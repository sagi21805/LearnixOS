use buddy::meta::{BuddyMeta, Regular};

pub union PageMeta {
    pub buddy: BuddyMeta<Regular>,
}
