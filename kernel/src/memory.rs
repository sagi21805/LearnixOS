use buddy::BuddyAllocator;
use common::constants::{
    MEMORY_MAP_LENGTH, MEMORY_MAP_OFFSET, PARSED_MEMORY_MAP,
    REGULAR_PAGE_SIZE,
};
use libk::{alloc::GlobalAllocator, okprintln};
use page::{Page, arena::PageMap};
use slab::{GlobalSlab, SlabAllocator};
use sync::once::Once;
use x86::memory_map::{MemoryMap, MemoryRegion, MemoryRegionExtended};

static MMAP: Once<MemoryMap> = Once::uninit();

#[global_allocator]
static GLOBAL_ALLOCATOR: GlobalAllocator = GlobalAllocator::uninit();

static BUDDY_ALLOCATOR: Once<BuddyAllocator<PageMap, Page>> =
    Once::uninit();

static SLAB_ALLOCATOR: Once<SlabAllocator<Page, PageMap>> = Once::uninit();

/// The kernel's slab allocator, for the zero-sized allocator types.
#[expect(
    dead_code,
    reason = "the kernel declares no slab type of its own yet"
)]
pub struct KernelSlab;

impl GlobalSlab for KernelSlab {
    type Block = Page;
    type Arena = PageMap;

    fn slab() -> &'static SlabAllocator<Page, PageMap> { &SLAB_ALLOCATOR }
}

/// A box of a `T` from its own slab cache, one pointer wide.
#[expect(
    dead_code,
    reason = "the kernel declares no slab type of its own yet"
)]
pub type SlabBox<T> = slab::SlabBox<T, KernelSlab>;

/// What the later boot steps need from the memory setup.
pub struct KernelMemory {
    pub mmap: &'static MemoryMap,
    pub slab: &'static SlabAllocator<Page, PageMap>,
}

/// Parses the memory map and starts the buddy, global and slab allocators.
///
/// # Safety
///
/// The contract of [`read_memory_map`] applies.
pub unsafe fn init() -> KernelMemory {
    let mmap = MMAP.init(unsafe { read_memory_map() });

    let buddy = BUDDY_ALLOCATOR
        .init_with(BuddyAllocator::<PageMap, Page>::new(mmap), |buddy| {
            buddy.get_mut().initialize(mmap)
        });
    // SAFETY: the buddy allocator takes pages only from its own arena.
    unsafe { GLOBAL_ALLOCATOR.set(buddy) };
    okprintln!("Initialized Buddy Allocator");

    let slab = SLAB_ALLOCATOR.init(SlabAllocator::new(buddy));
    okprintln!("Initialized Slab Allocator");

    KernelMemory { mmap, slab }
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
