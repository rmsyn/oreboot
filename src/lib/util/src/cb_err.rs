/**
 * coreboot error codes
 *
 * Common error definitions that can be used for any function. All error values
 * should be negative -- when useful, positive values can also be used to denote
 * success. Allocate a new group or errors every 100 values.
 */
#[derive(Clone, Copy)]
pub enum CbErr {
    /// Call completed successfully
    Success = 0,
    /// Generic error code
    Err = -1,
    /// Invalid argument
    ErrArg = -2,
    /// Function not implemented
    ErrNotImplemented = -3,

	/* NVRAM/CMOS errors */
    /// Option table disabled
    CMOSOtableDisabled = -100,
    /// Layout file not found
    CMOSLayoutNotFound = -101,
    /// Option string not found 
    CMOSOptionNotFound = -102,
    /// CMOS access error
    CMOSAccessError = -103,
    /// CMOS checksum is invalid
    CMOSChecksumInvalid = -104,

	/* Keyboard test failures */
    KbdControllerFailure = -200,
    KbdInterfaceFailure = -201,

	/* I2C controller failures */
    /// Device is not responding
    I2CNoDevice = -300,
    /// Device tells it's busy
    I2CBusy = -301,
    /// Data lost or spurious slave device response, try again?
    I2CProtocolError = -302,
    /// Transmission timed out
    I2CTimeout = -303,

    /* CBFS errors */
    /// Underlying I/O error
    CBFSIO = -400,
    /// File not found in directory
    CBFSNotFound = -401,
    /// Master hash validation failed
    CBFSHashMismatch = -402,
    /// Metadata cache overflowed
    CBFSCacheFull = -403,
}
