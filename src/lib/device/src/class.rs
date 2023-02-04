use core::ptr::NonNull;
use alloc::collections::LinkedList;

use crate::{Device, ErrorKind, ErrorType, class_id::ClassId};

/// An oreboot drive class, collecting together similar drivers
/// 
/// A `Class` provides an interface to a particular function, which is
/// implemented by one or more drivers. Every driver belongs to a uclass even
/// if it is the only driver in that uclass. An example uclass is GPIO, which
/// provides the ability to change read inputs, set and clear outputs, etc.
/// There may be drivers for on-chip SoC GPIO banks, I2C GPIO expanders and
/// PMIC IO lines, all made available in a unified way through the uclass.
#[repr(C)]
pub struct Class {
    /// Private data for this `Class` (do not access outside driver model)
    priv_: Option<NonNull<()>>,
    /// The driver for the `Class` itself, not to be confused with a `Driver`
    driver: Option<NonNull<ClassDriver>>,
    /// List of devices in this `Class` (devices are attached to their
    /// `Class` when their bind method is called)
    dev_head: LinkedList<Device>,
    /// Next `Class` in the linked list of `Class`es
    sibling_node: LinkedList<Class>,
}

impl Class {
    pub const fn new() -> Self {
        Self {
            priv_: None,
            driver: None,
            dev_head: LinkedList::new(),
            sibling_node: LinkedList::new(),
        }
    }

    /// Get access to private data
    pub fn private(&self) -> Option<&()> {
        if let Some(p) = self.priv_ {
            unsafe { Some(p.as_ref()) }
        } else {
            None
        }
    }

    pub fn driver(&self) -> Option<&ClassDriver> {
        if let Some(d) = self.driver {
            unsafe { Some(d.as_ref()) }
        } else {
            None
        }
    }

    pub fn dev_head(&self) -> &LinkedList<Device> {
        &self.dev_head
    }

    pub fn sibling(&self) -> &LinkedList<Class> {
        &self.sibling_node
    }
}

/// Driver for the `Class`
///
/// A `ClassDriver` provides a consistent interface to a set of related drivers.
#[repr(C)]
pub struct ClassDriver {
    /// Name of `ClassDriver`
	name: &'static str,
    /// ID number of this `Class`
	id: ClassId,
    /// If non-zero this is the size of the private data
    /// to be allocated in the `Class`'s ->priv pointer. If zero, then the `Class`
    /// driver is responsible for allocating any data required.
	priv_auto: i32,
    /// Each device can hold private data owned by the `Class`.
    /// If required this will be automatically allocated if this value is non-zero.
	per_device_auto: i32,
    /// Each device can hold platform data
    /// owned by the uclass as 'dev.class_plat'. If the value is non-zero,
    /// then this will be automatically allocated.
	per_device_plat_auto: i32,
    /// Each child device (of a parent in this
    /// `Class`) can hold parent data for the `Device`/`Class`. This value is only
    /// used as a fallback if this member is 0 in the driver.
	per_child_auto: i32,
    /// A bus likes to store information about
    /// its children. If non-zero this is the size of this data, to be allocated
    /// in the child device's parent_plat pointer. This value is only used as
    /// a fallback if this member is 0 in the driver.
	per_child_plat_auto: i32,
    /// Flags for this `Class`, see `(DM_UC_...)``
	flags: u32,
}

impl ClassDriver {
    pub const fn new() -> Self {
        Self {
            name: "",
            id: ClassId::Root,
            priv_auto: 0,
            per_device_auto: 0,
            per_device_plat_auto: 0,
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

    pub fn priv_auto(&self) -> i32 {
        self.priv_auto
    }

    pub fn per_device_auto(&self) -> i32 {
        self.per_device_auto
    }

    pub fn per_device_plat_auto(&self) -> i32 {
        self.per_device_plat_auto
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

impl ErrorType for ClassDriver {
    type Error = ErrorKind;
}

pub trait ClassDriverOps: ErrorType {
    fn post_bind(&mut self) -> Result<(), Self::Error>;
    fn pre_unbind(&mut self) -> Result<(), Self::Error>;
    fn pre_probe(&mut self) -> Result<(), Self::Error>;
    fn post_probe(&mut self) -> Result<(), Self::Error>;
    fn pre_remove(&mut self) -> Result<(), Self::Error>;
    fn child_post_bind(&mut self) -> Result<(), Self::Error>;
    fn child_pre_probe(&mut self) -> Result<(), Self::Error>;
    fn child_post_probe(&mut self) -> Result<(), Self::Error>;
    fn init(&mut self) -> Result<(), Self::Error>;
    fn destroy(&mut self) -> Result<(), Self::Error>;
}
