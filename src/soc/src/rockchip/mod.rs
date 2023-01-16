pub mod boot_mode;
pub mod bootrom;
pub mod config;
pub mod cru;

pub const fn bit(nr: u32) -> u32 {
    1u32 << nr
}
