/// struct i2c_msg - an I2C transaction segment beginning with START
/// @addr: Slave address, either seven or ten bits.  When this is a ten
///	bit address, I2C_M_TEN must be set in @flags.
/// @flags: I2C_M_RD is handled by all adapters.
/// @len: Number of data bytes in @buf being read from or written to the
///	I2C slave address.  For read transactions where I2C_M_RECV_LEN
///	is set, the caller guarantees that this buffer can hold up to
///	32 bytes in addition to the initial length byte sent by the
///	slave (plus, if used, the SMBus PEC).
/// @buf: The buffer into which data is read, or from which it's written.
///
/// An i2c_msg is the low level representation of one segment of an I2C
/// transaction.  It is visible to drivers in the @i2c_transfer() procedure.
///
/// All I2C adapters implement the standard rules for I2C transactions. Each
/// transaction begins with a START.  That is followed by the slave address,
/// and a bit encoding read versus write.  Then follow all the data bytes,
/// possibly including a byte with SMBus PEC.  The transfer terminates with
/// a NAK, or when all those bytes have been transferred and ACKed.  If this
/// is the last message in a group, it is followed by a STOP.  Otherwise it
/// is followed by the next @i2c_msg transaction segment, beginning with a
/// (repeated) START.
#[repr(C)]
pub struct I2cMsg {
    pub flags: u16,
    pub slave: u16,
    pub len: u16,
    pub buf: [u8; 2],
}

impl I2cMsg {
    /// read data, from slave to master
    pub const I2C_M_RD: u16 = 0x0001;
    /// this is a ten bit chip address
    pub const I2C_M_TEN: u16 = 0x0010;
    /// length will be first received byte
    pub const I2C_M_RECV_LEN: u16 = 0x0400;
    /// don't send a repeated START
    pub const I2C_M_NOSTART: u16 = 0x4000;
}

#[repr(C)]
pub enum I2cSpeed {
	I2cSpeedStandard	= 100000,
	I2cSpeedFast		= 400000,
	I2cSpeedFastPlus	= 1000000,
	I2cSpeedHigh		= 3400000,
	I2cSpeedFastUltra	= 5000000,
}

#[repr(C)]
pub enum I2cAddressMod {
    I2cMode7Bit,
    I2cMode10Bit,
}
