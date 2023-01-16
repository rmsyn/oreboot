#![allow(non_upper_case_globals)]

use crate::rockchip::bit;

pub const MHz: usize = 1_000_000;

#[cfg(feature = "rockchip_rk3399")]
pub mod rk3399;

#[repr(u32)]
#[derive(Clone, Copy, PartialEq)]
pub enum GlbRstSt {
    GlbPorRst,
    FstGlbRstSt = bit(0),
    SndGlbRstSt = bit(1),
    FstGlbTsadcRstSt = bit(2),
    SndGlbTsadcRstSt = bit(3),
    FstGlbWdtRstSt = bit(4),
    SndGlbWdtRstSt = bit(5),
}
