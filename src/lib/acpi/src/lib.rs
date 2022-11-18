pub mod acpigen;
pub mod device;
pub mod pld;
pub mod soc;

pub const COREBOOT_ACPI_ID: &str = "BOOT";

pub enum CorebootAcpiIds {
    /// BOOT0000
    CbTable = 0x0000,
    /// BOOTFFFF
    Max = 0xffff,
}

#[repr(C, packed)]
pub struct AcpiSwPstate {
    pub core_freq: u32,
    pub power: u32,
    pub transition_latency: u32,
    pub bus_master_latency: u32,
    pub control_value: u32,
    pub status_value: u32,
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct AcpiAddr {
    /// Address space ID
    space_id: u8,
    /// Register size in bits
    bit_width: u8,
    /// Register bit offset
    bit_offset: u8,
    /// Access size since ACPI 2.0c
    access_size: u8,
    /// Register address, low 32 bits
    addrl: u32,
    /// Register address, high 32 bits
    addrh: u32,
}

/// Low Power Idle State
pub struct AcpiLpiState<'a> {
    pub min_residency_us: u32,
    pub worst_case_wakeup_latency_us: u32,
    pub flags: u32,
    pub arch_context_lost_flags: u32,
    pub residency_counter_frequency_hz: u32,
    pub enabled_parent_state: u32,
    pub entry_method: AcpiAddr,
    pub residency_counter_register: AcpiAddr,
    pub usage_counter_register: AcpiAddr,
    pub state_name: &'a str,
}

#[repr(C, packed)]
pub struct AcpiCstate {
    pub ctype: u8,
    pub latency: u16,
    pub power: u32,
    pub resource: AcpiAddr,
}

#[repr(C, packed)]
pub struct AcpiTstate {
    pub percent: u32,
    pub power: u32,
    pub latency: u32,
    pub control: u32,
    pub status: u32,
}

#[repr(C)]
#[derive(Debug, PartialEq)]
pub enum UpcType {
    A,
    MiniAb,
    ExpressCard,
    Usb3A,
    Usb3B,
    Usb3MicroB,
    Usb3MicroAb,
    Usb3PowerB,
    CUsb2Only,
    CUsb2SsSwitch,
    CUsb2Ss,
    Proprietary = 0xff,
    // The following types are not directly defined in the ACPI
    // spec but are used by coreboot to identify a USB device type.
    Internal,
    Unused,
    Hub,
}

#[repr(C, packed)]
pub struct XpssSwPstate {
    core_freq: u64,
    power: u64,
    transition_latency: u64,
    bus_master_latency: u64,
    control_value: u64,
    status_value: u64,
    control_mask: u64,
    status_mask: u64,
}
