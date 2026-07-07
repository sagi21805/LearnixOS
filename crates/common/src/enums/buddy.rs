use num_enum::{ConstFromPrimitive, ConstIntoPrimitive};
use strum_macros::VariantArray;

#[repr(u8)]
#[derive(
    VariantArray,
    Clone,
    Copy,
    PartialEq,
    Debug,
    Eq,
    PartialOrd,
    Ord,
    ConstIntoPrimitive,
    ConstFromPrimitive,
)]
pub enum BuddyOrder {
    Order0 = 0,
    Order1 = 1,
    Order2 = 2,
    Order3 = 3,
    Order4 = 4,
    Order5 = 5,
    Order6 = 6,
    Order7 = 7,
    Order8 = 8,
    Order9 = 9,
    Order10 = 10,
    #[default]
    None,
}

impl BuddyOrder {
    pub const MIN: BuddyOrder = BuddyOrder::Order0;
    pub const MAX: BuddyOrder = BuddyOrder::Order10;

    pub fn next(self) -> Option<BuddyOrder> {
        match self {
            BuddyOrder::None => unreachable!(),
            BuddyOrder::MAX => None,
            _ => BuddyOrder::try_from(self as u8 + 1).ok(),
        }
    }

    pub fn prev(self) -> Option<BuddyOrder> {
        match self {
            BuddyOrder::None => unreachable!(),
            BuddyOrder::MIN => None,
            _ => BuddyOrder::try_from(self as u8 - 1).ok(),
        }
    }
}
