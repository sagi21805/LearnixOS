#[repr(u16)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum VendorID {
    Intel = 0x8086,
    VirtIO = 0x1AF4,
    Nvidia = 0x10DE,
    Realtek = 0x10EC,
    QEMU = 0x1B36,
    NonExistent = 0xFFFF,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u16)]
pub enum IntelDeviceID {
    HostBridge = 0x1237,
    PIIX3ISA = 0x700,
    PIIX3IDE = 0x701,
    PIIX3USB = 0x702,
    PIIX3ACPI = 0x703,
    ExpressDramController = 0x29C0,
    NetworkController = 0x100E, // e1000 again
    LPCInterface82801IB = 0x2410,
    SataControllerAHCI = 0x2822,
    NonExistent = 0xFFFF,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u16)]
pub enum VirtIODeviceID {
    Net = 0x1000,
    Block = 0x1001,
    Console = 0x1003,
    Balloon = 0x1005,
    NonExistent = 0xFFFF,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u16)]
pub enum QEMUDeviceID {
    StandardVGA = 0xB0C0,
    QXLGraphics = 0x0001,
    NonExistent = 0xFFFF,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u16)]
pub enum RealtekDeviceID {
    RTL8139NIC = 0x8139,
    NonExistent = 0xFFFF,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u16)]
pub enum NVIDIADeviceID {
    GPU = 0x1C81,
    NonExistent = 0xFFFF,
}

#[derive(Clone, Copy)]
pub union DeviceID {
    pub intel: IntelDeviceID,
    pub virtio: VirtIODeviceID,
    pub nvidia: NVIDIADeviceID,
    pub realtek: RealtekDeviceID,
    pub qemu: QEMUDeviceID,
    pub num: u16,
    pub none: (),
}

#[derive(Clone, Copy)]
pub struct VendorDevice {
    pub vendor: VendorID,
    pub device: DeviceID,
}

impl core::fmt::Debug for VendorDevice {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Vendor: {:?} ", self.vendor)?;
        write!(f, "Device: 0x{:x?}", unsafe { self.device.num })
    }
}

impl core::fmt::Debug for DeviceID {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Device: 0x{:x?}", unsafe { self.num })
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClassCode {
    Unclassified = 0x00,
    MassStorageController = 0x01,
    NetworkController = 0x02,
    DisplayController = 0x03,
    MultimediaController = 0x04,
    MemoryController = 0x05,
    Bridge = 0x06,
    SimpleCommunicationController = 0x07,
    BaseSystemPeripheral = 0x08,
    InputDeviceController = 0x09,
    DockingStation = 0x0A,
    Processor = 0x0B,
    SerialBusController = 0x0C,
    WirelessController = 0x0D,
    IntelligentController = 0x0E,
    SatelliteCommunicationController = 0x0F,
    EncryptionController = 0x10,
    SignalProcessingController = 0x11,
    ProcessingAccel = 0x12,
    NonEssentialInstrumentation = 0x13,
    Coprocessor = 0x40,
    Unassigned = 0xFF,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnclassifiedSubClass {
    NonVGA = 0x00,
    VGACompatible = 0x01,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MassStorageSubClass {
    SCSI = 0x00,
    IDE = 0x01,
    Floppy = 0x02,
    IPIBus = 0x03,
    RAID = 0x04,
    ATA = 0x05,
    SATA = 0x06,
    SAS = 0x07,
    NVM = 0x08,
    Other = 0x80,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkSubClass {
    Ethernet = 0x00,
    TokenRing = 0x01,
    FDDI = 0x02,
    ATM = 0x03,
    ISDN = 0x04,
    WorldFip = 0x05,
    PICMG = 0x06,
    Infiniband = 0x07,
    Fabric = 0x08,
    Other = 0x80,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplaySubClass {
    VGA = 0x00,
    XGA = 0x01,
    ThreeD = 0x02,
    Other = 0x80,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MultimediaSubClass {
    Video = 0x00,
    Audio = 0x01,
    ComputerTelephony = 0x02,
    HdAudio = 0x03,
    Other = 0x80,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemorySubClass {
    RAM = 0x00,
    Flash = 0x01,
    Other = 0x80,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeSubClass {
    Host = 0x00,
    ISA = 0x01,
    EISA = 0x02,
    MCA = 0x03,
    PCItoPCI = 0x04,
    PCMCIABridge = 0x05,
    NuBus = 0x06,
    CardBus = 0x07,
    RACEway = 0x08,
    PCItoPCISemi = 0x09,
    Infiniband = 0x0A,
    Other = 0x80,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimpleCommSubClass {
    Serial = 0x00,
    Parallel = 0x01,
    MultiportSerial = 0x02,
    Modem = 0x03,
    GPIB = 0x04,
    SmartCard = 0x05,
    Other = 0x80,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BaseSystemSubClass {
    PIC = 0x00,
    DMAController = 0x01,
    Timer = 0x02,
    RTC = 0x03,
    PCIHotplug = 0x04,
    Other = 0x80,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputSubClass {
    Keyboard = 0x00,
    Digitizer = 0x01,
    Mouse = 0x02,
    Scanner = 0x03,
    Gameport = 0x04,
    Other = 0x80,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockingStationSubClass {
    Generic = 0x00,
    Other = 0x80,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessorSubClass {
    P386 = 0x00,
    P486 = 0x01,
    Pentium = 0x02,
    Alpha = 0x10,
    PowerPC = 0x20,
    MIPS = 0x30,
    CoProc = 0x40,
    Other = 0x80,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SerialBusSubClass {
    FireWire = 0x00,
    ACCESSBus = 0x01,
    SSA = 0x02,
    USB = 0x03,
    FibreChannel = 0x04,
    SMBus = 0x05,
    InfiniBand = 0x06,
    IPMI = 0x07,
    SERCOS = 0x08,
    CANBus = 0x09,
    Other = 0x80,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WirelessSubClass {
    IRDA = 0x00,
    ConsumerIR = 0x01,
    RFController = 0x10,
    Bluetooth = 0x11,
    Broadband = 0x12,
    Ethernet8021a = 0x20,
    Ethernet8021b = 0x21,
    Other = 0x80,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntelligentSubClass {
    I20 = 0x00,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SatelliteCommSubClass {
    TV = 0x00,
    Audio = 0x01,
    Voice = 0x02,
    Data = 0x03,
    Other = 0x80,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncryptionSubClass {
    Network = 0x00,
    Entertainment = 0x10,
    Other = 0x80,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalProcessingSubClass {
    DPIOModule = 0x00,
    PerformanceCounters = 0x01,
    CommunicationSync = 0x10,
    Management = 0x20,
    Other = 0x80,
}

#[derive(Clone, Copy)]
pub union SubClass {
    pub unclassified: UnclassifiedSubClass,
    pub storage: MassStorageSubClass,
    pub network: NetworkSubClass,
    pub display: DisplaySubClass,
    pub multimedia: MultimediaSubClass,
    pub memory: MemorySubClass,
    pub bridge: BridgeSubClass,
    pub simple_comm: SimpleCommSubClass,
    pub base_system: BaseSystemSubClass,
    pub input: InputSubClass,
    pub docking: DockingStationSubClass,
    pub processor: ProcessorSubClass,
    pub serial_bus: SerialBusSubClass,
    pub wireless: WirelessSubClass,
    pub intelligent: IntelligentSubClass,
    pub satellite: SatelliteCommSubClass,
    pub encryption: EncryptionSubClass,
    pub signal_processing: SignalProcessingSubClass,
    pub num: u8,
    pub none: (),
}

impl core::fmt::Debug for SubClass {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:x}", unsafe { self.num })
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IDEControllerPI {
    IsaCompatibilityOnly = 0x00,
    PciNativeOnly = 0x05,
    IsaCompatibilityAndPciNative = 0x0A,
    PciNativeAndIsaCompatibility = 0x0F,
    IsaCompatibilityAndBusMastering = 0x80,
    PciNativeAndBusMastering = 0x85,
    IsaCompatibilityPciNativeAndBusMastering = 0x8A,
    PciNativeIsaCompatibilityAndBusMastering = 0x8F,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ATAControllerPI {
    SingleDMA = 0x20,
    ChainedDMA = 0x30,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SATAControllerPI {
    VendorSpecificInterface = 0x00,
    AHCI1 = 0x01,
    SataSerialStorageBus = 0x02,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SCSIControllerPI {
    SAS = 0x00,
    ScsiSerialStorageBus = 0x01,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NVMControllerPI {
    NVMHCI = 0x01,
    NVME = 0x02,
}
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VGAControllerPI {
    VgaController = 0x00,
    Compatibility8514Controller = 0x01,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PCItoPCIBridgePI {
    NormalDecode = 0x00,
    SubtractiveDecode = 0x01,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RACEwayBridgePI {
    TransparentMode = 0x00,
    EndpointMode = 0x01,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PCItoPCIBridgeSemiPI {
    SemiTransparentPrimary = 0x40,
    SemiTransparentSecondary = 0x80,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SerialControllerPI {
    Compatible8250 = 0x00,
    Compatible16450 = 0x01,
    Compatible16550 = 0x02,
    Compatible16650 = 0x03,
    Compatible16750 = 0x04,
    Compatible16850 = 0x05,
    Compatible16950 = 0x06,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParallelControllerPI {
    StandardParallelPort = 0x00,
    BiDirectionalParallelPort = 0x01,
    ECP1CompliantParallelPort = 0x02,
    IEEE1284Controller = 0x03,
    IEEE1284TargetDevice = 0xFE,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModemPI {
    GenericModem = 0x00,
    Hayes16450Compatible = 0x01,
    Hayes16550Compatible = 0x02,
    Hayes16650Compatible = 0x03,
    Hayes16750Compatible = 0x04,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PICPI {
    Generic8259Compatible = 0x00,
    PicIsaCompatible = 0x01,
    PicEisaCompatible = 0x02,
    IoApicInterruptController = 0x10,
    IoXApicInterruptController = 0x20,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DMAControllerPI {
    Generic8237Compatible = 0x00,
    DmaIsaCompatible = 0x01,
    DmaEisaCompatible = 0x02,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerPI {
    Generic8254Compatible = 0x00,
    TimerIsaCompatible = 0x01,
    TimerEisaCompatible = 0x02,
    HPET = 0x03,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RTCPI {
    GenericRtc = 0x00,
    RtcIsaCompatible = 0x01,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameportControllerPI {
    GameportGeneric = 0x00,
    GameportExtended = 0x10,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FireWirePI {
    FireWireGeneric = 0x00,
    FireWireOHCI = 0x10,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum USBPI {
    UHCI = 0x00,
    OHCI = 0x10,
    EHCI = 0x20,
    XHCI = 0x30,
    Unspecified = 0x80,
    UsbDevice = 0xFE,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IPMIPI {
    SMIC = 0x00,
    KeyboardControllerStyle = 0x01,
    BlockTransfer = 0x02,
}

#[derive(Clone, Copy)]
pub union ProgrammingInterface {
    pub ide: IDEControllerPI,
    pub ata: ATAControllerPI,
    pub sata: SATAControllerPI,
    pub scsi: SCSIControllerPI,
    pub nvm: NVMControllerPI,
    pub vga: VGAControllerPI,
    pub pci2pci: PCItoPCIBridgePI,
    pub raceway: RACEwayBridgePI,
    pub semi_pci2pci: PCItoPCIBridgeSemiPI,
    pub serial: SerialControllerPI,
    pub parallel: ParallelControllerPI,
    pub modem: ModemPI,
    pub pic: PICPI,
    pub dma: DMAControllerPI,
    pub timer: TimerPI,
    pub rtc: RTCPI,
    pub game_port: GameportControllerPI,
    pub firewire: FireWirePI,
    pub usb: USBPI,
    pub ipmi: IPMIPI,
    pub num: u8,
    pub none: (),
}

impl core::fmt::Debug for ProgrammingInterface {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:x}", unsafe { self.num })
    }
}

#[derive(Clone, Copy)]
pub struct PciDeviceType {
    pub prog_if: ProgrammingInterface,
    pub subclass: SubClass,
    pub class: ClassCode,
}

impl core::fmt::Debug for PciDeviceType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Class: {:?} ", self.class)?;
        write!(f, "SubClass: ")?;
        match self.class {
            ClassCode::Unclassified => {
                write!(f, "{:?}", unsafe { self.subclass.unclassified })
            }
            ClassCode::MassStorageController => {
                write!(f, "{:?}", unsafe { self.subclass.storage })?;
                write!(f, " ProgIf: ")?;
                match unsafe { self.subclass.storage } {
                    MassStorageSubClass::IDE => {
                        write!(f, "{:?}", unsafe { self.prog_if.ide })
                    }
                    MassStorageSubClass::ATA => {
                        write!(f, "{:?}", unsafe { self.prog_if.ata })
                    }
                    MassStorageSubClass::SATA => {
                        write!(f, "{:?}", unsafe { self.prog_if.sata })
                    }
                    MassStorageSubClass::SCSI => {
                        write!(f, "{:?}", unsafe { self.prog_if.scsi })
                    }
                    MassStorageSubClass::NVM => {
                        write!(f, "{:?}", unsafe { self.prog_if.nvm })
                    }
                    _ => write!(f, "{:?}", unsafe { self.prog_if.none }),
                }
            }
            ClassCode::NetworkController => {
                write!(f, "{:?}", unsafe { self.subclass.network })?;
                write!(f, " ProgIf: {:?}", unsafe { self.prog_if.none })
            }
            ClassCode::DisplayController => {
                write!(f, "{:?}", unsafe { self.subclass.display })?;
                write!(f, " ProgIf: ")?;
                match unsafe { self.subclass.display } {
                    DisplaySubClass::VGA => {
                        write!(f, "{:?}", unsafe { self.prog_if.vga })
                    }
                    _ => write!(f, "{:?}", unsafe { self.prog_if.none }),
                }
            }
            ClassCode::MultimediaController => {
                write!(f, "{:?}", unsafe { self.subclass.multimedia })?;
                write!(f, " ProgIf: {:?}", unsafe { self.prog_if.none })
            }
            ClassCode::MemoryController => {
                write!(f, "{:?}", unsafe { self.subclass.memory })
            }
            ClassCode::Bridge => {
                write!(f, "{:?}", unsafe { self.subclass.bridge })?;
                write!(f, " ProgIf: ")?;
                match unsafe { self.subclass.bridge } {
                    BridgeSubClass::PCItoPCI => {
                        write!(f, "{:?}", unsafe { self.prog_if.pci2pci })
                    }
                    BridgeSubClass::PCItoPCISemi => {
                        write!(f, "{:?}", unsafe { self.prog_if.pci2pci })
                    }
                    _ => write!(f, "{:?}", unsafe { self.prog_if.none }),
                }
            }
            ClassCode::SimpleCommunicationController => {
                write!(f, "{:?}", unsafe { self.subclass.simple_comm })?;
                write!(f, " ProgIf: ")?;
                match unsafe { self.subclass.simple_comm } {
                    SimpleCommSubClass::Serial => {
                        write!(f, "{:?}", unsafe { self.prog_if.serial })
                    }
                    SimpleCommSubClass::Parallel => {
                        write!(f, "{:?}", unsafe { self.prog_if.parallel })
                    }
                    SimpleCommSubClass::Modem => {
                        write!(f, "{:?}", unsafe { self.prog_if.modem })
                    }
                    _ => write!(f, "{:?}", unsafe { self.prog_if.none }),
                }
            }
            ClassCode::BaseSystemPeripheral => {
                write!(f, "{:?}", unsafe { self.subclass.base_system })?;
                write!(f, " ProgIf: ")?;
                match unsafe { self.subclass.base_system } {
                    BaseSystemSubClass::PIC => {
                        write!(f, "{:?}", unsafe { self.prog_if.pic })
                    }
                    BaseSystemSubClass::DMAController => {
                        write!(f, "{:?}", unsafe { self.prog_if.dma })
                    }
                    BaseSystemSubClass::Timer => {
                        write!(f, "{:?}", unsafe { self.prog_if.timer })
                    }
                    BaseSystemSubClass::RTC => {
                        write!(f, "{:?}", unsafe { self.prog_if.rtc })
                    }
                    _ => write!(f, "{:?}", unsafe { self.prog_if.none }),
                }
            }
            ClassCode::InputDeviceController => {
                write!(f, "{:?} ", unsafe { self.subclass.input })?;
                write!(f, " ProgIf: ")?;
                match unsafe { self.subclass.input } {
                    InputSubClass::Gameport => {
                        write!(f, "{:?}", unsafe { self.subclass.input })
                    }
                    _ => write!(f, "{:?}", unsafe { self.prog_if.none }),
                }
            }
            ClassCode::DockingStation => {
                write!(f, "{:?} ", unsafe { self.subclass.docking })?;
                write!(f, " ProgIf: {:?}", unsafe { self.prog_if.none })
            }
            ClassCode::Processor => {
                write!(f, "{:?} ", unsafe { self.subclass.processor })?;
                write!(f, " ProgIf: {:?}", unsafe { self.prog_if.none })
            }
            ClassCode::SerialBusController => {
                write!(f, "{:?} ", unsafe { self.subclass.serial_bus })?;
                write!(f, " ProgIf: ")?;
                match unsafe { self.subclass.serial_bus } {
                    SerialBusSubClass::FireWire => {
                        write!(f, "{:?}", unsafe { self.prog_if.firewire })
                    }
                    SerialBusSubClass::USB => {
                        write!(f, "{:?}", unsafe { self.prog_if.usb })
                    }
                    SerialBusSubClass::IPMI => {
                        write!(f, "{:?}", unsafe { self.prog_if.ipmi })
                    }
                    _ => write!(f, "{:?}", unsafe { self.prog_if.none }),
                }
            }
            ClassCode::WirelessController => {
                write!(f, "{:?}", unsafe { self.subclass.wireless })
            }
            ClassCode::IntelligentController => {
                write!(f, "{:?} ", unsafe { self.subclass.intelligent })?;
                write!(f, " ProgIf: {:?}", unsafe { self.prog_if.none })
            }
            ClassCode::SatelliteCommunicationController => {
                write!(f, "{:?} ", unsafe { self.subclass.satellite })?;
                write!(f, " ProgIf: {:?}", unsafe { self.prog_if.none })
            }
            ClassCode::EncryptionController => {
                write!(f, "{:?} ", unsafe { self.subclass.encryption })?;
                write!(f, " ProgIf: {:?}", unsafe { self.prog_if.none })
            }
            ClassCode::SignalProcessingController => {
                write!(f, "{:?} ", unsafe {
                    self.subclass.signal_processing
                })?;
                write!(f, " ProgIf: {:?}", unsafe { self.prog_if.none })
            }
            ClassCode::ProcessingAccel => {
                write!(f, "{:?} ", unsafe { self.subclass.none })?;
                write!(f, " ProgIf: {:?}", unsafe { self.prog_if.none })
            }

            ClassCode::NonEssentialInstrumentation => {
                write!(f, "{:?} ", unsafe { self.subclass.none })?;
                write!(f, " ProgIf: {:?}", unsafe { self.prog_if.none })
            }
            ClassCode::Coprocessor => {
                write!(f, "{:?} ", unsafe { self.subclass.none })?;
                write!(f, " ProgIf: {:?}", unsafe { self.prog_if.none })
            }
            ClassCode::Unassigned => {
                write!(f, "{:?} ", unsafe { self.subclass.none })?;
                write!(f, " ProgIf: {:?}", unsafe { self.prog_if.none })
            }
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum HeaderType {
    GeneralDevice = 0x00,
    PciToPciBridge = 0x01,
    PciToCardBusBridge = 0x02,
    GeneralDeviceMultiFunction = 0x80,
    PciToPciBridgeMultiFunction = 0x81,
    PciToCardBusBridgeMultiFunction = 0x82,
}

impl HeaderType {
    pub fn is_multifunction(self) -> bool {
        (self as u8) & 0x80 != 0
    }
}
