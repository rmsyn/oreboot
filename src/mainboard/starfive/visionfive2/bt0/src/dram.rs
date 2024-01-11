use crate::ddr_start::start;
use crate::ddrcsr::omc_init;
use crate::ddrphy::{train, util};
use crate::init::{self, read32, udelay, write32};
use crate::pll;

// TODO: support 1G
#[cfg(not(any(dram_size = "2G", dram_size = "4G", dram_size = "8G")))]
core::compile_error!("unsupported DRAM size or none set");

// see StarFive U-Boot drivers/ram/starfive/starfive_ddr.c
pub fn init() {
    // TODO: determine DRAM size from EEPROM at runtime, it's stored on board.
    // That requires I2C first, see `arch/riscv/cpu/jh7110/dram.c` in U-Boot.
    let dram_size = if cfg!(dram_size = "2G") {
        2
    } else if cfg!(dram_size = "4G") {
        4
    } else if cfg!(dram_size = "8G") {
        8
    } else {
        0 // does not actually occur due to build-time check
    };
    println!("DRAM size: {dram_size}G");
    unsafe {
        println!("[DRAM] init start");
        println!("[DRAM] set clk to OSC div2");
        init::clk_ddrc_osc_div2();
        println!("[DRAM] set PLL frequency");
        pll::pll1_set_freq(pll::PLL1_DDR2133_1066000000);
        udelay(500);
        println!("[DRAM] set clk to PLL1 div2");
        init::clk_ddrc_pll1_div2();
        udelay(200);

        println!("[DRAM] asserts");
        // DDR OSC
        init::set_bit(init::SYS_CRG_RESET_ASSERT1, init::RSTN_U0_DDR_OSC);
        while (read32(init::SYS_CRG_RESET_STATUS1) >> init::RSTN_U0_DDR_OSC) & 1 == 1 {
            udelay(1);
        }
        init::clear_bit(init::SYS_CRG_RESET_ASSERT1, init::RSTN_U0_DDR_OSC);
        while (read32(init::SYS_CRG_RESET_STATUS1) >> init::RSTN_U0_DDR_OSC) & 1 == 0 {
            udelay(1);
        }
        // DDR APB
        init::set_bit(init::SYS_CRG_RESET_ASSERT1, init::RSTN_U0_DDR_APB);
        while (read32(init::SYS_CRG_RESET_STATUS1) >> init::RSTN_U0_DDR_APB) & 1 == 1 {
            udelay(1);
        }
        init::clear_bit(init::SYS_CRG_RESET_ASSERT1, init::RSTN_U0_DDR_APB);
        while (read32(init::SYS_CRG_RESET_STATUS1) >> init::RSTN_U0_DDR_APB) & 1 == 0 {
            udelay(1);
        }
        // DDR AXI
        init::set_bit(init::SYS_CRG_RESET_ASSERT1, init::RSTN_U0_DDR_AXI);
        while (read32(init::SYS_CRG_RESET_STATUS1) >> init::RSTN_U0_DDR_AXI) & 1 == 1 {
            udelay(1);
        }
        init::clear_bit(init::SYS_CRG_RESET_ASSERT1, init::RSTN_U0_DDR_AXI);
        while (read32(init::SYS_CRG_RESET_STATUS1) >> init::RSTN_U0_DDR_AXI) & 1 == 0 {
            udelay(1);
        }

        // inlined from ddr_setup()
        println!("[DRAM] train"); // dram_pi::init in VF1 code
        train(init::DDR_PHY_CTRL_BASE);
        println!("[DRAM] util"); // dram_phy::init in VF1 code
        util(init::DDR_PHY_AC_BASE);
        println!("[DRAM] start");
        start(
            init::DDR_PHY_BASE,
            init::DDR_PHY_CTRL_BASE,
            init::DDR_PHY_AC_BASE,
        );
        println!("[DRAM] set clk to OSC div2");
        init::clk_ddrc_osc_div2();
        println!("[DRAM] boot");
        omc_init(
            init::DDR_PHY_BASE,
            init::DDR_CTRL_BASE,
            init::DDR_SEC_CTRL_BASE,
            init::DDR_PHY_CTRL_BASE,
        );
        println!("[DRAM] init done");
    }
}
