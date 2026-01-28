use crate::memory::allocators::slab::traits::{Slab, SlabPosition};

#[derive(Default, Clone, Copy, Debug)]
pub struct Unassigned;

pub trait UnassignSlab {
    type Target;

    fn as_unassigned(&self) -> Self::Target;
}

pub trait AssignSlab {
    type Target<U: Slab>;

    fn assign<T: Slab>(&self) -> Self::Target<T>;
}
