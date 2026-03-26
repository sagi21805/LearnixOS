use core::{marker::PhantomData, ptr::NonNull};

use common::{
    address_types::PhysicalAddress,
    constants::REGULAR_PAGE_SIZE,
    enums::{BUDDY_MAX_ORDER, BuddyOrder},
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

pub struct Head;
pub struct Regular;
pub struct Detached;
pub struct Unknown;

pub trait MetaState: private::Seald {
    type Prev<Block: Sized>;
    type Flags: Sized;
}
impl private::Seald for Head {}
impl MetaState for Head {
    type Prev<Block: Sized> = ();
    type Flags = ();
}
impl private::Seald for Regular {}
impl MetaState for Regular {
    type Prev<Block: Sized> = NonNull<Block>;
    type Flags = BuddyFlags;
}

impl private::Seald for Detached {}
impl MetaState for Detached {
    type Prev<Block: Sized> = Option<NonNull<Block>>;
    type Flags = BuddyFlags;
}
impl private::Seald for Unknown {}
impl MetaState for Unknown {
    type Flags = ();
    type Prev<Block: Sized> = ();
}

pub trait BuddyBlock: Sized {
    fn meta(&self) -> &BuddyMeta<Regular>;

    fn meta_mut(&mut self) -> &mut BuddyMeta<Regular>;

    fn from_meta(meta: NonNull<BuddyMeta<Regular>>) -> NonNull<Self>;
}

#[bitfields]
pub struct BuddyFlags {
    #[flag(flag_type = BuddyOrder)]
    pub order: B8,
    pub allocated: B1,
}

#[derive(Debug)]
pub struct BuddyMeta<SelfState: MetaState> {
    pub(crate) next: Option<NonNull<BuddyMeta<Regular>>>,
    pub(crate) prev: State::Prev<BuddyMeta<Unknown>>,
    pub(crate) flags: State::Flags,
    _state: PhantomData<SelfState>,
}

pub union BuddyMetaNode {
    pub regular: BuddyMeta<Regular>,
    pub detached: BuddyMeta<Detached>,
}

impl<PrevState: MetaState> const Default for BuddyMeta<Head, PrevState> {
    fn default() -> Self {
        Self {
            next: None,
            prev: (),
            flags: (),
            _state: PhantomData,
        }
    }
}

impl BuddyMeta<Regular> {
    fn new(prev: NonNull<BuddyMeta<Regular>>) -> Self {
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
    pub fn attach(&mut self, mut p: NonNull<BuddyMeta<Regular>>) {
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

impl BuddyMeta<Regular> {
    pub fn new(
        prev: NonNull<BuddyMeta<Regular>>,
        flags: BuddyFlags,
    ) -> Self {
        Self {
            next: None,
            prev,
            flags: BuddyFlags::default().order(BuddyOrder::None),
            _state: PhantomData,
        }
    }

    /// Detaches self from the list.
    pub fn detach(&mut self) -> NonNull<BuddyMeta<Regular>> {
        unsafe { self.prev.as_mut().next = self.next }

        if let Some(mut next) = self.next {
            unsafe { next.as_mut().prev = self.prev };
        }

        NonNull::from_mut(self)
    }
}

pub trait BuddyArena<Block: BuddyBlock> {
    // GENERATE ARUGMENTS
    pub fn init() -> (NonNull<Self>, [BuddyMeta<Head>; BUDDY_MAX_ORDER]);

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
