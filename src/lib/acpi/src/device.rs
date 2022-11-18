pub const ACPI_GPIO_REVISION_ID: usize = 1;
pub const ACPI_GPIO_MAX_PINS: usize = 8;

#[repr(C)]
pub enum GpioType {
    Interrupt,
    Io,
}

#[repr(C)]
pub enum GpioPull {
    PullDefault,
    Up,
    Down,
    PullNone,
}

#[repr(C)]
pub enum IoRestrict {
    RestrictNone,
    Input,
    Output,
    Preserve,
}

#[repr(C)]
pub enum IrqMode {
    EdgeTriggered,
    LevelTriggered,
}

#[repr(C)]
pub enum IrqPolarity {
    ActiveLow,
    ActiveHigh,
    ActiveBoth,
}

#[repr(C)]
pub enum IrqShared {
    Exclusive,
    Shared,
}

#[repr(C)]
pub enum IrqWake {
    NoWake,
    Wake,
}

#[repr(C)]
pub struct Irq {
    pin: u32,
    mode: IrqMode,
    polarity: IrqPolarity,
    shared: IrqShared,
    wake: IrqWake,
}

#[repr(C)]
pub struct Gpio<'a> {
    pub pin_count: i32,
    pub pins: [u16; ACPI_GPIO_MAX_PINS],
    pub gpio_type: GpioType,
    pub pull: GpioPull,
    pub resource: &'a str,
    /* GpioInt */
    pub interrupt_debounce_timeout: u16, /* 1/100 ms */
    pub irq: Irq,
    /* GpioIo */
    pub output_drive_strength: u16, /* 1/100 mA */
    pub io_shared: i32,
    pub io_restrict: IoRestrict,
    /*
     * As per ACPI spec, GpioIo does not have any polarity associated with it. Linux kernel
     * uses `active_low` argument within GPIO _DSD property to allow BIOS to indicate if the
     * corresponding GPIO should be treated as active low. Thus, if the GPIO has active high
     * polarity or if it does not have any polarity, then the `active_low` argument is
     * supposed to be set to 0.
     *
     * Reference:
     * https://www.kernel.org/doc/html/latest/firmware-guide/acpi/gpio-properties.html
     */
    pub active_low: bool,
}
