#![doc = include_str!("../README.md")]
#![no_std]
#![no_main]

mod configs;

use core::panic::PanicInfo;

#[no_mangle]
unsafe extern "C" fn main() -> usize {
    0
}

#[cfg_attr(not(test), panic_handler)]
fn panic(info: &PanicInfo) -> ! {
    if let Some(_location) = info.location() {
        // FIXME: uncomment with logging implementation
        //println!("panic in '{}' line {}", location.file(), location.line(),);
    } else {
        // FIXME: uncomment with logging implementation
        //println!("panic at unknown location");
    };
    loop {
        core::hint::spin_loop();
    }
}
