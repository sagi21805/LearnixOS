use core::fmt::Debug;

use buddy::meta::{BuddyMeta, Regular};

pub union PageMeta {
    pub buddy: BuddyMeta<Regular>,
}

impl Debug for PageMeta {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PageMeta")
            .field("buddy", unsafe { &self.buddy })
            .finish()
    }
}
