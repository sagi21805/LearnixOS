#![no_std]
#![feature(const_trait_impl)]
#![feature(const_convert)]
#![feature(const_result_trait_fn)]

extern crate alloc;

use alloc::{alloc::Allocator, vec::Vec};
use bitfields::bitfields;
use common::enums::{
    HeaderType, PciDeviceType, Port, VendorDevice, VendorID,
};
use core::{
    array,
    fmt::{self, Display, Formatter},
    mem::transmute,
};
use x86::instructions::port::PortExt;

const DEVICES_PER_BUS: u8 = 32;
const FUNCTIONS_PER_DEVICE: u8 = 8;
const HEADER_DWORDS: usize = 16;

/// Reads every PCI function on every bus into a `Vec` in `allocator`,
/// ordered by address.
///
/// `scan_pci(allocator) -> [00:00.0, 00:01.0, 00:1f.0, 00:1f.2, ..]`
///
/// # Panics
///
/// When `allocator` can't hold the list.
///
/// # Examples
///
/// ```no_run
/// let devices = pci::scan_pci(std::alloc::Global);
/// for device in &devices {
///     println!("{device}");
/// }
/// ```
pub fn scan_pci<A: Allocator>(allocator: A) -> Vec<PciDevice, A> {
    let mut devices = Vec::new_in(allocator);
    for bus in 0..=u8::MAX {
        for device in 0..DEVICES_PER_BUS {
            let Some(first) = PciDevice::read(PciAddress {
                bus,
                device,
                function: 0,
            }) else {
                continue;
            };
            let multifunction =
                first.header.common().header_type.is_multifunction();
            devices.push(first);
            if multifunction {
                devices.extend((1..FUNCTIONS_PER_DEVICE).filter_map(
                    |function| {
                        PciDevice::read(PciAddress {
                            bus,
                            device,
                            function,
                        })
                    },
                ));
            }
        }
    }
    devices
}

/// One PCI function and a copy of its configuration header.
pub struct PciDevice {
    pub header: PciDeviceHeader,
    pub address: PciAddress,
}

impl PciDevice {
    /// Reads the function at `address`, or returns `None` when the slot
    /// is empty.
    pub fn read(address: PciAddress) -> Option<Self> {
        let dword = |index: u8| {
            // SAFETY: this kernel runs on one CPU, and no interrupt
            // handler uses the configuration ports.
            unsafe { address.register(index * 4).read() }
        };
        // The vendor ID is the low half of the first dword, and an empty
        // slot reads as all ones.
        if dword(0) as u16 == VendorID::NonExistent as u16 {
            return None;
        }
        Some(PciDevice {
            header: PciDeviceHeader::from_dwords(array::from_fn(
                |index| dword(index as u8),
            )),
            address,
        })
    }
}

impl Display for PciDevice {
    /// `00:1f.2 8086:2922 01:06:01 MassStorageController`
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let common = self.header.common();
        let VendorDevice { vendor, device } = common.vendor_device;
        let device_type = common.device_type;
        write!(
            f,
            "{} {vendor:04x}:{device:04x} {:02x}:{:02x}:{:02x} ",
            self.address,
            device_type.class,
            device_type.subclass,
            device_type.prog_if,
        )?;
        match device_type.class_code() {
            Ok(class) => write!(f, "{class:?}"),
            Err(_) => write!(f, "Unknown"),
        }
    }
}

/// Where a PCI function sits in configuration space.
#[derive(Debug, Clone, Copy)]
pub struct PciAddress {
    pub bus: u8,
    pub device: u8,
    pub function: u8,
}

impl PciAddress {
    /// The cycle that selects the register at `offset` of this function.
    pub const fn register(self, offset: u8) -> PciConfigurationCycle {
        PciConfigurationCycle::new_unchecked(
            self.bus,
            self.device,
            self.function,
            offset,
        )
    }
}

impl Display for PciAddress {
    /// `00:1f.2`
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:02x}:{:02x}.{}", self.bus, self.device, self.function)
    }
}

#[bitfields]
pub struct PciConfigurationCycle {
    offset: B8,
    function: B3,
    device: B5,
    bus: B8,
    reserved: B7,
    enable: B1,
}

impl PciConfigurationCycle {
    /// Doesn't check that `device` and `function` are in range, or that
    /// `offset` is aligned to a dword.
    pub const fn new_unchecked(
        bus: u8,
        device: u8,
        function: u8,
        offset: u8,
    ) -> PciConfigurationCycle {
        PciConfigurationCycle::new()
            .bus(bus)
            .device(device)
            .function(function)
            .offset(offset)
            .enable(true)
    }

    /// Reads the register this cycle selects.
    ///
    /// # Safety
    ///
    /// Nothing else may use the configuration ports until this returns,
    /// because selecting the register and reading it are two port
    /// accesses.
    pub unsafe fn read(self) -> u32 {
        unsafe {
            Port::PciConfigAddress.outl(self.0);
            Port::PciConfigData.inl()
        }
    }

    /// Writes `val` to the register this cycle selects.
    ///
    /// # Safety
    ///
    /// Nothing else may use the configuration ports until this returns,
    /// because selecting the register and writing it are two port
    /// accesses. Writing can reconfigure the device, so the caller must
    /// know what it does with `val`.
    pub unsafe fn write(self, val: u32) {
        unsafe {
            Port::PciConfigAddress.outl(self.0);
            Port::PciConfigData.outl(val);
        }
    }
}

#[bitfields]
pub struct StatusRegister {
    reserved0: B3,
    interrupt_status: B1,
    capabilities_list: B1,
    capable_66mhz: B1,
    reserved1: B1,
    fast_back2back_capable: B1,
    master_data_parity_error: B1,
    #[flag(r)]
    // ReadOnly bits which represent the slowest time device
    // will assert DEVSEL 00 -> fast 01 -> medium 02 ->
    // slow bits 9-10 devsel
    devsel_time: B2,
    signaled_target_abort: B1,
    received_target_abort: B1,
    received_master_abort: B1,
    signaled_system_error: B1,
    detected_parity_error: B1,
}

#[bitfields]
pub struct CommandRegister {
    // Allow device to response to I/O space access
    io_space: B1,
    // Allow device to response to memory space access
    memory_space: B1,
    // Allow device to behave as bus master.
    bus_master: B1,
    // Allow device to monitor special cycle, otherwise
    // ignore them
    special_cycles: B1,
    // Allow device to generate memory writes and invalidate
    // commands. Otherwise memory write command must be
    // used.
    memory_write_inval_enable: B1,
    // Allow device to listen to vga palette writes, and
    // copy them for it own use (Legacy)
    vga_palette_snoop: B1,
    // If enabled device will take its normal action on
    // parity error otherwise it will set bit 15 on
    // status register, and will continue operation as
    // normal
    parity_error_response: B1,
    // SERR driver is enabled
    serr_enable: B1,
    // Allow device to generate back to back transactions
    fast_back2back_enable: B1,
    // Disable interrupts for this device
    interrupt_disable: B1,
}

#[bitfields]
pub struct BISTRegister {
    reserved: B7,
    bist_capable: B1,
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

#[derive(Debug, Clone, Copy)]
pub struct MemoryBaseAddressRegister(u32);

#[derive(Debug, Clone, Copy)]
pub struct IOBaseAddressRegister(u32);

#[repr(C)]
#[derive(Clone, Copy)]
pub union BaseAddressRegister {
    pub memory: MemoryBaseAddressRegister,
    pub io: IOBaseAddressRegister,
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

#[repr(C)]
pub union PciDeviceHeader {
    pub common: PciCommonHeader,
    pub general_device: GeneralDeviceHeader,
    pub pci2pci_bridge: Pci2PciBridge,
}

impl PciDeviceHeader {
    /// The header whose configuration space holds `dwords`.
    fn from_dwords(dwords: [u32; HEADER_DWORDS]) -> Self {
        // SAFETY: a header holds only integers, so any bits are valid.
        unsafe { transmute::<[u32; HEADER_DWORDS], Self>(dwords) }
    }

    pub fn identify(&self) -> HeaderType {
        // Doesn't matter which one we choose, common is the
        // same for all of them in the same offset.
        unsafe { self.common.header_type }
    }

    pub fn common(&self) -> &PciCommonHeader { unsafe { &self.common } }
}
