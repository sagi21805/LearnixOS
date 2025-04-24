use core::{alloc::{GlobalAlloc, Layout, LayoutError}, panic, ptr::{null, Alignment}};
use constants::values::REGULAR_PAGE_ALIGNMENT;
use super::super::paging::{address_types::{PhysicalAddress, VirtualAddress}, page_tables::PageTable};
use core::cell::UnsafeCell;

// TODO: static memory will be runtime variable
static TOTAL_MEMORY: usize = 0xffffffff;
static GLOBAL_ALLOCATOR: FreeListFrameAllocator = FreeListFrameAllocator::new();

pub struct FreeSpaceNode {
    start_address: PhysicalAddress,
    layout: Layout,
    next: Option<&'static mut Self>,
}


impl FreeSpaceNode {
    
    #[inline]
    const fn new(start_address: PhysicalAddress, mem_size: usize, alignment: usize) -> Self {
        Self {
            start_address,
            layout: unsafe { Layout::from_size_align_unchecked(mem_size, alignment) },
            next: None
        }
    }

    #[inline]
    const fn set_next(&mut self, next: &'static mut FreeSpaceNode) {
        self.next = Some(next);
    }

}

pub struct FreeListFrameAllocator {
    head: FreeSpaceNode
}

impl FreeListFrameAllocator {

    #[inline]
    pub const fn new() ->  Self {
        Self {
            head: FreeSpaceNode::new(
                PhysicalAddress::zero(), 
                TOTAL_MEMORY, 
                REGULAR_PAGE_ALIGNMENT.as_usize()
            )  
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
    inner: UnsafeCell<FreeListFrameAllocator>
}

/// TODO: Probably should add some kind of locking behavior when understanding it.
unsafe impl GlobalAlloc for Allocator {

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

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        todo!();
    }   
}

unsafe impl Sync for FreeListFrameAllocator {}
