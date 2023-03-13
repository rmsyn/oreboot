mod rk3399_common;

pub use rk3399_common::*;

pub const ROCKCHIP_DEVICE_SETTINGS: &str = "stdin=serial,usbkbd\0\
		                                    stdout=serial,vidconsole\0\
		                                    stderr=serial,vidconsole\0";
