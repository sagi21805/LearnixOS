//! The states of a [`BuddyMeta`].

use core::ptr::NonNull;

use crate::meta::{BuddyFlags, BuddyMeta};

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
