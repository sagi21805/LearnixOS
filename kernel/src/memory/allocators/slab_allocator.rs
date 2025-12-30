use core::alloc::Layout;

pub struct SlabCache<T: 'static> {
    // TODO ADD LOCK
    pub layout: Layout,
    pub objects: &'static mut [T],
}
