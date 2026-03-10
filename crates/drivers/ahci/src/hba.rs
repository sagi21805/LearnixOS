/// AHCI implementation for the learnix operating system
///
/// Implemented directly from https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/serial-ata-ahci-spec-rev1-3-1.pdf
extern crate alloc;

use core::{fmt::Debug, num::NonZero, panic, ptr::NonNull};

use common::{
    address_types::PhysicalAddress,
    constants::REGULAR_PAGE_ALIGNMENT,
    enums::{
        AtaCommand, DeviceDetection, DeviceType,
        InterfaceCommunicationControl, InterfaceInitialization,
        InterfacePowerManagement, InterfaceSpeed, PageSize,
    },
    error::{ConversionError, HbaError},
    read_volatile,
    volatile::Volatile,
    write_volatile,
};
use macros::bitfields;
use num_enum::UnsafeFromPrimitive;
use strum::IntoEnumIterator;
use x86::structures::paging::PageEntryFlags;

use crate::{
    DmaSetup, Fis, IdentityPacketData, PioSetupD2H, RegisterD2H,
    RegisterH2D, SetDeviceBits,
};

#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct AHCIBaseAddress(pub u32);

/// CAP
#[bitfields]
pub struct HBACapabilities {
    /// Number of ports
    number_of_ports: B5,
    /// Support external SATA
    #[flag(r)]
    sxs: B1,
    /// Enclosure management supported
    #[flag(r)]
    ems: B1,
    /// Command completion coalescing supported
    #[flag(r)]
    cccs: B1,
    /// Number of commands (value is between 1 and 32)
    number_of_commands: B5,
    #[flag(rc(0))]
    reserved0: B1,
    /// Partial state capable
    #[flag(r)]
    psc: B1,
    /// Programmed I/O multiple data request block
    #[flag(r)]
    pmd: B1,
    /// Frame information structure based switching supported
    #[flag(r)]
    fbss: B1,
    /// Support port multiplier
    #[flag(r)]
    spm: B1,
    /// Support AHCI mode only
    #[flag(r)]
    sam: B1,
    #[flag(rc(0))]
    reserved1: B1,
    /// Interface speed
    #[flag(r, flag_type = InterfaceSpeed)]
    interface_speed: B4,
    #[flag(rc(0))]
    reserved2: B1,
    /// Support activity LED
    #[flag(r)]
    sal: B1,
    /// Support aggressive link power management
    #[flag(r)]
    salp: B1,
    /// Support staggered spin-up
    #[flag(r)]
    sss: B1,
    /// Support mechanical presence switch
    #[flag(r)]
    smps: B1,
    /// Support S-notification register
    #[flag(r)]
    ssntf: B1,
    /// Support native command queuing
    #[flag(r)]
    snqc: B1,
    /// Support 64-bit addressing
    #[flag(r)]
    s64a: B1,
}

/// GHC
#[bitfields]
pub struct GlobalHostControl {
    /// HBA reset
    hr: B1,
    /// Interrupt enable
    ie: B1,
    /// MSI revert to single message
    mrsm: B1,
    #[flag(rc(0))]
    reserved: B28,
    /// AHCI enable — must be set for the HBA to operate in AHCI mode
    ae: B1,
}

/// IS
#[bitfields]
pub struct InterruptStatus {
    /// Port 0 interrupt pending status
    #[flag(rwc(1))]
    ip00: B1,
    /// Port 1 interrupt pending status
    #[flag(rwc(1))]
    ip01: B1,
    /// Port 2 interrupt pending status
    #[flag(rwc(1))]
    ip02: B1,
    /// Port 3 interrupt pending status
    #[flag(rwc(1))]
    ip03: B1,
    /// Port 4 interrupt pending status
    #[flag(rwc(1))]
    ip04: B1,
    /// Port 5 interrupt pending status
    #[flag(rwc(1))]
    ip05: B1,
    /// Port 6 interrupt pending status
    #[flag(rwc(1))]
    ip06: B1,
    /// Port 7 interrupt pending status
    #[flag(rwc(1))]
    ip07: B1,
    /// Port 8 interrupt pending status
    #[flag(rwc(1))]
    ip08: B1,
    /// Port 9 interrupt pending status
    #[flag(rwc(1))]
    ip09: B1,
    /// Port 10 interrupt pending status
    #[flag(rwc(1))]
    ip10: B1,
    /// Port 11 interrupt pending status
    #[flag(rwc(1))]
    ip11: B1,
    /// Port 12 interrupt pending status
    #[flag(rwc(1))]
    ip12: B1,
    /// Port 13 interrupt pending status
    #[flag(rwc(1))]
    ip13: B1,
    /// Port 14 interrupt pending status
    #[flag(rwc(1))]
    ip14: B1,
    /// Port 15 interrupt pending status
    #[flag(rwc(1))]
    ip15: B1,
    /// Port 16 interrupt pending status
    #[flag(rwc(1))]
    ip16: B1,
    /// Port 17 interrupt pending status
    #[flag(rwc(1))]
    ip17: B1,
    /// Port 18 interrupt pending status
    #[flag(rwc(1))]
    ip18: B1,
    /// Port 19 interrupt pending status
    #[flag(rwc(1))]
    ip19: B1,
    /// Port 20 interrupt pending status
    #[flag(rwc(1))]
    ip20: B1,
    /// Port 21 interrupt pending status
    #[flag(rwc(1))]
    ip21: B1,
    /// Port 22 interrupt pending status
    #[flag(rwc(1))]
    ip22: B1,
    /// Port 23 interrupt pending status
    #[flag(rwc(1))]
    ip23: B1,
    /// Port 24 interrupt pending status
    #[flag(rwc(1))]
    ip24: B1,
    /// Port 25 interrupt pending status
    #[flag(rwc(1))]
    ip25: B1,
    /// Port 26 interrupt pending status
    #[flag(rwc(1))]
    ip26: B1,
    /// Port 27 interrupt pending status
    #[flag(rwc(1))]
    ip27: B1,
    /// Port 28 interrupt pending status
    #[flag(rwc(1))]
    ip28: B1,
    /// Port 29 interrupt pending status
    #[flag(rwc(1))]
    ip29: B1,
    /// Port 30 interrupt pending status
    #[flag(rwc(1))]
    ip30: B1,
    /// Port 31 interrupt pending status
    #[flag(rwc(1))]
    ip31: B1,
}

/// PI
#[bitfields]
pub struct PortsImplemented {
    /// Port 0 is implemented
    p00: B1,
    /// Port 1 is implemented
    p01: B1,
    /// Port 2 is implemented
    p02: B1,
    /// Port 3 is implemented
    p03: B1,
    /// Port 4 is implemented
    p04: B1,
    /// Port 5 is implemented
    p05: B1,
    /// Port 6 is implemented
    p06: B1,
    /// Port 7 is implemented
    p07: B1,
    /// Port 8 is implemented
    p08: B1,
    /// Port 9 is implemented
    p09: B1,
    /// Port 10 is implemented
    p10: B1,
    /// Port 11 is implemented
    p11: B1,
    /// Port 12 is implemented
    p12: B1,
    /// Port 13 is implemented
    p13: B1,
    /// Port 14 is implemented
    p14: B1,
    /// Port 15 is implemented
    p15: B1,
    /// Port 16 is implemented
    p16: B1,
    /// Port 17 is implemented
    p17: B1,
    /// Port 18 is implemented
    p18: B1,
    /// Port 19 is implemented
    p19: B1,
    /// Port 20 is implemented
    p20: B1,
    /// Port 21 is implemented
    p21: B1,
    /// Port 22 is implemented
    p22: B1,
    /// Port 23 is implemented
    p23: B1,
    /// Port 24 is implemented
    p24: B1,
    /// Port 25 is implemented
    p25: B1,
    /// Port 26 is implemented
    p26: B1,
    /// Port 27 is implemented
    p27: B1,
    /// Port 28 is implemented
    p28: B1,
    /// Port 29 is implemented
    p29: B1,
    /// Port 30 is implemented
    p30: B1,
    /// Port 31 is implemented
    p31: B1,
}

/// VS
#[bitfields]
pub struct Version {
    /// Minor version number (bits 15:0)
    minor_version: B16,
    /// Major version number (bits 31:16)
    major_version: B16,
}

/// CCC_CTL
#[bitfields]
pub struct CommandCompletionCoalescingControl {
    /// Enable command completion coalescing
    enable: B1,
    #[flag(rc(0))]
    reserved: B7,
    /// Command completions — number of command completions necessary to
    /// cause a CCC interrupt
    command_completions: B8,
    /// Interrupt time in milliseconds
    interrupt_time_ms: B16,
}

/// CCC_PORTS
#[bitfields]
pub struct CommandCompletionCoalescingPorts {
    /// Port 0 CCC enabled
    prt00: B1,
    /// Port 1 CCC enabled
    prt01: B1,
    /// Port 2 CCC enabled
    prt02: B1,
    /// Port 3 CCC enabled
    prt03: B1,
    /// Port 4 CCC enabled
    prt04: B1,
    /// Port 5 CCC enabled
    prt05: B1,
    /// Port 6 CCC enabled
    prt06: B1,
    /// Port 7 CCC enabled
    prt07: B1,
    /// Port 8 CCC enabled
    prt08: B1,
    /// Port 9 CCC enabled
    prt09: B1,
    /// Port 10 CCC enabled
    prt10: B1,
    /// Port 11 CCC enabled
    prt11: B1,
    /// Port 12 CCC enabled
    prt12: B1,
    /// Port 13 CCC enabled
    prt13: B1,
    /// Port 14 CCC enabled
    prt14: B1,
    /// Port 15 CCC enabled
    prt15: B1,
    /// Port 16 CCC enabled
    prt16: B1,
    /// Port 17 CCC enabled
    prt17: B1,
    /// Port 18 CCC enabled
    prt18: B1,
    /// Port 19 CCC enabled
    prt19: B1,
    /// Port 20 CCC enabled
    prt20: B1,
    /// Port 21 CCC enabled
    prt21: B1,
    /// Port 22 CCC enabled
    prt22: B1,
    /// Port 23 CCC enabled
    prt23: B1,
    /// Port 24 CCC enabled
    prt24: B1,
    /// Port 25 CCC enabled
    prt25: B1,
    /// Port 26 CCC enabled
    prt26: B1,
    /// Port 27 CCC enabled
    prt27: B1,
    /// Port 28 CCC enabled
    prt28: B1,
    /// Port 29 CCC enabled
    prt29: B1,
    /// Port 30 CCC enabled
    prt30: B1,
    /// Port 31 CCC enabled
    prt31: B1,
}

/// EM_LOC
#[bitfields]
pub struct EnclosureManagementLocation {
    /// Buffer size in dwords (zero is invalid)
    buffer_size: B16,
    /// Dword offset of the EM buffer from ABAR
    dword_offset_from_abar: B16,
}

/// EM_CTL
#[bitfields]
pub struct EnclosureManagementControl {
    /// Message received
    #[flag(rwc(0))]
    mr: B1,
    #[flag(rc(0))]
    reserved0: B7,
    /// Transmit message
    tm: B1,
    /// Reset enclosure management
    reset: B1,
    #[flag(rc(0))]
    reserved1: B6,
    /// LED message type supported
    #[flag(r)]
    led: B1,
    /// SAF-TE enclosure management messages supported
    #[flag(r)]
    safte: B1,
    /// SES-2 enclosure management messages supported
    #[flag(r)]
    ses2: B1,
    /// SGPIO enclosure management messages supported
    #[flag(r)]
    sgpio: B1,
    #[flag(rc(0))]
    reserved2: B4,
    /// Single message buffer
    #[flag(r)]
    smb: B1,
    /// Transmit only
    #[flag(r)]
    xmt: B1,
    /// Activity LED hardware driven
    #[flag(r)]
    alhd: B1,
    /// Port multiplier support
    #[flag(r)]
    pm: B1,
    #[flag(rc(0))]
    reserved3: B4,
}

/// CAP2
#[bitfields]
pub struct HostCapabilitiesExtended {
    /// BIOS/OS handoff supported
    #[flag(r)]
    boh: B1,
    /// NVMHCI present
    #[flag(r)]
    nvmp: B1,
    /// Automatic partial to slumber transitions supported
    #[flag(r)]
    apst: B1,
    /// Support device sleep
    #[flag(r)]
    sds: B1,
    /// Aggressive device sleep management supported
    #[flag(r)]
    sadm: B1,
    /// DevSleep entrance from slumber only
    #[flag(r)]
    deso: B1,
    #[flag(rc(0))]
    reserved: B26,
}

/// BOHC
#[bitfields]
pub struct BiosOsControlStatus {
    /// BIOS owned semaphore
    bos: B1,
    /// OS owned semaphore
    oos: B1,
    /// SMI on OS ownership change enable
    sooe: B1,
    /// OS ownership change
    #[flag(rwc(0))]
    ooc: B1,
    /// BIOS busy
    bb: B1,
    #[flag(rc(0))]
    reserved: B27,
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
#[bitfields]
pub struct PortInterruptStatus {
    /// Device to Host Register FIS Interrupt
    #[flag(rwc(1))]
    dhrs: B1,
    /// PIO Setup FIS Interrupt
    #[flag(rwc(1))]
    pss: B1,
    /// DMA Setup FIS Interrupt
    #[flag(rwc(1))]
    dss: B1,
    /// Set Device Bits Interrupt
    #[flag(rwc(1))]
    sdbs: B1,
    /// Unknown FIS Interrupt
    #[flag(r)]
    ufs: B1,
    /// Descriptor Processed
    #[flag(rwc(1))]
    dps: B1,
    /// Port Connect Change Status
    #[flag(r)]
    pcs: B1,
    /// Device Mechanical Presence Status
    #[flag(rwc(1))]
    dmps: B1,
    #[flag(rc(0))]
    reserved0: B14,
    /// PhyRdy Change Status
    #[flag(r)]
    prcs: B1,
    /// Incorrect Port Multiplier Status
    #[flag(rwc(1))]
    ipms: B1,
    /// Overflow Status
    #[flag(rwc(1))]
    ofs: B1,
    #[flag(rc(0))]
    reserved1: B1,
    /// Interface Non-fatal Error Status
    #[flag(rwc(1))]
    infs: B1,
    /// Interface Fatal Error Status
    #[flag(rwc(1))]
    ifs: B1,
    /// Host Bus Data Error Status
    #[flag(rwc(1))]
    hbds: B1,
    /// Host bust fatal error status
    #[flag(rwc(1))]
    hbfs: B1,
    /// Task file error status
    #[flag(rwc(1))]
    tfes: B1,
    /// Cold port detect status
    #[flag(rwc(1))]
    cpds: B1,
}

impl PortInterruptStatus {
    pub fn clear_pending_interrupts(&mut self) {
        write_volatile!(self.0, u32::MAX);
    }
}

/// Port X Interrupt Enable
#[bitfields]
pub struct InterruptEnable {
    /// Device to Host Register FIS Interrupt Enable
    #[flag(rw)]
    dhre: B1,
    /// PIO Setup FIS Interrupt Enable
    #[flag(rw)]
    pse: B1,
    /// DMA Setup FIS Interrupt Enable
    #[flag(rw)]
    dse: B1,
    /// Set Device Bits FIS Interrupt Enable
    #[flag(rw)]
    sdbe: B1,
    /// Unknown FIS Interrupt Enable
    #[flag(rw)]
    ufe: B1,
    /// Descriptor Processed Interrupt Enable
    #[flag(rw)]
    dpe: B1,
    /// Port Change Interrupt Enable
    #[flag(rw)]
    pce: B1,
    /// Device Mechanical Presence Enable
    #[flag(rw)]
    dmpe: B1,
    #[flag(rc(0))]
    reserved0: B14,
    /// PhyRdy Change Interrupt Enable
    #[flag(rw)]
    prce: B1,
    /// Incorrect Port Multiplier Enable
    #[flag(rw)]
    ipme: B1,
    /// Overflow Enable
    #[flag(rw)]
    ofe: B1,
    #[flag(rc(0))]
    reserved1: B1,
    /// Interface Non-fatal Error Enable
    #[flag(rw)]
    infe: B1,
    /// Interface Fatal Error Enable
    #[flag(rw)]
    ife: B1,
    /// Host Bus Data Error Enable
    #[flag(rw)]
    hbde: B1,
    /// Host Bus Fatal Error Enable
    #[flag(rw)]
    hbfe: B1,
    /// Task File Error Enable
    #[flag(rw)]
    tfee: B1,
    /// Cold Presence Detect Enable
    #[flag(rw)]
    cpde: B1,
}

/// Port X Command and status
#[bitfields]
pub struct CmdStatus {
    /// Start
    st: B1,
    /// Spin-Up Device
    sud: B1,
    /// Power On Device
    pod: B1,
    /// Command List Override
    clo: B1,
    /// FIS Receive Enable
    fre: B1,
    #[flag(rc(0))]
    reserved0: B3,
    /// Current command slot being issued
    #[flag(r)]
    current_cmd: B5,
    /// Mechanical Presence Switch State
    #[flag(r)]
    mpss: B1,
    /// FIS Receive Running
    #[flag(r)]
    fr: B1,
    /// Command List Running
    #[flag(r)]
    cr: B1,
    /// Cold Presence State
    #[flag(r)]
    cps: B1,
    /// Port Multiplier Attached
    #[flag(rw)]
    pma: B1,
    /// Hot Plug Capable Port
    #[flag(r)]
    hpcp: B1,
    /// Mechanical Presence Switch Attached to Port
    #[flag(r)]
    mpsp: B1,
    /// Cold Presence Detection
    #[flag(r)]
    cpd: B1,
    /// External SATA Port
    #[flag(r)]
    esp: B1,
    /// FIS-based Switching Capable Port
    #[flag(r)]
    fbscp: B1,
    /// Automatic Partial to Slumber Transitions Enabled
    apste: B1,
    /// Device is ATAPI
    atapi: B1,
    /// Drive LED on ATAPI Enable
    dlae: B1,
    /// Aggressive Link Power Management Enable
    alpe: B1,
    /// Aggressive Slumber / Partial
    asp: B1,
    /// Interface Communication Control
    #[flag(rw, flag_type = InterfaceCommunicationControl)]
    icc: B4,
}

impl CmdStatus {
    pub fn get_current_cmd_checked(&mut self) -> u32 {
        if !self.is_st() {
            return 0;
        }
        self.get_current_cmd() as u32
    }

    pub fn start(&mut self) {
        while self.is_cr() {}
        self.set_fre(true);
        self.set_st(true);
    }

    pub fn stop(&mut self) {
        self.set_st(false);
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
        self.set_fre(false);
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
#[bitfields]
pub struct TaskFileData {
    /// Indicates error during transfer
    #[flag(r)]
    err: B1,
    #[flag(rc(0))]
    reserved0: B2,
    /// Indicates a data transfer request
    #[flag(r)]
    drq: B1,
    #[flag(rc(0))]
    reserved1: B3,
    /// Indicates that the interface is busy
    #[flag(r)]
    bsy: B1,
    error_byte: B8,
    #[flag(rc(0))]
    reserved2: B16,
}

/// Port X Signature
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
#[bitfields]
pub struct SataStatus {
    /// Device detection
    #[flag(flag_type = DeviceDetection)]
    device_detection: B4,
    /// Interface speed
    #[flag(flag_type = InterfaceSpeed)]
    interface_speed: B4,
    /// Interface power management
    #[flag(flag_type = InterfacePowerManagement)]
    power_management: B4,
    #[flag(rc(0))]
    reserved: B20,
}
/// Port X SATA control
#[bitfields]
pub struct SataControl {
    /// Device initialization
    #[flag(flag_type = InterfaceInitialization)]
    device_init: B4,
    /// Max interface speed restriction
    #[flag(flag_type = InterfaceSpeed)]
    max_speed: B4,
    /// Partial power management disabled
    #[flag(rw)]
    partial_disabled: B1,
    /// Slumber power management disabled
    #[flag(rw)]
    slumber_disabled: B1,
    /// Device sleep power management disabled
    #[flag(rw)]
    devslp_disabled: B1,
    #[flag(rc(0))]
    reserved0: B1,
    /// Select power management transitions
    select_power_management: B4,
    /// Port multiplier port
    port_multiplier: B4,
    #[flag(rc(0))]
    reserved1: B12,
}

#[bitfields]
pub struct DiagnosticError {
    phyrdy_change: B1,
    phy_internal: B1,
    comm_wake: B1,
    decoding_error: B1,
    disparity_error: B1,
    crc_error: B1,
    handshake_error: B1,
    link_sequence_error: B1,
    transport_state_error: B1,
    unknown_fistype: B1,
    exchanged: B1,
}

#[bitfields]
pub struct AhciError {
    recovered_data_integrity_err: B1,
    recovered_communication_err: B1,
    reserved: B6,
    data_intergrity_err: B1,
    persistent_comm_or_data_integrity_err: B1,
    protocol_err: B1,
    internal_err: B1,
}

/// Port X SATA error
#[bitfields]
pub struct SataError {
    /// AHCI error bits
    #[flag(flag_type = AhciError)]
    error: B16,
    /// Diagnostic error bits
    #[flag(flag_type = DiagnosticError)]
    diagnostic: B16,
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
#[bitfields]
pub struct SataNotification {
    /// Per-port-multiplier-port notification bits (ports 0-14)
    pm_notifications: B15,
    #[flag(rc(0))]
    reserved: B17,
}

impl SataNotification {
    /// Set port multiplier notification
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
#[bitfields]
pub struct FisSwitchControl {
    /// Enable, should be set if there is a port multiplier
    #[flag(rw)]
    en: B1,
    /// Device error clear
    #[flag(rwc(1))]
    dec: B1,
    /// Single device error
    #[flag(r)]
    sde: B1,
    #[flag(rc(0))]
    reserved0: B5,
    /// Set the port multiplier port number that should receive the next
    /// command
    device_to_issue: B4,
    /// The number of devices that FIS-Based switching has been optimized
    /// for. The minimum value for this field should be 0x2.
    #[flag(r)]
    active_device_optimization: B4,
    /// Port multiplier device that experienced fatal error
    #[flag(r)]
    device_with_error: B4,
    #[flag(rc(0))]
    reserved1: B12,
}

/// Port x Device sleep
#[bitfields]
pub struct DeviceSleep {
    /// Aggressive device sleep enable
    #[flag(r)]
    adse: B1,
    /// Device sleep present
    #[flag(r)]
    dsp: B1,
    /// Device sleep exit timeout
    ///
    /// TODO: currently only read only, if write needed, check
    /// documentation about extended cap and writing to this offset
    #[flag(r)]
    deto: B8,
    /// Minimum device sleep assertion time
    ///
    /// TODO: currently only read only, if write needed, check
    /// documentation about extended cap and writing to this offset
    #[flag(r)]
    mdat: B5,
    /// Raw dito value
    ///
    /// **Use [`dito_actual_ms`] for the actual wait time**
    #[flag(r)]
    dito: B10,
    /// Device Sleep Idle Timeout Multiplier
    #[flag(r)]
    dito_multiplier: B4,
    #[flag(rc(0))]
    reserved: B3,
}

impl DeviceSleep {
    /// The actual timeout, which is dito * (dito_multiplier + 1)
    pub fn dito_actual_ms(&self) -> u16 {
        self.get_dito() as u16 * (self.get_dito_multiplier() as u16 + 1)
    }
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
        self.fb.write((ptr & 0xffffffff) as u32);
        self.fbu.write((ptr >> 32) as u32);
    }

    pub fn set_status(&mut self, port: u8) {
        self.cmd.set_st(true);
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

    // pub fn identity_packet(&mut self, buf: *mut IdentityPacketData) {
    //     let fis = RegisterH2D::new(
    //         1 << 7,
    //         AtaCommand::IdentifyDevice,
    //         0,
    //         0,
    //         0,
    //         0,
    //         0,
    //     );
    //     let cmd = &mut self.cmd_list().entries[0];
    //     let cmd_table = &mut cmd.cmd_table::<8>();
    //     let prdt_ent = &mut cmd_table.table[0];
    //     write_volatile!(cmd_table.cfis, Fis { h2d: fis });
    //     prdt_ent.set_buffer(buf);
    //     prdt_ent.dbc.set_dbc(511);
    //     cmd.info.set_command_fis_len(size_of::<RegisterH2D>());
    //     cmd.info.set_prdtl(1);
    //     self.ci.issue_cmd(0);

    //     let mut timeout = 0xfffff;
    //     loop {
    //         if self.is.0 != 0 {
    //             if self.is.is_tfes() {
    //                 eprintln!("ERROR READING FROM DISK");
    //                 for error in self.serr.error() {
    //                     println!("{:?}", error);
    //                 }
    //                 if self.tfd.is_err() {
    //                     println!(
    //                         "TASK FILE DATA ERROR STATE\nERROR: {:08b}",
    //                         self.tfd.error()
    //                     );
    //                 }
    //             }
    //             println!("Finished!");
    //             println!("{:032b}", self.is.0);
    //             break;
    //         } else {
    //             timeout -= 1
    //         }

    //         if timeout == 0 {
    //             panic!("Timeout on identity packet read")
    //         }
    //     }
    //     unsafe {
    //         for w in (&mut *buf).serial_number.chunks_exact_mut(2) {
    //             w.swap(0, 1);
    //         }
    //         for w in (&mut *buf).model_num.chunks_exact_mut(2) {
    //             w.swap(0, 1);
    //         }
    //         for w in (&mut *buf).firmware_rev.chunks_exact_mut(2) {
    //             w.swap(0, 1);
    //         }
    //     }
    // }
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

#[bitfields]
pub struct CmdListDescriptionInfo {
    /// Length of command FIS (internally stored as dwords)
    cfl: B5,
    /// ATAPI
    #[flag(rw)]
    a: B1,
    /// Write
    #[flag(rw)]
    w: B1,
    /// Prefetchable
    #[flag(rw)]
    p: B1,
    /// Reset
    #[flag(rw)]
    r: B1,
    /// BIST
    #[flag(rw)]
    b: B1,
    /// Clear busy upon R_OK
    #[flag(rw)]
    c: B1,
    #[flag(rc(0))]
    reserved: B1,
    /// Port multiplier port
    pm_port: B4,
    /// Physical region descriptor table length
    prdtl: B16,
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
        self.ctba.write((ptr & 0xffffffff) as u32);
        self.ctbau.write((ptr >> 32) as u32);
    }
}

#[repr(C, align(1024))]
pub struct CmdList {
    pub entries: [CmdHeader; 32],
}

#[bitfields]
pub struct PrdtDescriptionInfo {
    /// Data byte count (max 4MiB, bit 0 is always set per spec)
    dbc: B22,
    #[flag(rc(0))]
    reserved: B9,
    /// Interrupt on completion
    #[flag(rw)]
    i: B1,
}

impl PrdtDescriptionInfo {
    /// Set the data byte count of the buffer on the prdt
    pub fn set_dbc_checked(&mut self, dbc: u32) {
        const MB: u32 = 1 << 20;
        assert!(dbc < 4 * MB, "DBC should be smaller then 4Mib");
        self.set_dbc(dbc | 1);
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
    pub fn new(
        a: PhysicalAddress,
    ) -> Result<NonNull<HBAMemoryRegisters>, HbaError> {
        if !a.is_aligned(REGULAR_PAGE_ALIGNMENT) {
            return Err(HbaError::AddressNotAligned);
        }

        // TODO: map this address
        // a.map(
        //     a.translate(),
        //     PageEntryFlags::regular_io_page_flags(),
        //     PageSize::Regular,
        // );

        let mut hba_ptr =
            a.translate().as_non_null::<HBAMemoryRegisters>();

        let hba = unsafe { hba_ptr.as_mut() };

        hba.ghc.ghc.set_ae(true);
        hba.ghc.ghc.set_ie(true);

        if hba.ghc.pi.0 >= (1 << 31) {
            panic!("There is no support for HBA's with more then 30 ports")
        }

        if hba.ghc.cap_ext.is_boh() {
            unimplemented!("Didn't implement bios os handoff")
        }

        Ok(hba_ptr)
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
