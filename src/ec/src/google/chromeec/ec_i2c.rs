use crate::google::chromeec::{
    crosec_proto::crosec_command_proto,
    ec::{ChromeECCommand, Error},
    ec_commands::{HostEventCode, EC_COMMAND_PROTOCOL_3},
};
use device::i2c::I2cMsg;
use drivers::context::Context;
use log::error;
use spin::rwlock::RwLock;

pub const PROTO3_FRAMING_BYTES: usize = 4;
pub const PROTO3_MAX_PACKET_SIZE: usize = 268;

pub const REQ_FRAME_INDEX: usize = 3;
pub const RESP_FRAME_INDEX: usize = 2;

pub const EC_GOOGLE_CHROMEEC_I2C_BUS: u16 = 0x00; // FIXME: default, needs proper kconfig
pub const EC_GOOGLE_CHROMEEC_I2C_CHIP: u16 = 0x1e; // FIXME: default, needs proper kconfig

#[repr(C, align(4))]
pub struct Proto3I2CBuf {
    pub framing_bytes: [u8; PROTO3_FRAMING_BYTES],
    pub data: [u8; PROTO3_MAX_PACKET_SIZE],
}

impl Proto3I2CBuf {
    pub const fn new() -> Self {
        Self {
            framing_bytes: [0u8; PROTO3_FRAMING_BYTES],
            data: [0u8; PROTO3_MAX_PACKET_SIZE],
        }
    }
}

pub static REQ_BUF: RwLock<Proto3I2CBuf> = RwLock::new(Proto3I2CBuf::new());
pub static RESP_BUF: RwLock<Proto3I2CBuf> = RwLock::new(Proto3I2CBuf::new());

#[repr(C)]
pub enum I2cSizes {
    CmdIndex,
    RespIndex,
    SegsPerCmd,
}

pub struct I2cEc {
    pub bus: i32,
    pub segs: [I2cMsg; I2cSizes::SegsPerCmd as usize],
}

impl Context for I2cEc {}

pub static EC_DEV: RwLock<I2cEc> = RwLock::new(
    I2cEc {
        bus: EC_GOOGLE_CHROMEEC_I2C_BUS as i32,
        segs: [
            I2cMsg {
                flags: 0,
                slave: EC_GOOGLE_CHROMEEC_I2C_CHIP,
                len: (PROTO3_FRAMING_BYTES - 3) as u16,
		        /* Framing byte to be transferred prior to request. */
                // FIXME: should be a pointer into REQ_BUF,
                // but mutable pointers can't be passed safely across threads, TBD
                buf: [0u8; 2],
            },
            I2cMsg {
                flags: I2cMsg::I2C_M_RD,
                slave: EC_GOOGLE_CHROMEEC_I2C_CHIP,
                len: (PROTO3_FRAMING_BYTES - 2) as u16,
                // FIXME: should be a pointer into RESP_BUF,
                // but mutable pointers can't be passed safely across threads, TBD
		        /* return code and total length before full response. */
                buf: [0u8; 2],
            },
        ]
    }
);

pub fn crosec_i2c_io(req_size: usize, resp_size: usize, context: &mut dyn Context) -> Result<(), Error> {
    if req_size > PROTO3_MAX_PACKET_SIZE || resp_size > PROTO3_MAX_PACKET_SIZE {
        return Err(Error::InvalidPacketSize);
    }
    if let Some(ec) = context.as_any_mut().downcast_mut::<I2cEc>() {
	    /* Place the framing byte and set size accordingly. */
        ec.segs[I2cSizes::CmdIndex as usize].len = (req_size + 1) as u16;
        ec.segs[I2cSizes::CmdIndex as usize].buf[0] = EC_COMMAND_PROTOCOL_3;
	    /* Return code and length returned prior to packet data. */
        ec.segs[I2cSizes::RespIndex as usize].len = (resp_size + 2) as u16;

        if i2c_transfer(ec.bus, &ec.segs).is_err() {
            error!("{}: Cannot complete read from i2c-{}:{:x}", "crosec_i2c_io", ec.bus, ec.segs[0].slave);
            return Err(Error::FailedI2cTransfer);
        }

        let ret_code = ec.segs[I2cSizes::RespIndex as usize].buf[0];
        let resp_len = ec.segs[I2cSizes::RespIndex as usize].buf[1];

        if ret_code != 0 {
            error!("EC command returned 0x{:x}", ret_code);
            return Err(Error::FailedI2cCommand(ret_code));
        }

        if resp_len > resp_size {
            error!("Response length mismatch {} vs {}", resp_len, resp_size);
            return Err(Error::I2cResponseLengthMismatch);
        }

        Ok(())
    }
}

pub fn google_chromeec_command(cec_command: ChromeECCommand) -> Result<(), Error> {
    crosec_command_proto(cec_command, crosec_i2c_io, &mut (*EC_DEV.write()))?;
    Ok(())
}

pub fn google_chromeec_get_event() -> HostEventCode {
    error!("{}: Not supported.", "google_chromeec_get_event");
    HostEventCode::None
}
