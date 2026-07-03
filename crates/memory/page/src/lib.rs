#![no_std]
#![feature(const_trait_impl)]

// pub mod arena;
pub mod map;
pub mod meta;

use core::ptr::NonNull;

use crate::meta::PageMeta;
use buddy::meta::{BuddyBlock, BuddyMeta, Regular};

pub struct Page {
    pub meta: PageMeta,
}

#[rustfmt::skip]
impl const BuddyBlock for Page {
    fn from_meta(meta: NonNull<BuddyMeta<Regular>>) -> NonNull<Self> {
        let offset = core::mem::offset_of!(Page, meta.buddy);
        unsafe {
            NonNull::new_unchecked(
                meta.as_ptr().cast::<u8>().sub(offset).cast::<Self>(),
            )
        }
    }

    fn meta_mut(&mut self) -> &mut BuddyMeta<Regular> {
        unsafe { NonNull::from_ref(&self.meta.buddy).as_mut() }
    }

    fn meta(&self) -> &BuddyMeta<Regular> {
        unsafe { NonNull::from_ref(&self.meta.buddy).as_ref() }
    }
}

// impl AssignSlab for NonNull<Page<()>> {
//     type Target<Unassigned: Slab> = NonNull<Page<Unassigned>>;

//     fn assign<T: Slab>(&self) -> NonNull<Page<T>> {
//         unsafe { NonNull::new_unchecked(self.as_ptr() as *mut Page<T>) }
//     }
// }

// impl<T: Slab> UnassignSlab for NonNull<Page<T>> {
//     type Target = NonNull<Page<()>>;

//     fn as_unassigned(&self) -> NonNull<Page<()>> {
//         unsafe { NonNull::new_unchecked(self.as_ptr() as *mut Page<()>)
// }     }
// }

// impl<T: Slab> Page<T> {
//     pub fn new(meta: PageMeta) -> Page<T> {
//         Page {
//             meta,
//             _phantom: PhantomData::<T>,
//         }
//     }

//     pub fn physical_address(&self) -> PhysicalAddress {
//         let index = (self as *const _ as usize
//             - unsafe { PAGES.as_ptr().addr() })
//             / size_of::<UnassignedPage>();

//         unsafe {
//             PhysicalAddress::new_unchecked(index * REGULAR_PAGE_SIZE)
//         }
//     }

//     /// Return the index of the page structure inside the [`PAGES`]
// array     /// pointed by this virtual address.
//     ///
//     /// **Note**: if you meant to get the page structure, consider using
//     /// [`Page<T>::from_virt`]
//     pub fn index_of(addr: VirtualAddress) -> usize {
//         addr.translate()
//             .expect("Address could not be translated")
//             .as_usize()
//             / REGULAR_PAGE_SIZE
//     }

//     /// Return the physical page structure that is pointed by this
// physical     /// address
//     pub fn from_virt(addr: VirtualAddress) -> NonNull<Page<T>> {
//         unsafe {
//             NonNull::from_ref(&PAGES[Page::<T>::index_of(addr)])
//                 .assign::<T>()
//         }
//     }
// }
