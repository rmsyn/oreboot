#[cfg(feature = "rk3399")]
pub const NR_DRAM_BANKS: usize = 1;

// Default number of DRAM banks
#[cfg(not(feature = "rk3399"))]
pub const NR_DRAM_BANKS: usize = 1;
