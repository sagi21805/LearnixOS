extern crate alloc;

use crate::{
    drivers::ata::ahci::AHCIBaseAddress,
    memory::allocators::page_allocator::allocator::PhysicalPageAllocator,
};
use alloc::vec::Vec;
use common::enums::{
    CascadedPicInterruptLine, ClassCode, DeviceID, HeaderType,
    PciDeviceType, Port, ProgrammingInterface, SubClass, VendorDevice,
    VendorID,
};
use cpu_utils::instructions::port::PortExt;
use learnix_macros::flag;

#[derive(Debug, Clone, Copy)]
pub struct PciConfigurationCycle(u32);

impl PciConfigurationCycle {
    flag!(enable, 31);

    /// Not checking device max numb er, function max number
    /// and offset alignment
    pub const fn new_unchecked(
        bus: u8,
        device: u8,
        function: u8,
        offset: u8,
    ) -> Self {
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

    pub fn read_common_header(
        bus: u8,
        device: u8,
        function: u8,
    ) -> PciCommonHeader {
        let mut uninit = PciCommonHeader::empty();
        let uninit_ptr =
            &mut uninit as *mut PciCommonHeader as usize as *mut u32;
        for offset in
            (0..size_of::<PciCommonHeader>()).step_by(size_of::<u32>())
        {
            unsafe {
                let header_data = Self::new_unchecked(
                    bus,
                    device,
                    function,
                    offset as u8,
                )
                .read();
                uninit_ptr.byte_add(offset).write_volatile(header_data);
            }
        }
        // uninit.bus = bus;
        // uninit.device = device;
        // uninit.function = function;
        uninit
    }

    pub fn read_pci_device(
        bus: u8,
        device: u8,
        function: u8,
        common: PciCommonHeader,
    ) -> PciDevice {
        let mut uninit = PciDeviceHeader { common };
        let uninit_ptr =
            &mut uninit as *mut PciDeviceHeader as usize as *mut u32;
        for offset in ((size_of::<PciCommonHeader>())
            ..size_of::<PciDeviceHeader>())
            .step_by(size_of::<u32>())
        {
            unsafe {
                let header_data = Self::new_unchecked(
                    bus,
                    device,
                    function,
                    offset as u8,
                )
                .read();
                uninit_ptr.byte_add(offset).write_volatile(header_data);
            }
        }
        PciDevice {
            header: uninit,
            bus,
            device,
            function,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct StatusRegister(u16);

impl StatusRegister {
    // Device detected a parity error
    flag!(detected_parity_error, 15);
    // Device asserted SERR
    flag!(signaled_system_error, 14);
    //Master  Device transaction was terminated by master
    // abort
    flag!(received_master_abort, 13);
    // Master Device transaction was terminated by target
    // abort
    flag!(received_target_abort, 12);
    // Target device terminated by target abort
    flag!(signaled_target_abort, 11);

    // ReadOnly bits which represent the slowest time device
    // will assert DEVSEL 00 -> fast 01 -> medium 02 ->
    // slow bits 9-10 devsel

    // Parity error detected and handled
    flag!(master_data_parity_error, 8);
    // Device is capable for fast back transaction not from
    // the same agent
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
    // If enabled device will take its normal action on
    // parity error otherwise it will set bit 15 on
    // status register, and will continue operation as
    // normal
    flag!(parity_error_response, 6);
    // Allow device to listen to vga palette writes, and
    // copy them for it own use (Legacy)
    flag!(vga_palette_snoop, 5);
    // Allow device to generate memory writes and invalidate
    // commands. Otherwise memory write command must be
    // used.
    flag!(memory_write_inval_enable, 4);
    // Allow device to monitor special cycle, otherwise
    // ignore them
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
    pub vendor_device: VendorDevice,
    pub command: CommandRegister,
    pub status: StatusRegister,
    pub revision: u8,
    pub device_type: PciDeviceType,
    pub cache_size: u8,
    pub latency_timer: u8,
    pub header_type: HeaderType,
    pub bist: BISTRegister,
}

impl PciCommonHeader {
    pub fn empty() -> Self {
        PciCommonHeader {
            vendor_device: VendorDevice {
                vendor: VendorID::NonExistent,
                device: DeviceID { none: () },
            },
            command: CommandRegister(0),
            status: StatusRegister(0),
            revision: 0,
            device_type: PciDeviceType {
                prog_if: ProgrammingInterface { none: () },
                subclass: SubClass { none: () },
                class: ClassCode::Unassigned,
            },
            cache_size: 0,
            latency_timer: 0,
            header_type: HeaderType::GeneralDevice,
            bist: BISTRegister(0),
            // bus: 0xff,
            // device: 0xff,
            // function: 0xff,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MemoryBaseAddressRegister(u32);

#[derive(Debug, Clone, Copy)]
pub struct IOBaseAddressRegister(u32);

#[derive(Clone, Copy)]
pub union BaseAddressRegister {
    pub memory: MemoryBaseAddressRegister,
    pub io: IOBaseAddressRegister,
    pub abar: AHCIBaseAddress,
}

#[derive(PartialEq, Eq)]
pub enum BaseAddressRegisterType {
    Memory,
    IO,
}

pub enum BaseAddressRegisterSize {
    Bit32 = 0,
    Reserved = 1,
    Bit64 = 2,
}

impl BaseAddressRegister {
    pub fn identify(&self) -> BaseAddressRegisterType {
        // Doesn't matter which variant we take, they are
        // both u32.
        unsafe {
            if self.memory.0 & 1 == 0 {
                BaseAddressRegisterType::Memory
            } else {
                BaseAddressRegisterType::IO
            }
        }
    }

    pub fn is_64bit(&self) -> bool {
        self.identify() == BaseAddressRegisterType::Memory
            && unsafe {
                self.memory.0 & BaseAddressRegisterSize::Bit64 as u32 != 0
            }
    }

    pub fn address(&self) -> usize {
        if !self.is_64bit() {
            (unsafe { self.io.0 } & 0xfffffff0) as usize
        } else {
            unimplemented!("Still didn't implemented 64bit addresses")
        }
    }
}

impl core::fmt::Debug for BaseAddressRegister {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "Memory: {:x?}", unsafe { self.memory })?;
        writeln!(f, "I/O: {:x?}", unsafe { self.io })
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct GeneralDeviceHeader {
    pub common: PciCommonHeader,
    pub bar0: BaseAddressRegister,
    pub bar1: BaseAddressRegister,
    pub bar2: BaseAddressRegister,
    pub bar3: BaseAddressRegister,
    pub bar4: BaseAddressRegister,
    pub bar5: BaseAddressRegister,
    pub cardbus_cis_ptr: u32,
    pub subsystem_vendor_id: u16,
    pub subsystem_id: u16,
    pub expansion_rom_base: u32,
    pub capabilities_ptr: u8,
    pub _reserved0: u8,
    pub _reserved1: u16,
    pub _reserved2: u32,
    pub interrupt_line: u8,
    pub interrupt_pin: u8,
    pub min_grant: u8,
    pub max_latency: u8,
}

impl GeneralDeviceHeader {
    pub fn empty_from_common(common: PciCommonHeader) -> Self {
        Self {
            common,
            bar0: unsafe {
                core::mem::transmute::<i32, BaseAddressRegister>(0)
            },
            bar1: unsafe {
                core::mem::transmute::<i32, BaseAddressRegister>(0)
            },
            bar2: unsafe {
                core::mem::transmute::<i32, BaseAddressRegister>(0)
            },
            bar3: unsafe {
                core::mem::transmute::<i32, BaseAddressRegister>(0)
            },
            bar4: unsafe {
                core::mem::transmute::<i32, BaseAddressRegister>(0)
            },
            bar5: unsafe {
                core::mem::transmute::<i32, BaseAddressRegister>(0)
            },
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
    bridge_control: u16,
}

pub union PciDeviceHeader {
    pub common: PciCommonHeader,
    pub general_device: GeneralDeviceHeader,
    pub pci2pci_bridge: Pci2PciBridge,
}

impl PciDeviceHeader {
    pub fn identify(&self) -> HeaderType {
        // Doesn't matter which one we choose, common is the
        // same for all of them in the same offset.
        unsafe { self.common.header_type }
    }

    pub fn common(&self) -> &PciCommonHeader {
        unsafe { &self.common }
    }
}

pub struct PciDevice {
    pub header: PciDeviceHeader,
    pub bus: u8,
    pub device: u8,
    pub function: u8,
}

impl PciDevice {
    pub fn enable_interrupts(&self, irq: CascadedPicInterruptLine) {}
}

// pub fn scan_pci() -> Vec<PciDevice, PhysicalPageAllocator> {
//     let mut v: Vec<PciDevice, PhysicalPageAllocator> =
//         Vec::with_capacity_in(64, unsafe {
//             ALLOCATOR.assume_init_ref().clone()
//         });
//     for bus in 0..=255 {
//         for device in 0..32 {
//             let common =
//                 PciConfigurationCycle::read_common_header(bus, device,
// 0);             if common.vendor_device.vendor == VendorID::NonExistent
// {                 continue;
//             }
//             v.push_within_capacity(
//                 PciConfigurationCycle::read_pci_device(
//                     bus, device, 0, common,
//                 ),
//             )
//             .unwrap_or_else(|_| {
//                 panic!("PCI Vec cannot push any more items")
//             });
//             if !common.header_type.is_multifunction() {
//                 continue;
//             }
//             for function in 1..8 {
//                 let common = PciConfigurationCycle::read_common_header(
//                     bus, device, function,
//                 );
//                 if common.vendor_device.vendor == VendorID::NonExistent
// {                     continue;
//                 }
//                 v.push_within_capacity(
//                     PciConfigurationCycle::read_pci_device(
//                         bus, device, function, common,
//                     ),
//                 )
//                 .unwrap_or_else(|_| {
//                     panic!("PCI Vec cannot push any more items")
//                 });
//             }
//         }
//     }
//     v
// }
