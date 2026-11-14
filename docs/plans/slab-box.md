# Plan 1: `SlabBox`

## Context

The goal is a box that allocates and frees a `T: Slab` from `T`'s own slab
cache, and is one pointer wide.

Today `CacheAllocator` holds a `&SlabAllocator`, so
`Box<T, CacheAllocator<..>>` is 16 bytes and every call site has to pass the
allocator. A zero-sized allocator type fixes both. `Box<T, A>` stores an `A`
next to the pointer, so when `A` has no fields the box is 8 bytes, and
`Drop` still knows which cache to free into because `T` is in `A`'s type.

The global allocator can't do this job. It only receives a `Layout`, which
is a size and an alignment, so it can never pick a type's own cache.

This plan also builds the base that
[plan 2](slab-global-allocator.md) reuses, a zero-sized way to reach the
kernel's slab.

Names follow the vocabulary the slab crate already has (`CacheAllocator`,
`SlabAllocator`). They are proposals. The code blocks are sketches of the
design and have not been compiled.

One branch, one commit per stage.

## The design in one picture

```rust
// What a call site looks like when the plan is done.
let inode = SlabBox::try_new_slab(Inode::default())?;   // 8 bytes wide
drop(inode);                                            // back to Inode's cache
```

```text
SlabBox<Inode>                       = Box<Inode, StaticCacheAllocator<Inode, KernelSlab>>
StaticCacheAllocator<Inode, KernelSlab>   zero-sized, knows the type and the slab
  └─ KernelSlab::slab()                   the kernel's static SlabAllocator
       └─ .cache::<Inode>()               the existing CacheAllocator
            └─ SlabCaches[Inode::SLAB_POSITION]
```

## Stage 1. Let other crates declare slab types

Today no kernel type can be `Slab`. The trait is sealed with a `pub(crate)`
module (`crates/memory/slab/src/traits.rs:47`), and the only
`define_slab_system!()` call is inside the slab crate
(`crates/memory/slab/src/lib.rs:74`) with no types in it.

### Check first

Confirm that `#[group(..)] impl SlabFamily for (A, B)` from typestate-groups
works in a second crate for a trait the slab crate declares.
`define_generics!` (`local_macros.rs:28-39`) shows what the group needs in
scope, which is `SlabFamily` and `SlabFamilyGroupMarker`. If it can't, the
fix belongs in typestate-groups.

### Replace the seal

`Slab` becomes an `unsafe trait`. Its contract is the fact
`SlabCaches::cache` (`cache.rs:43-48`) relies on. `sealed::Sealed` goes.

```rust
// crates/memory/slab/src/traits.rs

/// A type allocated from its own slab cache, the one at `SLAB_POSITION`
/// in [`SlabCaches`](crate::cache::SlabCaches).
///
/// # Safety
///
/// No other slab type of the program may have the same `SLAB_POSITION`,
/// and it must be below [`MAX_SLAB_TYPES`](crate::MAX_SLAB_TYPES).
pub unsafe trait Slab: SlabGeometry + SlabFamily + 'static {
    /// The index of this type's cache, which no other type shares.
    const SLAB_POSITION: usize;
}
```

### A fixed table of caches

The slab crate has to size the cache table without knowing how many types
the kernel adds. The table gets a fixed capacity.

```rust
// crates/memory/slab/src/lib.rs

/// The slab types the slab crate declares itself.
pub const BUILTIN_SLAB_COUNT: usize = SLAB_COUNT;

/// The most slab types a program can declare. A cache is 12 bytes, so
/// the whole table is 768 bytes.
pub const MAX_SLAB_TYPES: usize = 64;

pub struct SlabAllocator<Block, Arena>
where
    Block: BuddyBlock + SlabBlock + 'static,
    Arena: BuddyArena<Block> + SlabArena + 'static,
{
    caches: SpinMutex<SlabCaches<MAX_SLAB_TYPES>>,
    buddy: &'static BuddyAllocator<Arena, Block>,
}
```

I first planned a `const N: usize` parameter on `SlabAllocator`. Writing the
`GlobalSlab` trait of stage 2 showed the cost. Its method would return
`SlabAllocator<Block, Arena, { Self::SLABS }>`, which needs
`generic_const_exprs`, an incomplete feature. The fixed table avoids the
parameter everywhere, for 768 bytes.

### The declaration macro

It reuses the body of `number_slab_types!` (`local_macros.rs:139-159`) with
an offset.

```rust
// crates/memory/slab/src/local_macros.rs

/// Declares a crate's slab types, in the family group `$group`, numbered
/// after the slab crate's own types. Call it once per program.
///
/// ```ignore
/// declare_slabs!(KernelObjects: Inode, Pcb);
///
/// assert_eq!(Inode::SLAB_POSITION, BUILTIN_SLAB_COUNT);
/// assert_eq!(Pcb::SLAB_POSITION, BUILTIN_SLAB_COUNT + 1);
/// ```
#[macro_export]
macro_rules! declare_slabs {
    ($group:ident: $($t:ty),* $(,)?) => {
        #[::typestate_groups::group($group)]
        impl $crate::traits::SlabFamily for ($($t,)*) {
            const PFLAGS: ::x86::structures::paging::PageEntryFlags =
                ::x86::structures::paging::PageEntryFlags::regular_page_flags();
            const PSIZE: ::common::enums::PageSize =
                ::common::enums::PageSize::Regular;
        }

        $(
            impl $crate::traits::SlabGeometry for $t {
                type PreAllocatedArray = [
                    $crate::preallocated::PreAllocated<$t>;
                    <$t as $crate::traits::SlabGeometry>::OBJECTS_PER_SLAB
                ];
            }

            // SAFETY: each type takes the next index after the built-in
            // ones, and the assert below keeps them inside the table.
            unsafe impl $crate::traits::Slab for $t {
                const SLAB_POSITION: usize =
                    $crate::BUILTIN_SLAB_COUNT + ${index()};
            }
        )*

        const _: () = assert!(
            $crate::BUILTIN_SLAB_COUNT + ${count($t)}
                <= $crate::MAX_SLAB_TYPES,
            "the program's slab types fit the cache table"
        );
    };
}
```

A declared type also carries `#[typestate_groups::state]` on its definition,
as the generic sizes do (`local_macros.rs:22-24`).

Out of scope: generic slab types such as `Task<F>`. Whether `#[state]` and
`#[group]` accept them is a question for the executor work.

## Stage 2. The zero-sized typed allocator and `SlabBox`

All in the slab crate, in a new module next to `cache_allocator.rs`.

### `GlobalSlab`

The slab crate can't own the static, because `SlabAllocator` is generic over
`Block` and `Arena` and the real types live in the kernel. The trait hands
it out, and it is the only thing an implementor supplies.

```rust
// crates/memory/slab/src/static_allocator.rs

/// The one slab allocator of a program, reachable with no reference.
pub trait GlobalSlab: 'static {
    type Block: BuddyBlock + SlabBlock + 'static;
    type Arena: BuddyArena<Self::Block> + SlabArena + 'static;

    fn slab() -> &'static SlabAllocator<Self::Block, Self::Arena>;
}
```

### `StaticCacheAllocator`

It has no fields, and forwards to the existing `CacheAllocator`
(`cache_allocator.rs:65-128`), so no allocation logic is written twice.

```rust
/// Allocates `T`s from their slab cache in the slab of `G`.
///
/// It is zero-sized, so a box that uses it is one pointer wide.
pub struct StaticCacheAllocator<T: Slab, G: GlobalSlab>(
    PhantomData<fn() -> (T, G)>,
);

impl<T: Slab, G: GlobalSlab> StaticCacheAllocator<T, G> {
    pub const fn new() -> Self { Self(PhantomData) }
}

// SAFETY: every handle reaches the same cache of the same allocator.
unsafe impl<T: Slab, G: GlobalSlab> Allocator
    for StaticCacheAllocator<T, G>
{
    fn allocate(
        &self,
        layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        G::slab().cache::<T>().allocate(layout)
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        unsafe { G::slab().cache::<T>().deallocate(ptr, layout) }
    }
}
```

### `SlabBox` and its constructor

```rust
/// A box of a `T` from its own slab cache, one pointer wide.
pub type SlabBox<T, G> = Box<T, StaticCacheAllocator<T, G>>;

#[extend::ext(name = SlabBoxNew)]
pub impl<T: Slab, G: GlobalSlab> SlabBox<T, G> {
    /// Moves `value` into a slot of `T`'s cache.
    fn try_new_slab(value: T) -> Result<Self, AllocError> {
        Box::try_new_in(value, StaticCacheAllocator::new())
    }
}
```

It stays a real `Box`, so `Deref`, `Drop`, `Box::pin`, `into_raw` and
`from_raw_in` come for free. The executor needs the last three.

## Stage 3. Wire it into the kernel and use it once

```rust
// kernel/src/memory.rs

/// The kernel's slab allocator, for the zero-sized allocator types.
pub struct KernelSlab;

impl GlobalSlab for KernelSlab {
    type Block = Page;
    type Arena = PageMap;

    fn slab() -> &'static SlabAllocator<Page, PageMap> { &SLAB_ALLOCATOR }
}

pub type SlabBox<T> = slab::SlabBox<T, KernelSlab>;
```

`&SLAB_ALLOCATOR` goes through `Once`'s `Deref`, which panics before
`memory::init` with "a `Once` is read after it is initialized".

Then one real use. The type must be defined in the kernel crate. The slab
traits are foreign to the kernel, so the orphan rule stops it from
implementing them for a type of another crate. That rules out `Scrollback`
from `vga_display`, which an earlier draft of this plan named. The kernel
defines no long-lived object of its own yet, so the first use is a small
check type until the executor's task header or an inode exists.

```rust
// kernel/src/main.rs, temporary

#[typestate_groups::state]
struct SlabCheck([u64; 5]);

slab::declare_slabs!(KernelObjects: SlabCheck);

const _: () = assert!(size_of::<SlabBox<SlabCheck>>() == 8);
```

## Verification

1. `cargo xtask build` and clippy pass. The const asserts (the box is 8
   bytes, the types fit the table) are compile-level tests.
2. The headless boot reaches the PCI listing.
3. A temporary check in `_start`. Box a `SlabCheck`, print its address, drop
   it, box again, and see the same address. Then box two and see them
   `size_of::<SlabCheck>()` apart. Remove the check and the type before
   committing.
4. `graphify update .`.

## Changes made while implementing

- `#[group]` from typestate-groups works from a second crate, so the
  "check first" item passed with no change to typestate-groups.
- `number_slab_types!` numbers by recursion (`$position + 1`) and counts
  with `[$(stringify!($t)),*].len()`. An exported macro that uses
  `${index()}` makes every calling crate enable `macro_metavar_expr`.
- The table check is per type, `$position < MAX_SLAB_TYPES`. The single
  end check `$end <= MAX` tripped `clippy::int_plus_one`.
- `try_new_slab` returns `SlabBox<T, G>`, not `Self`, as the `#[ext]`
  trait has no `Sized` bound. The slab crate enables `allocator_ext` for
  `Box::try_new_in`. `StaticCacheAllocator` has `Default`, which clippy's
  `new_without_default` asks for.
- The boot check ran and was removed. A dropped `SlabBox` slot came back at
  the same address, and the next box was 40 bytes after it, the size of
  the check type.
- `KernelSlab` and the kernel's `SlabBox` alias carry
  `#[expect(dead_code)]` until the kernel declares its first slab type. The
  kernel adds the `typestate-groups` dependency at the same time, as
  `declare_slabs!` expands to `::typestate_groups` paths.

## Not changed by this plan

- The slab and buddy locks are plain spinlocks taken with interrupts on.
  Interrupt handlers still must not allocate or free.
- The slab crate has no host tests today, so this plan checks through the
  build and the boot.
- Book: chapter 5 has no slab section, so nothing goes stale.
