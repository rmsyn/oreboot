#![no_std]

#[cfg(feature = "amd")]
pub mod amd;

#[cfg(feature = "armltd")]
pub mod armltd;

#[cfg(feature = "lowrisc")]
pub mod lowrisc;

#[cfg(feature = "arm64")]
pub mod arm64;
