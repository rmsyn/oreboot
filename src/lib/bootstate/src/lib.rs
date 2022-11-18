/* SPDX-License-Identifier: GPL-2.0-only */
#![no_std]

/// The boot state machine provides a mechanism for calls to be made through-
/// out the main boot process. The boot process is separated into discrete
/// states. Upon a state's entry and exit and callbacks can be made. For
/// example:
///
///      Enter State
///           +
///           |
///           V
///   +-----------------+
///   | Entry callbacks |
///   +-----------------+
///   | State Actions   |
///   +-----------------+
///   | Exit callbacks  |
///   +-------+---------+
///           |
///           V
///       Next State
///
/// Below is the current flow from top to bottom:
///
///        start
///          |
///    BS_PRE_DEVICE
///          |
///    BS_DEV_INIT_CHIPS
///          |
///    BS_DEV_ENUMERATE
///          |
///    BS_DEV_RESOURCES
///          |
///    BS_DEV_ENABLE
///          |
///    BS_DEV_INIT
///          |
///    BS_POST_DEVICE
///          |
///    BS_OS_RESUME_CHECK -------- BS_OS_RESUME
///          |                          |
///    BS_WRITE_TABLES              os handoff
///          |
///    BS_PAYLOAD_LOAD
///          |
///    BS_PAYLOAD_BOOT
///          |
///      payload run
///
/// Brief description of states:
///   BS_PRE_DEVICE - before any device tree actions take place
///   BS_DEV_INIT_CHIPS - init all chips in device tree
///   BS_DEV_ENUMERATE - device tree probing
///   BS_DEV_RESOURCES - device tree resource allocation and assignment
///   BS_DEV_ENABLE - device tree enabling/disabling of devices
///   BS_DEV_INIT - device tree device initialization
///   BS_POST_DEVICE - all device tree actions performed
///   BS_OS_RESUME_CHECK - check for OS resume
///   BS_OS_RESUME - resume to OS
///   BS_WRITE_TABLES - write coreboot tables
///   BS_PAYLOAD_LOAD - Load payload into memory
///   BS_PAYLOAD_BOOT - Boot to payload
pub enum BootState {
    PreDevice,
    DevInitChips,
	DevEnumerate,
	DevResources,
	DevEnable,
	DevInit,
	PostDevice,
	OSResumeCheck,
	OSResume,
	WriteTables,
	PayloadLoad,
	PayloadBoot,
}

/// The boot_state_sequence_t describes when a callback is to be made. It is
/// called either before a state is entered or when a state is exited.
pub enum BootStateSequence {
    OnEntry,
    OnExit,
}
