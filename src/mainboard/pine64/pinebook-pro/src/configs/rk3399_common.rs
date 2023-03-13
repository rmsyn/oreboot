pub const CONFIG_SYS_CBSIZE:         u32 = 1024;

pub const COUNTER_FREQUENCY:           u32 = 24000000;
pub const CONFIG_ROCKCHIP_STIMER_BASE: u32 = 0xff8680a0;

pub const CONFIG_IRAM_BASE: u32 = 0xff8c0000;

pub const CONFIG_SYS_INIT_SP_ADDR: u32 = 0x00300000;
pub const CONFIG_SYS_LOAD_ADDR:    u32 = 0x00800800;

#[cfg(all(feature = "spl", feature = "tpl_bootrom_support"))]
mod spl_config {
    pub const CONFIG_SPL_STACK:          u32 = 0x00400000;
    pub const CONFIG_SPL_MAX_SIZE:       u32 = 0x40000;
    pub const CONFIG_SPL_BSS_START_ADDR: u32 = 0x00400000;
    pub const CONFIG_SPL_BSS_MAX_SIZE:   u32 = 0x2000;
}
#[cfg(all(feature = "spl", feature = "tpl_bootrom_support"))]
pub use spl_config::*;

#[cfg(not(any(feature = "spl", feature = "tpl_bootrom_support")))]
mod spl_config {
    pub const CONFIG_SPL_STACK:    u32 = 0xff8effff;
    pub const CONFIG_SPL_MAX_SIZE: u32 = 0x30000 - 0x2000;
    
    /*  BSS setup */
    pub const CONFIG_SPL_BSS_START_ADDR: u32 = 0xff8e0000;
    pub const CONFIG_SPL_BSS_MAX_SIZE:   u32 = 0x10000;
}
#[cfg(not(any(feature = "spl", feature = "tpl_bootrom_support")))]
pub use spl_config::*;


pub const CONFIG_SYS_BOOTM_LEN: u32 = 64 << 20; /* 64M */

// MMC/SD IP block
pub const CONFIG_ROCKCHIP_SDHCI_MAX_FREQ: u32 = 200000000;

// RAW SD card / eMMC locations.

// FAT sd card locations.
pub const CONFIG_SYS_SDRAM_BASE: u32 = 0;
pub const SDRAM_MAX_SIZE:        u32 = 0xf8000000;

#[cfg(feature = "spl")]
pub const ENV_MEM_LAYOUT_SETTINGS: &str = "scriptaddr=0x00500000\0\
                                           script_offset_f=0xffe000\0\
                                           script_size_f=0x2000\0\
                                           pxefile_addr_r=0x00600000\0\
                                           fdt_addr_r=0x01f00000\0\
                                           fdtoverlay_addr_r=0x02000000\0\
                                           kernel_addr_r=0x02080000\0\
                                           ramdisk_addr_r=0x06000000\0\
                                           kernel_comp_addr_r=0x08000000\0\
                                           kernel_comp_size=0x2000000\0";
