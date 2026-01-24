use core::{marker::PhantomData, ptr::NonNull};

use crate::memory::{
    allocators::slab::traits::SlabPosition,
    page::{map::PageMap, meta::PageMeta},
    unassigned::{AssignSlab, UnassignSlab, Unassigned},
};
use common::{
    address_types::PhysicalAddress, constants::REGULAR_PAGE_SIZE,
    late_init::LateInit,
};

pub mod map;
pub mod meta;

pub type UnassignedPage = Page<Unassigned>;

pub static mut PAGES: LateInit<PageMap> = LateInit::uninit();

pub struct Page<T: 'static + SlabPosition> {
    pub meta: PageMeta,
    _phantom: PhantomData<T>,
}

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

impl<T: 'static + SlabPosition> Page<T> {
    pub fn new(meta: PageMeta) -> Page<T> {
        Page {
            meta,
            _phantom: PhantomData::<T>,
        }
    }

    pub fn physical_address(&self) -> PhysicalAddress {
        let index = (self as *const _ as usize
            - unsafe { PAGES.as_ptr().addr() })
            / size_of::<UnassignedPage>();

        unsafe {
            PhysicalAddress::new_unchecked(index * REGULAR_PAGE_SIZE)
        }
    }

    pub const fn index_of_page(address: PhysicalAddress) -> usize {
        address.as_usize() / REGULAR_PAGE_SIZE
    }
}
