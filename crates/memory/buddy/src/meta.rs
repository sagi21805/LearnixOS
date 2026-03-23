use core::{marker::PhantomData, ptr::NonNull};

use common::{
    address_types::PhysicalAddress, constants::REGULAR_PAGE_SIZE,
    enums::BuddyOrder,
};

use macros::bitfields;
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
    type Flags: Sized;
}
impl private::Seald for Dummy {}
impl MetaState for Dummy {
    type Prev<Block: Sized> = ();
    type Flags = ();
}
impl private::Seald for Real {}
impl MetaState for Real {
    type Prev<Block: Sized> = NonNull<Block>;
    type Flags = BuddyFlags;
}

pub trait BuddyBlock: Sized {
    fn meta(&self) -> &BuddyMeta<Real>;

    fn meta_mut(&mut self) -> &mut BuddyMeta<Real>;

    fn from_meta(meta: NonNull<BuddyMeta<Real>>) -> NonNull<Self>;
}

#[bitfields]
pub struct BuddyFlags {
    #[flag(flag_type = BuddyOrder)]
    pub order: B8,
    pub allocated: B1,
}

//
#[derive(Debug)]
pub struct BuddyMeta<State: MetaState> {
    pub(crate) next: Option<NonNull<BuddyMeta<Real>>>,
    pub(crate) prev: State::Prev<BuddyMeta<State>>,
    pub(crate) flags: State::Flags,
    _state: PhantomData<State>,
}

impl const Default for BuddyMeta<Dummy> {
    fn default() -> Self {
        Self {
            next: None,
            prev: (),
            flags: (),
            _state: PhantomData,
        }
    }
}

impl BuddyMeta<Real> {
    fn new(prev: NonNull<BuddyMeta<Real>>) -> Self {
        Self {
            next: None,
            prev,
            flags: BuddyFlags::default().order(BuddyOrder::None),
            _state: PhantomData,
        }
    }
}

impl<State: MetaState> BuddyMeta<State> {
    #[inline]
    pub fn attach(&mut self, mut p: NonNull<BuddyMeta<Real>>) {
        unsafe { p.as_mut().next = self.next };
        if let Some(mut next) = self.next {
            unsafe { next.as_mut().prev = p };
        }
        self.next = Some(p)
    }

    #[inline]
    pub fn attach_block<Block: BuddyBlock>(&mut self, p: NonNull<Block>) {
        self.attach(NonNull::from_ref(unsafe { p.as_ref().meta() }));
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

    fn address_of(&self, block: NonNull<Block>) -> PhysicalAddress;

    fn split(
        &self,
        block: NonNull<Block>,
    ) -> (NonNull<Block>, NonNull<Block>);

    fn merge(
        &self,
        block: NonNull<Block>,
        buddy: NonNull<Block>,
    ) -> NonNull<Block>;
}
