use crate::{
    acpigen::{AcpiGen, Error, LOCAL0_OP},
    soc::amd::{
        common::block::amdblocks::gpio_defs::AMD_GPIO_FIRST_REMOTE_GPIO_NUMBER,
        picasso::soc::gpio::SOC_GPIO_TOTAL_PINS,
    },
};

use log::error;

impl AcpiGen {
    pub fn soc_gpio_op(&mut self, op: &str, gpio_num: u32) -> Result<(), Error> {
        if gpio_num as usize >= SOC_GPIO_TOTAL_PINS {
            error!(
                "Pin {} should be smaller than {}",
                gpio_num, SOC_GPIO_TOTAL_PINS
            );
            return Err(Error::InvalidGpioPins);
        }
        if SOC_GPIO_TOTAL_PINS >= AMD_GPIO_FIRST_REMOTE_GPIO_NUMBER
            && gpio_num as usize >= SOC_GPIO_TOTAL_PINS
        {
            error!(
                "Pin {} is a remote GPIO which isn't supported yet",
                gpio_num
            );
            return Err(Error::InvalidGpioPins);
        }
        self.emit_namestring(op)?;
        self.write_integer(gpio_num as u64)
    }

    pub fn soc_get_gpio_state(&mut self, op: &str, gpio_num: u32) -> Result<(), Error> {
        if gpio_num as usize >= SOC_GPIO_TOTAL_PINS {
            error!(
                "Pin {} should be smaller than {}",
                gpio_num, SOC_GPIO_TOTAL_PINS
            );
            return Err(Error::InvalidGpioPins);
        }
        if SOC_GPIO_TOTAL_PINS >= AMD_GPIO_FIRST_REMOTE_GPIO_NUMBER
            && gpio_num as usize >= SOC_GPIO_TOTAL_PINS
        {
            error!(
                "Pin {} is a remote GPIO which isn't supported yet",
                gpio_num
            );
            return Err(Error::InvalidGpioPins);
        }
        self.write_store()?;
        self.soc_gpio_op(op, gpio_num)?;
        self.emit_byte(LOCAL0_OP)
    }

    pub fn soc_read_rx_gpio(&mut self, gpio_num: u32) -> Result<(), Error> {
        self.soc_get_gpio_state("\\_SB.GRXS", gpio_num)
    }

    pub fn soc_get_tx_gpio(&mut self, gpio_num: u32) -> Result<(), Error> {
        self.soc_get_gpio_state("\\_SB.GTXS", gpio_num)
    }

    pub fn soc_set_tx_gpio(&mut self, gpio_num: u32) -> Result<(), Error> {
        self.soc_gpio_op("\\_SB.STXS", gpio_num)
    }

    pub fn soc_clear_tx_gpio(&mut self, gpio_num: u32) -> Result<(), Error> {
        self.soc_gpio_op("\\_SB.CTXS", gpio_num)
    }
}
