use super::MHz;

#[repr(C)]
pub(crate) struct ClkPriv {
    cru: Cru,
}

pub const KHz: usize = 1000;
pub const OSC_HZ: usize = 24 * MHz;
pub const LPLL_HZ: usize = 600 * MHz;
pub const BPLL_HZ: usize = 600 * MHz;
pub const GPLL_HZ: usize = 594 * MHz;
pub const CPLL_HZ: usize = 384 * MHz;
pub const PPLL_HZ: usize = 676 * MHz;

pub const PMU_PCLK_HZ: usize = 48 * MHz;

pub const ACLKM_CORE_L_HZ: usize = 300 * MHz;
pub const ATCLK_CORE_L_HZ: usize = 300 * MHz;
pub const PCLK_DBG_L_HZ: usize = 300 * MHz;

pub const PERIHP_ACLK_HZ: usize = 148500 * KHz;
pub const PERIHP_HCLK_HZ: usize = 148500 * KHz;
pub const PERIHP_PCLK_HZ: usize = 37125 * KHz;

pub const PERILP0_ACLK_HZ: usize = 99000 * KHz;
pub const PERILP0_HCLK_HZ: usize = 99000 * KHz;
pub const PERILP0_PCLK_HZ: usize = 49500 * KHz;

pub const PERILP1_HCLK_HZ: usize = 99000 * KHz;
pub const PERILP1_PCLK_HZ: usize = 49500 * KHz;

pub const PWM_CLK_HZ: usize = PMU_PCLK_HZ;

#[repr(C)]
#[derive(Clone, Copy, PartialEq)]
pub struct Cru {
    pub apll_l_con: [u32; 6],
    reserved: [u32; 2],
    pub apll_b_con: [u32; 6],
    reserved1: [u32; 2],
    pub dpll_con: [u32; 6],
    reserved2: [u32; 2],
    pub cpll_con: [u32; 6],
    reserved3: [u32; 2],
    pub gpll_con: [u32; 6],
    reserved4: [u32; 2],
    pub npll_con: [u32; 6],
    reserved5: [u32; 2],
    pub vpll_con: [u32; 6],
    reserved6: [u32; 0xa],
    pub clksel_con: [u32; 108],
    reserved7: [u32; 0x14],
    pub clkgate_con: [u32; 35],
    reserved8: [u32; 0x1d],
    pub softrst_con: [u32; 21],
    reserved9: [u32; 0x2b],
    pub glb_srst_fst_value: u32,
    pub glb_srst_snd_value: u32,
    pub glb_cnt_th: u32,
    pub misc_con: u32,
    pub glb_rst_con: u32,
    pub glb_rst_st: u32,
    reserved10: [u32; 0x1a],
    pub sdmmc_con: [u32; 2],
    pub sdio0_con: [u32; 2],
    pub sdio1_con: [u32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq)]
pub enum ApllLFrequencies {
    Mhz1600,
    Mhz600,
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq)]
pub enum ApllBFrequencies {
    Mhz600,
}
