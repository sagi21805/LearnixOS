extern crate alloc;
use crate::memory::allocators::page_allocator::{ALLOCATOR, allocator::PhysicalPageAllocator};
use alloc::vec::Vec;
use common::{
    enums::{ClassCode, DeviceID, HeaderType, Port, ProgrammingInterface, SubClass, VendorID},
    error::PciConfigurationError,
    flag,
};
use cpu_utils::instructions::port::PortExt;

#[derive(Debug, Clone, Copy)]
pub struct PciConfigurationCycle(u32);

impl PciConfigurationCycle {
    flag!(enable, 31);

    /// Not checking device max numb er, function max number and offset alignment
    pub const fn new_unchecked(bus: u8, device: u8, function: u8, offset: u8) -> Self {
        let config_address: u32 = ((bus as u32) << 16)
            | ((device as u32) << 11)
            | ((function as u32) << 8)
            | (offset as u32);
        Self(config_address).enable()
    }

    pub unsafe fn read(self) -> u32 {
        unsafe {
            Port::PciConfigAddress.outl(self.0);
            Port::PciConfigData.inl()
        }
    }

    pub unsafe fn write(self, val: u32) {
        unsafe {
            Port::PciConfigAddress.outl(self.0);
            Port::PciConfigData.outl(val);
        }
    }

    pub fn read_vendor_device(
        bus: u8,
        device: u8,
    ) -> Result<(VendorID, DeviceID), PciConfigurationError> {
        let config_address = Self::new_unchecked(bus, device, 0, 0);
        let vendor_config = unsafe { config_address.read() };
        let vendor = VendorID::from_u16((vendor_config & 0xffff) as u16)?;
        if vendor == VendorID::NonExistent {
            return Err(PciConfigurationError::NonExistentDevice(bus, device));
        }
        let device_id = DeviceID::from_vendor_dev_id(vendor, (vendor_config >> 16) as u16)?;
        Ok((vendor, device_id))
    }

    pub fn read_command_status(bus: u8, device: u8) -> (CommandRegister, StatusRegister) {
        let config_address = Self::new_unchecked(bus, device, 0, 4);
        let status_command = unsafe { config_address.read() };
        let command = CommandRegister((status_command & 0xffff) as u16);
        let status = StatusRegister((status_command >> 16) as u16);
        (command, status)
    }

    pub fn read_revision_type(
        bus: u8,
        device: u8,
    ) -> Result<(u8, ClassCode, SubClass, ProgrammingInterface), PciConfigurationError> {
        let config_address = Self::new_unchecked(bus, device, 0, 8);
        let value = unsafe { config_address.read() };
        let revision = (value & 0xff) as u8;
        let class_code = ClassCode::from_u8(((value >> 24) & 0xff) as u8)?;
        let subclass = SubClass::from_class_sub(class_code, ((value >> 16) & 0xff) as u8)?;
        let prog_if =
            ProgrammingInterface::from_subclass_u8(subclass, ((value >> 8) & 0xff) as u8)?;
        Ok((revision, class_code, subclass, prog_if))
    }

    pub fn read_header_meta(bus: u8, device: u8) -> (u8, u8, HeaderType, BISTRegister) {
        let config_address = Self::new_unchecked(bus, device, 0, 12);
        let value = unsafe { config_address.read() };
        let cache_size = (value & 0xff) as u8;
        let latency = ((value >> 8) & 0xff) as u8;
        let header_type: HeaderType = unsafe { core::mem::transmute(((value >> 16) & 0xff) as u8) };
        let bist = BISTRegister(((value >> 24) & 0xff) as u8);
        (cache_size, latency, header_type, bist)
    }

    pub fn read_common_header(
        bus: u8,
        device: u8,
    ) -> Result<PciCommonHeader, PciConfigurationError> {
        let (vendor, device_id) = Self::read_vendor_device(bus, device)?;
        let (command, status) = Self::read_command_status(bus, device);
        let (revision, class_code, subclass, prog_if) = Self::read_revision_type(bus, device)?;
        let (cache_size, latency, header_type, bist) = Self::read_header_meta(bus, device);
        return Ok(PciCommonHeader {
            vendor,
            device: device_id,
            command,
            status,
            revision,
            prog_if,
            subclass,
            class_code,
            cache_size,
            latency_timer: latency,
            header_type,
            bist,
        });
    }

    pub fn read_general_device_header(
        bus: u8,
        device: u8,
        common: PciCommonHeader,
    ) -> GeneralDeviceHeader {
        let mut empty = GeneralDeviceHeader::empty_from_common(common);
        let start_offset = size_of::<PciCommonHeader>();
        let end_offset = size_of::<GeneralDeviceHeader>();
        let empty_ptr = &mut empty as *mut GeneralDeviceHeader as usize as *mut u32;
        for offset in (start_offset..end_offset).step_by(size_of::<u32>()) {
            unsafe {
                let header_data = Self::new_unchecked(bus, device, 0, offset as u8).read();
                empty_ptr.byte_add(offset).write_volatile(header_data);
            }
        }
        empty
    }
}

#[derive(Debug, Clone, Copy)]
pub struct StatusRegister(u16);

impl StatusRegister {
    // Device detected a parity error
    flag!(detected_parity_error, 15);
    // Device asserted SERR
    flag!(signaled_system_error, 14);
    //Master  Device transaction was terminated by master abort
    flag!(received_master_abort, 13);
    // Master Device transaction was terminated by target abort
    flag!(received_target_abort, 12);
    // Target device terminated by target abort
    flag!(signaled_target_abort, 11);

    // ReadOnly bits which represent the slowest time device will assert DEVSEL
    // 00 -> fast 01 -> medium 02 -> slow
    // bits 9-10 devsel

    // Parity error detected and handled
    flag!(master_data_parity_error, 8);
    // Device is capable for fast back transaction not from the same agent
    flag!(fast_back2back_capable, 7);
    // Device is able to run at 66mhz
    flag!(capable_66mhz, 5);
    // Implements pointer for the capabilities list
    flag!(capabilities_list, 4);
    // Represents if interrupt is fired or not.
    flag!(interrupt_status, 3);
}

#[derive(Debug, Clone, Copy)]
pub struct CommandRegister(u16);

impl CommandRegister {
    // Disable interrupts for this device
    flag!(interrupt_disable, 10);
    // Allow device to generate back to back transactions
    flag!(fast_back2back_enable, 9);
    // SERR driver is enabled
    flag!(serr_enable, 8);
    // If enabled device will take its normal action on parity error
    // otherwise it will set bit 15 on status register, and will continue operation as normal
    flag!(parity_error_response, 6);
    // Allow device to listen to vga palette writes, and copy them for it own use (Legacy)
    flag!(vga_palette_snoop, 5);
    // Allow device to generate memory writes and invalidate commands. Otherwise memory write command must be used.
    flag!(memory_write_inval_enable, 4);
    // Allow device to monitor special cycle, otherwise ignore them
    flag!(special_cycles, 3);
    // Allow device to behave as bus master.
    flag!(bus_master, 2);
    // Allow device to response to memory space access
    flag!(memory_space, 1);
    // Allow device to response to I/O space access
    flag!(io_space, 0);
}

#[derive(Debug, Clone, Copy)]
pub struct BISTRegister(u8);

impl BISTRegister {
    flag!(bist_capable, 7);
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PciCommonHeader {
    vendor: VendorID,
    device: DeviceID,
    command: CommandRegister,
    status: StatusRegister,
    revision: u8,
    prog_if: ProgrammingInterface,
    subclass: SubClass,
    class_code: ClassCode,
    cache_size: u8,
    latency_timer: u8,
    header_type: HeaderType,
    bist: BISTRegister,
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct MemoryBaseAddressRegister(u32);

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct IOBaseAddressRegister(u32);

#[derive(Clone, Copy)]
pub union BaseAddressRegister {
    memory: MemoryBaseAddressRegister,
    io: IOBaseAddressRegister,
}

pub enum BaseAddressRegisterType {
    Memory,
    IO,
}

impl BaseAddressRegister {
    pub fn identify(&self) -> BaseAddressRegisterType {
        // Doesn't matter which variant we take, they are both u32.
        unsafe {
            if self.memory.0 & 1 == 0 {
                return BaseAddressRegisterType::Memory;
            } else {
                return BaseAddressRegisterType::IO;
            }
        }
    }
}

impl core::fmt::Debug for BaseAddressRegister {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "Memory: {:?}", unsafe { self.memory })?;
        writeln!(f, "I/O: {:?}", unsafe { self.io })
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct GeneralDeviceHeader {
    common: PciCommonHeader,
    bar0: BaseAddressRegister,
    bar1: BaseAddressRegister,
    bar2: BaseAddressRegister,
    bar3: BaseAddressRegister,
    bar4: BaseAddressRegister,
    bar5: BaseAddressRegister,
    cardbus_cis_ptr: u32,
    subsystem_vendor_id: u16,
    subsystem_id: u16,
    expansion_rom_base: u32,
    capabilities_ptr: u8,
    _reserved0: u8,
    _reserved1: u16,
    _reserved2: u32,
    interrupt_line: u8,
    interrupt_pin: u8,
    min_grant: u8,
    max_latency: u8,
}

impl GeneralDeviceHeader {
    pub fn empty_from_common(common: PciCommonHeader) -> Self {
        Self {
            common,
            bar0: unsafe { core::mem::transmute(0) },
            bar1: unsafe { core::mem::transmute(0) },
            bar2: unsafe { core::mem::transmute(0) },
            bar3: unsafe { core::mem::transmute(0) },
            bar4: unsafe { core::mem::transmute(0) },
            bar5: unsafe { core::mem::transmute(0) },
            cardbus_cis_ptr: 0,
            subsystem_vendor_id: 0,
            subsystem_id: 0,
            expansion_rom_base: 0,
            capabilities_ptr: 0,
            _reserved0: 0,
            _reserved1: 0,
            _reserved2: 0,
            interrupt_line: 0,
            interrupt_pin: 0,
            min_grant: 0,
            max_latency: 0,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Pci2PciBridge {
    common: PciCommonHeader,
    bar0: BaseAddressRegister,
    bar1: BaseAddressRegister,
    primary_bus_number: u8,
    secondary_bus_number: u8,
    subordinate_bus_number: u8,
    secondary_latency_timer: u8,
    io_base: u8,
    io_limit: u8,
    secondary_status: u16,
    memory_base: u16,
    memory_limit: u16,
    prefetchable_memory_base: u16,
    prefetchable_memory_limit: u16,
    prefetchable_base_upper: u32,
    prefetchable_limit_upper: u32,
    io_base_upper: u16,
    io_limit_upper: u16,
    capabilities_ptr: u8,
    expansion_rom_base: u32,
    interrupt_line: u8,
    interrupt_pin: u8,
    bridge_control: u8,
}

pub union PciDevice {
    pub common: PciCommonHeader,
    pub general_device: GeneralDeviceHeader,
    pub pci2pci_bridge: Pci2PciBridge,
}

impl PciDevice {
    pub fn identify(&self) -> HeaderType {
        // Doesn't matter which one we choose, common is the same for all of them in the same offset.
        unsafe { self.common.header_type }
    }

    pub fn common(&self) -> &PciCommonHeader {
        unsafe { &self.common }
    }
}

pub fn scan_pci() -> Result<Vec<PciDevice, PhysicalPageAllocator>, PciConfigurationError> {
    let mut v: Vec<PciDevice, PhysicalPageAllocator> =
        Vec::with_capacity_in(64, unsafe { ALLOCATOR.assume_init_ref().clone() });
    for bus in 0..=255 {
        for device in 0..32 {
            let common = match PciConfigurationCycle::read_common_header(bus, device) {
                Ok(h) => h,
                Err(PciConfigurationError::NonExistentDevice(_, _)) => return Ok(v),
                Err(e) => return Err(e),
            };
            match common.header_type {
                HeaderType::GeneralDevice => {
                    v.push_within_capacity(PciDevice {
                        general_device: PciConfigurationCycle::read_general_device_header(
                            bus, device, common,
                        ),
                    })
                    .unwrap_or_else(|_| panic!("PCI Vec cannot push any more items"));
                }
                _ => {
                    v.push_within_capacity(PciDevice { common })
                        .unwrap_or_else(|_| panic!("PCI Vec cannot push any more items"));
                }
            }
        }
    }
    Ok(v)
}
