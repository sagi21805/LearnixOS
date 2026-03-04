#[repr(u8)]
pub enum Interrupt {
    DivisionError = 0x0,
    Debug = 0x1,
    NonMaskableInterrupt = 0x2,
    Breakpoint = 0x3,
    Overflow = 0x4,
    BoundRangeExceeded = 0x5,
    InvalidOpcode = 0x6,
    DeviceNotFound = 0x7,
    DoubleFault = 0x8,
    CoprocessorSegmentOverrun = 0x9,
    InvalidTSS = 0xa,
    SegmentNotPresent = 0xb,
    StackSegmentFault = 0xc,
    GeneralProtection = 0xd,
    PageFault = 0xe,
    IntelReserved = 0xf,
    FloatingPointError = 0x10,
    AlignmentCheck = 0x11,
    MachineCheck = 0x12,
    SIMD = 0x13,
    Virtualization = 0x14,
    ControlProtection = 0x15,
    // Interrupts until 0x1f are reserved by Intel.
    Timer = 0x20,
    Keyboard = 0x21,
    Ahci = 0x2a,
}
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
pub enum InterruptType {
    Fault = 0xe,
    Trap = 0xf,
}

mod page_fault {

    use crate::error::ConversionError;
    use macros::bitfields;
    use num_enum::TryFromPrimitive;

    #[repr(u8)]
    #[derive(Debug, Clone, Copy, TryFromPrimitive)]
    #[num_enum(error_type(name = ConversionError<u8>, constructor = ConversionError::CantConvertFrom))]
    pub enum P {
        NonPresentPage = 0,
        ProtectionViolation = 1,
    }

    #[repr(u8)]
    #[derive(Debug, Clone, Copy, TryFromPrimitive)]
    #[num_enum(error_type(name = ConversionError<u8>, constructor = ConversionError::CantConvertFrom))]
    pub enum WR {
        ReadOperation = 0,
        WriteOperation = 1,
    }

    #[repr(u8)]
    #[derive(Debug, Clone, Copy, TryFromPrimitive)]
    #[num_enum(error_type(name = ConversionError<u8>, constructor = ConversionError::CantConvertFrom))]
    pub enum US {
        SuperviserCauseFault = 0,
        UserCausedFault = 1,
    }

    #[repr(u8)]
    #[derive(Debug, Clone, Copy, TryFromPrimitive)]
    #[num_enum(error_type(name = ConversionError<u8>, constructor = ConversionError::CantConvertFrom))]
    pub enum Reserved {
        NotReservedBit = 0,
        ReservedBitWasSet = 1,
    }

    #[repr(u8)]
    #[derive(Debug, Clone, Copy, TryFromPrimitive)]
    #[num_enum(error_type(name = ConversionError<u8>, constructor = ConversionError::CantConvertFrom))]
    pub enum Fetch {
        NotCauseByInstructionFetch = 0,
        CauseByInstructionFetch = 1,
    }

    #[repr(u8)]
    #[derive(Debug, Clone, Copy, TryFromPrimitive)]
    #[num_enum(error_type(name = ConversionError<u8>, constructor = ConversionError::CantConvertFrom))]
    pub enum PK {
        NotCauseByProtectionKey = 0,
        CauseByProtectionKey = 1,
    }

    #[repr(u8)]
    #[derive(Debug, Clone, Copy, TryFromPrimitive)]
    #[num_enum(error_type(name = ConversionError<u8>, constructor = ConversionError::CantConvertFrom))]
    pub enum SS {
        NotShadowStackAccess = 0,
        CauseByShadowStackAccess = 1,
    }

    #[repr(u8)]
    #[derive(Debug, Clone, Copy, TryFromPrimitive)]
    #[num_enum(error_type(name = ConversionError<u8>, constructor = ConversionError::CantConvertFrom))]
    pub enum Hlat {
        CauseDuringOrdinaryPagingAccess = 0,
        CausedDuringHLATPaging = 1,
    }

    #[repr(u8)]
    #[derive(Debug, Clone, Copy, TryFromPrimitive)]
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
