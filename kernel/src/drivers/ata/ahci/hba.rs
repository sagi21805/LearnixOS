/// AHCI implementation for the learnix operating system
///
/// Implemented directly from https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/serial-ata-ahci-spec-rev1-3-1.pdf
extern crate alloc;

use core::{fmt::Debug, num::NonZero, panic};

use common::{
    address_types::PhysicalAddress,
    constants::{
        PHYSICAL_MEMORY_OFFSET, REGULAR_PAGE_ALIGNMENT, REGULAR_PAGE_SIZE,
    },
    enums::{
        AtaCommand, Color, DeviceDetection, DeviceType,
        InterfaceCommunicationControl, InterfaceInitialization,
        InterfacePowerManagement, InterfaceSpeed,
        InterfaceSpeedRestriction, PageSize,
    },
    error::{AhciError, ConversionError, DiagnosticError, HbaError},
    read_volatile,
    volatile::Volatile,
    write_volatile,
};
use cpu_utils::structures::paging::PageEntryFlags;
use learnix_macros::{flag, ro_flag, rw1_flag, rwc_flag};
use num_enum::UnsafeFromPrimitive;
use strum::IntoEnumIterator;

use crate::{
    alloc_pages,
    drivers::{
        ata::ahci::{
            DmaSetup, Fis, IdentityPacketData, PioSetupD2H, RegisterD2H,
            RegisterH2D, SetDeviceBits,
        },
        vga_display::color_code::ColorCode,
    },
    eprintln,
    memory::allocators::extensions::PhysicalAddressExt,
    print, println,
};

#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct AHCIBaseAddress(pub u32);

/// CAP
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct HBACapabilities(pub u32);

impl HBACapabilities {
    // Support 64bit addressing
    ro_flag!(s64a, 31);

    // Support native command queuing
    ro_flag!(snqc, 30);

    // Support s-notification register
    ro_flag!(ssntf, 29);

    // Support mechanical presence switch
    ro_flag!(smps, 28);

    // Support staggered Spin-up
    ro_flag!(sss, 27);

    // Support aggressive link power management
    ro_flag!(salp, 26);

    // Support activity lead
    ro_flag!(sal, 25);

    pub fn interface_speed(&self) -> InterfaceSpeed {
        unsafe {
            core::mem::transmute(
                (((read_volatile!(self.0)) >> 20) & 0xf) as u8,
            )
        }
    }

    // Support AHCI mode only
    ro_flag!(sam, 18);

    // Support port multiplier
    ro_flag!(spm, 17);

    // Frame Information Structure based switching supported
    ro_flag!(fbss, 16);

    // Programmed I/O multiple Data request block
    ro_flag!(pmd, 15);

    // Slumber state capable
    ro_flag!(ssc, 15);

    // Partial state capable
    ro_flag!(psc, 14);

    // This value is between 1 and 32
    pub fn number_of_commands(&self) -> u8 {
        (((read_volatile!(self.0)) >> 8) & 0x1f) as u8
    }

    // Command completion coalescing supported
    ro_flag!(cccs, 7);

    // Enclosure management supported
    ro_flag!(ems, 6);

    // Support external SATA
    ro_flag!(sxs, 5);

    /// Returns the number of ports implemented
    pub fn number_of_ports(&self) -> u8 {
        (read_volatile!(self.0) & 0x1f) as u8
    }
}

/// GHC
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct GlobalHostControl(pub u32);

impl GlobalHostControl {
    // AHCI Enable. Must be set for the HBA to operate in AHCI mode.
    flag!(ae, 31);

    // MSI Revert to Single Message
    // 1.3.1)
    flag!(mrsm, 2);

    // Interrupt Enable
    flag!(ie, 1);

    // HBA Reset
    flag!(hr, 0);
}

/// IS
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct InterruptStatus(pub u32);

impl InterruptStatus {
    // Port Interrupt Pending Status. Corresponds to bits of the PI
    // register. Cleared by writing a '1' to the corresponding bit.
    pub fn is_port_pending(&self, port_num: u8) -> bool {
        (read_volatile!(self.0) & (1 << port_num)) != 0
    }

    pub fn clear(&mut self, port_num: u8) {
        write_volatile!(self.0, read_volatile!(self.0) | (1 << port_num));
    }

    pub fn clear_all(&mut self) {
        write_volatile!(self.0, 0);
    }

    // RWC flag for Port 0 Interrupt Pending Status
    rwc_flag!(ip01, 1);
    rwc_flag!(ip02, 2);
    rwc_flag!(ip03, 3);
    rwc_flag!(ip04, 4);
    rwc_flag!(ip05, 5);
    rwc_flag!(ip06, 6);
    rwc_flag!(ip07, 7);
    rwc_flag!(ip08, 8);
    rwc_flag!(ip09, 9);
    rwc_flag!(ip10, 10);
    rwc_flag!(ip11, 11);
    rwc_flag!(ip12, 12);
    rwc_flag!(ip13, 13);
    rwc_flag!(ip14, 14);
    rwc_flag!(ip15, 15);
    rwc_flag!(ip16, 16);
    rwc_flag!(ip17, 17);
    rwc_flag!(ip18, 18);
    rwc_flag!(ip19, 19);
    rwc_flag!(ip20, 20);
    rwc_flag!(ip21, 21);
    rwc_flag!(ip22, 22);
    rwc_flag!(ip23, 23);
    rwc_flag!(ip24, 24);
    rwc_flag!(ip25, 25);
    rwc_flag!(ip26, 26);
    rwc_flag!(ip27, 27);
    rwc_flag!(ip28, 28);
    rwc_flag!(ip29, 29);
    rwc_flag!(ip30, 30);
    rwc_flag!(ip31, 31);
}

// PI
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct PortsImplemented(pub u32);

impl PortsImplemented {
    // Port i is Implemented (P[i])
    pub fn is_port_implemented(&self, port_num: u8) -> bool {
        (read_volatile!(self.0) & (1 << port_num)) != 0
    }
}

// VS
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct Version(pub u32);

impl Version {
    // Major Version Number (Bits 31:16)
    pub fn major_version(&self) -> u16 {
        (read_volatile!(self.0) >> 16) as u16
    }

    // Minor Version Number (Bits 15:0)
    pub fn minor_version(&self) -> u16 {
        (read_volatile!(self.0) & 0xffff) as u16
    }
}

/// CCC_CTL
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct CommandCompletionCoalescingControl(pub u32);

impl CommandCompletionCoalescingControl {
    pub fn interrupt_time_ms(&self) -> u16 {
        ((read_volatile!(self.0) >> 16) & 0xffff) as u16
    }

    // Command Completions (CC): Number of command completions necessary to
    // cause a CCC interrupt
    pub fn command_completions(&self) -> u8 {
        ((read_volatile!(self.0) >> 8) & 0xff) as u8
    }

    flag!(enable, 0);
}

/// CCC_PORTS
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct CommandCompletionCoalescingPorts(pub u32);

impl CommandCompletionCoalescingPorts {
    pub fn set_port(&mut self, port_num: u8) {
        write_volatile!(self.0, read_volatile!(self.0) | (1 << port_num))
    }

    pub fn unset(&mut self, port_num: u8) {
        self.0 &= !(1 << port_num)
    }

    flag!(prt01, 1);
    flag!(prt02, 2);
    flag!(prt03, 3);
    flag!(prt04, 4);
    flag!(prt05, 5);
    flag!(prt06, 6);
    flag!(prt07, 7);
    flag!(prt08, 8);
    flag!(prt09, 9);
    flag!(prt10, 10);
    flag!(prt11, 11);
    flag!(prt12, 12);
    flag!(prt13, 13);
    flag!(prt14, 14);
    flag!(prt15, 15);
    flag!(prt16, 16);
    flag!(prt17, 17);
    flag!(prt18, 18);
    flag!(prt19, 19);
    flag!(prt20, 20);
    flag!(prt21, 21);
    flag!(prt22, 22);
    flag!(prt23, 23);
    flag!(prt24, 24);
    flag!(prt25, 25);
    flag!(prt26, 26);
    flag!(prt27, 27);
    flag!(prt28, 28);
    flag!(prt29, 29);
    flag!(prt30, 30);
    flag!(prt31, 31);
}

/// EM_LOC
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct EnclosureManagementLocation(pub u32);

impl EnclosureManagementLocation {
    pub fn dword_offset_from_abar(&self) -> usize {
        (read_volatile!(self.0) >> 16) as usize
    }

    /// ZERO is invalid
    /// TODO understand how to check if i have both receive and transmit
    pub fn buffet_size(&self) -> Option<NonZero<usize>> {
        NonZero::new((read_volatile!(self.0) & 0xffff) as usize)
    }
}

/// EM_CTL
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct EnclosureManagementControl(pub u32);

impl EnclosureManagementControl {
    // Port multiplier support
    ro_flag!(pm, 27);

    // Activity LED hardware driven
    ro_flag!(alhd, 26);

    // Transmit only
    ro_flag!(xmt, 25);

    // Single message buffer
    ro_flag!(smb, 24);

    // SGPIO Enclosure management messages
    ro_flag!(sgpio, 19);

    // SES2 Enclosure management massages
    ro_flag!(ses2, 18);

    // SAF-TE Enclosure management massages
    ro_flag!(safte, 17);

    // Led message type
    ro_flag!(led, 16);

    // Reset
    rw1_flag!(reset, 9);

    // Transmit massage
    rw1_flag!(tm, 8);

    // Message received
    rwc_flag!(mr, 0);
}

/// CAP2
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct HostCapabilitiesExtended(pub u32);

impl HostCapabilitiesExtended {
    // DevSleep entrance from slumber only
    ro_flag!(deso, 5);

    // Aggressive device sleep management
    ro_flag!(sadm, 4);

    // Support device sleep
    ro_flag!(sds, 3);

    // Automatic partial to slumber transitions
    ro_flag!(apst, 2);

    // NVMHCI present
    ro_flag!(nvmp, 1);

    // Bios/OS handoff
    ro_flag!(boh, 0);
}

// BOHC
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct BiosOsControlStatus(pub u32);

impl BiosOsControlStatus {
    // Bios Busy
    flag!(bb, 4);

    // OS ownership change
    rwc_flag!(ooc, 3);

    // SMI on OS ownership change enable
    flag!(sooe, 2);

    // OS Owned semaphore
    flag!(oos, 1);

    // BIOS owned semaphore
    flag!(bos, 0);
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct GenericHostControl {
    pub cap: HBACapabilities,
    pub ghc: GlobalHostControl,
    pub is: InterruptStatus,
    pub pi: PortsImplemented,
    pub vs: Version,
    pub ccc_ctl: CommandCompletionCoalescingControl,
    pub ccc_ports: CommandCompletionCoalescingPorts,
    pub em_loc: EnclosureManagementLocation,
    pub em_ctl: EnclosureManagementControl,
    pub cap_ext: HostCapabilitiesExtended,
    pub bohc: BiosOsControlStatus,
}

#[repr(C)]
pub struct VendorSpecificRegisters {
    _reserved: [u8; 0x74],
}

/// Port X Interrupt status
#[repr(transparent)]
pub struct PortInterruptStatus(pub u32);

impl PortInterruptStatus {
    // Cold port detect status
    rwc_flag!(cpds, 31);

    // Task file error status
    rwc_flag!(tfes, 30);

    // Host bust fatal error status
    rwc_flag!(hbfs, 29);

    // Host Bus Data Error Status
    rwc_flag!(hbds, 28);

    // Interface Fatal Error Status
    rwc_flag!(ifs, 27);

    // Interface Non-fatal Error Status
    rwc_flag!(infs, 26);

    // Overflow Status
    rwc_flag!(ofs, 24);

    // Incorrect Port Multiplier Status
    rwc_flag!(ipms, 23);

    // PhyRdy Change Status
    ro_flag!(prcs, 22);

    // Device Mechanical Presence Status
    rwc_flag!(dmps, 7);

    // Port Connect Change Status
    ro_flag!(pcs, 6);

    // Descriptor Processed
    rwc_flag!(dps, 5);

    // Unknown FIS Interrupt
    ro_flag!(ufs, 4);

    // Set Device Bits Interrupt
    rwc_flag!(sdbs, 3);

    // DMA Setup FIS Interrupt
    rwc_flag!(dss, 2);

    // PIO Setup FIS Interrupt
    rwc_flag!(pss, 1);

    // Device to Host Register FIS Interrupt
    rwc_flag!(dhrs, 0);

    pub fn clear_pending_interrupts(&mut self) {
        write_volatile!(self.0, u32::MAX);
    }
}

/// Port X Interrupt Enable
#[repr(transparent)]
pub struct InterruptEnable(pub u32);

impl InterruptEnable {
    // Cold Presence Detect Enable
    flag!(cpde, 31);

    // Task File Error Enable
    flag!(tfee, 30);

    // Host Bus Fatal Error Enable
    flag!(hbfe, 29);

    // Host Bus Data Error Enable
    flag!(hbde, 28);

    // Interface Fatal Error Enable
    flag!(ife, 27);

    // Interface Non-fatal Error Enable
    flag!(infe, 26);

    // Overflow Enable
    flag!(ofe, 24);

    // Incorrect Port Multiplier Enable
    flag!(ipme, 23);

    // PhyRdy Change Interrupt Enable
    flag!(prce, 22);

    // Device Mechanical Presence Enable
    flag!(dmpe, 7);

    // Port Change Interrupt Enable
    flag!(pce, 6);

    // Descriptor Processed Interrupt Enable
    flag!(dpe, 5);

    // Unknown FIS Interrupt Enable
    flag!(ufe, 4);

    // Set Device Bits FIS Interrupt Enable
    flag!(sdbe, 3);

    // DMA Setup FIS Interrupt Enable
    flag!(dse, 2);

    // PIO Setup FIS Interrupt Enable
    flag!(pse, 1);

    // Device to Host Register FIS Interrupt Enable
    flag!(dhre, 0);
}

/// Port X Command and status
#[repr(transparent)]
pub struct CmdStatus(pub u32);

impl CmdStatus {
    pub fn set_icc(&mut self, icc: InterfaceCommunicationControl) {
        write_volatile!(self.0, read_volatile!(self.0) & !(0xf << 28));
        write_volatile!(
            self.0,
            read_volatile!(self.0) | (icc as u32) << 28
        );
    }

    // Aggressive Slumber / Partial
    flag!(asp, 27);

    // Aggressive Link Power Management Enable
    flag!(alpe, 26);

    // Drive LED on ATAPI Enable
    flag!(dlae, 25);

    // Device is ATAPI
    flag!(atapi, 24);

    // Automatic Partial to Slumber Transitions Enabled
    flag!(apste, 23);

    // FIS-based Switching Capable Port
    ro_flag!(fbscp, 22);

    // External SATA Port
    ro_flag!(esp, 21);

    // Cold Presence Detection
    ro_flag!(cpd, 20);

    // Mechanical Presence Switch Attached to Port
    ro_flag!(mpsp, 19);

    // Hot Plug Capable Port
    ro_flag!(hpcp, 18);

    // Port Multiplier Attached
    flag!(pma, 17);

    // Cold Presence State
    ro_flag!(cps, 16);

    // Command List Running
    ro_flag!(cr, 15);

    // FIS Receive Running
    ro_flag!(fr, 14);

    // Mechanical Presence Switch State
    ro_flag!(mpss, 13);

    pub fn get_current_cmd(&mut self) -> u32 {
        if !self.is_st() {
            return 0;
        }
        (read_volatile!(self.0) >> 8) & 0x1f
    }

    // FIS Receive Enable
    flag!(fre, 4);

    // Command List Override
    flag!(clo, 3);

    // Power On Device
    flag!(pod, 2);

    // Spin-Up Device
    flag!(sud, 1);

    // Start
    flag!(st, 0);

    pub fn start(&mut self) {
        while self.is_cr() {}
        self.set_fre();
        self.set_st();
    }

    pub fn stop(&mut self) {
        self.unset_st();
        let mut timeout = 0xfffff;
        loop {
            timeout -= 1;
            if timeout == 0 {
                panic!("Timeout ended on port stop");
            }
            if self.is_cr() {
                continue;
            } else {
                break;
            }
        }
        self.unset_fre();
        let mut timeout = 0xfffff;
        loop {
            timeout -= 1;
            if timeout == 0 {
                panic!("Timeout ended on port stop");
            }
            if self.is_fr() {
                continue;
            } else {
                break;
            }
        }
    }
}

/// Port x Task File Data
#[repr(transparent)]
pub struct TaskFileData(pub u32);

impl TaskFileData {
    // Indicates error during transfer
    ro_flag!(err, 0);

    // Indicates a data transfer request
    ro_flag!(drq, 3);

    // Indicates that the interface is busy
    ro_flag!(bsy, 7);

    pub fn error(&self) -> u8 {
        (read_volatile!(self.0) >> 8) as u8
    }
}

/// Port X Signature
#[repr(C)]
pub struct Signature {
    pub sector_count: u8,
    pub lba_low: u8,
    pub lba_mid: u8,
    pub lba_high: u8,
}

impl Signature {
    pub fn device_type(&self) -> Result<DeviceType, ConversionError<u32>> {
        DeviceType::try_from(u32::from_le_bytes([
            self.sector_count,
            self.lba_low,
            self.lba_mid,
            self.lba_high,
        ]))
    }
}

/// Port X SATA Status
#[repr(transparent)]
pub struct SataStatus(pub u32);

impl SataStatus {
    pub fn power(
        &self,
    ) -> Result<InterfacePowerManagement, ConversionError<u8>> {
        let power = ((read_volatile!(self.0) >> 8) & 0xf) as u8;
        InterfacePowerManagement::try_from(power)
    }

    pub fn speed(&self) -> InterfaceSpeed {
        let speed = ((read_volatile!(self.0) >> 4) & 0xf) as u8;
        unsafe { InterfaceSpeed::unchecked_transmute_from(speed) }
    }

    pub fn detection(
        &self,
    ) -> Result<DeviceDetection, ConversionError<u8>> {
        let detection = (read_volatile!(self.0) & 0xf) as u8;
        DeviceDetection::try_from(detection)
    }
}

/// Port X SATA control
#[repr(transparent)]
pub struct SataControl(pub u32);

impl SataControl {
    pub fn port_multiplier(&self) -> u8 {
        ((read_volatile!(self.0) >> 16) & 0xf) as u8
    }

    pub fn select_power_management(&self) -> u8 {
        ((read_volatile!(self.0) >> 12) & 0xf) as u8
    }

    flag!(devslp_disabled, 10);
    flag!(slumber_disabled, 9);
    flag!(partial_disabled, 8);

    pub fn max_speed(&self) -> InterfaceSpeedRestriction {
        let speed = ((read_volatile!(self.0) >> 4) & 0xf) as u8;
        unsafe {
            InterfaceSpeedRestriction::unchecked_transmute_from(speed)
        }
    }

    pub fn set_max_speed(&mut self, speed: InterfaceSpeed) {
        if speed != InterfaceSpeed::DevNotPresent {
            write_volatile!(self.0, read_volatile!(self.0) & !(0xf << 4));
            write_volatile!(
                self.0,
                read_volatile!(self.0) | (speed as u32) << 4
            );
        }
    }

    pub fn device_initialization(
        &self,
    ) -> Result<InterfaceInitialization, ConversionError<u8>> {
        InterfaceInitialization::try_from(
            (read_volatile!(self.0) & 0xf) as u8,
        )
    }

    // TODO THIS COMMAND ANY MAYBE OTHER SHOULD PROBABLY MOVE TO THE PORT
    // SETTING BECAUSE THEY REQUIRE PxCMD.st BIT TO BE SET WHILE THEY ARE
    // SET
    pub fn set_device_initialization(
        &mut self,
        init: InterfaceInitialization,
    ) {
        write_volatile!(self.0, read_volatile!(self.0) & !0xf);
        write_volatile!(self.0, read_volatile!(self.0) | init as u32);
    }
}

/// Port X SATA error
#[repr(transparent)]
pub struct SataError(pub u32);

impl SataError {
    pub fn diagnostic(&self) -> impl Iterator<Item = DiagnosticError> {
        let diagnostic_errors =
            ((read_volatile!(self.0) >> 16) & 0xffff) as u16;
        DiagnosticError::iter()
            .filter(move |n| *n as u16 & diagnostic_errors != 0)
    }

    pub fn error(&self) -> impl Iterator<Item = AhciError> {
        let ahci_error = (read_volatile!(self.0) & 0xffff) as u16;
        AhciError::iter().filter(move |n| *n as u16 & ahci_error != 0)
    }

    pub fn zero_error(&mut self) {
        write_volatile!(self.0, read_volatile!(self.0) & !0xffff)
    }
}

/// Port X Sata Active
#[repr(transparent)]
pub struct SataActive(pub u32);

/// Port X Command issue
#[repr(transparent)]
pub struct CmdIssue(pub Volatile<u32>);

impl CmdIssue {
    pub fn issue_cmd(&mut self, cmd: u8) {
        self.0.write(self.0.read() | 1 << cmd);
    }
}

/// Port X SATA Notification
#[repr(transparent)]
pub struct SataNotification(pub u32);

impl SataNotification {
    /// Get port multiplier notification
    pub fn set_pm_notif(&mut self, pm_port: u8) {
        (0x0..0xf).contains(&pm_port).then(|| {
            write_volatile!(
                self.0,
                read_volatile!(self.0) | pm_port as u32
            )
        });
    }

    /// Get port multiplier notification
    pub fn get_pm_notif(&self, pm_port: u8) -> bool {
        if (0x0..0xf).contains(&pm_port) {
            (read_volatile!(self.0) & !0xffff) & (1 << pm_port) != 0
        } else {
            false
        }
    }
}

/// Port X Frame Information Structure based switching control
#[repr(transparent)]
pub struct FisSwitchControl(pub u32);

impl FisSwitchControl {
    /// Port multiplier device that experienced fatal error
    pub fn device_with_error(&self) -> u8 {
        ((read_volatile!(self.0) >> 16) & 0xf) as u8
    }

    /// The number of devices that FIS-Based switching has been optimized
    /// for. The minimum value for this field should be 0x2.
    pub fn active_device_optimization(&self) -> u8 {
        ((read_volatile!(self.0) >> 12) & 0xf) as u8
    }

    /// Set the port multiplier port number, that should receive the next
    /// command
    pub fn device_to_issue(&mut self, dev_num: u8) {
        write_volatile!(self.0, read_volatile!(self.0) & !(0xf << 8));
        write_volatile!(
            self.0,
            read_volatile!(self.0) | (dev_num as u32) << 8
        );
    }

    // Single device error
    ro_flag!(sde, 2);

    // Device error clear
    rw1_flag!(dec, 1);

    // Enable, should be set if there is a port multiplier
    flag!(en, 0);
}

/// Port x Device sleep
#[repr(transparent)]
pub struct DeviceSleep(pub u32);

impl DeviceSleep {
    /// Device Sleep Idle Timeout Multiplier
    pub fn dito_multiplier(&self) -> u8 {
        ((read_volatile!(self.0) >> 25) & 0xf) as u8
    }

    /// Raw dito value
    ///
    /// **Use [`dito_actual`] for the actual wait time**
    pub fn dito_ms(&self) -> u16 {
        ((read_volatile!(self.0) >> 15) & 0x3ff) as u16
    }

    /// The actual timeout, which is dito * (dito_multiplier + 1)
    pub fn dito_actual_ms(&self) -> u16 {
        self.dito_ms() * (self.dito_multiplier() + 1) as u16
    }

    /// Minimum device sleep assertion time
    ///
    /// TODO: currently only read only, if write needed, check
    /// documentation about extended cap and writing to this offset
    pub fn mdat(&self) -> u8 {
        ((read_volatile!(self.0) >> 10) & 0x1f) as u8
    }

    /// Device sleep exit timeout
    ///
    /// TODO: currently only read only, if write needed, check
    /// documentation about extended cap and writing to this offset
    pub fn deto_ms(&self) -> u8 {
        ((read_volatile!(self.0) >> 2) & 0xff) as u8
    }

    // Device sleep present
    ro_flag!(dsp, 1);

    // Aggressive device sleep enable
    ro_flag!(adse, 0);
}

/// Port X Vendor specific
#[repr(transparent)]
pub struct VendorSpecific(pub u32);

#[repr(C)]
pub struct PortControlRegisters {
    /// Port X Command list base address low
    pub clb: Volatile<u32>,
    /// Port X Command list base address high
    pub clbu: Volatile<u32>,
    /// Port X frame information structure base address low
    pub fb: Volatile<u32>,
    /// Port X frame information structure base address high
    pub fbu: Volatile<u32>,
    pub is: PortInterruptStatus,
    pub ie: InterruptEnable,
    pub cmd: CmdStatus,
    _reserved0: u32,
    pub tfd: TaskFileData,
    pub sig: Signature,
    pub ssts: SataStatus,
    pub sctl: SataControl,
    pub serr: SataError,
    pub sact: SataActive,
    pub ci: CmdIssue,
    pub sntf: SataNotification,
    pub fbs: FisSwitchControl,
    pub devslp: DeviceSleep,
    _reserved1: [u32; 10],
    pub vs: [VendorSpecific; 4],
}

impl PortControlRegisters {
    /// Return the full command list address by combining the low and high
    /// 32bit parts
    pub fn cmd_list(&mut self) -> &mut CmdList {
        let cmd_list_addr = ((self.clbu.read() as usize) << 32)
            | (self.clb.read() as usize & !((1 << 10) - 1));
        unsafe { &mut *(cmd_list_addr as *mut CmdList) }
    }

    pub fn set_cmd_list_address(&mut self, ptr: usize) {
        println!("CLB: {:x?}", ptr);
        self.clb.write((ptr & 0xffffffff) as u32);
        self.clbu.write((ptr >> 32) as u32);
    }

    /// Return the full frame information structure address by combining
    /// the low and high 32bit parts
    pub fn received_fis(&self) -> &ReceivedFis {
        let rfis_addr = ((self.fbu.read() as usize) << 32)
            | (self.fb.read() as usize & !((1 << 8) - 1));
        unsafe { &*(rfis_addr as *const ReceivedFis) }
    }

    pub fn set_received_fis_address(&mut self, ptr: usize) {
        println!("FB: {:x?}", ptr);
        self.fb.write((ptr & 0xffffffff) as u32);
        self.fbu.write((ptr >> 32) as u32);
    }

    pub fn set_status(&mut self, port: u8) {
        self.cmd.set_st();
        (0x0u8..=0x1fu8).contains(&port).then(|| {
            self.sact.0 &= !(0x1f << 8);
            self.sact.0 |= (port as u32) << 8;
        });
    }

    /// Return the index of an available command slot if one exists
    pub fn find_cmd_slot(&self) -> Option<usize> {
        let mut slots = self.ci.0.read() | self.sact.0;
        for i in 0usize..32 {
            if slots & 1 == 0 {
                return Some(i);
            } else {
                slots >>= 1
            }
        }
        None
    }

    pub fn identity_packet(&mut self, buf: *mut IdentityPacketData) {
        let fis = RegisterH2D::new(
            1 << 7,
            AtaCommand::IdentifyDevice,
            0,
            0,
            0,
            0,
            0,
        );
        let cmd = &mut self.cmd_list().entries[0];
        let cmd_table = &mut cmd.cmd_table::<8>();
        let prdt_ent = &mut cmd_table.table[0];
        write_volatile!(cmd_table.cfis, Fis { h2d: fis });
        prdt_ent.set_buffer(buf);
        prdt_ent.dbc.set_dbc(511);
        cmd.info.set_command_fis_len(size_of::<RegisterH2D>());
        cmd.info.set_prdtl(1);
        println!("Sending command!");
        self.ci.issue_cmd(0);

        let mut timeout = 0xfffff;
        loop {
            if self.is.0 != 0 {
                if self.is.is_tfes() {
                    eprintln!("ERROR READING FROM DISK");
                    for error in self.serr.error() {
                        println!("{:?}", error);
                    }
                    if self.tfd.is_err() {
                        println!(
                            "TASK FILE DATA ERROR STATE\nERROR: {:08b}",
                            self.tfd.error()
                        );
                    }
                }
                println!("Finished!");
                println!("{:032b}", self.is.0);
                break;
            } else {
                timeout -= 1
            }

            if timeout == 0 {
                panic!("Timeout on identity packet read")
            }
        }
        unsafe {
            for w in (&mut *buf).serial_number.chunks_exact_mut(2) {
                w.swap(0, 1);
            }
            for w in (&mut *buf).model_num.chunks_exact_mut(2) {
                w.swap(0, 1);
            }
            for w in (&mut *buf).firmware_rev.chunks_exact_mut(2) {
                w.swap(0, 1);
            }
        }
    }
}

/// TODO, DECIDE IF ITS OK THAT THIS IS ONE BYTE GREATER IN SIZE
#[repr(C, align(256))]
pub struct ReceivedFis {
    pub dsfis: Volatile<DmaSetup>,
    _reserved0: u32,
    pub psfis: Volatile<PioSetupD2H>,
    _reserved1: [u32; 3],
    pub rfis: Volatile<RegisterD2H>,
    _reserved2: u32,
    pub sdbfis: Volatile<SetDeviceBits>,
    pub ufis: Volatile<[u8; 64]>,
    _reserved3: [u32; 24],
}

#[derive(Default)]
pub struct CmdListDescriptionInfo(pub u32);

impl CmdListDescriptionInfo {
    /// Set the Physical region descriptor table length
    pub fn set_prdtl(&mut self, size: u16) {
        write_volatile!(
            self.0,
            read_volatile!(self.0) | (size as u32) << 16
        );
    }

    /// Set the port multiplier port
    pub fn set_pm_port(&mut self, pm_port: u8) {
        write_volatile!(
            self.0,
            read_volatile!(self.0) | ((pm_port & 0xf) as u32) << 12
        );
    }

    // Clear busy upon R_OK
    flag!(c, 10);

    // BIST
    flag!(b, 9);

    // Reset
    flag!(r, 8);

    // Prefetchable
    flag!(p, 7);

    // Write
    flag!(w, 6);

    // ATAPI
    flag!(a, 5);

    /// Length of command FIS len (internally converted to dw)
    pub fn set_command_fis_len(&mut self, len: usize) {
        assert!(len < 64, "Len must be smaller then 64");
        assert!(len > 8, "Len must be greater then 8 ");
        write_volatile!(
            self.0,
            read_volatile!(self.0) | (len / size_of::<u32>()) as u32
        );
    }
}

#[repr(C)]
pub struct CmdHeader {
    info: CmdListDescriptionInfo,
    prdb_byte_count: Volatile<u32>,
    /// Command table descriptor base address
    ctba: Volatile<u32>,
    /// Command table desciprtor base address upper
    ctbau: Volatile<u32>,
    _reserved: [u32; 4],
}

impl CmdHeader {
    pub fn cmd_table<const ENTRIES: usize>(
        &mut self,
    ) -> &mut CmdTable<ENTRIES> {
        let cmd_table_addr = ((self.ctbau.read() as usize) << 32)
            | (self.ctba.read() as usize);
        unsafe { &mut *(cmd_table_addr as *mut CmdTable<ENTRIES>) }
    }

    pub fn set_cmd_table(&mut self, ptr: usize) {
        println!("CMD TBL: {:x?}", ptr);
        self.ctba.write((ptr & 0xffffffff) as u32);
        self.ctbau.write((ptr >> 32) as u32);
    }
}

#[repr(C, align(1024))]
pub struct CmdList {
    pub entries: [CmdHeader; 32],
}

pub struct PrdtDescriptionInfo(pub u32);

impl PrdtDescriptionInfo {
    // Interrupt on completion
    flag!(i, 31);

    /// Set the data byte count of the buffer on the prdt
    pub fn set_dbc(&mut self, dbc: u32) {
        const MB: u32 = 1 << 20;
        assert!(dbc < 4 * MB, "DBC should be smaller then 4Mib");
        write_volatile!(self.0, read_volatile!(self.0) | dbc | 1);
    }
}

#[repr(C)]
pub struct CmdTableEntry {
    /// Data base address buffer
    dba: Volatile<u32>,
    /// Data base address buffer upper
    dbau: Volatile<u32>,
    _reserved: u32,
    /// Data byte count (A maximum of 4mb is available)
    dbc: PrdtDescriptionInfo,
}

impl CmdTableEntry {
    pub fn set_buffer<T>(&mut self, buf: *mut T) {
        let ptr = buf as usize;
        self.dba.write((ptr & 0xffffffff) as u32);
        self.dbau.write((ptr >> 32) as u32);
    }
}

#[repr(C, align(256))]
pub struct CmdTable<const ENTRIES: usize> {
    cfis: Fis,
    /// TODO
    acmd: [u8; 0x10],
    _reserved: [u8; 0x30],
    table: [CmdTableEntry; ENTRIES],
}

#[repr(C)]
/// Host Bus Adapter Memory Registers
pub struct HBAMemoryRegisters {
    pub ghc: GenericHostControl,
    pub _reserved: [u8; 0x60],
    pub vsr: VendorSpecificRegisters,

    // Not doing 32 ports on purporse!
    // Because it makes this structure larger then a page
    pub ports: [PortControlRegisters; 30],
}

impl HBAMemoryRegisters {
    pub fn new(a: PhysicalAddress) -> Result<&'static mut Self, HbaError> {
        if !a.is_aligned(REGULAR_PAGE_ALIGNMENT) {
            return Err(HbaError::AddressNotAligned);
        }

        a.map(
            a.translate(),
            PageEntryFlags::regular_io_page_flags(),
            PageSize::Regular,
        );

        let hba: &'static mut HBAMemoryRegisters =
            unsafe { &mut *a.translate().as_mut_ptr() };

        hba.ghc.ghc.set_ae();
        hba.ghc.ghc.set_ie();

        if hba.ghc.pi.0 >= (1 << 31) {
            panic!("There is no support for HBA's with more then 30 ports")
        }

        println!("BIOS / OS Handoff: {}", hba.ghc.cap_ext.is_boh());

        if hba.ghc.cap_ext.is_boh() {
            unimplemented!("Didn't implement bios os handoff")
        }

        Ok(hba)
    }

    /// Returns the amount of active devices found and set them into idle
    /// state.
    pub fn probe_init(&mut self) -> usize {
        //     println!(
        //         "Detected {} implemented ports",
        //         self.ghc.cap.number_of_ports()
        //     );

        //     println!(
        //         "Supported command slots: {}, Supported 64bit addresses:
        // {}",         self.ghc.cap.number_of_commands(),
        //         self.ghc.cap.is_s64a()
        //     );

        //     let mut count = 0;
        //     for (i, port) in self.ports.iter_mut().enumerate() {
        //         if self.ghc.pi.is_port_implemented(i as u8)
        //             && let Ok(power) = port.ssts.power()
        //             && let InterfacePowerManagement::Active = power
        //         {
        //             count += 1;
        //             println!("\nDetected device at port number: {}", i);
        //             print!("  Device Power: ");
        //             println!("{:?}", power ; color =
        // ColorCode::new(Color::Green, Color::Black));
        // print!("  Device Speed: ");             println!("{}",
        // port.ssts.speed() ; color = ColorCode::new(Color::Green,
        // Color::Black));             print!("  Device type: ");
        //             match port.sig.device_type() {
        //                 Ok(t) => {
        //                     println!("{:?}", t ; color =
        // ColorCode::new(Color::Green, Color::Black) )
        // }                 Err(e) => {
        //                     println!("{:?}", e ; color =
        // ColorCode::new(Color::Red, Color::Black) )
        // }             }
        //             port.cmd.stop();

        //             let clb_fbu_table = unsafe { alloc_pages!(1) };
        //             for i in (0..4096).step_by(size_of::<usize>()) {
        //                 unsafe {
        //                     core::ptr::write_volatile(
        //                         ((clb_fbu_table + i) +
        // PHYSICAL_MEMORY_OFFSET)                             as
        // *mut usize,                         0,
        //                     );
        //                 }
        //             }

        //             port.set_cmd_list_address(clb_fbu_table);
        //             port.set_received_fis_address(
        //                 clb_fbu_table + size_of::<CmdList>(),
        //             );

        //             // MAPPING the first header with 8 entries (0x100 in
        // total             // table size)
        //             let cmd_list = port.cmd_list();
        //             cmd_list.entries[0].set_cmd_table(
        //                 clb_fbu_table
        //                     + size_of::<CmdList>()
        //                     + size_of::<ReceivedFis>(),
        //             );

        //             port.cmd.set_fre();
        //             port.serr.zero_error();
        //             // port.ie.set_dhre();
        //             // port.ie.set_pse();
        //             // port.ie.set_dse();
        //             // port.ie.set_tfee();
        //             port.is.clear_pending_interrupts();
        //             self.ghc.is.clear_all();

        //             port.cmd.set_sud();
        //             port.cmd.set_pod();
        //
        // port.cmd.set_icc(InterfaceCommunicationControl::Active);

        //             loop {
        //                 if !port.tfd.is_bsy()
        //                     && !port.tfd.is_drq()
        //                     && matches!(
        //                         port.ssts.power().unwrap(),
        //                         InterfacePowerManagement::Active
        //                     )
        //                 {
        //                     break;
        //                 }
        //             }
        //             port.cmd.start();
        //             println!("Started port number: {}", i)
        //         }
        //     }
        todo!()
        // count
    }
}
