/// High 24 bits is tag, low 8 bits is type ;
pub const REBOOT_FLAG: usize = 0x5242_C300;
/// Normal boot
pub const BOOT_NORMAL: usize = REBOOT_FLAG + 0;
/// Enter loader rockusb mode
pub const BOOT_LOADER: usize = REBOOT_FLAG + 1;
/// Enter recovery
pub const BOOT_RECOVERY: usize = REBOOT_FLAG + 3;
/// Enter fastboot mode
pub const BOOT_FASTBOOT: usize = REBOOT_FLAG + 9;
/// Enter charging mode
pub const BOOT_CHARGING: usize = REBOOT_FLAG + 11;
/// Enter usb mass storage mode
pub const BOOT_UMS: usize = REBOOT_FLAG + 12;
/// Enter bootrom download mode
pub const BOOT_BROM_DOWNLOAD: usize = 0xEF08_A53C;
