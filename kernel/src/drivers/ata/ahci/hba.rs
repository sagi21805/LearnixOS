/// AHCI implementation for the learnix operating system
///
/// Implemented directly from https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/serial-ata-ahci-spec-rev1-3-1.pdf
extern crate alloc;

use core::num::NonZero;

use common::{
    address_types::{PhysicalAddress, VirtualAddress},
    constants::{REGULAR_PAGE_ALIGNMENT, REGULAR_PAGE_SIZE},
    enums::{
        DeviceDetection, DeviceType, InterfaceCommunicationControl,
        InterfaceInitialization, InterfacePowerManagement, InterfaceSpeed,
        InterfaceSpeedRestriction, PageSize, PicInterruptVectorOffset,
    },
    error::{AhciError, ConversionError, DiagnosticError, HbaError},
};
use cpu_utils::{instructions::port, structures::paging::PageEntryFlags};
use learnix_macros::{flag, ro_flag, rw1_flag, rwc_flag};
use num_enum::UnsafeFromPrimitive;
use strum::IntoEnumIterator;

use crate::{
    alloc_pages,
    drivers::ata::ahci::{
        DmaSetup, Fis, PioSetupD2H, RegisterD2H, SetDeviceBits,
    },
    memory::allocators::page_allocator::{
        allocator::PhysicalPageAllocator, extensions::PhysicalAddressExt,
    },
    println,
};

use alloc::vec::Vec;

#[derive(Copy, Clone)]
pub struct AHCIBaseAddress(pub u32);

/// CAP
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
        unsafe { core::mem::transmute(((self.0 >> 20) & 0xf) as u8) }
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
        ((self.0 >> 8) & 0x1f) as u8
    }

    // Command completion coalescing supported
    ro_flag!(cccs, 7);

    // Enclosure management supported
    ro_flag!(ems, 6);

    // Support external SATA
    ro_flag!(sxs, 5);

    /// Returns the number of ports implemented
    pub fn number_of_ports(&self) -> u8 {
        (self.0 & 0x1f) as u8
    }
}

/// GHC
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
#[derive(Debug, Clone, Copy)]
pub struct InterruptStatus(pub u32);

impl InterruptStatus {
    // Port Interrupt Pending Status. Corresponds to bits of the PI
    // register. Cleared by writing a '1' to the corresponding bit.
    pub fn is_port_pending(&self, port_num: u8) -> bool {
        (self.0 & (1 << port_num)) != 0
    }

    pub fn clear(&mut self, port_num: u8) {
        self.0 |= 1 << port_num;
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
#[derive(Debug, Clone, Copy)]
pub struct PortsImplemented(pub u32);

impl PortsImplemented {
    // Port i is Implemented (P[i])
    pub fn is_port_implemented(&self, port_num: u8) -> bool {
        (self.0 & (1 << port_num)) != 0
    }
}

// VS
#[derive(Debug, Clone, Copy)]
pub struct Version(pub u32);

impl Version {
    // Major Version Number (Bits 31:16)
    pub fn major_version(&self) -> u16 {
        (self.0 >> 16) as u16
    }

    // Minor Version Number (Bits 15:0)
    pub fn minor_version(&self) -> u16 {
        (self.0 & 0xFFFF) as u16
    }
}

/// CCC_CTL
#[derive(Debug, Clone, Copy)]
pub struct CommandCompletionCoalescingControl(pub u32);

impl CommandCompletionCoalescingControl {
    pub fn interrupt_time_ms(&self) -> u16 {
        const MASK: u32 = 0xFFFF;
        ((self.0 >> 16) & MASK) as u16
    }

    // Command Completions (CC): Number of command completions necessary to
    // cause a CCC interrupt
    pub fn command_completions(&self) -> u8 {
        const MASK: u32 = 0xFF;
        ((self.0 >> 8) & MASK) as u8
    }

    flag!(enable, 0);
}

/// CCC_PORTS
#[derive(Debug, Clone, Copy)]
pub struct CommandCompletionCoalescingPorts(pub u32);

impl CommandCompletionCoalescingPorts {
    pub fn set_port(&mut self, port_num: u8) {
        self.0 |= 1 << port_num
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
#[derive(Debug, Clone, Copy)]
pub struct EnclosureManagementLocation(pub u32);

impl EnclosureManagementLocation {
    pub fn dword_offset_from_abar(&self) -> usize {
        (self.0 >> 16) as usize
    }

    /// ZERO is invalid
    /// TODO understand how to check if i have both receive and transmit
    pub fn buffet_size(&self) -> Option<NonZero<usize>> {
        NonZero::new((self.0 & 0xffff) as usize)
    }
}

/// EM_CTL
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

/// Port X Command list base address low
pub struct CmdListAddressLow(pub u32);

/// Port X Command list base address high
pub struct CmdListAddressHigh(pub u32);

/// Port X Frame Information Structure base address low
pub struct FisAddressLow(pub u32);

/// Port X Frame Information Structure base address high
pub struct FisAddressHigh(pub u32);

/// Port X Interrupt status
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
}

/// Port X Interrupt Enable
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
pub struct CmdStatus(pub u32);

impl CmdStatus {
    pub fn set_icc(&mut self, icc: InterfaceCommunicationControl) {
        self.0 &= !(0xf << 28);
        self.0 |= (icc as u32) << 28;
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

    /// If None is returned, invalid ccs has entered (Value should be
    /// between 0x0 and 0x1f)
    pub fn set_current_cmd(&mut self, ccs: u8) {
        self.set_st();
        (0x0u8..=0x1fu8).contains(&ccs).then(|| {
            self.0 &= !(0x1f << 8);
            self.0 |= (ccs as u32) << 8;
        });
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
}

/// Port x Task File Data
pub struct TaskFileData(pub u32);

impl TaskFileData {
    // Indicates error during transfer
    ro_flag!(err, 0);

    // Indicates a data transfer request
    ro_flag!(drq, 3);

    // Indicates that the interface is busy
    ro_flag!(bsy, 7);

    pub fn error(&self) -> u8 {
        (self.0 >> 8) as u8
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
pub struct SataStatus(pub u32);

impl SataStatus {
    pub fn power(
        &self,
    ) -> Result<InterfacePowerManagement, ConversionError<u8>> {
        let power = ((self.0 >> 8) & 0xf) as u8;
        InterfacePowerManagement::try_from(power)
    }

    pub fn speed(&self) -> InterfaceSpeed {
        let speed = ((self.0 >> 4) & 0xf) as u8;
        unsafe { InterfaceSpeed::unchecked_transmute_from(speed) }
    }

    pub fn detection(
        &self,
    ) -> Result<DeviceDetection, ConversionError<u8>> {
        let detection = (self.0 & 0xf) as u8;
        DeviceDetection::try_from(detection)
    }
}

/// Port X SATA control
pub struct SataControl(pub u32);

impl SataControl {
    pub fn port_multiplier(&self) -> u8 {
        ((self.0 >> 16) & 0xf) as u8
    }

    pub fn select_power_management(&self) -> u8 {
        ((self.0 >> 12) & 0xf) as u8
    }

    flag!(devslp_disabled, 10);
    flag!(slumber_disabled, 9);
    flag!(partial_disabled, 8);

    pub fn max_speed(&self) -> InterfaceSpeedRestriction {
        let speed = ((self.0 >> 4) & 0xf) as u8;
        unsafe {
            InterfaceSpeedRestriction::unchecked_transmute_from(speed)
        }
    }

    pub fn set_max_speed(&mut self, speed: InterfaceSpeed) {
        if speed != InterfaceSpeed::DevNotPresent {
            self.0 &= !(0xf << 4);
            self.0 |= (speed as u32) << 4;
        }
    }

    pub fn device_initialization(
        &self,
    ) -> Result<InterfaceInitialization, ConversionError<u8>> {
        InterfaceInitialization::try_from((self.0 & 0xf) as u8)
    }

    // TODO THIS COMMAND ANY MAYBE OTHER SHOULD PROBABLY MOVE TO THE PORT
    // SETTING BECAUSE THEY REQUIRE PxCMD.st BIT TO BE SET WHILE THEY ARE
    // SET
    pub fn set_device_initialization(
        &mut self,
        init: InterfaceInitialization,
    ) {
        self.0 &= !0xf;
        self.0 |= init as u32;
    }
}

/// Port X SATA error
pub struct SataError(pub u32);

impl SataError {
    pub fn diagnostic(&self) -> impl Iterator<Item = DiagnosticError> {
        let diagnostic_errors = ((self.0 >> 16) & 0xffff) as u16;
        DiagnosticError::iter()
            .filter(move |n| *n as u16 & diagnostic_errors != 0)
    }

    pub fn error(&self) -> impl Iterator<Item = AhciError> {
        let ahci_error = (self.0 & 0xffff) as u16;
        AhciError::iter().filter(move |n| *n as u16 & ahci_error != 0)
    }
}

/// Port X Sata Active
pub struct SataActive(pub u32);

/// Port X Command issue
pub struct CmdIssue(pub u32);

/// Port X SATA Notification
pub struct SataNotification(pub u32);

impl SataNotification {
    /// Get port multiplier notification
    pub fn set_pm_notif(&mut self, pm_port: u8) {
        (0x0..0xf)
            .contains(&pm_port)
            .then(|| self.0 |= pm_port as u32);
    }

    /// Get port multiplier notification
    pub fn get_pm_notif(&self, pm_port: u8) -> bool {
        if (0x0..0xf).contains(&pm_port) {
            (self.0 & !0xffff) & (1 << pm_port) != 0
        } else {
            false
        }
    }
}

/// Port X Frame Information Structure based switching control
pub struct FisSwitchControl(pub u32);

impl FisSwitchControl {
    /// Port multiplier device that experienced fatal error
    pub fn device_with_error(&self) -> u8 {
        ((self.0 >> 16) & 0xf) as u8
    }

    /// The number of devices that FIS-Based switching has been optimized
    /// for. The minimum value for this field should be 0x2.
    pub fn active_device_optimization(&self) -> u8 {
        ((self.0 >> 12) & 0xf) as u8
    }

    /// Set the port multiplier port number, that should recieve the next
    /// command
    pub fn device_to_issue(&mut self, dev_num: u8) {
        self.0 &= !(0xf << 8);
        self.0 |= (dev_num as u32) << 8;
    }

    // Single device error
    ro_flag!(sde, 2);

    // Device error clear
    rw1_flag!(dec, 1);

    // Enable, should be set if there is a port multiplier
    flag!(en, 0);
}

/// Port x Device sleep
pub struct DeviceSleep(pub u32);

impl DeviceSleep {
    /// Device Sleep Idle Timeout Multiplier
    pub fn dito_multiplier(&self) -> u8 {
        ((self.0 >> 25) & 0xf) as u8
    }

    /// Raw dito value
    ///
    /// **Use [`dito_actual`] for the actual wait time**
    pub fn dito_ms(&self) -> u16 {
        ((self.0 >> 15) & 0x3ff) as u16
    }

    /// The actual timeout, which is dito * (dito_multiplier + 1)
    pub fn dito_actual_ms(&self) -> u16 {
        self.dito_ms() * (self.dito_multiplier() + 1) as u16
    }

    /// Minimu device sleep assertion time
    ///
    /// TODO: currently only read only, if write needed, check documentatio
    /// about extended cap and writing to this offset
    pub fn mdat(&self) -> u8 {
        ((self.0 >> 10) & 0x1f) as u8
    }

    /// Device sleep exit timeout
    ///
    /// TODO: currently only read only, if write needed, check documentatio
    /// about extended cap and writing to this offset
    pub fn deto_ms(&self) -> u8 {
        ((self.0 >> 2) & 0xff) as u8
    }

    // Device sleep present
    ro_flag!(dsp, 1);

    // Aggressive device sleep enable
    ro_flag!(adse, 0);
}

/// Port X Vendor specific
pub struct VendorSpecific(pub u32);

#[repr(C)]
pub struct PortControlRegisters {
    pub clb: CmdListAddressLow,
    pub clbu: CmdListAddressHigh,
    pub fb: FisAddressLow,
    pub fbu: FisAddressHigh,
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
    pub fn cmd_list(&mut self) -> &mut CommandList {
        let cmd_list_addr = ((self.clbu.0 as usize) << 32)
            | (self.clb.0 as usize & !((1 << 10) - 1));
        unsafe { &mut *(cmd_list_addr as *mut CommandList) }
    }

    /// Return the full frame information structure address by combining
    /// the low and high 32bit parts
    pub fn received_fis(&self) -> &RecievedFis {
        let rfis_addr = ((self.fbu.0 as usize) << 32)
            | (self.fb.0 as usize & !((1 << 8) - 1));
        unsafe { &*(rfis_addr as *const RecievedFis) }
    }

    pub fn set_status(&mut self, port: u8) {
        self.cmd.set_st();
        (0x0u8..=0x1fu8).contains(&port).then(|| {
            self.sact.0 &= !(0x1f << 8);
            self.sact.0 |= (port as u32) << 8;
        });
    }

    pub fn send_command(&mut self, port: u8) {
        self.cmd.set_st();
        (0x0u8..=0x1fu8).contains(&port).then(|| {
            self.ci.0 &= !(0x1f << 8);
            self.ci.0 |= (port as u32) << 8;
        });
    }
}

/// TODO, DECIDE IF ITS OK THAT THIS IS ONE BYTE GREATER IN SIZE
#[repr(C)]
pub struct RecievedFis {
    dsfis: DmaSetup,
    _reserved0: u32,
    psfis: PioSetupD2H,
    _reserved1: [u32; 3],
    rfis: RegisterD2H,
    _reserved2: u32,
    sdbfis: SetDeviceBits,
    ufis: [u8; 64],
    _reserved3: [u32; 24],
}

#[derive(Default)]
pub struct CmdListDescriptionInfo(pub u32);

impl CmdListDescriptionInfo {
    /// Set the Physical region descriptor table length
    pub fn set_prdtl(&mut self, size: u16) {
        self.0 |= (size as u32) << 16;
    }

    /// Set the port multiplier port
    pub fn set_pm_port(&mut self, pm_port: u8) {
        self.0 |= ((pm_port & 0xf) as u32) << 12
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

    /// Length of command FIS in dwords
    pub fn set_command_fis_len_dw(&mut self, len: u8) {
        assert!(len < 2, "Len must be smaller then 2");
        assert!(len > 16, "Len must be greater then 16 ");
        self.0 |= len as u32;
    }
}

#[repr(C)]
#[derive(Default)]
pub struct CommandListEntry {
    info: CmdListDescriptionInfo,
    prdb_byte_count: u32,
    /// Command table descriptor base address
    ctba: u32,
    /// Command table desciprtor base address upper
    ctbau: u32,
    _reserved: [u32; 4],
}

impl CommandListEntry {
    pub fn cmd_table<const ENTRIES: usize>(
        &mut self,
    ) -> &mut CommandTable<ENTRIES> {
        let cmd_table_addr =
            ((self.ctbau as usize) << 32) | (self.ctba as usize);
        unsafe { &mut *(cmd_table_addr as *mut CommandTable<ENTRIES>) }
    }
}

#[derive(Default)]
pub struct CommandList {
    pub entries: [CommandListEntry; 32],
}

#[derive(Clone, Copy, Default)]
pub struct PrdtDescriptionInfo(pub u32);

impl PrdtDescriptionInfo {
    // Interrupt on completion
    flag!(i, 31);

    /// Set the data byte count of the buffer on the prdt
    pub fn set_dbc(&mut self, dbc: u32) {
        const MB: u32 = 1 << 20;
        assert!(dbc < 4 * MB, "DBC should be smller then 4Mib");
    }
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct CommandTableEntry {
    /// Data base address buffer
    dba: u32,
    /// Data base address buffer upper
    dbau: u32,
    _reserved: u32,
    /// Data byte count (A maximum of 4mb is available)
    dbc: PrdtDescriptionInfo,
}

#[repr(C)]
pub struct CommandTable<const ENTRIES: usize = 14> {
    cfis: Fis,
    /// TODO
    acmd: [u8; 0x10],
    _reserved: [u8; 0x30],
    table: [CommandTableEntry; ENTRIES],
}

impl<const ENTRIES: usize> Default for CommandTable<ENTRIES> {
    fn default() -> Self {
        Self {
            cfis: Fis::default(),
            _reserved: [0; 0x30],
            acmd: [0; 0x10],
            table: [CommandTableEntry::default(); ENTRIES],
        }
    }
}

#[repr(align(4096))]
#[derive(Default)]
pub struct PortCommands<const ENTRIES: usize = 14> {
    pub cmd_list: CommandList,
    pub cmd_table: [CommandTable<ENTRIES>; 32],
}

impl<const ENTRIES: usize> PortCommands<ENTRIES> {
    pub fn empty() -> &'static mut PortCommands<ENTRIES> {
        let port_cmd_ptr = unsafe {
            alloc_pages!(size_of::<PortCommands>() / REGULAR_PAGE_SIZE)
                as *mut PortCommands<ENTRIES>
        };
        unsafe {
            core::ptr::write_volatile(
                port_cmd_ptr,
                PortCommands::<ENTRIES>::default(),
            );
            &mut *port_cmd_ptr
        }
    }
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
            return Err(HbaError::AdressNotAligned);
        }

        a.map(
            a.translate(),
            PageEntryFlags::regular_io_page_flags(),
            PageSize::Regular,
        );

        let hba: &'static mut HBAMemoryRegisters =
            unsafe { &mut *a.translate().as_mut_ptr() };

        if hba.ghc.pi.0 >= (1 << 31) {
            panic!("There is no support for HBA's with more then 30 ports")
        }

        println!(
            "Detected {} implemented ports",
            hba.ghc.cap.number_of_ports()
        );

        for port in 0..30 {
            if hba.ghc.pi.is_port_implemented(port) {
                let p = &hba.ports[port as usize];
                if let Ok(power) = p.ssts.power()
                    && let InterfacePowerManagement::Active = power
                {
                    println!("Detected device at port number: {}", port);
                    println!("  Device Power: {:?}", power);
                    println!("  Device Speed: {}", p.ssts.speed());
                    println!("  Device type: {:?}", p.sig.device_type());
                }
            }
        }

        Ok(hba)
    }
}

pub struct AhciDeviceController<const ENTRIES: usize = 14> {
    pub port: &'static mut PortControlRegisters,
    pub port_commands: &'static mut PortCommands,
}
