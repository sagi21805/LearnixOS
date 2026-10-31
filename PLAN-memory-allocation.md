# Plan: stabilize kernel memory allocation

Tracked in #89.

## Goal

Make kernel memory allocation stable and tested end to end. The buddy allocator hands out pages, the slab allocator gets its pages from the buddy allocator and serves small objects, and the kernel routes every allocation to one of them. Integration tests run on the host (native and Miri) and inside the kernel under QEMU.

Work happens on the `feat/memory-allocation` branch, one commit per phase.

Related: #43 (slab allocator), #54 and #86 (tests), #80 (buddy init above 170 MB).

## Where things stand

- `BuddyAllocator<PageMap, Page>` is the kernel's global allocator, installed through `GlobalAllocator::set` after the bump allocator.
- The slab crate has its caches, descriptors and lists, but no page source. `SlabDescriptor::new` is `todo!()` and calls the global `alloc(T::SLAB_LAYOUT)`. `SlabCache::take_object` reaches `todo!()` once its free and partial lists are empty.
- `SlabAllocator` holds `&SpinMutex<Arena>`. It can find a page on the free path but can't allocate one, because the buddy free lists live in `BuddyAllocator`, not in the arena.
- No allocator tests exist. The earlier slab tests used stand-in `TestPage` and `TestPageMap` types and were removed.

## Phases

### 1. `alloc_pages` and `free_pages` on the buddy allocator

Split `BuddyAllocator::allocate` in two. The first half (take the head of the order's free list, else `split_until`, then `allocate_meta`) becomes

```rust
pub fn alloc_pages(&self, order: usize) -> Option<NonNull<Block>>;
pub unsafe fn free_pages(&self, block: NonNull<Block>, order: usize);
```

and `Allocator::allocate` and `deallocate` call them. The slab layer gets the block's metadata, which is where its descriptor lives, and doesn't round-trip through a `Layout`. This mirrors Linux, where SLUB calls `alloc_pages` and gets a `struct page *`.

### 2. Slab pages come from the buddy allocator

- `SlabAllocator` holds `&'static BuddyAllocator<Arena, Block>` in place of `&'static SpinMutex<Arena>`.
- `SlabDescriptor::new` takes the block `alloc_pages(T::BUDDY_ORDER)` returned, writes a free descriptor into its `Page`, and builds `SlabObjects` from the block's address. The global `alloc` call in `descriptor/free.rs` goes away.
- `SlabCache::take_object` receives the page source as an argument when it needs a new slab. `SlabCaches` doesn't store it, so the caches stay independent of where pages come from.
- `Page` implements `SlabBlock`. `slab_descriptor_mut` casts `&raw mut self.slab`, which is sound because `SlabDescriptor` is `repr(C)` and its layout doesn't depend on `T`. `from_address` can't return `Self` by value, since a page's metadata is an entry in the `PageMap`. Replace it with the arena's `page_with_address`, or make it return `NonNull<Self>`.

### 3. The lock order

`deallocate` locks the buddy arena, then the slab caches. After phase 2, `take_object` runs under the slab lock and calls `alloc_pages`, which locks the buddy free lists and arena. One CPU freeing and another allocating would each hold the lock the other wants.

Fix it in `deallocate`. Drop the arena lock once the page is found, before locking the slab caches. The `PageMap` doesn't move, so the page pointer stays valid without the lock. Then the only order left is slab, then buddy. Document it on `CacheAllocator`.

### 4. A slab page must not read as free to the buddy allocator

`Page` is a union of `BuddyMeta` and `SlabDescriptor`, and both are `repr(C)`:

| Offset | `BuddyMeta<Regular>` | `SlabDescriptor` |
|---|---|---|
| 0 | `next` | `state` |
| 8 | `prev` | `objects` |
| 16 | `flags` (`order` bits 0 to 6, `allocated` bit 7) | `next` |

The slab's `next` lands on `BuddyFlags`. A slab with no `next` zeroes that byte, so the buddy allocator sees a free order-0 block and can merge it into its buddy while objects in it are still in use. Choose one fix:

- Reorder `SlabDescriptor` so offset 16 is a byte the slab never writes, with `allocated` kept set.
- Add a "slab page" bit both structs reserve, and have the buddy allocator check it first.
- Put the buddy flags first in `BuddyMeta`, and keep the slab metadata off that byte.

Add a `const` assert that `SlabDescriptor` fits in `BuddyMeta`, next to the other size asserts in `descriptor.rs`.

### 5. Higher-half `prev`

`FullFreeMeta::prev` is a 48-bit bitfield, and its setter panics on `0xffff_8000_0000_1000`. x86-64 addresses are canonical (bits 48 to 63 copy bit 47), so 48 bits hold every address if the conversion drops the top 16 bits on write and sign-extends bit 47 on read. `From<SlabAddress> for u64` hands over all 64 bits today.

### 6. One allocator for the kernel

The kernel owns both statics (`BUDDY_ALLOCATOR` already lives in `kernel/src/main.rs`) and builds `SlabAllocator` from them. A kernel allocator then routes each request:

- `allocate`: layouts a size class fits go to the slab, the rest to the buddy allocator.
- `deallocate`: route by the page, not by the layout. Blocks the buddy allocator handed out before the switch (the keyboard buffer, for one) can have small layouts, and routing those by layout would free them into a slab. The page's metadata says which allocator owns it, which phase 4's fix has to make readable.

`GlobalAllocator::set` requires the new allocator to take over every live block of the old one, and routing by page is what makes that hold.

### 7. Host integration tests

See the test plan below. These need:

- **A buddy arena over `std` memory.** `BuddyArena::new` takes a `MemoryMap`, so the test builds one that describes a `std` allocation.
- **A swappable address-to-page lookup.** `deallocate` calls `VirtualAddress::translate`, which reads CR3. That's privileged on the host and inline assembly that Miri rejects. Make the lookup part of the page source so a host arena can map addresses itself. Otherwise host tests cover allocation, and QEMU covers freeing.
- **Exposed provenance for the `PageMap`.** `SlabAddress` keeps only `addr()`, and `as_non_null` rebuilds the pointer with `with_exposed_provenance_mut`. On the host, call `expose_provenance()` once on the `PageMap` array's root pointer after allocating it.
- **Per-test state.** The slab caches are one `static`, and `cargo test` runs tests in parallel. Run these with `--test-threads=1`, or give each test its own caches.
- **A home outside `slab`.** `page` depends on `slab`, so a dev-dependency from `slab` on `page` builds two copies of `slab`. Use `crates/memory/page/tests/` or a small test crate depending on `buddy`, `page` and `slab`.

Run them with the existing aliases:

```bash
cargo test-host -p <crate>
cargo miri-host -p <crate>
MIRIFLAGS=-Zmiri-tree-borrows cargo miri-host -p <crate>
```

### 8. In-kernel tests

Add `cargo xtask test`, which builds the kernel with a `test` feature and boots it in QEMU. After the allocators are set up, the kernel runs every registered test and exits QEMU with a pass or fail code.

- **Exit code.** Start QEMU with `-device isa-debug-exit,iobase=0xf4,iosize=0x04`. Writing a value to port `0xf4` exits QEMU with status `(value << 1) | 1`, and `xtask` maps that to pass or fail.
- **Output.** Write test results to COM1 (port `0x3f8`) and start QEMU with `-serial stdio`, so results reach the terminal and CI without reading the VGA buffer. A 16550 UART driver is about 30 lines and worth writing by hand rather than pulling in a crate.
- **Test registration.** `#![feature(custom_test_frameworks)]` with a `test_runner`, or a hand-rolled `#[kernel_test]` macro that collects test functions into a linker section.
- **Faults.** A panic or CPU exception during a test reports failure and exits, so a broken test can't hang CI.

### 9. Compile-fail tests

`link` on an attached descriptor, `unlink` on a detached one, and `take_object` on a free or full one must not compile. `Link` and `Unlink` are `pub(crate)`, which doctests can't reach, so use trybuild in `tests/`, or make the traits `pub`.

## Test plan

Every test runs on the real components: `BuddyAllocator<PageMap, Page>` for pages, `Page` for descriptors, and `SlabAllocator` or `CacheAllocator` for objects. No stand-in types.

### Buddy, on the host

- `alloc_pages` of every order returns an aligned block of that size, and `free_pages` merges it back, so the free lists end where they started.
- Splitting a high-order block puts one buddy on each lower free list.
- Freeing two buddies in either order merges them, and freeing one alone doesn't.
- A memory map larger than 170 MB initializes, to reproduce #80.

### Slab, on the host

Allocation goes through `Box::new_in` and `Vec::with_capacity_in`.

- A fresh slab hands out every object, and the last take reports the slab full.
- Allocating from an empty cache and from a cache whose slabs are all full both succeed (phase 2).
- Values across several slabs keep their data after every other one is freed and the holes refilled.
- Slabs move between the free, partial and full lists, with lengths checked after each step. Free in an order that unlinks a middle node of the full list and of the partial list, as the head-only paths are the easy ones.
- A freed object is the next one allocated.
- A layout bigger than the class, or aligned past `OBJECT_ALIGNMENT`, is refused.
- Objects bigger than a page live in two-page slabs.
- A `Vec` fills one object and can't grow past it.
- A boxed linked list of a few hundred nodes is built, summed and freed.
- Random allocate and free steps, with every value checked. Fewer steps under `cfg!(miri)`.

### Buddy and slab together, on the host

- After a slab takes a block, the buddy allocator still sees it as allocated, and freeing its buddy doesn't merge them (phase 4).
- A `prev` set to a higher-half address reads back unchanged (phase 5).
- The kernel allocator sends small layouts to the slab and large ones to the buddy allocator, and frees a block allocated before the switch through the buddy allocator (phase 6).

### In the kernel, under QEMU

- The free path, including `translate` and the real page tables.
- `Box`, `Vec` and `String` through the global allocator after the switch to the kernel allocator.
- Allocation under interrupts. A timer or keyboard handler that allocates while the main thread holds no allocator lock.
- Allocate until the buddy allocator runs out, then free everything, and the free lists match their state at boot.

### Habits that caught bugs before

- Write every byte of each object on allocation, so Miri sees any overlap between objects and the slab's bookkeeping.
- Track handed-out addresses in a `HashSet`, so an object handed out twice fails at once.
- Run Miri under both Stacked Borrows and Tree Borrows. Stacked Borrows alone caught the bitfield setter bug.
- Plant a bug to check the tests catch it. Making `SlabObjects::slot` return the next slot made Miri stop at the out-of-bounds read in `take_object`, while the native run only failed later.

## Open questions

| Question | Options |
|---|---|
| Buddy flags under a slab (phase 4) | Reorder `SlabDescriptor`, a shared "slab page" bit, or buddy flags first in `BuddyMeta` |
| Address-to-page lookup on the host | Part of the page source, or host tests skip freeing |
| Where host integration tests live | `crates/memory/page/tests/`, or a test crate |
| In-kernel test registration | `custom_test_frameworks`, or a `#[kernel_test]` macro with a linker section |
| Descriptor addressing | Exposed provenance (works today), or a root pointer for strict provenance |
| Double free | A debug-only bitmap check, a free-list walk, or nothing |
| Kernel types with their own cache | A registration macro, or size classes only |

## Out of scope

- Returning free slabs to the buddy allocator (shrinking). When it lands, the `Page` gets valid `BuddyMeta` back before the buddy allocator sees the block.
- The virtual memory allocator (#44).

## Done when

- The kernel boots with the combined allocator as its global allocator, and nothing in `buddy` or `slab` reaches `todo!()` on the allocation or free path.
- `cargo test-host` and `cargo miri-host` (both borrow models) pass for the integration tests.
- `cargo xtask test` boots the kernel, runs the in-kernel tests and exits with their result.

