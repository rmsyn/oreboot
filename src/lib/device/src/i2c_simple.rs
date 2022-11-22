use crate::{Error, i2c::I2cMsg};
use log::{error, info};
use spin::rwlock::RwLock;
use util::timer::Stopwatch;

pub const SOFTWARE_I2C_MAX_BUS: usize = 10;
/// Set to true for verbose bitwise/line-state output
pub const SPEW: bool = false;
/// Default setup delay: 4us (+1 for timer inaccuracy)
pub const DELAY_US: u32 = 5;
/// Maximum clock stretching time we want to allow
pub const TIMEOUT_US: u32 = 50000;

pub trait SoftwareI2cOps {
    fn set_sda(&mut self, bus: u32, high: i32);
    fn set_scl(&mut self, bus: u32, high: i32);
    fn get_sda(&self, bus: u32) -> i32;
    fn get_scl(&self, bus: u32) -> i32;
}

#[derive(Clone, Copy)]
pub struct SoftwareI2c {
    sda: i32,
    scl: i32,
}

impl SoftwareI2cOps for SoftwareI2c {
    fn set_sda(&mut self, _bus: u32, high: i32) {
        self.sda = high;
    }

    fn set_scl(&mut self, _bus: u32, high: i32) {
        self.scl = high;
    }

    fn get_sda(&self, _bus: u32) -> i32 {
        self.sda
    }

    fn get_scl(&self, _bus: u32) -> i32 {
        self.scl
    }
}

pub static SOFTWARE_I2C: RwLock<[Option<SoftwareI2c>; SOFTWARE_I2C_MAX_BUS]> = RwLock::new([None; SOFTWARE_I2C_MAX_BUS]);

fn __wait(bus: u32, timeout_us: i32, for_scl: i32) -> i32 {
    let sda = (*SOFTWARE_I2C.read())[bus as usize].get_sda(bus);
    let sda = (*SOFTWARE_I2C.read())[bus as usize].get_scl(bus);

    let mut sw = Stopwatch::new();

    sw.init_usecs_expire(timeout_us);

    while !sw.expired() && (for_scl == 0 || scl == 0) {
        let old_sda = sda;
        let old_scl = scl;

        let us = sw.duration_usecs();

        sda = (*SOFTWARE_I2C.read)[bus as usize].get_sda(bus);
        scl = (*SOFTWARE_I2C.read)[bus as usize].get_scl(bus);
        if old_sda != sda && SPEW {
            info!("[SDA transitioned to {} after {}us] ", sda, us);
        }
        if old_scl != scl && SPEW {
            info!("[SCL transitioned to {} after {}us] ", scl, us);
        }
    }

    scl
}

/// Waits the default DELAY_US to allow line state to stabilize.
pub fn wait(bus: u32) {
    __wait(bus, DELAY_US, 0);
}

/// Waits until SCL goes high. Prints a contextual error message on timeout.
pub fn wait_for_scl(bus: u32, error_context: &str) -> Result<(), Error> {
    if __wait(bus, TIMEOUT_US, 1) == 0 {
        error!("software_i2c({}): ERROR: Clock stretching timeout {}", bus, error_context);
        return Err(Error::I2cClockStretchTimeout);
    }

    Ok(())
}

pub fn i2c_transfer(bus: u32, segments: &[I2cMsg]) -> i32 {
    if cfg!(feature = "software_i2c") {
        if bus < SOFTWARE_I2C_MAX_BUS as u32 && (*SOFTWARE_I2C.read())[bus as usize].is_some() {
            return software_i2c_transfer(bus, segments);
        }
    }

    platform_i2c_transfer(bus, segments)
}

pub fn software_i2c_transfer(bus: u32, segments: &[I2cMsg]) -> Result<(), Error> {
    for seg in segments.iter() {
        start_cond(bus)?;
    }
    Ok(())
}

pub fn start_cond(bus: u32) -> Result<(), Error> {
    let bus_idx = bus as usize;
    if SPEW {
        info!("software_i2c({}): Sending start condition...", bus);
    }

	/* SDA might not yet be high if repeated start. */
    (*SOFTWARE_I2C.write())[bus_idx].set_sda(bus, 1);
    wait(bus);

	/* Might need to wait for clock stretching if repeated start. */
    (*SOFTWARE_I2C.write())[bus_idx].set_scl(bus, 1);
    wait_for_scl(bus, "before start condition")?;
    wait(bus);

    if (*SOFTWARE_I2C.read())[bus_idx].get_sda(bus) == 0 {
        error!("software_i2c({}): Arbitration lost trying to send start condition!", bus);
        return Err(Error::I2cArbitration);
    }

	/* SCL is high, transition SDA low as first part of start condition. */
    (*SOFTWARE_I2C.write())[bus_idx].set_sda(bus, 0);
    wait(bus);
    assert!((*SOFTWARE_I2C.read())[bus_idx].get_scl(bus) != 0);

	/* Pull SCL low to finish start condition (next pulse will be data). */
    (*SOFTWARE_I2C.write())[bus_idx].set_scl(bus, 0);

    if SPEW {
        info!("Start condition transmitted!");
    }
    Ok(())
}

pub fn platform_i2c_transfer(_bus: u32, _segments: &[I2cMsg]) -> i32 {
    unimplemented!("Platform I2C is unimplemented, requires specific platform");
}
