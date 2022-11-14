use bitfield::bitfield;
use util::region::Region;
use crate::spi::spi_generic::{SPISlave, Error};

bitfield! {
    pub struct SPIFlashPartID(u16);
    pub nr_sectors_shift, set_nr_sectors_shift: 4, 0;
    pub fast_read_dual_output_support, set_fast_read_dual_output_support: 1, 4;
    pub fast_read_dual_io_support, set_fast_read_dual_io_support: 1, 5;
    reserved_for_flags, _: 2, 6;
    pub protection_granularity_shift, set_protection_granularity_shift: 5, 8;
    pub bp_bits, set_bp_bits: 3, 13;
}

bitfield! {
    pub struct SPIFlashFlags(u8);
    pub dual_output, set_dual_output: 1, 0;
    pub dual_io, set_dual_io: 1, 1;
    reserved, _: 6, 2;
}

impl Clone for SPIFlashFlags {
    fn clone(&self) -> Self {
        let mut f = Self(0); {
            f.set_dual_output(self.dual_output());
            f.set_dual_io(self.dual_io());
            f
        }
    }
}
impl Copy for SPIFlashFlags {}

pub union SPIFlashFlagsUnion {
    pub raw: u8,
    pub flags: SPIFlashFlags,
}

/**
 * SPI write protection is enforced by locking the status register.
 * The following modes are known. It depends on the flash chip if the
 * mode is actually supported.
 *
 * PRESERVE : Keep the previous status register lock-down setting (noop)
 * NONE     : Status register isn't locked
 * PIN      : Status register is locked as long as the ~WP pin is active
 * REBOOT   : Status register is locked until power failure
 * PERMANENT: Status register is permanently locked
 */
pub enum SPIFlashStatusRegLockdown {
    WriteProtectionPreserve = -1,
    WriteProtectionNone = 0,
    WriteProtectionPin = 1,
    WriteProtectionReboot = 2,
    WriteProtectionPermanent = 3,
}

/**
 * Representation of SPI flash operations:
 * read:	Flash read operation.
 * write:	Flash write operation.
 * erase:	Flash erase operation.
 * status:	Read flash status register.
 */
pub struct SPIFlashOps {
    pub read: Option<fn(&SPIFlash, u32, usize, &[u8]) -> Result<(), Error>>,
    pub write: Option<fn(&SPIFlash, u32, usize, &[u8]) -> Result<(), Error>>,
    pub erase: Option<fn(&SPIFlash, u32, usize) -> Result<(), Error>>,
    pub status: Option<fn(&SPIFlash, &u8) -> Result<(), Error>>,
}

#[repr(C, packed)]
pub struct SPIFlash {
    spi: SPISlave,
    vendor: u8,
    flags: SPIFlashFlagsUnion,
    model: u16,
    size: u32,
    sector_size: u32,
    page_size: u32,
    erase_cmd: u8,
    status_cmd: u8,
    pp_cmd: u8, /* Page program command */
    wren_cmd: u8, /* Write Enable command */
    ops: Option<SPIFlashOps>,
    /* If Some all protection callbacks exist */
    prot_ops: Option<SPIFlashProtectionOps>,
    part: Option<SPIFlashPartID>,
}

/// Current code assumes all callbacks are supplied in this object.
pub struct SPIFlashProtectionOps {
	/*
	 * Returns 1 if the whole region is software write protected.
	 * Hardware write protection mechanism aren't accounted.
	 * If the write protection could be changed, due to unlocked status
	 * register for example, 0 should be returned.
	 * Returns 0 on success.
	 */
    pub get_write: Option<fn(&SPIFlash, &Region) -> Result<(), Error>>,

	/*
	 * Enable the status register write protection, if supported on the
	 * requested region, and optionally enable status register lock-down.
	 * Returns 0 if the whole region was software write protected.
	 * Hardware write protection mechanism aren't accounted.
	 * If the status register is locked and the requested configuration
	 * doesn't match the selected one, return an error.
	 * Only a single region is supported !
	 *
	 * @return 0 on success
	 */
    pub set_write: Option<fn(&SPIFlash, &Region, SPIFlashStatusRegLockdown) -> Result<(), Error>>,
}
