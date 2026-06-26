use core::ptr::NonNull;

use common::{
    address_types::PhysicalAddress, enums::BuddyOrder, late_init::LateInit,
};

use macros::bitfields;
use thiserror::Error;
use x86::memory_map::MemoryMap;

#[derive(Debug, Error)]
pub enum BuddyError {
    #[error("Cannot find a buddy for a block that is MAX_ORDER")]
    MaxOrder,
}

mod private {
    pub trait Seald {}
}

/// The first node on the list.
#[derive(Copy, Clone)]
pub struct Head;
#[derive(Copy, Clone)]
/// A node on the list, that is not the first.
pub struct Regular;
/// A node that was on the list, but is no longer.
#[derive(Copy, Clone)]
pub struct Detached;

/// Intermidiate state represents a node that is in the list, but is not
/// known to be a head or a regular node.
#[derive(Copy, Clone)]
pub struct Intermidiate;

impl private::Seald for Intermidiate {}
impl MetaState for Intermidiate {
    type Next = Option<NonNull<BuddyMeta<Regular>>>;
    type Prev = ();
    type Flags = ();
}

pub trait MetaState: private::Seald {
    type Next: Sized;
    type Prev: Sized;
    type Flags: Sized;
}
impl private::Seald for Head {}
impl MetaState for Head {
    type Next = Option<NonNull<BuddyMeta<Regular>>>;
    type Prev = ();
    type Flags = ();
}
impl private::Seald for Regular {}
impl MetaState for Regular {
    type Next = Option<NonNull<BuddyMeta<Regular>>>;
    type Prev = NonNull<BuddyMeta<Head>>;
    type Flags = BuddyFlags;
}

impl private::Seald for Detached {}
impl MetaState for Detached {
    type Next = Option<NonNull<()>>;
    type Prev = Option<NonNull<()>>;
    type Flags = BuddyFlags;
}
pub trait BuddyBlock: Sized {
    fn meta(&self) -> &BuddyMeta<Regular>;

    fn meta_mut(&mut self) -> &mut BuddyMeta<Regular>;

    fn from_meta(meta: NonNull<BuddyMeta<Regular>>) -> NonNull<Self>;
}

#[bitfields]
pub struct BuddyFlags {
    #[flag(flag_type = BuddyOrder)]
    pub order: B7,
    pub allocated: B1,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct BuddyMeta<State: MetaState> {
    pub(crate) next: State::Next,
    pub(crate) prev: State::Prev,
    pub(crate) flags: State::Flags,
}

#[derive(Copy, Clone)]
pub union BuddyMetaType {
    pub regular: BuddyMeta<Regular>,
    pub intermediate: BuddyMeta<Intermidiate>,
    pub detached: BuddyMeta<Detached>,
    pub head: BuddyMeta<Head>,
}

impl<S> BuddyMeta<S>
where
    S: MetaState<Next = Option<NonNull<BuddyMeta<Regular>>>>,
{
    #[inline]
    pub fn attach(&mut self, mut p: NonNull<BuddyMeta<Regular>>) {
        unsafe { p.as_mut().next = self.next };
        if let Some(mut next) = self.next {
            unsafe { next.as_mut().prev = p.cast() };
        }
        self.next = Some(p.cast())
    }

    #[inline]
    pub fn attach_block<Block: BuddyBlock>(&mut self, p: NonNull<Block>) {
        self.attach(NonNull::from_ref(unsafe { p.as_ref().meta() }));
    }
}

impl From<BuddyMeta<Head>> for BuddyMeta<Intermidiate> {
    fn from(value: BuddyMeta<Head>) -> Self {
        Self {
            next: value.next,
            prev: (),
            flags: (),
        }
    }
}

impl Default for BuddyMeta<Head> {
    fn default() -> Self {
        Self {
            next: None,
            prev: (),
            flags: (),
        }
    }
}

impl BuddyMeta<Detached> {
    pub fn new(order: BuddyOrder) -> Self {
        Self {
            next: None,
            prev: None,
            flags: BuddyFlags::new().order(order).allocated(false),
        }
    }
}

impl BuddyMeta<Regular> {
    pub fn new(
        prev: NonNull<BuddyMeta<Regular>>,
        flags: BuddyFlags,
    ) -> BuddyMeta<Regular> {
        BuddyMeta {
            next: None,
            prev: prev.cast(),
            flags,
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

pub trait BuddyArena<Block: BuddyBlock>: Sized {
    // GENERATE ARUGMENTS
    fn init(uninit: &'static mut LateInit<Self>, mmap: MemoryMap);

    fn iter(&self) -> impl ExactSizeIterator<Item = NonNull<Block>>;

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
