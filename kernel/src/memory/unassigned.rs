use crate::memory::allocators::slab::traits::SlabPosition;

#[derive(Default, Clone, Copy, Debug)]
pub struct Unassigned;

pub trait UnassignSlab {
    type Target;

    fn as_unassigned(&self) -> Self::Target;
}

pub trait AssignSlab {
    type Target<U: SlabPosition>;

    fn assign<T: SlabPosition>(&self) -> Self::Target<T>;
}
