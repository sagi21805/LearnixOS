mod arena;
mod list;
mod state;

pub use arena::*;
pub use state::*;

use core::{fmt::Debug, ptr::NonNull};

use bitfields::bitfields;
use common::enums::BuddyOrder;

#[bitfields]
pub struct BuddyFlags {
    #[flag(flag_type = BuddyOrder)]
    pub order: B7,
    pub allocated: B1,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct BuddyMeta<State: MetaState> {
    pub next: State::Next,
    pub prev: State::Prev,
    pub flags: State::Flags,
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

impl BuddyMeta<Regular> {
    pub fn new<
        S: MetaState<Next = Option<NonNull<BuddyMeta<Regular>>>>,
    >(
        prev: NonNull<BuddyMeta<S>>,
        flags: BuddyFlags,
    ) -> BuddyMeta<Regular> {
        BuddyMeta {
            next: None,
            prev: prev.cast(),
            flags,
        }
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
        while let Some(n) = next {
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

unsafe impl Send for BuddyMeta<Regular> {}
unsafe impl Send for BuddyMeta<Head> {}
