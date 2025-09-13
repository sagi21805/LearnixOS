use crate::error::PciConfigurationError;

#[repr(u16)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum VendorID {
    Intel,
    VirtIO,
    AMD,
    Nvidia,
    Realtek,
    QEMU,
    RedHat,
    Broadcom,
    VMware,
    NonExistent,
}

impl VendorID {
    pub fn from_u16(vendor: u16) -> Result<Self, PciConfigurationError> {
        match vendor {
            0x8086 => Ok(Self::Intel),
            0x1AF4 => Ok(Self::VirtIO),
            0x1022 => Ok(Self::AMD),
            0x10DE => Ok(Self::Nvidia),
            0x10EC => Ok(Self::Realtek),
            0x1234 => Ok(Self::QEMU),
            0x1B36 => Ok(Self::RedHat),
            0x14E4 => Ok(Self::Broadcom),
            0x15AD => Ok(Self::VMware),
            0xFFFF => Ok(Self::NonExistent),
            _ => Err(PciConfigurationError::UnknownVendor(vendor)),
        }
    }
    pub fn to_u16(self) -> u16 {
        match self {
            VendorID::Intel => 0x8086,
            VendorID::VirtIO => 0x1AF4,
            VendorID::AMD => 0x1022,
            VendorID::Nvidia => 0x10DE,
            VendorID::Realtek => 0x10EC,
            VendorID::QEMU => 0x1234,
            VendorID::RedHat => 0x1B36,
            VendorID::Broadcom => 0x14E4,
            VendorID::VMware => 0x15AD,
            VendorID::NonExistent => 0xFFFF,
        }
    }
}
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DeviceID {
    // Intel 440FX / PIIX3 chipset devices
    Intel440FXHostBridge,
    IntelPIIX3ISA,
    IntelPIIX3IDE,
    IntelPIIX3USB,
    IntelPIIX3ACPI,
    IntelEthernetController,

    // Intel Q35 qemu chipset
    IntelExpressDramController,
    IntelNetworkController,
    LPCInterfaceController,

    // USB Controllers
    USB2EhciController,

    // VirtIO devices
    VirtIONet,
    VirtIOBlock,
    VirtIOConsole,
    VirtIOBalloon,

    // QEMU VGA / Graphics
    QEMUStandardVGA,
    QXLGraphics,

    // Realtek
    RTL8139NIC,

    // NVIDIA
    NVIDIAGPU,

    NonExistent,
}

impl DeviceID {
    pub fn from_vendor_dev_id(
        vendor: VendorID,
        device: u16,
    ) -> Result<Self, PciConfigurationError> {
        match vendor {
            // Intel
            VendorID::Intel => match device {
                0x1237 => Ok(Self::Intel440FXHostBridge),
                0x7000 => Ok(Self::IntelPIIX3ISA),
                0x7010 => Ok(Self::IntelPIIX3IDE),
                0x7020 => Ok(Self::IntelPIIX3USB),
                0x7113 => Ok(Self::IntelPIIX3ACPI),
                0x29C0 => Ok(Self::IntelExpressDramController),
                0x100E => Ok(Self::IntelEthernetController),
                0x10D3 => Ok(Self::IntelNetworkController),
                0x2918 => Ok(Self::LPCInterfaceController),
                0x24CD => Ok(Self::USB2EhciController),
                _ => Err(PciConfigurationError::UnknownDevice(device as u16)),
            },

            // VirtIO
            VendorID::VirtIO => match device {
                0x1000 => Ok(Self::VirtIONet),
                0x1001 => Ok(Self::VirtIOBlock),
                0x1002 => Ok(Self::VirtIOBalloon),
                0x1003 => Ok(Self::VirtIOConsole),
                _ => Err(PciConfigurationError::UnknownDevice(device as u16)),
            },

            // // QEMU graphics
            VendorID::QEMU => match device {
                0x1111 => Ok(Self::QEMUStandardVGA),
                _ => Err(PciConfigurationError::UnknownDevice(device as u16)),
            },
            VendorID::RedHat => match device {
                0x0100 => Ok(Self::QXLGraphics),
                _ => Err(PciConfigurationError::UnknownDevice(device as u16)),
            },

            // // Realtek
            VendorID::Realtek => match device {
                0x8139 => Ok(Self::RTL8139NIC),
                _ => Err(PciConfigurationError::UnknownDevice(device as u16)),
            },
            // // NVIDIA
            VendorID::Nvidia => match device {
                0x1C82 => Ok(Self::NVIDIAGPU),
                _ => Err(PciConfigurationError::UnknownDevice(device as u16)),
            },
            VendorID::NonExistent => Ok(Self::NonExistent),
            _ => Err(PciConfigurationError::UnknownDevice(device)),
        }
    }

    pub fn to_u16(self) -> u16 {
        match self {
            // Intel 440FX + PIIX3 chipset
            Self::Intel440FXHostBridge => 0x1237,
            Self::IntelPIIX3ISA => 0x7000,
            Self::IntelPIIX3IDE => 0x7010,
            Self::IntelPIIX3USB => 0x7020,
            Self::IntelPIIX3ACPI => 0x7113,
            Self::IntelEthernetController => 0x100E,
            Self::IntelExpressDramController => 0x29C0,
            Self::IntelNetworkController => 0x10D3,
            Self::LPCInterfaceController => 0x2918,
            Self::USB2EhciController => 0x24CD,

            // VirtIO devices
            Self::VirtIONet => 0x1000,
            Self::VirtIOBlock => 0x1001,
            Self::VirtIOBalloon => 0x1002,
            Self::VirtIOConsole => 0x1003,

            // QEMU / RedHat graphics
            Self::QEMUStandardVGA => 0x1111,
            Self::QXLGraphics => 0x0100,

            // Realtek
            Self::RTL8139NIC => 0x8139,

            // NVIDIA
            Self::NVIDIAGPU => 0x1C82,

            // Special case: nonexistent
            Self::NonExistent => 0xFFFF,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClassCode {
    Unclassified,
    MassStorageController,
    NetworkController,
    DisplayController,
    MultimediaController,
    MemoryController,
    Bridge,
    SimpleCommunicationController,
    BaseSystemPeripheral,
    InputDeviceController,
    DockingStation,
    Processor,
    SerialBusController,
    WirelessController,
    IntelligentController,
    SatelliteCommunicationController,
    EncryptionController,
    SignalProcessingController,
    ProcessingAccel,
    NonEssentialInstrumentation,
    Coprocessor,
    Unassigned,
}

impl ClassCode {
    pub fn from_u8(class: u8) -> Result<Self, PciConfigurationError> {
        match class {
            0x00 => Ok(Self::Unclassified),
            0x01 => Ok(Self::MassStorageController),
            0x02 => Ok(Self::NetworkController),
            0x03 => Ok(Self::DisplayController),
            0x04 => Ok(Self::MultimediaController),
            0x05 => Ok(Self::MemoryController),
            0x06 => Ok(Self::Bridge),
            0x07 => Ok(Self::SimpleCommunicationController),
            0x08 => Ok(Self::BaseSystemPeripheral),
            0x09 => Ok(Self::InputDeviceController),
            0x0A => Ok(Self::DockingStation),
            0x0B => Ok(Self::Processor),
            0x0C => Ok(Self::SerialBusController),
            0x0D => Ok(Self::WirelessController),
            0x0E => Ok(Self::IntelligentController),
            0x0F => Ok(Self::SatelliteCommunicationController),
            0x10 => Ok(Self::EncryptionController),
            0x11 => Ok(Self::SignalProcessingController),
            0x12 => Ok(Self::ProcessingAccel),
            0x13 => Ok(Self::NonEssentialInstrumentation),
            0x40 => Ok(Self::Coprocessor),
            0xFF => Ok(Self::Unassigned),
            _ => Err(PciConfigurationError::UnknownClassCode(class as u16)),
        }
    }

    pub fn to_u8(self) -> u8 {
        match self {
            Self::Unclassified => 0x00,
            Self::MassStorageController => 0x01,
            Self::NetworkController => 0x02,
            Self::DisplayController => 0x03,
            Self::MultimediaController => 0x04,
            Self::MemoryController => 0x05,
            Self::Bridge => 0x06,
            Self::SimpleCommunicationController => 0x07,
            Self::BaseSystemPeripheral => 0x08,
            Self::InputDeviceController => 0x09,
            Self::DockingStation => 0x0A,
            Self::Processor => 0x0B,
            Self::SerialBusController => 0x0C,
            Self::WirelessController => 0x0D,
            Self::IntelligentController => 0x0E,
            Self::SatelliteCommunicationController => 0x0F,
            Self::EncryptionController => 0x10,
            Self::SignalProcessingController => 0x11,
            Self::ProcessingAccel => 0x12,
            Self::NonEssentialInstrumentation => 0x13,
            Self::Coprocessor => 0x40,
            Self::Unassigned => 0xFF,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubClass {
    // 0x00: Unclassified
    NonVGAUnclassified,        // 0x00
    VGACompatibleUnclassified, // 0x01

    // 0x01: Mass Storage Controller
    SCSIController,   // 0x00
    IDEController,    // 0x01
    FloppyController, // 0x02
    IPIBusController, // 0x03
    RAIDController,   // 0x04
    ATAController,    // 0x05
    SATAController,   // 0x06
    SASController,    // 0x07
    NVMController,    // 0x08
    MassStorageOther, // 0x80

    // 0x02: Network Controller
    Ethernet,                    // 0x00
    TokenRing,                   // 0x01
    FDDI,                        // 0x02
    ATM,                         // 0x03
    ISDN,                        // 0x04
    WorldFip,                    // 0x05
    PICMGMultiComputing,         // 0x06
    InfinibandNetworkController, // 0x07
    Fabric,                      // 0x08
    NetworkOther,                // 0x80

    // 0x03: Display Controller
    VGAController,    // 0x00
    XGAController,    // 0x01
    ThreeDController, // 0x02
    DisplayOther,     // 0x80

    // 0x04: Multimedia Controller
    Video,             // 0x00
    Audio,             // 0x01
    ComputerTelephony, // 0x02
    HdAudio,           // 0x03
    MultimediaOther,   // 0x80

    // 0x05: Memory Controller
    RAMController,   // 0x00
    FlashController, // 0x01
    MemoryOther,     // 0x80

    // 0x06: Bridge Device
    HostBridge,         // 0x00
    ISABridge,          // 0x01
    EISABridge,         // 0x02
    MCABridge,          // 0x03
    PCItoPCIBridge,     // 0x04
    PCMCIABridge,       // 0x05
    NuBusBridge,        // 0x06
    CardBusBridge,      // 0x07
    RACEwayBridge,      // 0x08
    PCItoPCIBridgeSemi, // 0x09
    InfinibandBridge,   // 0x0A
    BridgeOther,        // 0x80

    // 0x07: Simple Communication
    SerialController,   // 0x00
    ParallelController, // 0x01
    MultiportSerial,    // 0x02
    Modem,              // 0x03
    GPIB,               // 0x04
    SmartCard,          // 0x05
    CommOther,          // 0x80

    // 0x08: Base System Peripherals
    PIC,             // 0x00
    DMAController,   // 0x01
    Timer,           // 0x02
    RTC,             // 0x03
    PCIHotplug,      // 0x04
    BaseSystemOther, // 0x80

    // 0x09: Input Devices
    KeyboardController, // 0x00
    Digitizer,          // 0x01
    MouseController,    // 0x02
    ScannerController,  // 0x03
    GameportController, // 0x04
    InputOther,         // 0x80

    // 0x0A: Docking Station
    GenericDocking, // 0x00
    DockingOther,   // 0x80

    // 0x0B: Processor
    Processor386,     // 0x00
    Processor486,     // 0x01
    ProcessorPentium, // 0x02
    ProcessorAlpha,   // 0x10
    ProcessorPowerPC, // 0x20
    ProcessorMIPS,    // 0x30
    ProcessorCoProc,  // 0x40
    ProcessorOther,   // 0x80

    // 0x0C: Serial Bus Controllers
    FireWire,             // 0x00
    ACCESSBus,            // 0x01
    SSA,                  // 0x02
    USB,                  // 0x03
    FibreChannel,         // 0x04
    SMBus,                // 0x05
    InfiniBandController, // 0x06
    IPMI,                 // 0x07
    SERCOS,               // 0x08
    CANBus,               // 0x09
    SerialBusOther,       // 0x80

    // 0x0D: Wireless Controller
    IRDA,          // 0x00
    ConsumerIR,    // 0x01
    RFController,  // 0x10
    Bluetooth,     // 0x11
    Broadband,     // 0x12
    Ethernet8021a, // 0x20
    Ethernet8021b, // 0x21
    WirelessOther, // 0x80

    // 0x0E: Intelligent I/O
    I20, // 0x00

    // 0x0F: Satellite Communication
    TV,       // 0x00
    AudioSat, // 0x01
    VoiceSat, // 0x02
    DataSat,  // 0x03
    OtherSat, // 0x80

    // 0x10: Encryption/Decryption
    NetworkCrypto,       // 0x00
    EntertainmentCrypto, // 0x10
    CryptoOther,         // 0x80

    // 0x11: Signal Processing
    DPIOModule,                 // 0x00
    PerformanceCounters,        // 0x01
    CommunicationSync,          // 0x10
    SignalProcessingManagement, // 0x20
    SignalProcessingOther,      // 0x80

    // 0xFF: Unassigned
    Unassigned, // 0xFF
}

impl SubClass {
    pub fn from_class_sub(class: ClassCode, sub: u8) -> Result<Self, PciConfigurationError> {
        match class {
            ClassCode::Unclassified => match sub {
                0x00 => Ok(Self::NonVGAUnclassified),
                0x01 => Ok(Self::VGACompatibleUnclassified),
                _ => Err(PciConfigurationError::UnknownSubClass(sub as u16)),
            },

            ClassCode::MassStorageController => match sub {
                0x00 => Ok(Self::SCSIController),
                0x01 => Ok(Self::IDEController),
                0x02 => Ok(Self::FloppyController),
                0x03 => Ok(Self::IPIBusController),
                0x04 => Ok(Self::RAIDController),
                0x05 => Ok(Self::ATAController),
                0x06 => Ok(Self::SATAController),
                0x07 => Ok(Self::SASController),
                0x08 => Ok(Self::NVMController),
                0x80 => Ok(Self::MassStorageOther),
                _ => Err(PciConfigurationError::UnknownSubClass(sub as u16)),
            },

            ClassCode::NetworkController => match sub {
                0x00 => Ok(Self::Ethernet),
                0x01 => Ok(Self::TokenRing),
                0x02 => Ok(Self::FDDI),
                0x03 => Ok(Self::ATM),
                0x04 => Ok(Self::ISDN),
                0x05 => Ok(Self::WorldFip),
                0x06 => Ok(Self::PICMGMultiComputing),
                0x07 => Ok(Self::InfinibandNetworkController),
                0x08 => Ok(Self::Fabric),
                0x80 => Ok(Self::NetworkOther),
                _ => Err(PciConfigurationError::UnknownSubClass(sub as u16)),
            },

            ClassCode::DisplayController => match sub {
                0x00 => Ok(Self::VGAController),
                0x01 => Ok(Self::XGAController),
                0x02 => Ok(Self::ThreeDController),
                0x80 => Ok(Self::DisplayOther),
                _ => Err(PciConfigurationError::UnknownSubClass(sub as u16)),
            },

            ClassCode::MultimediaController => match sub {
                0x00 => Ok(Self::Video),
                0x01 => Ok(Self::Audio),
                0x02 => Ok(Self::ComputerTelephony),
                0x03 => Ok(Self::HdAudio),
                0x80 => Ok(Self::MultimediaOther),
                _ => Err(PciConfigurationError::UnknownSubClass(sub as u16)),
            },

            ClassCode::MemoryController => match sub {
                0x00 => Ok(Self::RAMController),
                0x01 => Ok(Self::FlashController),
                0x80 => Ok(Self::MemoryOther),
                _ => Err(PciConfigurationError::UnknownSubClass(sub as u16)),
            },

            ClassCode::Bridge => match sub {
                0x00 => Ok(Self::HostBridge),
                0x01 => Ok(Self::ISABridge),
                0x02 => Ok(Self::EISABridge),
                0x03 => Ok(Self::MCABridge),
                0x04 => Ok(Self::PCItoPCIBridge),
                0x05 => Ok(Self::PCMCIABridge),
                0x06 => Ok(Self::NuBusBridge),
                0x07 => Ok(Self::CardBusBridge),
                0x08 => Ok(Self::RACEwayBridge),
                0x09 => Ok(Self::PCItoPCIBridgeSemi),
                0x0A => Ok(Self::InfinibandBridge),
                0x80 => Ok(Self::BridgeOther),
                _ => Err(PciConfigurationError::UnknownSubClass(sub as u16)),
            },

            ClassCode::SimpleCommunicationController => match sub {
                0x00 => Ok(Self::SerialController),
                0x01 => Ok(Self::ParallelController),
                0x02 => Ok(Self::MultiportSerial),
                0x03 => Ok(Self::Modem),
                0x04 => Ok(Self::GPIB),
                0x05 => Ok(Self::SmartCard),
                0x80 => Ok(Self::CommOther),
                _ => Err(PciConfigurationError::UnknownSubClass(sub as u16)),
            },

            ClassCode::BaseSystemPeripheral => match sub {
                0x00 => Ok(Self::PIC),
                0x01 => Ok(Self::DMAController),
                0x02 => Ok(Self::Timer),
                0x03 => Ok(Self::RTC),
                0x04 => Ok(Self::PCIHotplug),
                0x80 => Ok(Self::BaseSystemOther),
                _ => Err(PciConfigurationError::UnknownSubClass(sub as u16)),
            },

            ClassCode::InputDeviceController => match sub {
                0x00 => Ok(Self::KeyboardController),
                0x01 => Ok(Self::Digitizer),
                0x02 => Ok(Self::MouseController),
                0x03 => Ok(Self::ScannerController),
                0x04 => Ok(Self::GameportController),
                0x80 => Ok(Self::InputOther),
                _ => Err(PciConfigurationError::UnknownSubClass(sub as u16)),
            },

            ClassCode::DockingStation => match sub {
                0x00 => Ok(Self::GenericDocking),
                0x80 => Ok(Self::DockingOther),
                _ => Err(PciConfigurationError::UnknownSubClass(sub as u16)),
            },

            ClassCode::Processor => match sub {
                0x00 => Ok(Self::Processor386),
                0x01 => Ok(Self::Processor486),
                0x02 => Ok(Self::ProcessorPentium),
                0x10 => Ok(Self::ProcessorAlpha),
                0x20 => Ok(Self::ProcessorPowerPC),
                0x30 => Ok(Self::ProcessorMIPS),
                0x40 => Ok(Self::ProcessorCoProc),
                0x80 => Ok(Self::ProcessorOther),
                _ => Err(PciConfigurationError::UnknownSubClass(sub as u16)),
            },

            ClassCode::SerialBusController => match sub {
                0x00 => Ok(Self::FireWire),
                0x01 => Ok(Self::ACCESSBus),
                0x02 => Ok(Self::SSA),
                0x03 => Ok(Self::USB),
                0x04 => Ok(Self::FibreChannel),
                0x05 => Ok(Self::SMBus),
                0x06 => Ok(Self::InfiniBandController),
                0x07 => Ok(Self::IPMI),
                0x08 => Ok(Self::SERCOS),
                0x09 => Ok(Self::CANBus),
                0x80 => Ok(Self::SerialBusOther),
                _ => Err(PciConfigurationError::UnknownSubClass(sub as u16)),
            },

            ClassCode::WirelessController => match sub {
                0x00 => Ok(Self::IRDA),
                0x01 => Ok(Self::ConsumerIR),
                0x10 => Ok(Self::RFController),
                0x11 => Ok(Self::Bluetooth),
                0x12 => Ok(Self::Broadband),
                0x20 => Ok(Self::Ethernet8021a),
                0x21 => Ok(Self::Ethernet8021b),
                0x80 => Ok(Self::WirelessOther),
                _ => Err(PciConfigurationError::UnknownSubClass(sub as u16)),
            },

            ClassCode::IntelligentController => match sub {
                0x00 => Ok(Self::I20),
                _ => Err(PciConfigurationError::UnknownSubClass(sub as u16)),
            },

            ClassCode::SatelliteCommunicationController => match sub {
                0x00 => Ok(Self::TV),
                0x01 => Ok(Self::AudioSat),
                0x02 => Ok(Self::VoiceSat),
                0x03 => Ok(Self::DataSat),
                0x80 => Ok(Self::OtherSat),
                _ => Err(PciConfigurationError::UnknownSubClass(sub as u16)),
            },

            ClassCode::EncryptionController => match sub {
                0x00 => Ok(Self::NetworkCrypto),
                0x10 => Ok(Self::EntertainmentCrypto),
                0x80 => Ok(Self::CryptoOther),
                _ => Err(PciConfigurationError::UnknownSubClass(sub as u16)),
            },

            // 0x11: Signal Processing
            ClassCode::SignalProcessingController => match sub {
                0x00 => Ok(Self::DPIOModule),
                0x01 => Ok(Self::PerformanceCounters),
                0x10 => Ok(Self::CommunicationSync),
                0x20 => Ok(Self::SignalProcessingManagement),
                0x80 => Ok(Self::SignalProcessingOther),
                _ => Err(PciConfigurationError::UnknownSubClass(sub as u16)),
            },

            ClassCode::ProcessingAccel => Ok(Self::Unassigned),
            ClassCode::NonEssentialInstrumentation => Ok(Self::Unassigned),
            ClassCode::Coprocessor => Ok(Self::Unassigned),
            ClassCode::Unassigned => Ok(Self::Unassigned),
        }
    }

    pub fn to_u8(self) -> u8 {
        use SubClass::*;
        match self {
            NonVGAUnclassified => 0x00,
            VGACompatibleUnclassified => 0x01,

            SCSIController => 0x00,
            IDEController => 0x01,
            FloppyController => 0x02,
            IPIBusController => 0x03,
            RAIDController => 0x04,
            ATAController => 0x05,
            SATAController => 0x06,
            SASController => 0x07,
            NVMController => 0x08,
            MassStorageOther => 0x80,

            Ethernet => 0x00,
            TokenRing => 0x01,
            FDDI => 0x02,
            ATM => 0x03,
            ISDN => 0x04,
            WorldFip => 0x05,
            PICMGMultiComputing => 0x06,
            InfinibandNetworkController => 0x07,
            Fabric => 0x08,
            NetworkOther => 0x80,

            VGAController => 0x00,
            XGAController => 0x01,
            ThreeDController => 0x02,
            DisplayOther => 0x80,

            Video => 0x00,
            Audio => 0x01,
            ComputerTelephony => 0x02,
            HdAudio => 0x03,
            MultimediaOther => 0x80,

            RAMController => 0x00,
            FlashController => 0x01,
            MemoryOther => 0x80,

            HostBridge => 0x00,
            ISABridge => 0x01,
            EISABridge => 0x02,
            MCABridge => 0x03,
            PCItoPCIBridge => 0x04,
            PCMCIABridge => 0x05,
            NuBusBridge => 0x06,
            CardBusBridge => 0x07,
            RACEwayBridge => 0x08,
            PCItoPCIBridgeSemi => 0x09,
            InfinibandBridge => 0x0A,
            BridgeOther => 0x80,

            SerialController => 0x00,
            ParallelController => 0x01,
            MultiportSerial => 0x02,
            Modem => 0x03,
            GPIB => 0x04,
            SmartCard => 0x05,
            CommOther => 0x80,

            PIC => 0x00,
            DMAController => 0x01,
            Timer => 0x02,
            RTC => 0x03,
            PCIHotplug => 0x04,
            BaseSystemOther => 0x80,

            KeyboardController => 0x00,
            Digitizer => 0x01,
            MouseController => 0x02,
            ScannerController => 0x03,
            GameportController => 0x04,
            InputOther => 0x80,

            GenericDocking => 0x00,
            DockingOther => 0x80,

            Processor386 => 0x00,
            Processor486 => 0x01,
            ProcessorPentium => 0x02,
            ProcessorAlpha => 0x10,
            ProcessorPowerPC => 0x20,
            ProcessorMIPS => 0x30,
            ProcessorCoProc => 0x40,
            ProcessorOther => 0x80,

            FireWire => 0x00,
            ACCESSBus => 0x01,
            SSA => 0x02,
            USB => 0x03,
            FibreChannel => 0x04,
            SMBus => 0x05,
            InfiniBandController => 0x06,
            IPMI => 0x07,
            SERCOS => 0x08,
            CANBus => 0x09,
            SerialBusOther => 0x80,

            IRDA => 0x00,
            ConsumerIR => 0x01,
            RFController => 0x10,
            Bluetooth => 0x11,
            Broadband => 0x12,
            Ethernet8021a => 0x20,
            Ethernet8021b => 0x21,
            WirelessOther => 0x80,

            I20 => 0x00,

            TV => 0x00,
            AudioSat => 0x01,
            VoiceSat => 0x02,
            DataSat => 0x03,
            OtherSat => 0x80,

            NetworkCrypto => 0x00,
            EntertainmentCrypto => 0x10,
            CryptoOther => 0x80,

            DPIOModule => 0x00,
            PerformanceCounters => 0x01,
            CommunicationSync => 0x10,
            SignalProcessingManagement => 0x20,
            SignalProcessingOther => 0x80,

            Unassigned => 0xFF,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgrammingInterface {
    // IDE Controller
    IsaCompatibilityOnly,
    PciNativeOnly,
    IsaCompatibilityAndPciNative, // starts and ISA and can be switched
    PciNativeAndIsaCompatibility, // starts and PCI and can be switched
    IsaCompatibilityAndBusMastering,
    PciNativeAndBusMastering,
    IsaCompatibilityPciNativeAndBusMastering,
    PciNativeIsaCompatibilityAndBusMastering,

    // ATA Controller
    SingleDMA,
    ChainedDMA,

    // Serial ATA
    VendorSpecificInterface,
    AHCI1,
    SataSerialStorageBus,

    // Serial Scsi Controller
    SAS,
    ScsiSerialStorageBus,

    // NVM Controller
    NVMHCI,
    NVME,

    // VGA Controller
    VgaController,
    Compatibility8514Controller,

    // PCI to PCI Bridge
    NormalDecode,
    SubtractiveDecode,

    // RACEway Bridge
    TransparentMode,
    EndpointMode,

    // Pci to PCi Bridge Semi
    SemiTransparentPrimary,
    SemiTransparentSecondary,

    // Serial Controller
    Compatible8250,
    Compatible16450,
    Compatible16550,
    Compatible16650,
    Compatible16750,
    Compatible16850,
    Compatible16950,

    // Parallel Controller
    StandardParallelPort,
    BiDirectionalParallelPort,
    ECP1CompliantParallelPort,
    IEEE1284Controller,
    IEEE1284TargetDevice,

    // Modem
    GenericModem,
    Hayes16450Compatible,
    Hayes16550Compatible,
    Hayes16650Compatible,
    Hayes16750Compatible,

    // PIC
    Generic8259Compatible,
    PicIsaCompatible,
    PicEisaCompatible,
    IoApicInterruptController,
    IoXApicInterruptController,

    // DMA Controller
    Generic8237Compatible,
    DmaIsaCompatible,
    DmaEisaCompatible,

    // Timer
    Generic8254Compatible,
    TimerIsaCompatible,
    TimerEisaCompatible,
    HPET,

    // RTC Controller
    GenericRtc,
    RtcIsaCompatible,

    // Gameport Controller
    GameportGeneric,
    GameportExtended,

    // FireWire
    FireWireGeneric,
    FireWireOHCI,

    // USB Controller
    UHCI,
    OHCI,
    EHCI,
    XHCI,
    Unspecified,
    UsbDevice,

    // IPMI Interface
    SMIC,
    KeyboardControllerStyle,
    BlockTransfer,

    Unassigned,
}

impl ProgrammingInterface {
    pub fn from_subclass_u8(subclass: SubClass, pi: u8) -> Result<Self, PciConfigurationError> {
        match subclass {
            SubClass::IDEController => match pi {
                0x00 => Ok(Self::IsaCompatibilityOnly),
                0x05 => Ok(Self::PciNativeOnly),
                0x0A => Ok(Self::IsaCompatibilityAndPciNative),
                0x0F => Ok(Self::PciNativeAndIsaCompatibility),
                0x80 => Ok(Self::IsaCompatibilityAndBusMastering),
                0x85 => Ok(Self::PciNativeAndBusMastering),
                0x8A => Ok(Self::IsaCompatibilityPciNativeAndBusMastering),
                0x8F => Ok(Self::PciNativeIsaCompatibilityAndBusMastering),
                _ => Err(PciConfigurationError::UnknownProgrammableInterface(
                    pi as u16,
                )),
            },

            SubClass::ATAController => match pi {
                0x20 => Ok(Self::SingleDMA),
                0x30 => Ok(Self::ChainedDMA),
                _ => Err(PciConfigurationError::UnknownProgrammableInterface(
                    pi as u16,
                )),
            },

            SubClass::SATAController => match pi {
                0x00 => Ok(Self::VendorSpecificInterface),
                0x01 => Ok(Self::AHCI1),
                0x02 => Ok(Self::SataSerialStorageBus),
                _ => Err(PciConfigurationError::UnknownProgrammableInterface(
                    pi as u16,
                )),
            },

            SubClass::SCSIController => match pi {
                0x00 => Ok(Self::SAS),
                0x01 => Ok(Self::ScsiSerialStorageBus),
                _ => Err(PciConfigurationError::UnknownProgrammableInterface(
                    pi as u16,
                )),
            },

            SubClass::NVMController => match pi {
                0x01 => Ok(Self::NVMHCI),
                0x02 => Ok(Self::NVME),
                _ => Err(PciConfigurationError::UnknownProgrammableInterface(
                    pi as u16,
                )),
            },

            SubClass::VGAController => match pi {
                0x00 => Ok(Self::VgaController),
                0x01 => Ok(Self::Compatibility8514Controller),
                _ => Err(PciConfigurationError::UnknownProgrammableInterface(
                    pi as u16,
                )),
            },

            SubClass::PCItoPCIBridge => match pi {
                0x00 => Ok(Self::NormalDecode),
                0x01 => Ok(Self::SubtractiveDecode),
                _ => Err(PciConfigurationError::UnknownProgrammableInterface(
                    pi as u16,
                )),
            },

            SubClass::RACEwayBridge => match pi {
                0x00 => Ok(Self::TransparentMode),
                0x01 => Ok(Self::EndpointMode),
                _ => Err(PciConfigurationError::UnknownProgrammableInterface(
                    pi as u16,
                )),
            },

            SubClass::PCItoPCIBridgeSemi => match pi {
                0x40 => Ok(Self::SemiTransparentPrimary),
                0x80 => Ok(Self::SemiTransparentSecondary),
                _ => Err(PciConfigurationError::UnknownProgrammableInterface(
                    pi as u16,
                )),
            },

            SubClass::SerialController => match pi {
                0x00 => Ok(Self::Compatible8250),
                0x01 => Ok(Self::Compatible16450),
                0x02 => Ok(Self::Compatible16550),
                0x03 => Ok(Self::Compatible16650),
                0x04 => Ok(Self::Compatible16750),
                0x05 => Ok(Self::Compatible16850),
                0x06 => Ok(Self::Compatible16950),
                _ => Err(PciConfigurationError::UnknownProgrammableInterface(
                    pi as u16,
                )),
            },

            SubClass::ParallelController => match pi {
                0x00 => Ok(Self::StandardParallelPort),
                0x01 => Ok(Self::BiDirectionalParallelPort),
                0x02 => Ok(Self::ECP1CompliantParallelPort),
                0x03 => Ok(Self::IEEE1284Controller),
                0xFE => Ok(Self::IEEE1284TargetDevice),
                _ => Err(PciConfigurationError::UnknownProgrammableInterface(
                    pi as u16,
                )),
            },

            SubClass::Modem => match pi {
                0x00 => Ok(Self::GenericModem),
                0x01 => Ok(Self::Hayes16450Compatible),
                0x02 => Ok(Self::Hayes16550Compatible),
                0x03 => Ok(Self::Hayes16650Compatible),
                0x04 => Ok(Self::Hayes16750Compatible),
                _ => Err(PciConfigurationError::UnknownProgrammableInterface(
                    pi as u16,
                )),
            },

            SubClass::PIC => match pi {
                0x00 => Ok(Self::Generic8259Compatible),
                0x01 => Ok(Self::PicIsaCompatible),
                0x02 => Ok(Self::PicEisaCompatible),
                0x10 => Ok(Self::IoApicInterruptController),
                0x20 => Ok(Self::IoXApicInterruptController),
                _ => Err(PciConfigurationError::UnknownProgrammableInterface(
                    pi as u16,
                )),
            },

            SubClass::DMAController => match pi {
                0x00 => Ok(Self::Generic8237Compatible),
                0x01 => Ok(Self::DmaIsaCompatible),
                0x02 => Ok(Self::DmaEisaCompatible),
                _ => Err(PciConfigurationError::UnknownProgrammableInterface(
                    pi as u16,
                )),
            },

            SubClass::Timer => match pi {
                0x00 => Ok(Self::Generic8254Compatible),
                0x01 => Ok(Self::TimerIsaCompatible),
                0x02 => Ok(Self::TimerEisaCompatible),
                0x03 => Ok(Self::HPET),
                _ => Err(PciConfigurationError::UnknownProgrammableInterface(
                    pi as u16,
                )),
            },

            SubClass::RTC => match pi {
                0x00 => Ok(Self::GenericRtc),
                0x01 => Ok(Self::RtcIsaCompatible),
                _ => Err(PciConfigurationError::UnknownProgrammableInterface(
                    pi as u16,
                )),
            },

            SubClass::GameportController => match pi {
                0x00 => Ok(Self::GameportGeneric),
                0x10 => Ok(Self::GameportExtended),
                _ => Err(PciConfigurationError::UnknownProgrammableInterface(
                    pi as u16,
                )),
            },

            SubClass::FireWire => match pi {
                0x00 => Ok(Self::FireWireGeneric),
                0x10 => Ok(Self::FireWireOHCI),
                _ => Err(PciConfigurationError::UnknownProgrammableInterface(
                    pi as u16,
                )),
            },

            SubClass::USB => match pi {
                0x00 => Ok(Self::UHCI),
                0x10 => Ok(Self::OHCI),
                0x20 => Ok(Self::EHCI),
                0x30 => Ok(Self::XHCI),
                0x80 => Ok(Self::Unspecified),
                0xFE => Ok(Self::UsbDevice),
                _ => Err(PciConfigurationError::UnknownProgrammableInterface(
                    pi as u16,
                )),
            },

            SubClass::IPMI => match pi {
                0x00 => Ok(Self::SMIC),
                0x01 => Ok(Self::KeyboardControllerStyle),
                0x02 => Ok(Self::BlockTransfer),
                _ => Err(PciConfigurationError::UnknownProgrammableInterface(
                    pi as u16,
                )),
            },

            _ => Ok(Self::Unassigned),
        }
    }
    pub fn to_u8(self) -> u8 {
        match self {
            // IDEController
            Self::IsaCompatibilityOnly => 0x00,
            Self::PciNativeOnly => 0x05,
            Self::IsaCompatibilityAndPciNative => 0x0A,
            Self::PciNativeAndIsaCompatibility => 0x0F,
            Self::IsaCompatibilityAndBusMastering => 0x80,
            Self::PciNativeAndBusMastering => 0x85,
            Self::IsaCompatibilityPciNativeAndBusMastering => 0x8A,
            Self::PciNativeIsaCompatibilityAndBusMastering => 0x8F,

            // ATAController
            Self::SingleDMA => 0x20,
            Self::ChainedDMA => 0x30,

            // SATAController
            Self::VendorSpecificInterface => 0x00,
            Self::AHCI1 => 0x01,
            Self::SataSerialStorageBus => 0x02,

            // SCSIController
            Self::SAS => 0x00,
            Self::ScsiSerialStorageBus => 0x01,

            // NVMController
            Self::NVMHCI => 0x01,
            Self::NVME => 0x02,

            // VGAController
            Self::VgaController => 0x00,
            Self::Compatibility8514Controller => 0x01,

            // PCItoPCIBridge
            Self::NormalDecode => 0x00,
            Self::SubtractiveDecode => 0x01,

            // RACEwayBridge
            Self::TransparentMode => 0x00,
            Self::EndpointMode => 0x01,

            // PCItoPCIBridgeSemi
            Self::SemiTransparentPrimary => 0x40,
            Self::SemiTransparentSecondary => 0x80,

            // SerialController
            Self::Compatible8250 => 0x00,
            Self::Compatible16450 => 0x01,
            Self::Compatible16550 => 0x02,
            Self::Compatible16650 => 0x03,
            Self::Compatible16750 => 0x04,
            Self::Compatible16850 => 0x05,
            Self::Compatible16950 => 0x06,

            // ParallelController
            Self::StandardParallelPort => 0x00,
            Self::BiDirectionalParallelPort => 0x01,
            Self::ECP1CompliantParallelPort => 0x02,
            Self::IEEE1284Controller => 0x03,
            Self::IEEE1284TargetDevice => 0xFE,

            // Modem
            Self::GenericModem => 0x00,
            Self::Hayes16450Compatible => 0x01,
            Self::Hayes16550Compatible => 0x02,
            Self::Hayes16650Compatible => 0x03,
            Self::Hayes16750Compatible => 0x04,

            // PIC
            Self::Generic8259Compatible => 0x00,
            Self::PicIsaCompatible => 0x01,
            Self::PicEisaCompatible => 0x02,
            Self::IoApicInterruptController => 0x10,
            Self::IoXApicInterruptController => 0x20,

            // DMAController
            Self::Generic8237Compatible => 0x00,
            Self::DmaIsaCompatible => 0x01,
            Self::DmaEisaCompatible => 0x02,

            // Timer
            Self::Generic8254Compatible => 0x00,
            Self::TimerIsaCompatible => 0x01,
            Self::TimerEisaCompatible => 0x02,
            Self::HPET => 0x03,

            // RTC
            Self::GenericRtc => 0x00,
            Self::RtcIsaCompatible => 0x01,

            // GameportController
            Self::GameportGeneric => 0x00,
            Self::GameportExtended => 0x10,

            // FireWire
            Self::FireWireGeneric => 0x00,
            Self::FireWireOHCI => 0x10,

            // USB
            Self::UHCI => 0x00,
            Self::OHCI => 0x10,
            Self::EHCI => 0x20,
            Self::XHCI => 0x30,
            Self::Unspecified => 0x80,
            Self::UsbDevice => 0xFE,

            // IPMI
            Self::SMIC => 0x00,
            Self::KeyboardControllerStyle => 0x01,
            Self::BlockTransfer => 0x02,

            Self::Unassigned => 0x00,
        }
    }
}
