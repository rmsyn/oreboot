pub mod jmp;

pub fn hang() -> ! {
    loop {
        aarch64_cpu::asm::wfi()
    }
}
