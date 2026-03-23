use core::{marker::PhantomData, ptr::NonNull};

use common::{
    address_types::PhysicalAddress, constants::REGULAR_PAGE_SIZE,
    enums::BuddyOrder,
};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum BuddyError {
    #[error("Cannot find a buddy for a block that is MAX_ORDER")]
    MaxOrder,
}

mod private {
    pub trait Seald {}
}

pub struct Dummy;
pub struct Real;

pub trait MetaState: private::Seald {
    type Prev<Block: Sized>;
}
impl private::Seald for Dummy {}
impl MetaState for Dummy {
    type Prev<Block: Sized> = ();
}
impl private::Seald for Real {}
impl MetaState for Real {
    type Prev<Block: Sized> = NonNull<Block>;
}

pub trait BuddyBlock: Sized {
    fn meta(&self) -> &BuddyMeta<Real>;

    fn meta_mut(&mut self) -> &mut BuddyMeta<Real>;

    fn from_meta(meta: NonNull<BuddyMeta<Real>>) -> NonNull<Self>;
}
// TODO: MOVE TO BUDDY META REAL
// fn is_allocated(&self) -> bool {
//     self.meta().next.is_none() && self.meta().prev.is_none()
// }
#[derive(Debug)]
pub struct BuddyMeta<State: MetaState> {
    pub next: Option<NonNull<BuddyMeta<Real>>>,
    pub prev: State::Prev<BuddyMeta<State>>,
    pub order: Option<BuddyOrder>,
    _state: PhantomData<State>,
}

impl const Default for BuddyMeta<Dummy> {
    fn default() -> Self {
        Self {
            next: None,
            prev: (),
            order: None,
            _state: PhantomData,
        }
    }
}

impl<State: MetaState> BuddyMeta<State> {
    pub fn attach(&mut self, mut p: NonNull<BuddyMeta<Real>>) {
        unsafe { p.as_mut().next = self.next };
        if let Some(mut next) = self.next {
            unsafe { next.as_mut().prev = p };
        }
        self.next = Some(p)
    }
}

impl BuddyMeta<Real> {
    /// Detaches self from the list.
    pub fn detach(&mut self) -> NonNull<BuddyMeta<Real>> {
        unsafe { self.prev.as_mut().next = self.next }

        if let Some(mut next) = self.next {
            unsafe { next.as_mut().prev = self.prev };
        }

        NonNull::from_mut(self)
    }
}

pub trait BuddyArena<Block: BuddyBlock> {
    fn iter(&self) -> impl Iterator<Item = NonNull<Block>>;

    fn buddy_of(
        &self,
        block: NonNull<Block>,
    ) -> Result<NonNull<Block>, BuddyError>;

    fn address_of(
        &self,
        block: NonNull<BuddyMeta<Real>>,
    ) -> PhysicalAddress;

    fn split(
        &self,
        block: NonNull<Block>,
    ) -> (NonNull<Block>, NonNull<Block>);

    fn merge(&self, block: NonNull<Block>, buddy: NonNull<Block>);
}
