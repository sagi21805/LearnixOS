use common::{
    enums::{ClassCode, DeviceID, Port, ProgrammingInterface, SubClass, VendorID},
    error::PciConfigurationError,
    flag,
};
use cpu_utils::instructions::port::PortExt;

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
        let header_type = HeaderType(((value >> 16) & 0xff) as u8);
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
}

#[derive(Debug)]
pub struct StatusRegister(u16);

impl StatusRegister {
    flag!(detected_parity_error, 15);
    flag!(signaled_system_error, 14);
    flag!(received_master_abort, 13);
    flag!(received_target_abort, 12);
    flag!(signaled_target_abort, 11);
    // bits 9-10 devsel
    flag!(master_data_parity_error, 8);
    flag!(fast_backup_capable, 7);
    flag!(capable_66mhz, 5);
    flag!(capabilities_list, 4);
    flag!(interrupt_status, 3);
}

#[derive(Debug)]
pub struct CommandRegister(u16);

#[derive(Debug)]
pub struct HeaderType(u8);

#[derive(Debug)]
pub struct BISTRegister(u8);

#[derive(Debug)]
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
