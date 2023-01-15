use core::arch::asm;
use oreboot_asm::io::{readl, writel};
use oreboot_cpu::arm64::{
    hang,
    jmp::{setjmp, longjmp, JmpBuf},
};
use super::{boot_mode::BOOT_BROM_DOWNLOAD, config};

/// Locations of the boot-device identifier in SRAM
pub const BROM_BOOTSOURCE_ID_ADDR: usize = config::IRAM_BASE + 0x10;

#[link_section = ".data"]
static mut BROM_CTX: JmpBuf = JmpBuf::new();

/// back_to_bootrom() - return to bootrom (for TPL/SPL), passing a
///                     result code
///
/// Transfer control back to the Rockchip BROM, restoring necessary
/// register context and passing a command/result code to the BROM
/// to instruct its next actions (e.g. continue boot sequence, enter
/// download mode, ...).
///
/// This function does not return.
///
/// @brom_cmd: indicates how the bootrom should continue the boot
///            sequence (e.g. load the next stage)
#[repr(C)]
#[derive(Clone, Copy, PartialEq)]
pub enum BootromCmd {
	// These can not start at 0, as 0 has a special meaning
	// for setjmp().
    Reserved = 0,
    /// Continue boot-sequence
    NextStage = 1,
    /// Have BROM enter download-mode
    EnterDnl,
    Invalid = 255,
}

impl From<i32> for BootromCmd {
    fn from(i: i32) -> Self {
        match i {
            0 => Self::Reserved,
            1 => Self::NextStage,
            2 => Self::EnterDnl,
            _ => Self::Invalid,
        }
    }
}

/// Boot-device identifiers as used by the BROM
#[repr(C)]
#[derive(Clone, Copy, PartialEq)]
pub enum Bootsource {
    Nand = 1,
    Emmc = 2,
    SpiNor = 3,
    SpiNand = 4,
    Sd = 5,
    Usb = 10,
}

impl Bootsource {
    pub const fn last() -> Self {
        Self::Usb
    }
}

fn _back_to_bootrom(brom_cmd: BootromCmd) {
    unsafe { longjmp(&BROM_CTX, brom_cmd as i32) };
}

pub fn back_to_bootrom(brom_cmd: BootromCmd) {
    // with logging, uncomment
    // if cfg!(feature = "logging") {
    //     println!("Returning to boot ROM...");
    // }
    _back_to_bootrom(brom_cmd);
}

/// We back to bootrom download mode if get a
/// BOOT_BROM_DOWNLOAD flag in boot mode register
///
/// note: the boot mode register is configured by
/// application(next stage bootloader, kernel, etc),
/// and the bootrom never check this register, so we need
/// to check it and back to bootrom at very early bootstage(before
/// some basic configurations(such as interrupts) been
/// changed by TPL/SPL, as the bootrom download operation
/// relies on many default settings(such as interrupts) by
/// itself.
pub fn check_back_to_brom_dnl_flag() -> bool {
    if config::ROCKCHIP_BOOT_MODE_REG != 0 {
        let boot_mode = readl(config::ROCKCHIP_BOOT_MODE_REG);
        if boot_mode == BOOT_BROM_DOWNLOAD as u32 {
            writel(0, config::ROCKCHIP_BOOT_MODE_REG);
            return true;
        }
    }

    false
}

/// All rockchip brom implementations enter with a valid stack-pointer,
/// so this can safely be implemented in rust (providing a single
/// implementation both for armv7 and aarch64).
pub fn save_boot_params() -> i32 {
    let ret = unsafe { setjmp(&BROM_CTX) };

    match BootromCmd::from(ret) {
        BootromCmd::Reserved => {
            if check_back_to_brom_dnl_flag() {
                _back_to_bootrom(BootromCmd::EnterDnl);
            }
		    // This is the initial pass through this function
		    // (i.e. saving the context), setjmp just setup up the
		    // brom_ctx: transfer back into the startup-code at
		    // 'save_boot_params_ret' and let the compiler know
		    // that this will not return.
            unsafe { save_boot_params_ret() };
            // does not return
            loop { unsafe { asm!("nop") } }
        },
		// To instruct the BROM to boot the next stage, we
		// need to return 0 to it: i.e. we need to rewrite
		// the return code once more.
        BootromCmd::NextStage => return 0,
		// A non-zero return value will instruct the BROM enter
		// download mode.
        BootromCmd::EnterDnl => return 1,
        // uncomment with logging:
        // if cfg!(feature = "logging") {
        //     println!("FATAL: unexpected command to back_to_bootrom()");
        // }
        _ => hang(),
    }
}

// FIXME: may need to leave this in start.S
// From u-boot: u-boot/arch/arm/cpu/armv8/start.S
pub unsafe extern "C" fn save_boot_params_ret() {
    asm!("adr  x0, _start",
         "ands x0, x0, #0xfff",
         "b.eq 1f",
         "0:",
         // FATAL, can't continue.
         // U-Boot needs to be loaded at a 4K aligned address.
         //
         // We use ADRP and ADD to load some symbol addresses during startup.
         //
         // The ADD uses an absolute (non pc-relative) lo12 relocation
         // this requiring 4K alignment.
         // 1:
         "wfi",
         "b 0b",
         "1:",
    );
}
