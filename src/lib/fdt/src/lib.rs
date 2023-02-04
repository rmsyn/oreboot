/// Indicates where the devicetree came from
/// 
/// These are listed in approximate order of desirability after FdtSource::None
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FdtSource {
    /// Appended to oreboot. This is the normal approach if U-Boot
    ///	is the only firmware being booted
    Separate,
    /// Found in a multi-dtb FIT. This should be used when oreboot must
    /// select a devicetree from many options
    Fit,
    /// Located by custom board code. This should only be used when
    /// the prior stage does not support FDTSRC_PASSAGE
    Board,
    /// Embedded into oreboot executable. This should only be used when
    /// oreboot is packaged as an ELF file, e.g. for debugging purposes
    Embed,
    /// Provided by the fdtcontroladdr environment variable. This should
    /// be used for debugging/development only
    Env,
    /// No devicetree at all
    None,
}
