use core::arch::asm;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct JmpBufData {
    regs: [u64; 13],
}

impl JmpBufData {
    pub const fn new() -> Self {
        Self { regs: [0; 13] }
    }

    pub fn regs(&self) -> &[u64] {
        self.regs.as_ref()
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct JmpBuf([JmpBufData; 1]);

impl JmpBuf {
    pub const fn new() -> Self {
        Self([JmpBufData::new(); 1])
    }

    pub fn data(&self) -> &[JmpBufData] {
        &self.0
    }
}

#[link_section = ".text.setjmp"]
pub fn setjmp(jmp: &JmpBuf) -> i32 {
    let ret: i32;
    unsafe {
        asm!(
            // Preserve all callee-saved registers and the SP
            "stp x19, x20, [x0,#0]",
            "stp x21, x22, [x0,#16]",
            "stp x23, x24, [x0,#32]",
            "stp x25, x26, [x0,#48]",
            "stp x27, x28, [x0,#64]",
            "stp x29, x30, [x0,#80]",
            "mov x2, sp",
            "str x2, [x0,#96]",
            "mov x1, #0",
            "ret",
            in("x0") jmp.data()[0].regs().as_ptr(),
            out("x1") ret,
        );
    }
    ret
}

#[link_section = ".text.longjmp"]
pub fn longjmp(jmp: &JmpBuf, mut ret: i32) -> i32 {
    unsafe {
        asm!(
            "ldp x19, x20, [x0,#0]",
            "ldp x21, x22, [x0,#16]",
            "ldp x23, x24, [x0,#32]",
            "ldp x25, x26, [x0,#48]",
            "ldp x27, x28, [x0,#64]",
            "ldp x29, x30, [x0,#80]",
            "ldr x2, [x0,#96]",
            "mov sp, x2",
            // Move the return value in place, but return 1 if passed 0
            "adds x0, xzr, x1",
            "csinc x1, x0, xzr, ne",
            "ret",
            in("x0") jmp.data()[0].regs().as_ptr(),
            inout("x1") ret,
        );
    }
    ret
}
