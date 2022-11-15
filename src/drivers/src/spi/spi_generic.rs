use crate::context::Context;
use crate::spi::spi_flash::SPIFlash;
use util::region::Region;

/* Controller-specific definitions: */

pub struct SPICtrlrBuses {
    pub ctrlr: Option<SPICtrlr>,
    pub bus_start: u32,
    pub bus_end: u32,
}

/// Representation of SPI operation status.
#[derive(Debug)]
pub enum SPIOpStatus {
    NotExecuted,
    Success,
    Failure,
}

#[derive(Debug)]
pub enum CtrlrProtType {
    ReadProtect,
    WriteProtect,
    ReadWriteProtect,
}

/**
 * Representation of a SPI operation.
 *
 * dout:	Pointer to data to send.
 * din:	Pointer to store received data.
 */
pub struct SPIOp<'a, 'b> {
    pub dout: &'a [u8],
    pub din: &'b mut [u8],
    pub status: SPIOpStatus,
}

/**----------------------------------------------------------------------
 * Representation of a SPI controller. Note the xfer() and xfer_vector()
 * callbacks are meant to process full duplex transactions. If the
 * controller cannot handle these transactions then return an error when
 * din and dout are both set. See spi_xfer() below for more details.
 *
 * claim_bus:		Claim SPI bus and prepare for communication.
 * release_bus:	Release SPI bus.
 * setup:		Setup given SPI device bus.
 * xfer:		Perform one SPI transfer operation.
 * xfer_vector:	Vector of SPI transfer operations.
 * xfer_dual:		(optional) Perform one SPI transfer in Dual SPI mode.
 * max_xfer_size:	Maximum transfer size supported by the controller
 *			(0 = invalid,
 *			 SPI_CTRLR_DEFAULT_MAX_XFER_SIZE = unlimited)
 * flags:		See SPI_CNTRLR_* enums above.
 *
 * Following member is provided by specialized SPI controllers that are
 * actually SPI flash controllers.
 *
 * flash_probe:	Specialized probe function provided by SPI flash
 *			controllers.
 * flash_protect: Protect a region of flash using the SPI flash controller.
 */
#[derive(Clone, Copy)]
pub struct SPICtrlr {
    pub claim_bus: Option<fn(&SPISlave) -> Result<(), Error>>,
    pub release_bus: Option<fn(&SPISlave)>,
    pub setup: Option<fn(&SPISlave) -> Result<(), Error>>,
    pub xfer: Option<fn(&SPISlave, &[u8], &mut [u8]) -> Result<(), Error>>,
    pub xfer_vector: Option<fn(&SPISlave, &[SPIOp]) -> Result<(), Error>>,
    pub xfer_dual: Option<fn(&SPISlave, &[u8], &mut [u8])>,
    pub max_xfer_size: u32,
    pub flags: u32,
    pub flash_probe: Option<fn(&SPISlave, &SPIFlash)>,
    pub flash_protect: Option<fn(&SPIFlash, &Region, CtrlrProtType)>,
}

#[derive(Debug)]
pub enum Error {
    MissingSPIBus,
    MissingSPICtrlr,
    MissingSPIXfer,
    MissingSPIReleaseBus,
}

/**----------------------------------------------------------------------
 * Representation of a SPI slave, i.e. what we're communicating with.
 *
 *   bus:	ID of the bus that the slave is attached to.
 *   cs:	ID of the chip select connected to the slave.
 *   ctrlr:	Pointer to SPI controller structure.
 */
#[derive(Clone, Copy)]
pub struct SPISlave {
    bus: u32,
    cs: u32,
    ctrlr: Option<SPICtrlr>,
}

impl SPISlave {
    pub const fn new() -> Self {
        Self {
            bus: 0,
            cs: 0,
            ctrlr: None,
        }
    }

    pub fn clear(&mut self) {
        self.bus = 0;
        self.cs = 0;
        self.ctrlr = None;
    }

    pub fn setup(&mut self, bus: u32, cs: u32, ctrlr_map: &[SPICtrlrBuses]) -> Result<(), Error> {
        self.clear();

        for ctrlr in ctrlr_map.iter() {
            if ctrlr.bus_start <= bus && ctrlr.bus_end >= bus {
                self.ctrlr = ctrlr.ctrlr.clone();
                break;
            }
        }

        if let Some(ctrlr) = &self.ctrlr {
            self.bus = bus;
            self.cs = cs;

            if let Some(setup) = ctrlr.setup {
                setup(self)?;
            }

            Ok(())
        } else {
            println!("Can't find SPI bus {}", bus);
            Err(Error::MissingSPIBus)
        }
    }

    pub fn claim_bus(&self) -> Result<(), Error> {
        if let Some(ctrlr) = self.ctrlr {
            if let Some(claim_bus) = ctrlr.claim_bus {
                claim_bus(&self)?;
            }
        }
        Ok(())
    }

    pub fn xfer(&self, req_buf: &[u8], res_buf: &mut [u8]) -> Result<(), Error> {
        if let Some(ctrlr) = self.ctrlr {
            if let Some(xfer) = ctrlr.xfer {
                xfer(&self, req_buf, res_buf)
            } else {
                Err(Error::MissingSPIXfer)
            }
        } else {
            Err(Error::MissingSPICtrlr)
        }
    }

    pub fn release_bus(&self) -> Result<(), Error> {
        if let Some(ctrlr) = self.ctrlr {
            if let Some(release_bus) = ctrlr.release_bus {
                release_bus(&self);
                Ok(())
            } else {
                Err(Error::MissingSPIReleaseBus)
            }
        } else {
            Err(Error::MissingSPICtrlr)
        }
    }
}

impl Context for SPISlave {}
