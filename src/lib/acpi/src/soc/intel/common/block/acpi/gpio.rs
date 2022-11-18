use crate::acpigen::{AcpiGen, Error, LOCAL0_OP};

impl AcpiGen {
    pub fn soc_gpio_op(&mut self, op: &str, gpio_num: u32) -> Result<(), Error> {
        /* op (gpio_num) */
        self.emit_namestring(op)?;
        self.write_integer(gpio_num as u64)
    }

    pub fn soc_get_gpio_state(&mut self, op: &str, gpio_num: u32) -> Result<(), Error> {
        /* Store (op (gpio_num), Local0) */
        self.write_store()?;
        self.soc_gpio_op(op, gpio_num)?;
        self.emit_byte(LOCAL0_OP)
    }

    pub fn soc_read_rx_gpio(&mut self, gpio_num: u32) -> Result<(), Error> {
        self.soc_get_gpio_state("\\_SB.PCI0.GRXS", gpio_num)
    }

    pub fn soc_get_tx_gpio(&mut self, gpio_num: u32) -> Result<(), Error> {
        self.soc_get_gpio_state("\\_SB.PCI0.GTXS", gpio_num)
    }

    pub fn soc_set_tx_gpio(&mut self, gpio_num: u32) -> Result<(), Error> {
        self.soc_gpio_op("\\_SB.PCI0.STXS", gpio_num)
    }

    pub fn soc_clear_tx_gpio(&mut self, gpio_num: u32) -> Result<(), Error> {
        self.soc_gpio_op("\\_SB.PCI0.CTXS", gpio_num)
    }
}
