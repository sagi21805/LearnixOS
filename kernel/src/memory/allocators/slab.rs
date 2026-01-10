use core::{fmt::Debug, mem::ManuallyDrop};

/// Preallocated object in the slab allocator.
///
/// When a slab is initialized, each position will include the index of the
/// next free object, when the object is allocated this index will be
/// overwrite by the objects data thus wasting no space on the freelist.
pub union PreallocatedObject<T: 'static> {
    allocated: ManuallyDrop<T>,
    next_free: usize,
}

impl<T> Debug for PreallocatedObject<T> {
    fn fmt(&self, _f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Ok(())
    }
}

#[derive(Debug)]
pub struct SlabDescriptor<T: 'static> {
    /// The index in the objects array of the next free objet
    pub next_free: usize,
    pub objects: &'static mut [PreallocatedObject<T>],
    pub next: Option<&'static mut SlabDescriptor<T>>,
}

#[derive(Debug)]
pub struct SlabCache<T: 'static> {
    // TODO ADD LOCK
    pub buddy_order: usize,
    pub partial: SlabDescriptor<T>,
    pub full: SlabDescriptor<T>,
    pub free: SlabDescriptor<T>,
}
