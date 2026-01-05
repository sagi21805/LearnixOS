use core::alloc::Layout;

#[derive(Debug)]
pub struct SlabCache<T: 'static> {
    // TODO ADD LOCK
    pub layout: Layout,
    pub objects: &'static mut [T],
}
