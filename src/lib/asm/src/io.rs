use core::ptr::{read_volatile, write_volatile};

///! IO functions taken from u-boot headers: <arch/arm/include/asm/io.h>

fn _raw_readb(a: usize) -> u8 {
    unsafe { read_volatile::<u8>(a as *const u8) }
}

fn _raw_readw(a: usize) -> u16 {
    unsafe { read_volatile::<u16>(a as *const u16) }
}

fn _raw_readl(a: usize) -> u32 {
    unsafe { read_volatile::<u32>(a as *const u32) }
}

fn _raw_readq(a: usize) -> u64 {
    unsafe { read_volatile::<u64>(a as *const u64) }
}

fn _raw_writeb(v: u8, a: usize) {
    unsafe { write_volatile::<u8>(a as *mut u8, v) }
}

fn _raw_writew(v: u16, a: usize) {
    unsafe { write_volatile::<u16>(a as *mut u16, v) }
}

fn _raw_writel(v: u32, a: usize) {
    unsafe { write_volatile::<u32>(a as *mut u32, v) }
}

fn _raw_writeq(v: u64, a: usize) {
    unsafe { write_volatile::<u64>(a as *mut u64, v) }
}

fn readb_relaxed(c: usize) -> u8 {
    _raw_readb(c)
}

fn readw_relaxed(c: usize) -> u16 {
    let u = _raw_readw(c);
    let b = u.to_le_bytes();
    u16::from_ne_bytes(u16::from_le_bytes(b).to_ne_bytes())
}

fn readl_relaxed(c: usize) -> u32 {
    let u = _raw_readl(c);
    let b = u.to_le_bytes();
    u32::from_ne_bytes(u32::from_le_bytes(b).to_ne_bytes())
}

fn readq_relaxed(c: usize) -> u64 {
    let u = _raw_readq(c);
    let b = u.to_le_bytes();
    u64::from_ne_bytes(u64::from_le_bytes(b).to_ne_bytes())
}

fn writeb_relaxed(v: u8, c: usize) {
    _raw_writeb(v, c);
}

fn writew_relaxed(v: u16, c: usize) {
    _raw_writew(v, c);
}

fn writel_relaxed(v: u32, c: usize) {
    _raw_writel(v, c);
}

fn writeq_relaxed(v: u64, c: usize) {
    _raw_writeq(v, c);
}

pub fn readb(c: usize) -> u8 {
    readb_relaxed(c)
}

pub fn readw(c: usize) -> u16 {
    readw_relaxed(c)
}

pub fn readl(c: usize) -> u32 {
    readl_relaxed(c)
}

pub fn readq(c: usize) -> u64 {
    readq_relaxed(c)
}

pub fn writeb(v: u8, c: usize) {
    writeb_relaxed(v, c);
}

pub fn writew(v: u16, c: usize) {
    writew_relaxed(v, c);
}

pub fn writel(v: u32, c: usize) {
    writel_relaxed(v, c);
}

pub fn writeq(v: u64, c: usize) {
    writeq_relaxed(v, c);
}
