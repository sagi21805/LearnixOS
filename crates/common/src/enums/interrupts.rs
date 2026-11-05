use crate::error::ConversionError;
use num_enum::{ConstIntoPrimitive, ConstTryFromPrimitive};

#[repr(u8)]
#[derive(Clone, Debug, Copy)]
pub enum InterruptStackTable {
    None = 0,
    IST1 = 1,
    IST2 = 2,
    IST3 = 3,
    IST4 = 4,
    IST5 = 5,
    IST6 = 6,
    IST7 = 7,
}

#[repr(u8)]
#[derive(
    Debug, Clone, Copy, ConstTryFromPrimitive, ConstIntoPrimitive,
)]
#[num_enum(error_type(name = ConversionError<u8>, constructor = ConversionError::CantConvertFrom))]
pub enum GateType {
    /// Clears `IF` on entry, so the handler runs with interrupts off.
    Interrupt = 0xe,
    /// Leaves `IF` as it was.
    Trap = 0xf,
}

mod page_fault {

    use crate::error::ConversionError;
    use bitfields::bitfields;
    use num_enum::ConstTryFromPrimitive;

    #[repr(u8)]
    #[derive(Debug, Clone, Copy, ConstTryFromPrimitive)]
    #[num_enum(error_type(name = ConversionError<u8>, constructor = ConversionError::CantConvertFrom))]
    pub enum P {
        NonPresentPage = 0,
        ProtectionViolation = 1,
    }

    #[repr(u8)]
    #[derive(Debug, Clone, Copy, ConstTryFromPrimitive)]
    #[num_enum(error_type(name = ConversionError<u8>, constructor = ConversionError::CantConvertFrom))]
    pub enum WR {
        ReadOperation = 0,
        WriteOperation = 1,
    }

    #[repr(u8)]
    #[derive(Debug, Clone, Copy, ConstTryFromPrimitive)]
    #[num_enum(error_type(name = ConversionError<u8>, constructor = ConversionError::CantConvertFrom))]
    pub enum US {
        SuperviserCauseFault = 0,
        UserCausedFault = 1,
    }

    #[repr(u8)]
    #[derive(Debug, Clone, Copy, ConstTryFromPrimitive)]
    #[num_enum(error_type(name = ConversionError<u8>, constructor = ConversionError::CantConvertFrom))]
    pub enum Reserved {
        NotReservedBit = 0,
        ReservedBitWasSet = 1,
    }

    #[repr(u8)]
    #[derive(Debug, Clone, Copy, ConstTryFromPrimitive)]
    #[num_enum(error_type(name = ConversionError<u8>, constructor = ConversionError::CantConvertFrom))]
    pub enum Fetch {
        NotCauseByInstructionFetch = 0,
        CauseByInstructionFetch = 1,
    }

    #[repr(u8)]
    #[derive(Debug, Clone, Copy, ConstTryFromPrimitive)]
    #[num_enum(error_type(name = ConversionError<u8>, constructor = ConversionError::CantConvertFrom))]
    pub enum PK {
        NotCauseByProtectionKey = 0,
        CauseByProtectionKey = 1,
    }

    #[repr(u8)]
    #[derive(Debug, Clone, Copy, ConstTryFromPrimitive)]
    #[num_enum(error_type(name = ConversionError<u8>, constructor = ConversionError::CantConvertFrom))]
    pub enum SS {
        NotShadowStackAccess = 0,
        CauseByShadowStackAccess = 1,
    }

    #[repr(u8)]
    #[derive(Debug, Clone, Copy, ConstTryFromPrimitive)]
    #[num_enum(error_type(name = ConversionError<u8>, constructor = ConversionError::CantConvertFrom))]
    pub enum Hlat {
        CauseDuringOrdinaryPagingAccess = 0,
        CausedDuringHLATPaging = 1,
    }

    #[repr(u8)]
    #[derive(Debug, Clone, Copy, ConstTryFromPrimitive)]
    #[num_enum(error_type(name = ConversionError<u8>, constructor = ConversionError::CantConvertFrom))]
    pub enum Sgx {
        NotRelatedToSgx = 0,
        RelatedToS = 1,
    }

    #[bitfields]
    struct PageFaultError {
        #[flag(r, flag_type = P)]
        p: B1,
        #[flag(r, flag_type = WR)]
        wr: B1,
        #[flag(r, flag_type = US)]
        us: B1,
        #[flag(r, flag_type = Reserved)]
        rsvd: B1,
        #[flag(r, flag_type = Fetch)]
        fetch: B1,
        #[flag(r, flag_type = PK)]
        pk: B1,
        #[flag(r, flag_type = SS)]
        ss: B1,
        #[flag(r, flag_type = Hlat)]
        hlat: B1,
        #[flag(r, flag_type = Sgx)]
        sgx: B1,
    }
}
