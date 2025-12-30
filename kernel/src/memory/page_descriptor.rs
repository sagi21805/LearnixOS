use core::{intrinsics::size_of, mem::MaybeUninit};

use common::constants::{REGULAR_PAGE_ALIGNMENT, REGULAR_PAGE_SIZE};

use crate::{alloc_pages, memory::allocators::slab_allocator::SlabCache};

pub struct Unassigned;

pub type UnassignedPage = Page<Unassigned>;

impl UnassignedPage {
    pub fn assign<T>(&self) -> &Page<T> {
        let ptr = self as *const _ as usize;
        unsafe { &*(ptr as *const Page<T>) }
    }

    pub fn assign_mut<T>(&mut self) -> &mut Page<T> {
        let ptr = self as *const _ as usize;
        unsafe { &mut *(ptr as *mut Page<T>) }
    }
}

pub static PAGES: MaybeUninit<&'static mut [UnassignedPage]> =
    MaybeUninit::uninit();

pub struct Page<T: 'static> {
    pub owner: Option<&'static SlabCache<T>>,
    pub counter: u64,
}

pub fn pages_init(usable_mem: usize) {
    let num_pages = usable_mem / REGULAR_PAGE_SIZE;

    let capacity = (num_pages * size_of::<UnassignedPage>())
        .next_multiple_of(REGULAR_PAGE_SIZE);

    let array_address =
        unsafe { alloc_pages!(capacity / REGULAR_PAGE_SIZE) };
}
