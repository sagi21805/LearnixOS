use crate::{
    memory::{
        allocators::slab_allocator::SlabCache, memory_map::ParsedMemoryMap,
    },
    println,
};
use common::constants::{
    PAGE_ALLOCATOR_OFFSET, REGULAR_PAGE_ALIGNMENT, REGULAR_PAGE_SIZE,
};
use core::{
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
};
use strum::VariantArray;
use strum_macros::VariantArray;

#[derive(Default)]
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

pub static mut PAGES: LateInit<&'static mut [UnassignedPage]> =
    LateInit::uninit();

#[derive(Default, Clone, Copy)]
pub struct BuddyBlockMeta {
    next: Option<*mut UnassignedPage>,
    prev: Option<*mut UnassignedPage>,
    order: Option<BuddyOrder>,
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

#[derive(Default)]
pub struct BuddyAllocator {
    freelist: [BuddyBlockMeta; BUDDY_MAX_ORDER],
}

impl BuddyAllocator {
    pub fn alloc_pages(&self, num_pages: usize) -> usize {
        assert!(
            num_pages < (1 << BUDDY_MAX_ORDER),
            "Size cannot be greater then: {}",
            1 << BUDDY_MAX_ORDER
        );
        let order = num_pages.next_power_of_two().leading_zeros();
    }

    pub fn init(&'static mut self) {
        self.freelist[BUDDY_MAX_ORDER - 1] =
            unsafe { PAGES[0].buddy_meta };

        let mut iter = unsafe { PAGES.iter_mut().peekable() };
        let mut prev = None;

        while let Some(curr) = iter.next() {
            curr.buddy_meta.next = iter.peek().map(|v| {
                *v as *const UnassignedPage as *mut UnassignedPage
            });
            curr.buddy_meta.prev = prev;
            prev = Some(curr)
        }
    }
}

#[derive(Default)]
pub struct Page<T: 'static> {
    pub owner: Option<&'static SlabCache<T>>,
    pub buddy_meta: BuddyBlockMeta,
}

pub struct LateInit<T>(MaybeUninit<T>);

impl<T> LateInit<T> {
    pub const fn uninit() -> LateInit<T> {
        LateInit::<T>(MaybeUninit::uninit())
    }

    pub const fn write(&mut self, val: T) {
        self.0.write(val);
    }
}

impl<T> Deref for LateInit<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.0.assume_init_ref() }
    }
}

impl<T> DerefMut for LateInit<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.0.assume_init_mut() }
    }
}

pub fn pages_init(map: &ParsedMemoryMap) -> usize {
    let last = map.last().unwrap();
    let last_page = (last.base_address + last.length) as usize
        & !REGULAR_PAGE_ALIGNMENT.as_usize();
    let total_pages = last_page / REGULAR_PAGE_SIZE;
    println!("Last Page: {}, Total Pages: {}", last_page, total_pages);

    unsafe {
        PAGES.write(core::slice::from_raw_parts_mut(
            PAGE_ALLOCATOR_OFFSET as *mut UnassignedPage,
            total_pages,
        ));

        PAGES
            .iter_mut()
            .for_each(|p| *p = UnassignedPage::default());

        PAGES.as_ptr_range().end as usize
    }
}
