#![no_std]
#![feature(const_trait_impl)]

pub mod arena;
pub mod meta;
mod placement;

use core::{
    cell::UnsafeCell,
    fmt::Debug,
    mem::{ManuallyDrop, offset_of},
    ptr::NonNull,
};

use buddy::meta::{
    BlockFlagBits, BlockFlags, BuddyBlock, BuddyLinks, Regular,
};

use slab::{
    descriptor::{SlabDescriptor, Used},
    traits::{Slab, SlabBlock, Unknown},
};

/// The arena's entry for one physical page.
///
/// The flags are the part anyone may read. The body is the part only the
/// page's owner may touch, buddy's links while the page is free, or its
/// slab descriptor while a slab owns it.
#[repr(C)]
pub struct Page {
    flags: BlockFlags,
    body: UnsafeCell<PageBody>,
}

/// What the owner of a page keeps in it.
#[repr(C)]
union PageBody {
    buddy: BuddyLinks<Regular>,
    slab: ManuallyDrop<SlabDescriptor<Unknown, Used>>,
}

impl Page {
    /// A page with `bits`, in no free list.
    pub const fn new(bits: BlockFlagBits) -> Page {
        Page {
            flags: BlockFlags::new(bits),
            body: UnsafeCell::new(PageBody {
                buddy: BuddyLinks::new(),
            }),
        }
    }
}

impl Debug for Page {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut ds = f.debug_struct("Page");
        ds.field("flags", &self.flags);
        if !self.flags.is_allocated() {
            // SAFETY: a free page's body holds its links.
            ds.field("links", unsafe { self.links().as_ref() });
        }
        ds.finish()
    }
}

impl BuddyBlock for Page {
    fn flags(&self) -> &BlockFlags { &self.flags }

    unsafe fn links(&self) -> NonNull<BuddyLinks<Regular>> {
        debug_assert!(
            !self.flags.is_allocated(),
            "only a free page has links"
        );
        // SAFETY: the body sits inside `self`, so its address isn't null.
        unsafe {
            NonNull::new_unchecked(&raw mut (*self.body.get()).buddy)
        }
    }

    fn from_links(links: NonNull<BuddyLinks<Regular>>) -> NonNull<Self> {
        // SAFETY: links live in the body of their page.
        unsafe { links.byte_sub(offset_of!(Page, body)) }.cast()
    }
}

impl SlabBlock for Page {
    unsafe fn slab_descriptor<T: Slab>(&self) -> &SlabDescriptor<T, Used> {
        // SAFETY: a descriptor's layout doesn't depend on `T`, and the
        // caller guarantees this page holds a slab of `T`.
        unsafe { &*(&raw const (*self.body.get()).slab).cast() }
    }

    unsafe fn slab_descriptor_mut<T: Slab>(
        &self,
    ) -> &mut SlabDescriptor<T, Used> {
        // SAFETY: a descriptor's layout doesn't depend on `T`, and the
        // caller owns this page as a slab of `T`.
        unsafe { &mut *(&raw mut (*self.body.get()).slab).cast() }
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
