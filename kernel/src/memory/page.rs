use core::{marker::PhantomData, ptr::NonNull};

use crate::memory::{
    allocators::{extensions::VirtualAddressExt, slab::traits::Slab},
    page::{map::PageMap, meta::PageMeta},
    unassigned::{AssignSlab, UnassignSlab},
};
use common::{
    address_types::{PhysicalAddress, VirtualAddress},
    constants::REGULAR_PAGE_SIZE,
    late_init::LateInit,
};

pub mod map;
pub mod meta;

pub type UnassignedPage = Page<()>;

pub static mut PAGES: LateInit<PageMap> = LateInit::uninit();

pub struct Page<T: Slab> {
    pub meta: PageMeta,
    _phantom: PhantomData<T>,
}

impl AssignSlab for NonNull<Page<()>> {
    type Target<Unassigned: Slab> = NonNull<Page<Unassigned>>;

    fn assign<T: Slab>(&self) -> NonNull<Page<T>> {
        unsafe { NonNull::new_unchecked(self.as_ptr() as *mut Page<T>) }
    }
}

impl<T: Slab> UnassignSlab for NonNull<Page<T>> {
    type Target = NonNull<Page<()>>;

    fn as_unassigned(&self) -> NonNull<Page<()>> {
        unsafe { NonNull::new_unchecked(self.as_ptr() as *mut Page<()>) }
    }
}

impl<T: Slab> Page<T> {
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

    /// Return the index of the page structure inside the [`PAGES`] array
    /// pointed by this virtual address.
    ///
    /// **Note**: if you meant to get the page structure, consider using
    /// [`Page<T>::from_virt`]
    pub fn index_of(addr: VirtualAddress) -> usize {
        addr.translate().as_usize() / REGULAR_PAGE_SIZE
    }

    /// Return the physical page structure that is pointed by this physical
    /// address
    pub fn from_virt(addr: VirtualAddress) -> NonNull<Page<T>> {
        unsafe {
            NonNull::from_ref(&PAGES[Page::<T>::index_of(addr)])
                .assign::<T>()
        }
    }
}
