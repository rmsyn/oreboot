use crate::config::NR_DRAM_BANKS;

/// Board information passed to Linux kernel from oreboot
///
/// (originally bd_info from U-Boot)
#[repr(C)]
pub struct BoardInfo {
    /// start of FLASH memory
    flashstart: u32,
    /// size of FLASH memory
    flashsize: u32,
    /// reserved area for startup monitor
    flashoffset: u32,
    /// start of SRAM memory
    sramstart: u32,
    /// size of SRAM memory
    sramsize: u32,
    /// boot / reboot flag (Unused)
    bootflags: u32,
    /// IP Address
    ip_addr: u32,
    /// Ethernet speed in Mbps
    ethspeed: u16,
    /// Internal Freq, in MHz
    intfreq: u32,
    /// Bus Freq, in MHz
    busfreq: u32,
    /// unique id for this board
    arch_number: u32,
    /// where this board expects params
    boot_params: u32,
    /// RAM configuration
    dram: [BoardInfoDram; NR_DRAM_BANKS],
}

impl BoardInfo {
    pub const fn new() -> Self {
        Self {
            flashstart: 0,
            flashsize: 0,
            flashoffset: 0,
            sramstart: 0,
            sramsize: 0,
            bootflags: 0,
            ip_addr: 0,
            ethspeed: 0,
            intfreq: 0,
            busfreq: 0,
            arch_number: 0,
            boot_params: 0,
            dram: [BoardInfoDram::new(); NR_DRAM_BANKS],
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BoardInfoDram {
    start: u64,
    size: usize,
}

impl BoardInfoDram {
    pub const fn new() -> Self {
        Self {
            start: 0,
            size: 0,
        }
    }

    pub fn start(&self) -> u64 {
        self.start
    }

    pub fn size(&self) -> usize {
        self.size
    }
}
