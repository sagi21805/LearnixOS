use core::num::NonZero;

use common::enums::{AHCIInterfaceSpeed, InterfaceCommunicationControl};
use learnix_macros::{flag, ro_flag, rw1_flag, rwc_flag};

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

    pub fn interface_speed(&self) -> AHCIInterfaceSpeed {
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
    /// between 0x0 and 0xf)
    pub fn set_current_cmd(&mut self, ccs: u8) -> Option<()> {
        (0x0u8..=0xfu8).contains(&ccs).then(|| {
            self.0 &= !(0xff << 8);
            self.0 |= (ccs as u32) << 8;
        })
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
pub struct Signature {
    pub sector_count: u8,
    pub lba_low: u8,
    pub lba_mid: u8,
    pub lba_high: u8,
}

/// Port X SATA Status
pub struct SataStatus(pub u32);

/// Port X SATA control
pub struct SataControl(pub u32);

/// Port X SATA error
pub struct SataError(pub u32);

/// Port X Sata Active
pub struct SataActive(pub u32);

/// Port X Command issue
pub struct CmdIssue(pub u32);

/// Port X SATA Notification
pub struct SataNotification(pub u32);

/// Port X Frame Information Structure based switching control
pub struct FisSwitchControl(pub u32);

/// Port x Device sleep
pub struct DeviceSleep(pub u32);

/// Port X Vendor specific
pub struct VendorSpecific(pub u32);

#[repr(C)]
pub struct PortControlRegisters {
    clb: CmdListAddressLow,
    clbu: CmdListAddressHigh,
    fb: FisAddressLow,
    fbu: FisAddressHigh,
    is: PortInterruptStatus,
    ie: InterruptEnable,
    cmd: CmdStatus,
    _reserved0: u32,
    tfd: TaskFileData,
    sig: Signature,
    ssts: SataStatus,
    sctl: SataControl,
    serr: SataError,
    sact: SataActive,
    ci: CmdIssue,
    sntf: SataNotification,
    fbs: FisSwitchControl,
    devslp: DeviceSleep,
    _reserved1: [u32; 10],
    vs: [VendorSpecific; 4],
}

impl PortControlRegisters {
    /// Return the full command list address by combining the low and high
    /// 32bit parts
    pub fn cmd_list_address(&self) -> usize {
        ((self.clbu.0 as usize) << 32)
            | (self.clb.0 as usize & !((1 << 10) - 1))
    }

    /// Return the full frame information structure address by combining
    /// the low and high 32bit parts
    pub fn fis_address(&self) -> usize {
        ((self.fbu.0 as usize) << 32)
            | (self.fb.0 as usize & !((1 << 8) - 1))
    }
}

#[repr(C)]
/// Host Bus Adapter Memory Registers
pub struct HBAMemoryRegisters {
    pub ghc: GenericHostControl,
    pub _reserved: [u8; 0x60],
    pub vsr: VendorSpecificRegisters,
    pub ports: [PortControlRegisters; 32],
}
