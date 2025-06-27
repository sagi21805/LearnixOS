use super::super::paging::{
    address_types::{PhysicalAddress, VirtualAddress},
    page_tables::PageTable,
};
use common::constants::values::REGULAR_PAGE_ALIGNMENT;
use core::cell::UnsafeCell;
use core::{
    alloc::{GlobalAlloc, Layout, LayoutError},
    panic,
    ptr::{Alignment, null},
};

// TODO: static memory will be runtime variable
static TOTAL_MEMORY: usize = 0xffffffff;
static GLOBAL_ALLOCATOR: FreeListFrameAllocator = FreeListFrameAllocator::new();

pub struct FreeSpaceNode {
    start_address: PhysicalAddress,
    layout: Layout,
    next: Option<&'static mut Self>,
}

impl FreeSpaceNode {
    /// Creates a new `FreeSpaceNode` with the specified start address, memory size, and alignment.
    ///
    /// # Parameters
    /// - `start_address`: The starting physical address of the free memory region.
    /// - `mem_size`: The size of the free memory region in bytes.
    /// - `alignment`: The alignment requirement for the memory region.
    ///
    /// # Examples
    ///
    /// ```
    /// let node = FreeSpaceNode::new(PhysicalAddress(0x1000), 4096, 4096);
    /// assert_eq!(node.start_address, PhysicalAddress(0x1000));
    /// ```
    #[inline]
    const fn new(start_address: PhysicalAddress, mem_size: usize, alignment: usize) -> Self {
        Self {
            start_address,
            layout: unsafe { Layout::from_size_align_unchecked(mem_size, alignment) },
            next: None,
        }
    }

    /// Sets the next node in the free list to the specified node.
    #[inline]
    const fn set_next(&mut self, next: &'static mut FreeSpaceNode) {
        self.next = Some(next);
    }
}

pub struct FreeListFrameAllocator {
    head: FreeSpaceNode,
}

impl FreeListFrameAllocator {
    /// Creates a new free-list frame allocator covering the entire available memory.
    ///
    /// The allocator is initialized with a single free node spanning from address zero to the total memory size, aligned to the regular page alignment.
    ///
    /// # Examples
    ///
    /// ```
    /// let allocator = FreeListFrameAllocator::new();
    /// ```
    #[inline]
    pub const fn new() -> Self {
        Self {
            head: FreeSpaceNode::new(
                PhysicalAddress::zero(),
                TOTAL_MEMORY,
                REGULAR_PAGE_ALIGNMENT.as_usize(),
            ),
        }
    }

    // pub fn find_free_block(&mut self, layout: Layout) -> Option<&'static mut FreeSpaceNode> {
    //     let mut current = Some(&mut self.head); // Get mutable reference to head

    //     loop {

    //         if let Some(ref region) = current  {
    //             if region.layout.size() >= layout.size() && region.layout.align() >= layout.align() {
    //                 let ret = current.take();

    //                 return ret;
    //             }
    //         }

    //     }

    // }
}

struct Allocator {
    inner: UnsafeCell<FreeListFrameAllocator>,
}

/// TODO: Probably should add some kind of locking behavior when understanding it.
unsafe impl GlobalAlloc for Allocator {
    /// Allocates a memory block with the specified layout from the free list.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it may return a pointer to uninitialized or invalid memory if the allocator is not properly initialized or if the requested layout cannot be satisfied.
    ///
    /// # Returns
    ///
    /// A pointer to the beginning of the allocated memory block, or panics if allocation fails.
    ///
    /// # Panics
    ///
    /// Panics if the requested alignment does not match the page alignment or if no suitable free block is available.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Usage requires a properly initialized allocator and valid layout.
    /// let layout = Layout::from_size_align(4096, 4096).unwrap();
    /// let ptr = unsafe { ALLOCATOR.alloc(layout) };
    /// assert!(!ptr.is_null());
    /// ```
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        //     if layout.align() != PAGE_ALIGNMENT.as_usize() {
        //         panic!("Page Error!");
        //     }
        //     let mutable = unsafe { self.inner.as_mut_unchecked() };
        //     // match mutable.find_free_block(layout) {
        //         Some(block) => {
        //             let frame_address = block.start_address.clone();
        //             block.start_address += layout.size();
        //             return frame_address.address() as *mut u8
        //         }
        //         None => {
        //             panic!("Memory error, there is no free block")
        //         }
        //     }
        todo!()
    }

    /// Deallocates a previously allocated memory block, returning it to the free list.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `ptr` was allocated by this allocator and that `layout` matches the original allocation.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Usage depends on allocator integration; see allocator documentation.
    /// ```
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        todo!();
    }
}

unsafe impl Sync for FreeListFrameAllocator {}
