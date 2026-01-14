use core::ptr::NonNull;

use crate::{
    memory::{
        allocators::{
            page_allocator::buddy::BuddyBlockMeta,
            slab::{cache::SlabCache, traits::SlabPosition},
        },
        memory_map::ParsedMemoryMap,
    },
    println,
};
use common::{
    constants::{
        PAGE_ALLOCATOR_OFFSET, REGULAR_PAGE_ALIGNMENT, REGULAR_PAGE_SIZE,
    },
    enums::BuddyOrder,
    late_init::LateInit,
    write_volatile,
};

#[derive(Default, Clone, Copy, Debug)]
pub struct Unassigned;

pub type UnassignedPage = Page<Unassigned>;

impl UnassignedPage {
    pub fn assign<T: SlabPosition>(&self) -> &Page<T> {
        let ptr = self as *const _ as usize;
        unsafe { &*(ptr as *const Page<T>) }
    }

    pub fn assign_mut<T: SlabPosition>(&mut self) -> &mut Page<T> {
        let ptr = self as *const _ as usize;
        unsafe { &mut *(ptr as *mut Page<T>) }
    }
}

pub static mut PAGES: LateInit<&'static mut [UnassignedPage]> =
    LateInit::uninit();

#[derive(Debug)]
pub struct Page<T: 'static + SlabPosition> {
    pub owner: Option<NonNull<SlabCache<T>>>,
    pub buddy_meta: BuddyBlockMeta,
}

impl<T: 'static + SlabPosition> Page<T> {
    pub fn as_unassigned(&self) -> &UnassignedPage {
        let ptr = self as *const _ as usize;
        unsafe { &*(ptr as *const UnassignedPage) }
    }

    pub fn as_unassigned_mut(&mut self) -> &mut UnassignedPage {
        let ptr = self as *const _ as usize;
        unsafe { &mut *(ptr as *mut UnassignedPage) }
    }

    pub fn physical_address(&self) -> usize {
        let index = (self.as_unassigned() as *const _ as usize
            - PAGE_ALLOCATOR_OFFSET)
            / size_of::<UnassignedPage>();

        index * REGULAR_PAGE_SIZE
    }

    pub fn get_buddy(&self) -> Option<*mut Page<T>> {
        let order = self.buddy_meta.order?;
        if let BuddyOrder::MAX = order {
            None
        } else {
            Some(
                (self as *const _ as usize
                    ^ ((1 << order as usize)
                        * size_of::<UnassignedPage>()))
                    as *mut Page<T>,
            )
        }
    }

    /// TODO: Make an unsafe split if relevant
    ///
    /// # Safety
    /// This function does not attach the new references!
    pub unsafe fn split(
        &mut self,
    ) -> Option<(*mut Page<T>, *mut Page<T>)> {
        // Reduce it's order to find it's order.

        let prev_order =
            BuddyOrder::try_from(self.buddy_meta.order? as u8 - 1)
                .unwrap();

        write_volatile!(self.buddy_meta.order, Some(prev_order));
        let index = ((self.as_unassigned() as *const _ as usize
            - PAGE_ALLOCATOR_OFFSET)
            / size_of::<UnassignedPage>())
            + (1 << prev_order as usize);

        // Find it's half
        let buddy = unsafe { PAGES[index].assign_mut::<T>() };

        // Set the order of the buddy.
        write_volatile!(buddy.buddy_meta.order, Some(prev_order));

        Some((self as *mut Page<T>, buddy as *mut Page<T>))
    }
}

pub fn pages_init(map: &ParsedMemoryMap) -> usize {
    let last = map.last().unwrap();
    let last_page = (last.base_address + last.length) as usize
        & !REGULAR_PAGE_ALIGNMENT.as_usize();
    let total_pages = last_page / REGULAR_PAGE_SIZE;

    println!(
        "Last Page: {}, Total Pages: {}, size_of_array: {:x?} Kib",
        last_page,
        total_pages,
        total_pages * size_of::<Page<Unassigned>>() / 1024
    );
    unsafe {
        PAGES.write(core::slice::from_raw_parts_mut(
            PAGE_ALLOCATOR_OFFSET as *mut UnassignedPage,
            total_pages,
        ));

        for p in PAGES.iter_mut() {
            core::ptr::write_volatile(
                p as *mut UnassignedPage,
                UnassignedPage {
                    buddy_meta: BuddyBlockMeta::default(),
                    owner: None,
                },
            );
        }
        PAGES.as_ptr_range().end as usize
    }
}
