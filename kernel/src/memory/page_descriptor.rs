use crate::{
    alloc_pages,
    memory::{
        allocators::slab_allocator::SlabCache, memory_map::MemoryRegion,
    },
};
use common::constants::{REGULAR_PAGE_ALIGNMENT, REGULAR_PAGE_SIZE};
use core::{intrinsics::size_of, mem::MaybeUninit};
use strum::VariantArray;
use strum_macros::VariantArray;

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

#[derive(Default)]
pub struct BuddyBlockMeta {
    next: Option<&'static UnassignedPage>,
    prev: Option<&'static UnassignedPage>,
    order: BuddyOrder,
}

pub const BUDDY_MAX_ORDER: usize = BuddyOrder::VARIANTS.len();

#[derive(VariantArray, Clone, Copy, PartialEq, Eq)]
pub enum BuddyOrder {
    Order0 = 0,
    Order1 = 1,
    Order2 = 2,
    Order3 = 3,
    Order4 = 4,
    Order5 = 5,
    Order6 = 6,
    Order7 = 7,
    Order8 = 8,
    Order9 = 9,
}

impl BuddyOrder {
    pub const MAX: BuddyOrder = BuddyOrder::Order9;
    pub const MIN: BuddyOrder = BuddyOrder::Order0;
}

impl Default for BuddyOrder {
    fn default() -> Self {
        BuddyOrder::MAX
    }
}

#[derive(Default)]
pub struct BuddyAllocator {
    freelist: [BuddyBlockMeta; BUDDY_MAX_ORDER],
}

impl BuddyAllocator {
    pub fn alloc_pages() -> usize {
        unimplemented!()
    }
}

pub struct Page<T: 'static> {
    pub owner: Option<&'static SlabCache<T>>,
    pub buddy: BuddyBlockMeta,
}

pub fn pages_init(map: &mut [MemoryRegion]) {
    // let num_pages = usable_mem / REGULAR_PAGE_SIZE;

    // let capacity = (num_pages * size_of::<UnassignedPage>())
    //     .next_multiple_of(REGULAR_PAGE_SIZE);

    // let array_address =
    //     unsafe { alloc_pages!(capacity / REGULAR_PAGE_SIZE) };
}
