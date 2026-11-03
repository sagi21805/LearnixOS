use core::ptr::NonNull;

use bitfields::bitfields;
use common::page_index::PageIndex;

use crate::{
    descriptor::{SlabDescriptor, SlabState},
    slab_objects::SlabObjects,
    traits::Slab,
};

/// The arena that holds every slab's descriptor, which finds a slab's
/// pages, flags and neighbours from its descriptor or its index.
pub trait SlabArena {
    /// The slots of the slab whose descriptor is `descriptor`.
    fn objects_of<T: Slab, S: SlabState>(
        &self,
        descriptor: &SlabDescriptor<T, S>,
    ) -> SlabObjects<T>;

    /// The index of the page that holds `descriptor`.
    fn index_of<T: Slab, S: SlabState>(
        &self,
        descriptor: &SlabDescriptor<T, S>,
    ) -> PageIndex;

    /// The descriptor in the page at `index`.
    ///
    /// # Safety
    ///
    /// A live descriptor of `T` in state `S` must sit at `index`.
    unsafe fn descriptor_at<T: Slab, S: SlabState>(
        &self,
        index: PageIndex,
    ) -> NonNull<SlabDescriptor<T, S>>;

    /// The list bits of the slab whose descriptor is `descriptor`.
    fn slab_flags_of<T: Slab, S: SlabState>(
        &self,
        descriptor: &SlabDescriptor<T, S>,
    ) -> SlabFlags;

    /// Replaces the list bits of the slab whose descriptor is
    /// `descriptor`. The `&mut` descriptor shows the caller manages the
    /// slab, so it owns the slab's page.
    fn set_slab_flags_of<T: Slab, S: SlabState>(
        &self,
        descriptor: &mut SlabDescriptor<T, S>,
        flags: SlabFlags,
    );
}

/// A slab's list bits, in the owner byte of its page's flags.
#[bitfields]
pub struct SlabFlags {
    /// Whether the slab is in the partial list.
    pub partial: B1,
    /// Whether the slab is the first node of the partial list.
    pub head: B1,
    #[flag(r)]
    pub reserved: B6,
}
