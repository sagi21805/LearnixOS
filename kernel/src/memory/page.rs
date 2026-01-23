use core::{mem::ManuallyDrop, ptr::NonNull};

use crate::{
    memory::{
        allocators::slab::{
            descriptor::SlabDescriptor, traits::SlabPosition,
        },
        memory_map::ParsedMemoryMap,
        page::meta::{BuddyPageMeta, PageMeta},
        unassigned::{AssignSlab, UnassignSlab, Unassigned},
    },
    println,
};
use common::{
    address_types::PhysicalAddress,
    constants::{PAGE_ALLOCATOR_OFFSET, REGULAR_PAGE_SIZE},
    enums::BuddyOrder,
    late_init::LateInit,
    write_volatile,
};

pub mod map;
pub mod meta;

pub type UnassignedPage = Page<Unassigned>;

pub static mut PAGES: LateInit<&'static mut [UnassignedPage]> =
    LateInit::uninit();

impl AssignSlab for NonNull<Page<Unassigned>> {
    type Target<Unassigned: SlabPosition> = NonNull<Page<Unassigned>>;

    fn assign<T: SlabPosition>(&self) -> NonNull<Page<T>> {
        unsafe { NonNull::new_unchecked(self.as_ptr() as *mut Page<T>) }
    }
}

impl<T: SlabPosition> UnassignSlab for NonNull<Page<T>> {
    type Target = NonNull<Page<Unassigned>>;

    fn as_unassigned(&self) -> NonNull<Page<Unassigned>> {
        unsafe {
            NonNull::new_unchecked(self.as_ptr() as *mut Page<Unassigned>)
        }
    }
}

pub struct Page<T: 'static + SlabPosition> {
    pub owner: Option<NonNull<SlabDescriptor<T>>>,
    pub meta: PageMeta,
}

impl<T: 'static + SlabPosition> Page<T> {
    pub fn physical_address(&self) -> PhysicalAddress {
        let index = (self as *const _ as usize
            - unsafe { PAGES.as_ptr().addr() })
            / size_of::<UnassignedPage>();

        unsafe {
            PhysicalAddress::new_unchecked(index * REGULAR_PAGE_SIZE)
        }
    }

    pub fn get_buddy(&self) -> Option<*mut Page<T>> {
        let order = unsafe { self.meta.buddy.order? };
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
    #[allow(clippy::type_complexity)]
    pub unsafe fn split(
        &mut self,
    ) -> Option<(NonNull<Page<T>>, NonNull<Page<T>>)> {
        // Reduce it's order to find it's order.

        let prev_order = BuddyOrder::try_from(
            unsafe { self.meta.buddy.order? } as u8 - 1,
        )
        .unwrap();

        write_volatile!((*self.meta.buddy).order, Some(prev_order));

        let index = ((self as *const _ as usize
            - unsafe { PAGES.as_ptr().addr() })
            / size_of::<UnassignedPage>())
            + (1 << prev_order as usize);

        // Find it's half
        let mut buddy =
            unsafe { NonNull::from_mut(&mut PAGES[index]).assign::<T>() };

        // Set the order of the buddy.
        write_volatile!(
            (*buddy.as_mut().meta.buddy).order,
            Some(prev_order)
        );

        Some((NonNull::from_mut(self), buddy))
    }

    /// Try to merge this page with it's buddy.
    ///
    /// Note: This function should not be recursive
    pub unsafe fn merge(&self) {
        todo!("")
    }

    pub const fn index_of_page(address: PhysicalAddress) -> usize {
        address.as_usize() / REGULAR_PAGE_SIZE
    }
}

pub fn pages_init(mmap: ParsedMemoryMap) -> usize {
    let last = mmap.last().unwrap();
    let last_address = (last.base_address + last.length) as usize;
    let total_pages = last_address / REGULAR_PAGE_SIZE;

    println!(
        "Last address: {}, Total Pages: {}, size_of_array: {:x?} Kib",
        last_address,
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
                    meta: PageMeta {
                        buddy: ManuallyDrop::new(BuddyPageMeta::default()),
                    },
                    owner: None,
                },
            );
        }
        PAGES.as_ptr_range().end as usize
    }
}
