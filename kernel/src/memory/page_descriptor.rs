use crate::{
    memory::{allocators::slab::SlabCache, memory_map::ParsedMemoryMap},
    println,
};
use common::{
    constants::{
        PAGE_ALLOCATOR_OFFSET, REGULAR_PAGE_ALIGNMENT, REGULAR_PAGE_SIZE,
    },
    enums::{BUDDY_MAX_ORDER, BuddyOrder},
};
use core::{
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
};

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

impl BuddyBlockMeta {
    pub fn detach<T>(&mut self) -> Option<*mut Page<T>> {
        let detached = self.next? as *mut Page<T>; // None if there is no page to detach
        self.next = unsafe { (*detached).buddy_meta.next };
        Some(detached)
    }

    pub fn attach<T>(&mut self, attachment: *mut Page<T>) {
        let attachment_ref =
            unsafe { &mut *attachment }.as_unassigned_mut();
        attachment_ref.buddy_meta.next = self.next;
        self.next = Some(attachment_ref as *mut UnassignedPage)
    }
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
        let order = num_pages.next_power_of_two().leading_zeros() as usize;

        let page = self.freelist[order].next.unwrap_or_else(|| {
            self.split_until(order)
                .expect("Out of memory, swap is not implemented")
        });

        get_page_address(page)
    }

    pub fn free_pages(&self, address: usize) {
        unimplemented!()
    }

    /// This function assumes that `wanted_order` is empty, and won't check
    /// it.
    pub fn split_until(
        &self,
        wanted_order: usize,
    ) -> Option<*mut UnassignedPage> {
        let closet_order = ((wanted_order + 1)..BUDDY_MAX_ORDER)
            .find(|i| self.freelist[*i].next.is_some())?;

        let mut next_split = &self.freelist[closet_order];

        // for current_split in
        //     ((wanted_order + 1)..closet_order).rev().peekable()
        // {
        //     let page = self.freelist[current_split]
        //         .detach::<Unassigned>()
        //         .expect("Error in logic");
        // }
        None
    }

    /// TODO: Make an unsafe split if relevant
    ///
    /// # Safety
    /// This function does not attach the new references!
    pub unsafe fn split(
        &mut self,
        order: usize,
    ) -> Option<(&mut UnassignedPage, &mut UnassignedPage)> {
        let meta = &mut self.freelist[order];

        // Detach the page from it's order list.
        if let Some(page) = meta.detach::<Unassigned>() {
            let page_ref = unsafe { &mut (*page) };

            // Reduce it's order to find it's order.
            let prev_order =
                BuddyOrder::try_from(order as u8 - 1).unwrap();
            page_ref.buddy_meta.order = Some(prev_order);

            // Find it's buddy new buddy.
            let buddy = unsafe {
                &mut (*page_ref
                    .get_buddy()
                    .expect("Buddy order given is the max order"))
            };

            // Set the order of the buddy.
            buddy.buddy_meta.order = Some(prev_order);

            return Some((page_ref, buddy));
        }
        None
    }

    pub fn merge(&self) {
        unimplemented!()
    }

    pub fn init(&'static mut self) {
        self.freelist[BUDDY_MAX_ORDER - 1] =
            unsafe { PAGES[0].buddy_meta };

        let mut iter = unsafe {
            PAGES
                .iter_mut()
                .step_by(BuddyOrder::MAX as usize)
                .peekable()
        };
        let mut prev = None;

        while let Some(curr) = iter.next() {
            curr.buddy_meta.next = iter.peek().map(|v| {
                *v as *const UnassignedPage as *mut UnassignedPage
            });
            curr.buddy_meta.prev = prev;
            curr.buddy_meta.order = Some(BuddyOrder::MAX);
            prev = Some(curr)
        }
    }
}

#[derive(Default)]
pub struct Page<T: 'static> {
    pub owner: Option<&'static SlabCache<T>>,
    pub buddy_meta: BuddyBlockMeta,
}

impl<T: 'static> Page<T> {
    pub fn as_unassigned(&self) -> &UnassignedPage {
        let ptr = self as *const _ as usize;
        unsafe { &*(ptr as *const UnassignedPage) }
    }

    pub fn as_unassigned_mut(&mut self) -> &mut UnassignedPage {
        let ptr = self as *const _ as usize;
        unsafe { &mut *(ptr as *mut UnassignedPage) }
    }

    pub fn get_buddy(&self) -> Option<*mut Page<T>> {
        if let Some(order) = self.buddy_meta.order {
            if let BuddyOrder::MAX = order {
                return None;
            } else {
                return Some(
                    (self as *const _ as usize ^ (1 << order as usize))
                        as *mut Page<T>,
                );
            }
        }
        None
    }
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

pub fn get_page_address<T>(page: *const Page<T>) -> usize {
    let index = unsafe {
        PAGES.as_ptr().offset_from((&*page).as_unassigned()) as usize
    };
    index * REGULAR_PAGE_SIZE
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
