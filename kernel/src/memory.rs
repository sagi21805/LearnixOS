use alloc::alloc::StaticAllocator;
use buddy::BuddyAllocator;
use common::constants::{
    MEMORY_MAP_LENGTH, MEMORY_MAP_OFFSET, PARSED_MEMORY_MAP,
    REGULAR_PAGE_SIZE,
};
use core::{
    alloc::{AllocError, Allocator, Layout},
    ptr::NonNull,
};
use libk::okprintln;
use page::{Page, arena::PageMap};
use slab::{SlabAllocator, SlabSystem, static_allocator::GlobalSlab};
use sync::once::Once;
use x86::memory_map::{MemoryMap, MemoryRegion, MemoryRegionExtended};

use crate::task::{KernelTasks, Task};

pub static MMAP: Once<MemoryMap> = Once::uninit();

#[global_allocator]
static GLOBAL_ALLOCATOR: KernelAllocator = KernelAllocator;

static BUDDY_ALLOCATOR: Once<BuddyAllocator<PageMap, Page>> =
    Once::uninit();

static SLAB_ALLOCATOR: Once<SlabAllocator<KernelSlab>> = Once::uninit();

slab::slab_caches! {
    /// The caches of the kernel's own slab types.
    pub struct KernelCaches {
        KernelTasks { task: Task }
    }
}

/// The kernel's slab system, for the zero-sized allocator types.
pub struct KernelSlab;

impl SlabSystem for KernelSlab {
    type Block = Page;
    type Arena = PageMap;
    type Caches = KernelCaches;
}

// SAFETY: `SLAB_ALLOCATOR` is a static, so every call returns the same
// allocator.
unsafe impl GlobalSlab for KernelSlab {
    fn slab() -> &'static SlabAllocator<KernelSlab> { &SLAB_ALLOCATOR }
}

/// The kernel's global allocator. A layout that a generic size holds goes
/// to the slab, and every other layout to buddy.
///
/// It is zero-sized, and reaches both allocators through their statics,
/// so it works once [`init`] returns.
///
/// ```ignore
/// let small = Box::new(0u64); // a slot of `Generic8`
/// let large = Box::new([0u8; 8192]); // two pages from buddy
/// ```
pub struct KernelAllocator;

// SAFETY: both methods route a layout the same way, so a block returns to
// the allocator it came from.
unsafe impl Allocator for KernelAllocator {
    fn allocate(
        &self,
        layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        if SLAB_ALLOCATOR.serves(layout) {
            SLAB_ALLOCATOR.allocate(layout)
        } else {
            BUDDY_ALLOCATOR.allocate(layout)
        }
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        if SLAB_ALLOCATOR.serves(layout) {
            // SAFETY: `allocate` sent this layout to the slab too.
            unsafe { SLAB_ALLOCATOR.deallocate(ptr, layout) }
        } else {
            // SAFETY: `allocate` sent this layout to buddy too.
            unsafe { BUDDY_ALLOCATOR.deallocate(ptr, layout) }
        }
    }
}

// SAFETY: blocks are invalidated only by `deallocate`, and the two
// statics never change once set.
unsafe impl StaticAllocator for KernelAllocator {}

// SAFETY: the slab takes its pages from buddy directly and buddy from its
// arena, so a call never recurses into the global allocator.
unsafe impl core::alloc::GlobalAllocator for KernelAllocator {}

/// A box of a `T` from its own slab cache, one pointer wide.
#[expect(
    dead_code,
    reason = "the executor boxes tasks through their allocator, and \
              nothing else boxes a slab type yet"
)]
pub type SlabBox<T> = slab::SlabBox<T, KernelSlab>;

/// Parses the memory map and starts the buddy and slab allocators. The
/// global allocator works once this returns.
///
/// # Safety
///
/// The contract of [`read_memory_map`] applies.
pub unsafe fn init() {
    let mmap = MMAP.init(unsafe { read_memory_map() });

    let buddy = BUDDY_ALLOCATOR
        .init_with(BuddyAllocator::<PageMap, Page>::new(mmap), |buddy| {
            buddy.get_mut().initialize(mmap)
        });
    okprintln!("Initialized Buddy Allocator");

    let _ = SLAB_ALLOCATOR.init(SlabAllocator::new(buddy));
    okprintln!("Initialized Slab Allocator");
}

/// Parses the raw map the second stage stored into [`PARSED_MEMORY_MAP`].
///
/// # Safety
///
/// The second stage must have stored the map at [`MEMORY_MAP_OFFSET`] and
/// its length at [`MEMORY_MAP_LENGTH`]. Nothing else may use that memory
/// or the page at [`PARSED_MEMORY_MAP`].
unsafe fn read_memory_map() -> MemoryMap {
    let (raw, buf) = unsafe {
        let len = *(MEMORY_MAP_LENGTH as *const u32) as usize;
        (
            core::slice::from_raw_parts_mut(
                MEMORY_MAP_OFFSET as *mut MemoryRegionExtended,
                len,
            ),
            core::slice::from_raw_parts_mut(
                PARSED_MEMORY_MAP as *mut MemoryRegion,
                REGULAR_PAGE_SIZE / size_of::<MemoryRegion>(),
            ),
        )
    };

    MemoryMap::parse_map(raw, buf)
        .expect("the BIOS reports a usable region and the map fits a page")
}
