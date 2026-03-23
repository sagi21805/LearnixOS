use crate::error::ConversionError;
use num_enum::{ConstIntoPrimitive, ConstTryFromPrimitive};
use strum::VariantArray;
use strum_macros::VariantArray;

pub const BUDDY_MAX_ORDER: usize = BuddyOrder::VARIANTS.len();

// TODO: understand how to put option<enum> in bitfield struct.
#[repr(u8)]
#[derive(
    VariantArray,
    Clone,
    Copy,
    PartialEq,
    Debug,
    Eq,
    ConstTryFromPrimitive,
    ConstIntoPrimitive,
)]
#[num_enum(error_type(name = ConversionError<u8>, constructor = ConversionError::CantConvertFrom))]
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
    None,
}

impl BuddyOrder {
    pub const MIN: BuddyOrder = *BuddyOrder::VARIANTS.first().unwrap();
    pub const MAX: BuddyOrder = *BuddyOrder::VARIANTS.last().unwrap();
}
