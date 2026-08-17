use macros::bitfields;

use crate::descriptor::SlabAddress;

#[bitfields]
#[derive(PartialEq, Eq)]
pub struct PartialMeta {
    pub next_free_idx: B16,
    pub total_allocated: B31,
    pub partial: B1,
}

const impl Default for PartialMeta {
    fn default() -> Self {
        PartialMeta::new()
            .next_free_idx(0)
            .total_allocated(0)
            .partial(true)
    }
}

#[bitfields]
pub struct FullFreeMeta {
    #[flag(flag_type = SlabAddress)]
    pub prev: B48,
    #[flag(r)]
    pub reserved: B15,
    pub partial: B1,
}

const impl Default for FullFreeMeta {
    fn default() -> Self {
        Self::new().prev(SlabAddress::default()).partial(false)
    }
}

#[bitfields]
pub struct RawMeta {
    #[flag(r)]
    pub reserved: B63,
    pub partial: B1,
}
