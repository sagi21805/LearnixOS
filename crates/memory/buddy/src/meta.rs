use core::{fmt::Debug, ptr::NonNull};

use common::{
    address_types::PhysicalAddress, enums::BuddyOrder, volatile::Volatile,
};

use macros::bitfields;
use thiserror::Error;
use x86::memory_map::MemoryMap;

#[derive(Debug, Error)]
pub enum BuddyError {
    #[error("Cannot find a buddy for a block that is BuddyOrder::MAX")]
    MaxOrder,
    #[error("Page is part of a larger order")]
    PageInLargerOrder,
    #[error("Cannot split a block that is BuddyOrder::MIN")]
    Unsplitable,
    #[error("The buddy of this block is outside of memory range")]
    BuddyOutOfRange,
    #[error("Page is not part of the arena")]
    PageNotInArena,
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

pub const trait BuddyBlock: Sized {
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
    pub next: Volatile<State::Next>,
    pub prev: Volatile<State::Prev>,
    pub flags: State::Flags,
}

#[derive(Copy, Clone)]
pub union BuddyMetaType {
    pub regular: BuddyMeta<Regular>,
    pub head: BuddyMeta<Head>,
}

impl<S> BuddyMeta<S>
where
    S: MetaState<Next = Option<NonNull<BuddyMeta<Regular>>>>,
{
    #[inline]
    pub fn attach(&mut self, mut p: NonNull<BuddyMeta<Regular>>) {
        unsafe { p.as_mut().next = self.next };
        if let Some(mut next) = self.next.read() {
            unsafe { next.as_mut().prev = Volatile::new(p.cast()) };
        }
        self.next.write(Some(p.cast()));
        unsafe { p.as_mut().prev.write(NonNull::from_mut(self).cast()) };
    }

    #[inline]
    pub fn attach_block<Block: const BuddyBlock>(
        &mut self,
        mut p: NonNull<Block>,
    ) {
        self.attach(NonNull::from_mut(unsafe { p.as_mut().meta_mut() }));
    }
}

impl Default for BuddyMeta<Head> {
    fn default() -> Self {
        Self {
            next: Volatile::new(None),
            prev: Volatile::new(()),
            flags: (),
        }
    }
}

impl BuddyMeta<Regular> {
    pub fn new<
        S: MetaState<Next = Option<NonNull<BuddyMeta<Regular>>>>,
    >(
        prev: NonNull<BuddyMeta<S>>,
        flags: BuddyFlags,
    ) -> BuddyMeta<Regular> {
        BuddyMeta {
            next: Volatile::new(None),
            prev: Volatile::new(prev.cast()),
            flags,
        }
    }

    /// Detaches self from the list.
    pub fn detach(&mut self) -> NonNull<BuddyMeta<Regular>> {
        unsafe { self.prev.read().as_mut().next.write(self.next.read()) }

        if let Some(mut next) = self.next.read() {
            unsafe { next.as_mut().prev.write(self.prev.read()) };
        }

        NonNull::from_mut(self)
    }
}

impl<S> ::core::fmt::Debug for BuddyMeta<S>
where
    S: MetaState<Next: Debug, Prev: Debug, Flags: Debug>,
{
    default fn fmt(
        &self,
        f: &mut core::fmt::Formatter<'_>,
    ) -> core::fmt::Result {
        let mut ds = f.debug_struct("BuddyMeta");

        ds.field("next", &self.next)
            .field("prev", &self.prev)
            .field("flags", &self.flags);

        #[cfg(debug_assertions)]
        ds.field("self_ptr", &NonNull::from_ref(self));

        ds.finish()
    }
}

impl ::core::fmt::Debug for BuddyMeta<Head> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut list_fmt = f.debug_list();

        let mut next = self.next;
        let mut i = 0;
        while let Some(n) = next.read() {
            list_fmt.entry(unsafe { n.as_ref() });
            next = unsafe { n.as_ref().next };
            if i == 32 {
                return list_fmt.finish();
            }
            i += 1;
        }

        list_fmt.finish()
    }
}

pub trait BuddyArena<Block: BuddyBlock>: Sized {
    fn new(mmap: &MemoryMap, head: &mut BuddyMeta<Head>) -> Self;

    /// Returns an iterator over all blocks in this arena.
    fn iter(&self) -> impl ExactSizeIterator<Item = NonNull<Block>>;

    /// Returns the buddy of a block.
    fn buddy_of(
        &self,
        block: NonNull<Block>,
    ) -> Result<NonNull<Block>, BuddyError>;

    /// Returns the physical allocated address by this block.
    ///
    /// The first block of Order1 for example allocates 0..4096
    fn address_of(&self, block: NonNull<Block>) -> PhysicalAddress;

    /// Return the corresponding block in the arena for a given physical
    /// address, if one exists.
    fn page_with_address(
        &self,
        address: PhysicalAddress,
    ) -> Result<NonNull<Block>, BuddyError>;

    /// Split a block into two smaller blocks of previous order.
    fn split(
        &self,
        block: NonNull<Block>,
    ) -> Result<(NonNull<Block>, NonNull<Block>), BuddyError>;

    /// Merge two blocks with the same order to a next order block.
    ///
    /// The two blocks must be buddies, have the same order, and be free.
    fn merge(
        &self,
        block: NonNull<Block>,
        buddy: NonNull<Block>,
    ) -> Result<NonNull<Block>, BuddyError>;

    // /// Detach a block from the middle of the arena, returning the
    // detached /// block.
    // fn detach_mid(&self, block: NonNull<Block>) -> NonNull<Block>;

    /// Returns the block nth block of the arena, if one exists.
    fn at(&self, n: usize) -> Option<NonNull<Block>>;

    /// Returns the (section index, section offset) of the given page.
    ///
    /// A section is a contiguous range of pages aligned to
    /// [`BuddyOrder::MAX`].
    ///
    /// # Safety
    ///
    /// The page must be contained within the arena's memory range.
    unsafe fn section_index_of(
        &self,
        block: NonNull<Block>,
    ) -> (usize, usize);
}

unsafe impl Send for BuddyMeta<Regular> {}
unsafe impl Send for BuddyMeta<Head> {}
