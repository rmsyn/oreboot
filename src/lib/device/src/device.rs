use alloc::collections::LinkedList;
use core::ptr::NonNull;
use crate::{Class, ClassId, ErrorKind, ErrorType};

/// Device manager flags
#[repr(u32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DmFlag {
    /// Driver is active (probed). Cleared when it is removed
    Activated = 1 << 0,
    /// DM is responsible for allocating and freeing plat
    AllocPdata = 1 << 1,
    /// DM should init this device prior to relocation
    PreReloc = 1 << 2,
    /// DM is responsible for allocating and freeing parent_plat
    AllocParentPdata = 1 << 3,
    /// DM is responsible for allocating and freeing class_plat
    AllocClassPdata = 1 << 4,
    /// Allocate driver private data on a DMA boundary
    AllocPrivDma = 1 << 5,
    /// Device is bound
    Bound = 1 << 6,
    /// Device name is allocated and should be freed on unbind()
    NameAlloced = 1 << 7,
    /// Device has platform data provided by of-platdata
    OfPlatdata = 1 << 8,
    /// Call driver remove function to stop currently active DMA transfers or
    /// give DMA buffers back to the HW / controller. This may be needed for
    /// some drivers to do some final stage cleanup before the OS is called
    /// (oreboot exit)
    ActiveDma = 1 << 9,
    /// Call driver remove function to do some final configuration, before
    /// oreboot exits and the OS is started
    OsPrepare = 1 << 10,
    /// DM does not enable/disable the power domains corresponding to this device
    DefaultPdCtrlOff = 1 << 11,
    /// Driver plat has been read. Cleared when the device is removed
    PlatdataValid = 1 << 12,
    /// Device is removed without switching off its power domain. This might
    /// be required, i. e. for serial console (debug) output when booting OS.
    LeavePdOn = 1 << 13,
    /// Device is vital to the operation of other devices. It is possible to remove
    /// removed this device after all regular devices are removed. This is useful
    /// e.g. for clock, which need to be active during the device-removal phase.
    Vital = 1 << 14,
    /// Device must be probed after it was bound
    ProbeAfterBind = 1 << 15,
}

/// One or multiple of these flags are passed to [`Device::remove`] so that
/// a selective device removal as specified by the remove-stage and the
/// driver flags can be done.
///
/// DO NOT use these flags in your driver's @flags value...
///	use the above [`DmFlag`] values instead
#[repr(u32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DmRemove {
	/// Normal remove, remove all devices
    Normal = 1 << 0,
	/// Remove devices with active DMA
    ActiveDma = DmFlag::ActiveDma as u32,
	/// Remove devices which need some final OS preparation steps
    OsPrepare = DmFlag::OsPrepare as u32,
	/// Remove only devices that are not marked vital
    NonVital = DmFlag::Vital as u32,
	/// Remove devices with any active flag
    ActiveAll = DmFlag::ActiveDma as u32 | Self::OsPrepare as u32,
	/// Don't power down any attached power domains
    NoPd = 1 << 1,
}

/// An instance of a driver
///
/// This holds information about a device, which is a driver bound to a
/// particular port or peripheral (essentially a driver instance).
///
/// A device will come into existence through a 'bind' call, either due to
/// a `oreboot_drvinfo!()` macro (in which case plat is non-NULL) or a node
/// in the device tree (in which case of_offset is >= 0). In the latter case
/// we translate the device tree information into plat in a function
/// implemented by the driver of_to_plat method (called just before the
/// probe method if the device has a device tree node.
///
/// All three of plat, priv and class_priv can be allocated by the
/// driver, or you can use the auto members of struct driver and
/// struct uclass_driver to have driver model do this automatically.
#[repr(C)]
pub struct Device {
    /// The driver used by this device
    driver: Option<&'static Driver>,
    /// Name of device, typically the FDT node name
    name: &'static str,
    /// Configuration data for this device (do not access outside driver model)
    plat_: Option<NonNull<libc::c_void>>,
    /// The parent bus's configuration data for this device (do not access outside driver model)
    parent_plat_: Option<NonNull<libc::c_void>>,
    /// The uclass's configuration data for this device (do not access driver model)
    class_plat_: Option<NonNull<libc::c_void>>,
    /// Driver data word for the entry that matched this device with its driver
    driver_data: u32,
    /// Parent of this device, or NULL for the top level device
    parent: Option<NonNull<Device>>,
    /// Private data for this device (do not access outside driver model)
    priv_: Option<NonNull<libc::c_void>>,
    /// Pointer to uclass for this device
    class: Option<NonNull<Class>>,
    /// The uclass's private data for this device (do not access outside driver model)
    class_priv_: Option<NonNull<libc::c_void>>,
    /// The parent's private data for this device (do not access outside driver model)
    parent_priv_: Option<NonNull<libc::c_void>>,
    /// Used by Class to link its devices
    class_node: LinkedList<Class>,
    /// List of children of this device
    child_head: LinkedList<Device>,
    /// Next device in list of all devices
    sibling_node: LinkedList<Class>,
    /// Allocated sequence number for this device (-1 = none). This is set up
    seq: i32,
    /// Flags for this device [`DmFlag`] (do not access outside driver model)
    /// when the device is bound and is unique within the device's uclass. If the
    /// device has an alias in the devicetree then that is used to set the sequence
    /// number. Otherwise, the next available number is used. Sequence numbers are
    /// used by certain commands that need device to be numbered (e.g. 'mmc dev').
    /// (do not access outside driver model)
    flags: DmFlag,
    /// List of memory allocations associated with this device.
    /// When CONFIG_DEVRES is enabled, devm_kmalloc() and friends will
    /// add to this list. Memory so-allocated will be freed
    /// automatically when the device is removed / unbound
    devres_head: LinkedList<Device>,
    /// Offset between the physical address space (CPU's) and the
    /// device's bus address space
    dma_offset: u32,
    /// IOMMU device associated with this device
    iommu: Option<NonNull<Device>>,
}

impl Device {
    pub const fn new() -> Self {
        Self {
            driver: None,
            name: "",
            plat_: None,
            parent_plat_: None,
            class_plat_: None,
            driver_data: 0,
            parent: None,
            priv_: None,
            class: None,
            class_priv_: None,
            parent_priv_: None,
            class_node: LinkedList::new(),
            child_head: LinkedList::new(),
            sibling_node: LinkedList::new(),
            seq: 0,
            flags: DmFlag::Activated,
            devres_head: LinkedList::new(),
            dma_offset: 0,
            iommu: None,
        }
    }

    pub fn driver(&self) -> Option<&Driver> {
        self.driver
    }

    pub fn name(&self) -> &str {
        self.name
    }

    pub fn driver_data(&self) -> u32 {
        self.driver_data
    }

    pub fn parent(&self) -> Option<&Device> {
        if let Some(d) = self.parent {
            Some(unsafe{ d.as_ref() })
        } else {
            None
        }
    }

    pub fn class(&self) -> Option<&Class> {
        if let Some(c) = self.class {
            Some(unsafe{ c.as_ref() })
        } else {
            None
        }
    }

    pub fn class_node(&self) -> &LinkedList<Class> {
        &self.class_node
    }

    pub fn child_head(&self) -> &LinkedList<Device> {
        &self.child_head
    }

    pub fn sibling_node(&self) -> &LinkedList<Class> {
        &self.sibling_node
    }

    pub fn seq(&self) -> i32 {
        self.seq
    }

    pub fn flags(&self) -> DmFlag {
        self.flags
    }

    pub fn devres_head(&self) -> &LinkedList<Device> {
        &self.devres_head
    }

    pub fn dma_offset(&self) -> u32 {
        self.dma_offset
    }

    pub fn iommu(&self) -> Option<&Device> {
        if let Some(i) = self.iommu {
            Some(unsafe{ i.as_ref() })
        } else {
            None
        }
    }
}

/// A driver for a feature or peripheral
///
/// This holds methods for setting up a new device, and also removing it.
/// The device needs information to set itself up - this is provided either
/// by plat or a device tree node (which we find by looking up
/// matching compatible strings with of_match).
///
/// Drivers all belong to a uclass, representing a class of devices of the
/// same type. Common elements of the drivers can be implemented in the uclass,
/// or the uclass can provide a consistent interface to the drivers within
/// it.
#[repr(C)]
pub struct Driver {
    /// Device name
    name: &'static str,
    /// Identifies the uclass we belong to
    id: ClassId,
    /// List of compatible strings to match, and any identifying data
    /// for each.
    of_match: DeviceId,
    /// If non-zero this is the size of the private data
    /// to be allocated in the device's ->priv pointer. If zero, then the driver
    /// is responsible for allocating any data required.
    priv_auto: i32,
    /// If non-zero this is the size of the
    /// platform data to be allocated in the device's ->plat pointer.
    /// This is typically only useful for device-tree-aware drivers (those with
    /// an of_match), since drivers which use plat will have the data
    /// provided in the U_BOOT_DRVINFO() instantiation.
    plat_auto: i32,
    /// Each device can hold private data owned by
    /// its parent. If required this will be automatically allocated if this
    /// value is non-zero.
    per_child_auto: i32,
    /// A bus likes to store information about
    /// its children. If non-zero this is the size of this data, to be allocated
    /// in the child's parent_plat pointer.
    per_child_plat_auto: i32,
    /// driver flags - see `DM_FLAGS_...`
    flags: u32,
}

impl Driver {
    pub const fn new() -> Self {
        Self {
            name: "",
            id: ClassId::Root,
            of_match: DeviceId::new(),
            priv_auto: 0,
            plat_auto: 0,
            per_child_auto: 0,
            per_child_plat_auto: 0,
            flags: 0,
        }
    }

    pub fn name(&self) -> &str {
        self.name
    }

    pub fn id(&self) -> ClassId {
        self.id
    }

    pub fn of_match(&self) -> &DeviceId {
        &self.of_match
    }

    pub fn priv_auto(&self) -> i32 {
        self.priv_auto
    }

    pub fn plat_auto(&self) -> i32 {
        self.plat_auto
    }

    pub fn per_child_auto(&self) -> i32 {
        self.per_child_auto
    }

    pub fn per_child_plat_auto(&self) -> i32 {
        self.per_child_plat_auto
    }

    pub fn flags(&self) -> u32 {
        self.flags
    }
}

impl ErrorType for Driver {
    type Error = ErrorKind;
}

pub trait DriverOps: ErrorType {
    /// Called to bind a device to its driver
    fn bind(&mut self) -> Result<(), Self::Error>;
    /// Called to probe a device, i.e. activate it
    fn probe(&mut self) -> Result<(), Self::Error>;
    /// Called to remove a device, i.e. de-activate it
    fn remove(&mut self) -> Result<(), Self::Error>;
    /// Called to unbind a device from its driver
    fn unbind(&mut self) -> Result<(), Self::Error>;
    /// Called before probe to decode device tree data
    fn of_to_plat(&mut self) -> Result<(), Self::Error>;
    /// Called after a new child has been bound
    fn child_post_bind(&mut self) -> Result<(), Self::Error>;
    /// Called before a child device is probed. The device has
    /// memory allocated but it has not yet been probed.
    fn child_pre_probe(&mut self) -> Result<(), Self::Error>;
    /// Called after a child device is removed. The device
    /// has memory allocated but its device_remove() method has been called.
    fn child_post_remove(&mut self) -> Result<(), Self::Error>;
}

/// Lists the compatible strings supported by a driver
#[repr(C)]
pub struct DeviceId {
    /// Compatible string
    compatible: &'static str,
    /// Data for this compatible string
    data: u32,
}

impl DeviceId {
    pub const fn new() -> Self {
        Self {
            compatible: "",
            data: 0,
        }
    }

    pub fn compatible(&self) -> &str {
        self.compatible
    }

    pub fn data(&self) -> u32 {
        self.data
    }
}
