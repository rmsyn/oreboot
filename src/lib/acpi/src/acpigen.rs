/* SPDX-License-Identifier: GPL-2.0-only */

#[cfg(any(feature = "amd", feature = "intel"))]
use crate::device::Gpio;
use crate::{
    pld::Pld, AcpiAddr, AcpiCstate, AcpiLpiState, AcpiSwPstate, AcpiTstate, CorebootAcpiIds,
    UpcType, XpssSwPstate, COREBOOT_ACPI_ID,
};
use device::{
    device_util::GlobalSearch,
    path::DEVICE_PATH_MAX,
    resource::{Resource, ResourceArg, IORESOURCE_IO, IORESOURCE_MEM, IORESOURCE_RESERVE},
    soundwire::SoundwireAddress,
    Device, Error as DeviceError,
};
use util::hexstrtobin::hexstrtobin;

use core::fmt::Write;
use heapless::{String, Vec};
use log::{debug, error};

/// If you need to change this, change acpigen_write_len_f and
/// acpigen_pop_len
const ACPIGEN_MAXLEN: usize = 0xfffff;
/// How much nesting do we support?
const ACPIGEN_LENSTACK_SIZE: usize = 10;

const ACPI_CPU_STRING: &str = "\\_SB.CP";

pub const UUID_LEN: usize = 16;
pub const CPPC_PACKAGE_NAME: &str = "GCPC";

pub const FIELD_ANYACC: usize = 0;
pub const FIELD_BYTEACC: usize = 1;
pub const FIELD_WORDACC: usize = 2;
pub const FIELD_DWORDACC: usize = 3;
pub const FIELD_QWORDACC: usize = 4;
pub const FIELD_BUFFERACC: usize = 5;
pub const FIELD_NOLOCK: usize = 0 << 4;
pub const FIELD_LOCK: usize = 1 << 4;
pub const FIELD_PRESERVE: usize = 0 << 5;
pub const FIELD_WRITEASONES: usize = 1 << 5;
pub const FIELD_WRITEASZEROS: usize = 2 << 5;

#[derive(Debug)]
pub enum Error {
    CurrentTooLong,
    HIDString,
    Device(DeviceError),
    UUIDTooShort,
    InvalidCppcVersion(u32),
    InvalidFieldOffset,
    InvalidFieldType,
    InvalidGpioPins,
}

#[repr(C)]
pub enum FieldType {
    Offset,
    NameString,
    Reserved,
    FieldTypeMax,
}

#[repr(C)]
pub struct FieldList<'a> {
    field_type: FieldType,
    name: &'a str,
    bits: u32,
}

impl<'a> FieldList<'a> {
    pub fn offset(bits: u32) -> Self {
        Self {
            field_type: FieldType::Offset,
            name: "",
            bits: bits * 8,
        }
    }

    pub fn namestr(name: &'a str, bits: u32) -> Self {
        Self {
            field_type: FieldType::NameString,
            name,
            bits,
        }
    }

    pub fn reserved(bits: u32) -> Self {
        Self {
            field_type: FieldType::Reserved,
            name: "",
            bits,
        }
    }
}

#[repr(C)]
pub enum RegionSpace {
    SystemMemory,
    SystemIo,
    PciConfig,
    EmbeddedControl,
    Smbus,
    Cmos,
    PciBarTarget,
    Ipmi,
    GpioRegion,
    GpSerialBus,
    Pcc,
    FixedHardware = 0x7F,
    RegionSpaceMax,
}

#[repr(C)]
pub struct OpRegion<'a> {
    name: &'a str,
    region_space: RegionSpace,
    region_offset: u32,
    region_len: u32,
}

impl<'a> OpRegion<'a> {
    pub fn create(
        name: &'a str,
        region_space: RegionSpace,
        region_offset: u32,
        region_len: u32,
    ) -> Self {
        Self {
            name,
            region_space,
            region_offset,
            region_len,
        }
    }
}

#[repr(C)]
pub enum PsdCoord {
    SwAll = 0xfc,
    SwAny = 0xfd,
    HwAll = 0xfe,
}

#[repr(C)]
pub enum CsdCoord {
    HwAll = 0xfe,
}

/// Version 1 has 15 fields, version 2 has 19, and version 3 has 21 */
#[repr(C)]
pub enum CppcFields {
    HighestPerf,    /* can be DWORD */
    NominalPerf,    /* can be DWORD */
    LowestNonlPerf, /* can be DWORD */
    LowestPerf,     /* can be DWORD */
    GuaranteedPerf,
    DesiredPerf,
    MinPerf,
    MaxPerf,
    PerfReduceTolerance,
    TimeWindow,
    CounterWrap, /* can be DWORD */
    RefPerfCounter,
    DeliveredPerfCounter,
    PerfLimited,
    Enable, /* can be System I/O */
    MaxFieldsVer1,
    AutoActivityWindow,
    PerfPref,
    RefPerf, /* can be DWORD */
    MaxFieldsVer2,
    NominalFreq, /* can be DWORD */
    MaxFieldsVer3,
}

#[repr(C)]
pub union CppcUnion {
    reg: AcpiAddr,
    dword: u32,
}

#[repr(C)]
#[derive(Debug, PartialEq)]
pub enum CppcType {
    Reg,
    Dword,
}

#[repr(C)]
pub struct CppcEntry {
    cppc_type: CppcType,
    cppc_union: CppcUnion,
}

#[repr(C)]
pub struct CppcConfig {
    /// Must be 1, 2, or 3
    version: u32,
    /// The generic acpi_addr_t structure is being used, though
    /// anything besides PPC or FFIXED generally requires checking
    /// if the OS has advertised support for it (via _OSC).
    entries: [CppcEntry; CppcFields::MaxFieldsVer3 as usize],
}

pub trait CallbackArg: Sync {}

pub struct DsmUuid<'a, 'b, const N: usize> {
    pub uuid: &'a str,
    pub callbacks: [Option<fn(&dyn CallbackArg)>; N],
    pub count: usize,
    pub arg: &'b dyn CallbackArg,
}

impl<'a, 'b, const N: usize> DsmUuid<'a, 'b, N> {
    pub fn create(
        uuid: &'a str,
        callbacks: [Option<fn(&dyn CallbackArg)>; N],
        count: usize,
        arg: &'b dyn CallbackArg,
    ) -> Self {
        Self {
            uuid,
            callbacks,
            count,
            arg,
        }
    }
}

/// ACPI Op/Prefix Codes
pub const ZERO_OP: u8 = 0x00;
pub const ONE_OP: u8 = 0x01;
pub const ALIAS_OP: u8 = 0x06;
pub const NAME_OP: u8 = 0x08;
pub const BYTE_PREFIX: u8 = 0x0A;
pub const WORD_PREFIX: u8 = 0x0B;
pub const DWORD_PREFIX: u8 = 0x0C;
pub const STRING_PREFIX: u8 = 0x0D;
pub const QWORD_PREFIX: u8 = 0x0E;
pub const SCOPE_OP: u8 = 0x10;
pub const BUFFER_OP: u8 = 0x11;
pub const PACKAGE_OP: u8 = 0x12;
pub const VARIABLE_PACKAGE_OP: u8 = 0x13;
pub const METHOD_OP: u8 = 0x14;
pub const EXTERNAL_OP: u8 = 0x15;
pub const DUAL_NAME_PREFIX: u8 = 0x2E;
pub const MULTI_NAME_PREFIX: u8 = 0x2F;
pub const EXT_OP_PREFIX: u8 = 0x5B;
pub const MUTEX_OP: u8 = 0x01;
pub const EVENT_OP: u8 = 0x01;
pub const SF_RIGHT_OP: u8 = 0x10;
pub const SF_LEFT_OP: u8 = 0x11;
pub const COND_REFOF_OP: u8 = 0x12;
pub const CREATEFIELD_OP: u8 = 0x13;
pub const LOAD_TABLE_OP: u8 = 0x1f;
pub const LOAD_OP: u8 = 0x20;
pub const STALL_OP: u8 = 0x21;
pub const SLEEP_OP: u8 = 0x22;
pub const ACQUIRE_OP: u8 = 0x23;
pub const SIGNAL_OP: u8 = 0x24;
pub const WAIT_OP: u8 = 0x25;
pub const RST_OP: u8 = 0x26;
pub const RELEASE_OP: u8 = 0x27;
pub const FROM_BCD_OP: u8 = 0x28;
pub const TO_BCD_OP: u8 = 0x29;
pub const UNLOAD_OP: u8 = 0x2A;
pub const REVISON_OP: u8 = 0x30;
pub const DEBUG_OP: u8 = 0x31;
pub const FATAL_OP: u8 = 0x32;
pub const TIMER_OP: u8 = 0x33;
pub const OPREGION_OP: u8 = 0x80;
pub const FIELD_OP: u8 = 0x81;
pub const DEVICE_OP: u8 = 0x82;
pub const PROCESSOR_OP: u8 = 0x83;
pub const POWER_RES_OP: u8 = 0x84;
pub const THERMAL_ZONE_OP: u8 = 0x85;
pub const INDEX_FIELD_OP: u8 = 0x86;
pub const BANK_FIELD_OP: u8 = 0x87;
pub const DATA_REGION_OP: u8 = 0x88;
pub const ROOT_PREFIX: u8 = 0x5C;
pub const PARENT_PREFIX: u8 = 0x5E;
pub const LOCAL0_OP: u8 = 0x60;
pub const LOCAL1_OP: u8 = 0x61;
pub const LOCAL2_OP: u8 = 0x62;
pub const LOCAL3_OP: u8 = 0x63;
pub const LOCAL4_OP: u8 = 0x64;
pub const LOCAL5_OP: u8 = 0x65;
pub const LOCAL6_OP: u8 = 0x66;
pub const LOCAL7_OP: u8 = 0x67;
pub const ARG0_OP: u8 = 0x68;
pub const ARG1_OP: u8 = 0x69;
pub const ARG2_OP: u8 = 0x6A;
pub const ARG3_OP: u8 = 0x6B;
pub const ARG4_OP: u8 = 0x6C;
pub const ARG5_OP: u8 = 0x6D;
pub const ARG6_OP: u8 = 0x6E;
pub const STORE_OP: u8 = 0x70;
pub const REF_OF_OP: u8 = 0x71;
pub const ADD_OP: u8 = 0x72;
pub const CONCATENATE_OP: u8 = 0x73;
pub const SUBTRACT_OP: u8 = 0x74;
pub const INCREMENT_OP: u8 = 0x75;
pub const DECREMENT_OP: u8 = 0x76;
pub const MULTIPLY_OP: u8 = 0x77;
pub const DIVIDE_OP: u8 = 0x78;
pub const SHIFT_LEFT_OP: u8 = 0x79;
pub const SHIFT_RIGHT_OP: u8 = 0x7A;
pub const AND_OP: u8 = 0x7B;
pub const NAND_OP: u8 = 0x7C;
pub const OR_OP: u8 = 0x7D;
pub const NOR_OP: u8 = 0x7E;
pub const XOR_OP: u8 = 0x7F;
pub const NOT_OP: u8 = 0x80;
pub const FD_SHIFT_LEFT_BIT_OR: u8 = 0x81;
pub const FD_SHIFT_RIGHT_BIT_OR: u8 = 0x82;
pub const DEREF_OP: u8 = 0x83;
pub const CONCATENATE_TEMP_OP: u8 = 0x84;
pub const MOD_OP: u8 = 0x85;
pub const NOTIFY_OP: u8 = 0x86;
pub const SIZEOF_OP: u8 = 0x87;
pub const INDEX_OP: u8 = 0x88;
pub const MATCH_OP: u8 = 0x89;
pub const CREATE_DWORD_OP: u8 = 0x8A;
pub const CREATE_WORD_OP: u8 = 0x8B;
pub const CREATE_BYTE_OP: u8 = 0x8C;
pub const CREATE_BIT_OP: u8 = 0x8D;
pub const OBJ_TYPE_OP: u8 = 0x8E;
pub const CREATE_QWORD_OP: u8 = 0x8F;
pub const LAND_OP: u8 = 0x90;
pub const LOR_OP: u8 = 0x91;
pub const LNOT_OP: u8 = 0x92;
pub const LEQUAL_OP: u8 = 0x93;
pub const LGREATER_OP: u8 = 0x94;
pub const LLESS_OP: u8 = 0x95;
pub const TO_BUFFER_OP: u8 = 0x96;
pub const TO_DEC_STRING_OP: u8 = 0x97;
pub const TO_HEX_STRING_OP: u8 = 0x98;
pub const TO_INTEGER_OP: u8 = 0x99;
pub const TO_STRING_OP: u8 = 0x9C;
pub const CP_OBJ_OP: u8 = 0x9D;
pub const MID_OP: u8 = 0x9E;
pub const CONTINUE_OP: u8 = 0x9F;
pub const IF_OP: u8 = 0xA0;
pub const ELSE_OP: u8 = 0xA1;
pub const WHILE_OP: u8 = 0xA2;
pub const NOOP_OP: u8 = 0xA3;
pub const RETURN_OP: u8 = 0xA4;
pub const BREAK_OP: u8 = 0xA5;
pub const COMMENT_OP: u8 = 0xA9;
pub const BREAKPIONT_OP: u8 = 0xCC;
pub const ONES_OP: u8 = 0xFF;

pub struct AcpiGen {
    gencurrent: String<ACPIGEN_MAXLEN>,
    len_stack: Vec<String<ACPIGEN_MAXLEN>, ACPIGEN_LENSTACK_SIZE>,
    ltop: usize,
}

impl ResourceArg for AcpiGen {}

impl AcpiGen {
    pub const fn new() -> Self {
        Self {
            gencurrent: String::new(),
            len_stack: Vec::new(),
            ltop: 0,
        }
    }

    pub fn write_len_f(&mut self) -> Result<(), Error> {
        assert!(self.ltop < ACPIGEN_LENSTACK_SIZE - 1);
        self.len_stack[self.ltop] = self.gencurrent.clone();
        self.gencurrent.clear();
        self.ltop += 1;
        self.emit_byte(0)?;
        self.emit_byte(0)?;
        self.emit_byte(0)
    }

    pub fn pop_len(&mut self) {
        assert!(self.ltop > 0);
        self.ltop -= 1;
        // SAFETY: all ACPI strings should be valid UTF-8
        let p = unsafe { self.len_stack[self.ltop].as_mut_vec() };
        let len = self.gencurrent.len() - p.len();
        assert!(len <= ACPIGEN_MAXLEN);
        assert!(p.len() >= 3);
        // generate store length for 0xfffff max
        p[0] = 0x80 | (len as u8 & 0x0f);
        p[1] = ((len >> 4) & 0xff) as u8;
        p[2] = ((len >> 12) & 0xff) as u8;
    }

    pub fn set_current(&mut self, curr: &str) -> Result<(), Error> {
        self.gencurrent.clear();
        self.gencurrent
            .push_str(curr)
            .map_err(|_| Error::CurrentTooLong)
    }

    pub fn get_current(&self) -> &str {
        &self.gencurrent
    }

    pub fn emit_byte(&mut self, c: u8) -> Result<(), Error> {
        self.gencurrent
            .push(c as char)
            .map_err(|_| Error::CurrentTooLong)
    }

    pub fn emit_ext_op(&mut self, op: u8) -> Result<(), Error> {
        self.emit_byte(EXT_OP_PREFIX)?;
        self.emit_byte(op)
    }

    pub fn emit_word(&mut self, data: u32) -> Result<(), Error> {
        self.emit_byte((data & 0xff) as u8)?;
        self.emit_byte(((data >> 8) & 0xff) as u8)
    }

    pub fn emit_dword(&mut self, data: u32) -> Result<(), Error> {
        self.emit_byte((data & 0xff) as u8)?;
        self.emit_byte(((data >> 8) & 0xff) as u8)?;
        self.emit_byte(((data >> 16) & 0xff) as u8)?;
        self.emit_byte(((data >> 24) & 0xff) as u8)
    }

    pub fn write_package(&mut self, nr_el: u8) -> Result<&str, Error> {
        self.emit_byte(PACKAGE_OP)?;
        self.write_len_f()?;
        self.emit_byte(nr_el)?;
        let p = self.get_current();
        Ok(&p[..p.len() - 2])
    }

    pub fn write_byte(&mut self, data: u32) -> Result<(), Error> {
        self.emit_byte(BYTE_PREFIX)?;
        self.emit_byte((data & 0xff) as u8)
    }

    pub fn write_word(&mut self, data: u32) -> Result<(), Error> {
        self.emit_byte(WORD_PREFIX)?;
        self.emit_word(data)
    }

    pub fn write_dword(&mut self, data: u32) -> Result<(), Error> {
        self.emit_byte(DWORD_PREFIX)?;
        self.emit_dword(data)
    }

    pub fn write_qword(&mut self, data: u64) -> Result<(), Error> {
        self.emit_byte(QWORD_PREFIX)?;
        self.emit_dword((data & 0xffffffff) as u32)?;
        self.emit_dword(((data >> 32) & 0xffffffff) as u32)
    }

    pub fn write_zero(&mut self) -> Result<(), Error> {
        self.emit_byte(ZERO_OP)
    }

    pub fn write_one(&mut self) -> Result<(), Error> {
        self.emit_byte(ONE_OP)
    }

    pub fn write_ones(&mut self) -> Result<(), Error> {
        self.emit_byte(ONES_OP)
    }

    pub fn write_integer(&mut self, data: u64) -> Result<(), Error> {
        if data == 0 {
            self.write_zero()
        } else if data == 1 {
            self.write_one()
        } else if data <= 0xff {
            self.write_byte(data as u32)
        } else if data <= 0xffff {
            self.write_word(data as u32)
        } else if data <= 0xffffffff {
            self.write_dword(data as u32)
        } else {
            self.write_qword(data)
        }
    }

    pub fn write_name_byte(&mut self, name: &str, val: u8) -> Result<(), Error> {
        self.write_name(name)?;
        self.write_byte(val as u32)
    }

    pub fn write_name_dword(&mut self, name: &str, val: u32) -> Result<(), Error> {
        self.write_name(name)?;
        self.write_dword(val)
    }

    pub fn write_name_qword(&mut self, name: &str, val: u64) -> Result<(), Error> {
        self.write_name(name)?;
        self.write_qword(val)
    }

    pub fn write_name_integer(&mut self, name: &str, val: u64) -> Result<(), Error> {
        self.write_name(name)?;
        self.write_integer(val)
    }

    pub fn write_name_string(&mut self, name: &str, string: &str) -> Result<(), Error> {
        self.write_name(name)?;
        self.write_string(string)
    }

    pub fn write_name_unicode(&mut self, name: &str, string: &str) -> Result<(), Error> {
        let len = string.len() + 1;
        self.write_name(name)?;
        self.emit_byte(BUFFER_OP)?;
        self.write_len_f()?;
        self.write_integer(len as u64)?;
        for c in string.chars() {
            self.emit_word(if c as u8 > 0 { c as u32 } else { b'?' as u32 })?;
        }
        self.pop_len();
        Ok(())
    }

    pub fn emit_stream(&mut self, data: &str) -> Result<(), Error> {
        for b in data.chars() {
            self.emit_byte(b as u8)?;
        }
        Ok(())
    }

    pub fn emit_string(&mut self, string: &str) -> Result<(), Error> {
        self.emit_stream(string)?;
        self.emit_byte(b'\0')
    }

    pub fn write_string(&mut self, string: &str) -> Result<(), Error> {
        self.emit_byte(STRING_PREFIX)?;
        self.emit_string(string)
    }

    pub fn write_coreboot_hid(&mut self, id: CorebootAcpiIds) -> Result<(), Error> {
        let mut hid: String<8> = String::new();
        write!(&mut hid, "{}{:04x}", COREBOOT_ACPI_ID, id as u16).map_err(|_| Error::HIDString)?;
        self.write_name_string("_HID", &hid)
    }

    pub fn emit_namestring(&mut self, namepath: &str) -> Result<(), Error> {
        let mut idx = 0;

        // We can start with a '\'
        if &namepath[..1] == "\\" {
            self.emit_byte(b'\\')?;
            idx += 1;
        }

        // And there can be any number of '^'
        while &namepath[idx..idx + 1] == "^" {
            self.emit_byte(b'^')?;
            idx += 1;
        }

        // If we have only \\ or only ^...^ Then we need to put a null name (0x00)
        if &namepath[idx..idx + 1] == "\0" {
            self.emit_byte(ZERO_OP)?;
            return Ok(());
        }

        let mut dotcount = 0;
        let mut dotpos = 0;
        for i in idx..namepath.len() {
            if &namepath[i..i + 1] == "." {
                dotcount += 1;
                dotpos = i;
            }
        }

        if dotcount == 0 {
            self.emit_simple_namestring(namepath)
        } else if dotcount == 1 {
            self.emit_double_namestring(namepath, dotpos)
        } else {
            self.emit_multi_namestring(namepath)
        }
    }

    pub fn write_name(&mut self, name: &str) -> Result<(), Error> {
        self.emit_byte(NAME_OP)?;
        self.emit_namestring(name)
    }

    pub fn emit_simple_namestring(&mut self, name: &str) -> Result<(), Error> {
        let ud = "____";
        for i in 0..4 {
            if &name[i..i + 1] == "\0" || &name[i..i + 1] == "." {
                self.emit_stream(&ud[..4 - i])?;
                break;
            }
            self.emit_byte(name[i..i + 1].as_bytes()[0])?;
        }
        Ok(())
    }

    pub fn emit_double_namestring(&mut self, name: &str, dotpos: usize) -> Result<(), Error> {
        self.emit_byte(DUAL_NAME_PREFIX)?;
        self.emit_simple_namestring(name)?;
        self.emit_simple_namestring(&name[dotpos + 1..])
    }

    pub fn emit_multi_namestring(&mut self, name: &str) -> Result<(), Error> {
        let mut count = 0;
        let mut idx = 0;
        self.emit_byte(MULTI_NAME_PREFIX)?;
        self.emit_byte(ZERO_OP)?;

        while &name[idx..idx + 1] != "\0" {
            self.emit_simple_namestring(&name[idx..])?;
            while &name[idx..idx + 1] != "." && &name[idx..idx + 1] != "\0" {
                idx += 1;
            }
            if &name[idx..idx + 1] == "." {
                idx += 1;
            }
            count += 1;
        }

        // SAFETY: all ACPI name strings should be valid UTF-8
        let bytes = unsafe { self.gencurrent.as_bytes_mut() };
        bytes[0] = count as u8;
        Ok(())
    }

    pub fn write_scope(&mut self, name: &str) -> Result<(), Error> {
        self.emit_byte(SCOPE_OP)?;
        self.write_len_f()?;
        self.emit_namestring(name)
    }

    pub fn get_package_op_element(
        &mut self,
        package_op: u8,
        element: u32,
        dest_op: u8,
    ) -> Result<(), Error> {
        // <dest_op> = DeRefOf (<package_op>[<element>])
        self.write_store()?;
        self.emit_byte(DEREF_OP)?;
        self.emit_byte(INDEX_OP)?;
        self.emit_byte(package_op)?;
        self.write_integer(element as u64)?;
        // Ignore Index() Destination
        self.emit_byte(ZERO_OP)?;
        self.emit_byte(dest_op)
    }

    pub fn set_package_op_element_int(
        &mut self,
        package_op: u8,
        element: u32,
        src: u64,
    ) -> Result<(), Error> {
        // DeRefOf (<package>[<element>]) = <src>
        self.write_store()?;
        self.write_integer(src)?;
        self.emit_byte(DEREF_OP)?;
        self.emit_byte(INDEX_OP)?;
        self.emit_byte(package_op)?;
        self.write_integer(element as u64)?;
        // Ignore Index() Destination
        self.emit_byte(ZERO_OP)
    }

    pub fn get_package_element(
        &mut self,
        package: &str,
        element: u32,
        dest_op: u8,
    ) -> Result<(), Error> {
        // <dest_op> = <package>[<element>]
        self.write_store()?;
        self.emit_byte(INDEX_OP)?;
        self.emit_namestring(package)?;
        self.write_integer(element as u64)?;
        // Ignore Index() Destination
        self.emit_byte(ZERO_OP)?;
        self.emit_byte(dest_op)
    }

    pub fn set_package_element_int(
        &mut self,
        package: &str,
        element: u32,
        src: u64,
    ) -> Result<(), Error> {
        // <package>[<element>] = <src>
        self.write_store()?;
        self.write_integer(src)?;
        self.emit_byte(INDEX_OP)?;
        self.emit_namestring(package)?;
        self.write_integer(element as u64)?;
        // Ignore Index() Destination
        self.emit_byte(ZERO_OP)
    }

    pub fn set_package_element_namestr(
        &mut self,
        package: &str,
        element: u32,
        src: &str,
    ) -> Result<(), Error> {
        // <package>[<element>] = <src>
        self.write_store()?;
        self.emit_namestring(src)?;
        self.emit_byte(INDEX_OP)?;
        self.emit_namestring(package)?;
        self.write_integer(element as u64)?;
        // Ignore Index() Destination
        self.emit_byte(ZERO_OP)
    }

    pub fn write_processor(
        &mut self,
        cpuindex: u8,
        pblock_addr: u32,
        pblock_len: u8,
    ) -> Result<(), Error> {
        // Processor (\_SB.CPcpuindex, cpuindex, pblock_addr, pblock_len)
        let mut pscope: String<15> = String::new();
        self.emit_ext_op(PROCESSOR_OP)?;
        self.write_len_f()?;
        write!(&mut pscope, "{}{:02}", ACPI_CPU_STRING, cpuindex).unwrap();
        self.emit_namestring(&pscope)?;
        self.emit_byte(cpuindex)?;
        self.emit_dword(pblock_addr)?;
        self.emit_byte(pblock_len)
    }

    pub fn write_processor_package(
        &mut self,
        name: &str,
        first_core: u32,
        core_count: u32,
    ) -> Result<(), Error> {
        let mut pscope: String<15> = String::new();

        self.write_name(name)?;
        self.write_package(core_count as u8)?;
        for i in first_core..first_core + core_count {
            write!(&mut pscope, "{}{:02}", ACPI_CPU_STRING, i).unwrap();
            self.emit_namestring(&pscope)?;
        }
        self.pop_len();
        Ok(())
    }

    pub fn write_processor_cnot(&mut self, number_of_cores: u32) -> Result<(), Error> {
        self.write_method("\\_SB.CNOT", 1)?;
        for core_id in 0..number_of_cores {
            let mut buffer: String<DEVICE_PATH_MAX> = String::new();
            write!(buffer, "{}{:02}", ACPI_CPU_STRING, core_id).unwrap();
            self.emit_byte(NOTIFY_OP)?;
            self.emit_namestring(&buffer)?;
            self.emit_byte(ARG0_OP)?;
        }
        self.pop_len();
        Ok(())
    }

    /// Generate ACPI AML code for OperationRegion
    /// Arg0: Pointer to struct opregion opreg = OPREGION(rname, space, offset, len)
    /// where rname is region name, space is region space, offset is region offset &
    /// len is region length.
    /// OperationRegion(regionname, regionspace, regionoffset, regionlength)
    pub fn write_opregion(&mut self, opreg: &OpRegion) -> Result<(), Error> {
        /* OpregionOp */
        self.emit_ext_op(OPREGION_OP)?;
        /* NameString 4 chars only */
        self.emit_simple_namestring(opreg.name)?;
        /* RegionSpace */
        self.emit_byte(opreg.region_space as u8)?;
        /* RegionOffset & RegionLen, it can be byte word or double word */
        self.write_integer(opreg.region_offset as u64)?;
        self.write_integer(opreg.region_len as u64)
    }

    pub fn write_field_length(&mut self, mut len: u32) -> Result<(), Error> {
        let mut emit = [0u8; 4];

        let mut i = 1;
        if len < 0x40 {
            emit[0] = (len & 0x3f) as u8;
        } else {
            emit[0] = (len & 0xf) as u8;
            len >>= 4;
            while len != 0 {
                emit[i] = (len & 0xff) as u8;
                i += 1;
                len >>= 8;
            }
        }
        emit[0] |= ((i - 1) << 6) as u8;

        for j in 0..i {
            self.emit_byte(emit[j])?;
        }

        Ok(())
    }

    pub fn write_field_offset(&mut self, offset: u32, current_bit_pos: u32) -> Result<(), Error> {
        const FUNC_NAME: &str = "write_field_offset";

        if offset < current_bit_pos {
            error!("{}: Cannot move offset backward", FUNC_NAME);
            return Err(Error::InvalidFieldOffset);
        }

        let diff_bits = offset - current_bit_pos;

        if diff_bits > 0xfffffff {
            error!("{}: Offset very large to encode", FUNC_NAME);
            return Err(Error::InvalidFieldOffset);
        }

        self.emit_byte(0)?;
        self.write_field_length(diff_bits)
    }

    pub fn write_field_name(&mut self, name: &str, size: u32) -> Result<(), Error> {
        self.emit_simple_namestring(name)?;
        self.write_field_length(size)
    }

    pub fn write_field_reserved(&mut self, size: u32) -> Result<(), Error> {
        self.emit_byte(0)?;
        self.write_field_length(size)
    }

    /// Generate ACPI AML code for Field
    /// Arg0: region name
    /// Arg1: Pointer to struct fieldlist.
    /// Arg2: no. of entries in Arg1
    /// Arg3: flags which indicate filed access type, lock rule  & update rule.
    /// Example with fieldlist
    /// struct fieldlist l[] = {
    ///	FIELDLIST_OFFSET(0x84),
    ///	FIELDLIST_NAMESTR("PMCS", 2),
    ///	FIELDLIST_RESERVED(6),
    ///	};
    /// acpigen_write_field("UART", l, ARRAY_SIZE(l), FIELD_ANYACC | FIELD_NOLOCK |
    ///								FIELD_PRESERVE);
    /// Output:
    /// Field (UART, AnyAcc, NoLock, Preserve)
    ///	{
    ///		Offset (0x84),
    ///		PMCS,   2,
    ///              , 6,
    ///	}
    pub fn write_field(&mut self, name: &str, l: &[FieldList], flags: u8) -> Result<(), Error> {
        let mut current_bit_pos = 0;

        /* FieldOp */
        self.emit_ext_op(FIELD_OP)?;
        /* Package Length */
        self.write_len_f()?;
        /* NameString 4 chars only */
        self.emit_simple_namestring(name)?;
        /* Field Flag */
        self.emit_byte(flags)?;

        for list in l.iter() {
            match list.field_type {
                FieldType::NameString => {
                    self.write_field_name(list.name, list.bits)?;
                    current_bit_pos += list.bits;
                }
                FieldType::Reserved => {
                    self.write_field_reserved(list.bits)?;
                    current_bit_pos += list.bits;
                }
                FieldType::Offset => {
                    self.write_field_offset(list.bits, current_bit_pos)?;
                    current_bit_pos = list.bits;
                }
                _ => {
                    error!(
                        "{}: Invalid field type 0x{:x}",
                        "write_field", list.field_type as u8
                    );
                    return Err(Error::InvalidFieldType);
                }
            };
        }
        self.pop_len();

        Ok(())
    }

    pub fn write_method(&mut self, name: &str, nargs: u32) -> Result<(), Error> {
        self.__write_method(name, (nargs & 7) as u8)
    }

    pub fn write_method_serialized(&mut self, name: &str, nargs: u32) -> Result<(), Error> {
        self.__write_method(name, ((nargs & 7) | (1 << 3)) as u8)
    }

    fn __write_method(&mut self, name: &str, flags: u8) -> Result<(), Error> {
        self.emit_byte(METHOD_OP)?;
        self.write_len_f()?;
        self.emit_namestring(name)?;
        self.emit_byte(flags)
    }

    pub fn write_device(&mut self, name: &str) -> Result<(), Error> {
        self.emit_ext_op(DEVICE_OP)?;
        self.write_len_f()?;
        self.emit_namestring(name)
    }

    pub fn write_thermal_zone(&mut self, name: &str) -> Result<(), Error> {
        self.emit_ext_op(THERMAL_ZONE_OP)?;
        self.write_len_f()?;
        self.emit_namestring(name)
    }

    pub fn write_sta(&mut self, status: u8) -> Result<(), Error> {
        // Method (_STA, 0, NotSerialized) { Return (status) }
        self.write_method("_STA", 0)?;
        self.emit_byte(RETURN_OP)?;
        self.write_byte(status as u32)?;
        self.pop_len();
        Ok(())
    }

    pub fn write_sta_ext(&mut self, namestring: &str) -> Result<(), Error> {
        self.write_method("_STA", 0)?;
        self.emit_byte(RETURN_OP)?;
        self.emit_namestring(namestring)?;
        self.pop_len();
        Ok(())
    }

    pub fn write_lpi_package(&mut self, level: u64, states: &[AcpiLpiState]) -> Result<(), Error> {
        /*
         * Name (_LPI, Package (0x06)  // _LPI: Low Power Idle States
         * {
         *     0x0000,
         *     0x0000000000000000,
         *     0x0003,
         *     Package (0x0A)
         *     {
         *         0x00000002,
         *         0x00000001,
         *         0x00000001,
         *         0x00000000,
         *         0x00000000,
         *         0x00000000,
         *         ResourceTemplate ()
         *         {
         *             Register (FFixedHW,
         *                 0x02,               // Bit Width
         *                 0x02,               // Bit Offset
         *                 0x0000000000000000, // Address
         *                 ,)
         *         },
         *
         *        ResourceTemplate ()
         *        {
         *            Register (SystemMemory,
         *                0x00,               // Bit Width
         *                0x00,               // Bit Offset
         *                0x0000000000000000, // Address
         *                ,)
         *        },
         *
         *        ResourceTemplate ()
         *        {
         *            Register (SystemMemory,
         *                0x00,               // Bit Width
         *                0x00,               // Bit Offset
         *                0x0000000000000000, // Address
         *                ,)
         *        },
         *
         *        "C1"
         *    },
         *    ...
         * }
         */
        self.write_name("_LPI")?;
        self.write_package(3 + states.len() as u8)?;
        self.write_word(0)?;
        self.write_qword(level)?;
        self.write_word(states.len() as u32)?;
        for state in states.iter() {
            self.write_package(0xa)?;
            self.write_dword(state.min_residency_us)?;
            self.write_dword(state.worst_case_wakeup_latency_us)?;
            self.write_dword(state.flags)?;
            self.write_dword(state.arch_context_lost_flags)?;
            self.write_dword(state.residency_counter_frequency_hz)?;
            self.write_dword(state.enabled_parent_state)?;
            self.write_register_resource(&state.entry_method)?;
            self.write_register_resource(&state.residency_counter_register)?;
            self.write_register_resource(&state.usage_counter_register)?;
            self.write_string(state.state_name)?;
            self.pop_len();
        }
        self.pop_len();
        Ok(())
    }

    /// Generates a func with max supported P-states
    pub fn write_ppc(&mut self, nr: u8) -> Result<(), Error> {
        /*
         * Method (_PPC, 0, NotSerialized)
         * {
         *      Return (nr)
         * }
         */
        self.write_method("_PPC", 0)?;
        self.emit_byte(RETURN_OP)?;
        // arg
        self.write_byte(nr as u32)?;
        self.pop_len();
        Ok(())
    }

    /// Generates a func with max supported P-states saved
    /// in the variable PPCM.
    pub fn write_ppc_nvs(&mut self) -> Result<(), Error> {
        /*
         * Method (_PPC, 0, NotSerialized)
         * {
         *      Return (PPCM)
         * }
         */
        self.write_method("_PPC", 0)?;
        self.emit_byte(RETURN_OP)?;
        // arg
        self.emit_namestring("PPCM")?;
        self.pop_len();
        Ok(())
    }

    pub fn write_tpc(&mut self, gnvs_tpc_limit: &str) -> Result<(), Error> {
        /*
         * Sample _TPC method
         * Method (_TPC, 0, NotSerialized
         * {
         *      Return (\TLVL)
         * }
         */
        self.write_method("_TPC", 0)?;
        self.emit_byte(RETURN_OP)?;
        self.emit_namestring(gnvs_tpc_limit)?;
        self.pop_len();
        Ok(())
    }

    pub fn write_prw(&mut self, wake: u32, level: u32) -> Result<(), Error> {
        /*
         * Name (_PRW, Package () { wake, level }
         */
        self.write_name("_PRW")?;
        self.write_package(2)?;
        self.write_integer(wake as u64)?;
        self.write_integer(level as u64)?;
        self.pop_len();
        Ok(())
    }

    pub fn write_pss_package(
        &mut self,
        core_freq: u32,
        power: u32,
        trans_lat: u32,
        busm_lat: u32,
        control: u32,
        status: u32,
    ) -> Result<(), Error> {
        self.write_package(6)?;
        self.write_dword(core_freq)?;
        self.write_dword(power)?;
        self.write_dword(trans_lat)?;
        self.write_dword(busm_lat)?;
        self.write_dword(control)?;
        self.write_dword(status)?;
        self.pop_len();
        debug!(
            "PSS: {}MHz power {} control 0x{:x} status 0x{:x}",
            core_freq, power, control, status
        );
        Ok(())
    }

    pub fn write_pss_object(&mut self, pstate_values: &[AcpiSwPstate]) -> Result<(), Error> {
        self.write_name("_PSS")?;
        self.write_package(pstate_values.len() as u8)?;

        for pstate in pstate_values.iter() {
            self.write_pss_package(
                pstate.core_freq,
                pstate.power,
                pstate.transition_latency,
                pstate.bus_master_latency,
                pstate.control_value,
                pstate.status_value,
            )?;
        }

        self.pop_len();

        Ok(())
    }

    pub fn write_psd_package(
        &mut self,
        domain: u32,
        numprocs: u32,
        coordtype: PsdCoord,
    ) -> Result<(), Error> {
        self.write_name("_PSD")?;
        self.write_package(1)?;
        self.write_package(5)?;
        self.write_byte(5)?; // 5 values
        self.write_byte(0)?; // revision 0
        self.write_dword(domain)?;
        self.write_dword(coordtype as u32)?;
        self.write_dword(numprocs)?;
        self.pop_len();
        self.pop_len();
        Ok(())
    }

    pub fn write_cst_package_entry(&mut self, cstate: &AcpiCstate) -> Result<(), Error> {
        self.write_package(4)?;
        self.write_register_resource(&cstate.resource)?;
        self.write_byte(cstate.ctype as u32)?;
        self.write_word(cstate.latency as u32)?;
        self.write_dword(cstate.power)?;
        self.pop_len();
        Ok(())
    }

    pub fn write_cst_package(&mut self, cstates: &[AcpiCstate]) -> Result<(), Error> {
        self.write_name("_CST")?;
        self.write_package((cstates.len() + 1) as u8)?;
        self.write_integer(cstates.len() as u64)?;

        for cstate in cstates.iter() {
            self.write_cst_package_entry(cstate)?;
        }

        self.pop_len();

        Ok(())
    }

    pub fn write_csd_package(
        &mut self,
        domain: u32,
        numprocs: u32,
        coordtype: CsdCoord,
        index: u32,
    ) -> Result<(), Error> {
        self.write_name("_CSD")?;
        self.write_package(1)?;
        self.write_package(6)?;
        self.write_integer(6)?; // 6 values
        self.write_byte(0)?; // revision 0
        self.write_dword(domain)?;
        self.write_dword(coordtype as u32)?;
        self.write_dword(numprocs)?;
        self.write_dword(index)?;
        self.pop_len();
        self.pop_len();

        Ok(())
    }

    pub fn write_tss_package(&mut self, tstate_list: &[AcpiTstate]) -> Result<(), Error> {
        /*
         *	Sample _TSS package with 100% and 50% duty cycles
         *	Name (_TSS, Package (0x02)
         *	{
         *		Package(){100, 1000, 0, 0x00, 0)
         *		Package(){50, 520, 0, 0x18, 0)
         *	})
         */

        self.write_name("_TSS")?;
        self.write_package(tstate_list.len() as u8)?;

        for tstate in tstate_list.iter() {
            self.write_package(5)?;
            self.write_dword(tstate.percent)?;
            self.write_dword(tstate.power)?;
            self.write_dword(tstate.latency)?;
            self.write_dword(tstate.control)?;
            self.write_dword(tstate.status)?;
            self.pop_len();
        }

        self.pop_len();

        Ok(())
    }

    pub fn write_tsd_package(
        &mut self,
        domain: u32,
        numprocs: u32,
        coordtype: PsdCoord,
    ) -> Result<(), Error> {
        self.write_name("_TSD")?;
        self.write_package(1)?;
        self.write_package(5)?;
        self.write_byte(5)?; // 5 values
        self.write_byte(0)?; // revision 0
        self.write_dword(domain)?;
        self.write_dword(coordtype as u32)?;
        self.write_dword(numprocs)?;
        self.pop_len();
        self.pop_len();

        Ok(())
    }

    pub fn write_mem32fixed(&mut self, readwrite: i32, base: u32, size: u32) -> Result<(), Error> {
        /*
         * ACPI 4.0 section 6.4.3.4: 32-Bit Fixed Memory Range Descriptor
         * Byte 0:
         *   Bit7  : 1 => big item
         *   Bit6-0: 0000110 (0x6) => 32-bit fixed memory
         */
        self.emit_byte(0x86)?;
        /* Byte 1+2: length (0x0009) */
        self.emit_byte(0x09)?;
        self.emit_byte(0x00)?;
        /* bit1-7 are ignored */
        self.emit_byte(if readwrite != 0 { 0x01 } else { 0x00 })?;
        self.emit_dword(base)?;
        self.emit_dword(size)
    }

    pub fn write_register(&mut self, addr: &AcpiAddr) -> Result<(), Error> {
        self.emit_byte(0x82)?; /* Register Descriptor */
        self.emit_byte(0x0c)?; /* Register Length 7:0 */
        self.emit_byte(0x00)?; /* Register Length 15:8 */
        self.emit_byte(addr.space_id)?; /* Address Space ID */
        self.emit_byte(addr.bit_width)?; /* Register Bit Width */
        self.emit_byte(addr.bit_offset)?; /* Register Bit Offset */
        self.emit_byte(addr.access_size)?; /* Register Access Size */
        self.emit_byte(addr.addrl as u8)?; /* Register Address Low */
        self.emit_byte(addr.addrh as u8) /* Register Address High */
    }

    pub fn write_register_resource(&mut self, addr: &AcpiAddr) -> Result<(), Error> {
        self.write_resourcetemplate_header()?;
        self.write_register(addr)?;
        self.write_resourcetemplate_footer()
    }

    pub fn write_irq(&mut self, mask: u16) -> Result<(), Error> {
        /*
         * ACPI 3.0b section 6.4.2.1: IRQ Descriptor
         * Byte 0:
         *   Bit7  : 0 => small item
         *   Bit6-3: 0100 (0x4) => IRQ port descriptor
         *   Bit2-0: 010 (0x2) => 2 Bytes long
         */
        self.emit_byte(0x22)?;
        self.emit_byte((mask & 0xff) as u8)?;
        self.emit_byte(((mask >> 8) & 0xff) as u8)
    }

    pub fn write_io16(
        &mut self,
        min: u16,
        max: u16,
        align: u8,
        len: u8,
        decode16: u8,
    ) -> Result<(), Error> {
        /*
         * ACPI 4.0 section 6.4.2.6: I/O Port Descriptor
         * Byte 0:
         *   Bit7  : 0 => small item
         *   Bit6-3: 1000 (0x8) => I/O port descriptor
         *   Bit2-0: 111 (0x7) => 7 Bytes long
         */
        self.emit_byte(0x47)?;
        /* Does the device decode all 16 or just 10 bits? */
        /* bit1-7 are ignored */
        self.emit_byte(if decode16 != 0 { 0x01 } else { 0x00 })?;
        /* minimum base address the device may be configured for */
        self.emit_byte((min & 0xff) as u8)?;
        self.emit_byte(((min >> 8) & 0xff) as u8)?;
        /* maximum base address the device may be configured for */
        self.emit_byte((max & 0xff) as u8)?;
        self.emit_byte(((max >> 8) & 0xff) as u8)?;
        /* alignment for min base */
        self.emit_byte((align & 0xff) as u8)?;
        self.emit_byte((len & 0xff) as u8)
    }

    pub fn add_mainboard_rsvd_mem32(&mut self, _dev: &Device, res: &Resource) -> Result<(), Error> {
        self.write_mem32fixed(0, res.base as u32, res.size as u32)
    }

    pub fn add_mainboard_rsvd_io(&mut self, _dev: &Device, res: &Resource) -> Result<(), Error> {
        let mut base = res.base;
        let mut size = res.size;
        while size > 0 {
            let sz = if size > 255 { 255 } else { size as u64 };
            self.write_io16(base as u16, base as u16, 0, sz as u8, 1)?;
            size -= sz;
            base += sz;
        }

        Ok(())
    }

    pub fn write_resourcetemplate_header(&mut self) -> Result<(), Error> {
        /*
         * A ResourceTemplate() is a Buffer() with a
         * (Byte|Word|DWord) containing the length, followed by one or more
         * resource items, terminated by the end tag.
         * (small item 0xf, len 1)
         */
        self.emit_byte(BUFFER_OP)?;
        self.write_len_f()?;
        self.emit_byte(WORD_PREFIX)?;
        self.len_stack[self.ltop] = self.get_current().into();
        self.ltop += 1;
        /* Add 2 dummy bytes for the ACPI word (keep aligned with
        the calculation in acpigen_write_resourcetemplate() below). */
        self.emit_byte(0x00)?;
        self.emit_byte(0x00)
    }

    pub fn write_resourcetemplate_footer(&mut self) -> Result<(), Error> {
        self.ltop -= 1;
        /*
         * end tag (acpi 4.0 Section 6.4.2.8)
         * 0x79 <checksum>
         * 0x00 is treated as a good checksum according to the spec
         * and is what iasl generates.
         */
        self.emit_byte(0x79)?;
        self.emit_byte(0x00)?;

        /* Start counting past the 2-bytes length added in
        acpigen_write_resourcetemplate() above. */
        let curlen = self.get_current().len();
        let p = &mut self.len_stack[self.ltop];
        let len = curlen - (p.len() - 2);

        /* patch len word */
        let p = unsafe { p[..1].as_bytes_mut() };
        p[0] = len as u8 & 0xff;
        p[1] = (len >> 8) as u8 & 0xff;
        /* patch len field */
        self.pop_len();

        Ok(())
    }

    pub fn write_mainboard_resource_template(&mut self, root_dev: &Device) -> Result<(), Error> {
        self.write_resourcetemplate_header()?;

        // Add reserved memory ranges
        self.search_global_resources(
            root_dev,
            IORESOURCE_MEM | IORESOURCE_RESERVE,
            IORESOURCE_MEM | IORESOURCE_RESERVE,
            Self::add_mainboard_rsvd_mem32,
        )?;

        // Add reserved memory ranges
        self.search_global_resources(
            root_dev,
            IORESOURCE_IO | IORESOURCE_RESERVE,
            IORESOURCE_IO | IORESOURCE_RESERVE,
            Self::add_mainboard_rsvd_io,
        )?;

        self.write_resourcetemplate_footer()
    }

    pub fn write_mainboard_resources(
        &mut self,
        scope: &str,
        name: &str,
        root_dev: &Device,
    ) -> Result<(), Error> {
        self.write_scope(scope)?;
        self.write_name(name)?;
        self.write_mainboard_resource_template(root_dev)?;
        self.pop_len();

        Ok(())
    }

    pub fn emit_eisaid(&mut self, eisaid: &str) -> Result<(), Error> {
        let mut compact = 0u32;

        /* Clamping individual values would be better but
          there is a disagreement over what is a valid
          EISA id, so accept anything and don't clamp,
          parent code should create a valid EISAid.
        */
        let ebytes = eisaid.as_bytes();
        compact |= ((ebytes[0] - b'A' + 1) as u32) << 26;
        compact |= ((ebytes[1] - b'A' + 1) as u32) << 21;
        compact |= ((ebytes[2] - b'A' + 1) as u32) << 16;
        compact |= (hex2bin(ebytes[3] as char) as u32) << 12;
        compact |= (hex2bin(ebytes[4] as char) as u32) << 8;
        compact |= (hex2bin(ebytes[5] as char) as u32) << 4;
        compact |= hex2bin(ebytes[5] as char) as u32;

        self.emit_byte(0xc)?;
        self.emit_byte(((compact >> 24) & 0xff) as u8)?;
        self.emit_byte(((compact >> 16) & 0xff) as u8)?;
        self.emit_byte(((compact >> 8) & 0xff) as u8)?;
        self.emit_byte((compact & 0xff) as u8)
    }

    /// ToUUID(uuid)
    ///
    /// ACPI 6.1 Section 19.6.136 table 19-385 defines a special output
    /// order for the bytes that make up a UUID Buffer object.
    /// UUID byte order for input:
    ///   aabbccdd-eeff-gghh-iijj-kkllmmnnoopp
    /// UUID byte order for output:
    ///   ddccbbaa-ffee-hhgg-iijj-kkllmmnnoopp
    pub fn write_uuid(&mut self, uuid: &str) -> Result<(), Error> {
        let mut buf = [0u8; UUID_LEN];
        let order: [usize; UUID_LEN] = [3, 2, 1, 0, 5, 4, 7, 6, 8, 9, 10, 11, 12, 13, 14, 15];

        // Parse UUID string into bytes
        if hexstrtobin(uuid, &mut buf) < UUID_LEN {
            return Err(Error::UUIDTooShort);
        }

        /* BufferOp */
        self.emit_byte(BUFFER_OP)?;
        self.write_len_f()?;

        /* Buffer length in bytes */
        self.write_word(UUID_LEN as u32)?;

        for i in 0..UUID_LEN {
            self.emit_byte(buf[order[i]])?;
        }

        self.pop_len();

        Ok(())
    }

    /// Name (_PRx, Package(One) { name })
    /// ...
    /// PowerResource (name, level, order)
    pub fn write_power_res(
        &mut self,
        name: &str,
        level: u8,
        order: u16,
        dev_states: &[&str],
    ) -> Result<(), Error> {
        for dev_state in dev_states.iter() {
            self.write_name(dev_state)?;
            self.write_package(1)?;
            self.emit_simple_namestring(name)?;
            self.pop_len();
        }

        self.emit_ext_op(POWER_RES_OP)?;

        self.write_len_f()?;

        self.emit_simple_namestring(name)?;
        self.emit_byte(level)?;
        self.emit_word(order as u32)
    }

    pub fn write_sleep(&mut self, sleep_ms: u64) -> Result<(), Error> {
        self.emit_ext_op(SLEEP_OP)?;
        self.write_integer(sleep_ms)
    }

    pub fn write_store(&mut self) -> Result<(), Error> {
        self.emit_byte(STORE_OP)
    }

    /// Store (src, dst)
    pub fn write_store_ops(&mut self, src: u8, dst: u8) -> Result<(), Error> {
        self.write_store()?;
        self.emit_byte(src)?;
        self.emit_byte(dst)
    }

    /// Store (src, "namestr")
    pub fn write_stor_op_to_namestr(&mut self, src: u8, dst: &str) -> Result<(), Error> {
        self.write_store()?;
        self.emit_byte(src)?;
        self.emit_namestring(dst)
    }

    /// Store (src, "namestr")
    pub fn write_store_int_to_namestr(&mut self, src: u64, dst: &str) -> Result<(), Error> {
        self.write_store()?;
        self.write_integer(src)?;
        self.emit_namestring(dst)
    }

    /// Store (src, dst)
    pub fn write_store_int_to_op(&mut self, src: u64, dst: u8) -> Result<(), Error> {
        self.write_store()?;
        self.write_integer(src)?;
        self.emit_byte(dst)
    }

    /// Or (arg1, arg2, res)
    pub fn write_or(&mut self, arg1: u8, arg2: u8, res: u8) -> Result<(), Error> {
        self.emit_byte(OR_OP)?;
        self.emit_byte(arg1)?;
        self.emit_byte(arg2)?;
        self.emit_byte(res)
    }

    /// Xor (arg1, arg2, res)
    pub fn write_xor(&mut self, arg1: u8, arg2: u8, res: u8) -> Result<(), Error> {
        self.emit_byte(XOR_OP)?;
        self.emit_byte(arg1)?;
        self.emit_byte(arg2)?;
        self.emit_byte(res)
    }

    /// And (arg1, arg2, res)
    pub fn write_and(&mut self, arg1: u8, arg2: u8, res: u8) -> Result<(), Error> {
        self.emit_byte(AND_OP)?;
        self.emit_byte(arg1)?;
        self.emit_byte(arg2)?;
        self.emit_byte(res)
    }

    /// Not (arg1, res)
    pub fn write_not(&mut self, arg: u8, res: u8) -> Result<(), Error> {
        self.emit_byte(NOT_OP)?;
        self.emit_byte(arg)?;
        self.emit_byte(res)
    }

    /// Store (str, DEBUG)
    pub fn write_debug_string(&mut self, string: &str) -> Result<(), Error> {
        self.write_store()?;
        self.write_string(string)?;
        self.emit_ext_op(DEBUG_OP)
    }

    /// Store (val, DEBUG)
    pub fn write_debug_integer(&mut self, val: u64) -> Result<(), Error> {
        self.write_store()?;
        self.write_integer(val)?;
        self.emit_ext_op(DEBUG_OP)
    }

    /// Store (op, DEBUG)
    pub fn write_debug_op(&mut self, op: u8) -> Result<(), Error> {
        self.write_store()?;
        self.emit_byte(op)?;
        self.emit_ext_op(DEBUG_OP)
    }

    /// Store (str, DEBUG)
    pub fn write_debug_namestr(&mut self, string: &str) -> Result<(), Error> {
        self.write_store()?;
        self.emit_namestring(string)?;
        self.emit_ext_op(DEBUG_OP)
    }

    pub fn write_if(&mut self) -> Result<(), Error> {
        self.emit_byte(IF_OP)?;
        self.write_len_f()
    }

    pub fn write_if_end(&mut self) -> Result<(), Error> {
        self.pop_len();
        Ok(())
    }

    /// If (And (arg1, arg2))
    pub fn write_if_and(&mut self, arg1: u8, arg2: u8) -> Result<(), Error> {
        self.write_if()?;
        self.emit_byte(AND_OP)?;
        self.emit_byte(arg1)?;
        self.emit_byte(arg2)
    }

    /// Generates ACPI code for checking if operand1 and operand2 are equal.
    /// Both operand1 and operand2 are ACPI ops.
    ///
    /// If (Lequal (op,1 op2))
    pub fn write_if_lequal_op_op(&mut self, op1: u8, op2: u8) -> Result<(), Error> {
        self.write_if()?;
        self.emit_byte(LEQUAL_OP)?;
        self.emit_byte(op1)?;
        self.emit_byte(op2)
    }

    /// Generates ACPI code for checking if operand1 and operand2 are equal, where,
    /// operand1 is ACPI op and operand2 is an integer.
    ///
    /// If (Lequal (op, val))
    pub fn write_if_lequal_op_int(&mut self, op1: u8, val: u64) -> Result<(), Error> {
        self.write_if()?;
        self.emit_byte(LEQUAL_OP)?;
        self.emit_byte(op1)?;
        self.write_integer(val)
    }

    /// Generates ACPI code for checking if operand1 and operand2 are equal, where,
    /// operand1 is namestring and operand2 is an integer.
    ///
    /// If (Lequal ("namestr", val))
    pub fn write_if_lequal_namestr_int(&mut self, namestr: &str, val: u64) -> Result<(), Error> {
        self.write_if()?;
        self.emit_byte(LEQUAL_OP)?;
        self.emit_namestring(namestr)?;
        self.write_integer(val)
    }

    /// Generates ACPI code to check at runtime if an object named `namestring`
    /// exists, and leaves the If scope open to continue execute code when this
    /// is true. NOTE: Requires matching acpigen_write_if_end().
    ///
    /// If (CondRefOf (NAME))
    pub fn write_if_cond_ref_of(&mut self, namestring: &str) -> Result<(), Error> {
        self.write_if()?;
        self.emit_ext_op(COND_REFOF_OP)?;
        self.emit_namestring(namestring)?;
        self.emit_byte(ZERO_OP) // ignore COND_REFOF_OP destination
    }

    /// Closes previously opened if statement and generates ACPI code for else statement.
    pub fn write_else(&mut self) -> Result<(), Error> {
        self.pop_len();
        self.emit_byte(ELSE_OP)?;
        self.write_len_f()
    }

    pub fn write_shiftleft_op_int(&mut self, src_result: u8, count: u64) -> Result<(), Error> {
        self.emit_byte(SHIFT_LEFT_OP)?;
        self.emit_byte(src_result)?;
        self.write_integer(count)?;
        self.emit_byte(ZERO_OP)
    }

    pub fn write_to_buffer(&mut self, src: u8, dst: u8) -> Result<(), Error> {
        self.emit_byte(TO_BUFFER_OP)?;
        self.emit_byte(src)?;
        self.emit_byte(dst)
    }

    pub fn write_to_integer(&mut self, src: u8, dst: u8) -> Result<(), Error> {
        self.emit_byte(TO_INTEGER_OP)?;
        self.emit_byte(src)?;
        self.emit_byte(dst)
    }

    pub fn write_to_integer_from_namestring(
        &mut self,
        source: &str,
        dst_op: u8,
    ) -> Result<(), Error> {
        self.emit_byte(TO_INTEGER_OP)?;
        self.emit_namestring(source)?;
        self.emit_byte(dst_op)
    }

    pub fn write_byte_buffer(&mut self, arr: &[u8]) -> Result<(), Error> {
        self.emit_byte(BUFFER_OP)?;
        self.write_len_f()?;
        self.write_integer(arr.len() as u64)?;

        for &b in arr.iter() {
            self.emit_byte(b)?;
        }

        self.pop_len();

        Ok(())
    }

    pub fn write_return_byte_buffer(&mut self, arr: &[u8]) -> Result<(), Error> {
        self.emit_byte(RETURN_OP)?;
        self.write_byte_buffer(arr)
    }

    pub fn write_return_singleton_buffer(&mut self, arg: u8) -> Result<(), Error> {
        self.write_return_byte_buffer(&[arg])
    }

    pub fn write_return_op(&mut self, arg: u8) -> Result<(), Error> {
        self.emit_byte(RETURN_OP)?;
        self.emit_byte(arg)
    }

    pub fn write_return_byte(&mut self, arg: u8) -> Result<(), Error> {
        self.emit_byte(RETURN_OP)?;
        self.write_byte(arg as u32)
    }

    pub fn write_return_integer(&mut self, arg: u64) -> Result<(), Error> {
        self.emit_byte(RETURN_OP)?;
        self.write_integer(arg)
    }

    pub fn write_return_namestr(&mut self, arg: &str) -> Result<(), Error> {
        self.emit_byte(RETURN_OP)?;
        self.emit_namestring(arg)
    }

    pub fn write_return_string(&mut self, arg: &str) -> Result<(), Error> {
        self.emit_byte(RETURN_OP)?;
        self.write_string(arg)
    }

    pub fn write_upc(&mut self, upc_type: UpcType) -> Result<(), Error> {
        self.write_name("_UPC")?;
        self.write_package(4)?;
        // Connectable
        self.write_byte(if upc_type == UpcType::Unused { 0 } else { 0xff })?;
        // Type
        self.write_byte(upc_type as u32)?;
        // Reserved0
        self.write_zero()?;
        // Reserved1
        self.write_zero()?;
        self.pop_len();

        Ok(())
    }

    pub fn write_pld(&mut self, pld: &Pld) -> Result<(), Error> {
        let buf = pld.to_buffer();

        self.write_name("_PLD")?;
        self.write_package(1)?;
        self.write_byte_buffer(&buf)?;
        self.pop_len();

        Ok(())
    }

    pub fn write_dsm<const N: usize>(
        &mut self,
        uuid: &str,
        callbacks: [Option<fn(&dyn CallbackArg)>; N],
        count: usize,
        arg: &dyn CallbackArg,
    ) -> Result<(), Error> {
        let id: DsmUuid<N> = DsmUuid::create(uuid, callbacks, count, arg);
        self.write_dsm_uuid_arr(&[id])
    }

    pub fn dsm_uuid_enum_functions<const N: usize>(
        &mut self,
        id: &DsmUuid<N>,
    ) -> Result<(), Error> {
        let mut buffer = [0u8; N];
        let mut set = false;
        let mut cb_idx = 0;
        for i in 0..N {
            for j in 0..8 {
                if cb_idx >= N {
                    break;
                }

                if id.callbacks[cb_idx].is_some() {
                    set = true;
                    buffer[i] |= (1 << j) as u8;
                }

                cb_idx += 1;
            }
        }

        if set {
            buffer[0] |= 1 << 0;
        }

        self.write_return_byte_buffer(&buffer)
    }

    pub fn write_dsm_uuid<const N: usize>(&mut self, id: &DsmUuid<N>) -> Result<(), Error> {
        /* If (LEqual (Local0, ToUUID(uuid))) */
        self.write_if()?;
        self.emit_byte(LEQUAL_OP)?;
        self.emit_byte(LOCAL0_OP)?;
        self.write_uuid(id.uuid)?;

        /* ToInteger (Arg2, Local1) */
        self.write_to_integer(ARG2_OP, LOCAL1_OP)?;

        self.write_if_lequal_op_int(LOCAL1_OP, 0)?;

        if let Some(cb) = &id.callbacks[0] {
            cb(id.arg);
        } else if id.count != 0 {
            self.dsm_uuid_enum_functions(id)?;
        }
        self.write_if_end()?;

        for i in 1..id.count {
            /* If (LEqual (Local1, i)) */
            self.write_if_lequal_op_int(LOCAL1_OP, i as u64)?;

            /* Callback to write if handler. */
            if let Some(cb) = &id.callbacks[i] {
                cb(id.arg);
            }

            self.write_if_end()?;
        }

        /* Default case: Return (Buffer (One) { 0x0 }) */
        self.write_return_singleton_buffer(0x0)?;

        self.write_if_end() /* If (LEqual (Local0, ToUUID(uuid))) */
    }

    /// Generate ACPI AML code for _DSM method.
    /// This function takes as input array of uuid for the device, set of callbacks
    /// and argument to pass into the callbacks. Callbacks should ensure that Local0
    /// and Local1 are left untouched. Use of Local2-Local7 is permitted in
    /// callbacks.
    ///
    /// Arguments passed into _DSM method:
    /// Arg0 = UUID
    /// Arg1 = Revision
    /// Arg2 = Function index
    /// Arg3 = Function specific arguments
    ///
    /// AML code generated would look like:
    /// Method (_DSM, 4, Serialized) {
    ///	ToBuffer (Arg0, Local0)
    ///	If (LEqual (Local0, ToUUID(uuid))) {
    ///		ToInteger (Arg2, Local1)
    ///		If (LEqual (Local1, 0)) {
    ///			<acpigen by callback[0]>
    ///		}
    ///		...
    ///		If (LEqual (Local1, n)) {
    ///			<acpigen by callback[n]>
    ///		}
    ///		Return (Buffer (One) { 0x0 })
    ///	}
    ///	...
    ///	If (LEqual (Local0, ToUUID(uuidn))) {
    ///	...
    ///	}
    ///	Return (Buffer (One) { 0x0 })
    /// }
    pub fn write_dsm_uuid_arr<const N: usize>(&mut self, ids: &[DsmUuid<N>]) -> Result<(), Error> {
        /* Method (_DSM, 4, Serialized) */
        self.write_method_serialized("_DSM", 0x4)?;

        /* ToBuffer (Arg0, Local0) */
        self.write_to_buffer(ARG0_OP, LOCAL0_OP)?;

        for id in ids.iter() {
            self.write_dsm_uuid(id)?;
        }

        /* Return (Buffer (One) { 0x0 }) */
        self.write_return_singleton_buffer(0x0)?;

        self.pop_len();

        Ok(())
    }

    pub fn write_cppc_package(&mut self, config: &CppcConfig) -> Result<(), Error> {
        let max = match config.version {
            1 => CppcFields::MaxFieldsVer1 as u32,
            2 => CppcFields::MaxFieldsVer2 as u32,
            3 => CppcFields::MaxFieldsVer3 as u32,
            _ => {
                error!("CPPC version {} is not implemented", config.version);
                return Err(Error::InvalidCppcVersion(config.version));
            }
        };
        self.write_name(CPPC_PACKAGE_NAME)?;

        /* Adding 2 to account for length and version fields */
        self.write_package((max + 2) as u8)?;
        self.write_dword(max + 2)?;

        self.write_byte(config.version)?;

        for entry in config.entries[..max as usize].iter() {
            // SAFETY: accessing union types should be safe,
            // since union should be initialized along with CppcType
            if entry.cppc_type == CppcType::Dword {
                self.write_dword(unsafe { entry.cppc_union.dword })?;
            } else {
                self.write_register_resource(&unsafe { entry.cppc_union.reg })?;
            }
        }

        self.pop_len();

        Ok(())
    }

    pub fn write_cppc_method(&mut self) -> Result<(), Error> {
        let mut pscope: String<16> = String::new();
        write!(&mut pscope, "{}.{}", ACPI_CPU_STRING, CPPC_PACKAGE_NAME).unwrap();

        self.write_method("_CPC", 0)?;
        self.emit_byte(RETURN_OP)?;
        self.emit_namestring(&pscope)?;
        self.pop_len();

        Ok(())
    }

    /// Generate ACPI AML code for _ROM method.
    /// This function takes as input ROM data and ROM length.
    ///
    /// The ACPI spec isn't clear about what should happen at the end of the
    /// ROM. Tests showed that it shouldn't truncate, but fill the remaining
    /// bytes in the returned buffer with zeros.
    ///
    /// Arguments passed into _DSM method:
    /// Arg0 = Offset in Bytes
    /// Arg1 = Bytes to return
    ///
    /// Example:
    ///   acpigen_write_rom(0xdeadbeef, 0x10000)
    ///
    /// AML code generated would look like:
    /// Method (_ROM, 2, NotSerialized) {
    ///
    ///	OperationRegion("ROMS", SYSTEMMEMORY, 0xdeadbeef, 0x10000)
    ///	Field (ROMS, AnyAcc, NoLock, Preserve)
    ///	{
    ///		Offset (0),
    ///		RBF0,   0x80000
    ///	}
    ///
    ///	Store (Arg0, Local0)
    ///	Store (Arg1, Local1)
    ///
    ///	If (LGreater (Local1, 0x1000))
    ///	{
    ///		Store (0x1000, Local1)
    ///	}
    ///
    ///	Store (Local1, Local3)
    ///
    ///	If (LGreater (Local0, 0x10000))
    ///	{
    ///		Return(Buffer(Local1){0})
    ///	}
    ///
    ///	If (LGreater (Local0, 0x0f000))
    ///	{
    ///		Subtract (0x10000, Local0, Local2)
    ///		If (LGreater (Local1, Local2))
    ///		{
    ///			Store (Local2, Local1)
    ///		}
    ///	}
    ///
    ///	Name (ROM1, Buffer (Local3) {0})
    ///
    ///	Multiply (Local0, 0x08, Local0)
    ///	Multiply (Local1, 0x08, Local1)
    ///
    ///	CreateField (RBF0, Local0, Local1, TMPB)
    ///	Store (TMPB, ROM1)
    ///	Return (ROM1)
    /// }
    pub fn write_rom(&mut self, bios: usize, length: usize) -> Result<(), Error> {
        assert!(bios != 0x0);
        assert!(length != 0);

        /* Method (_ROM, 2, Serialized) */
        self.write_method_serialized("_ROM", 2)?;

        let opreg = OpRegion::create(
            "ROMS",
            RegionSpace::SystemMemory,
            bios as u32,
            length as u32,
        );
        self.write_opregion(&opreg)?;

        let l = [
            FieldList::offset(0),
            FieldList::namestr("RBF0", (8 * length) as u32),
        ];

        /* Field (ROMS, AnyAcc, NoLock, Preserve)
         * {
         *  Offset (0),
         *  RBF0,   0x80000
         * } */
        self.write_field(
            opreg.name,
            &l,
            (FIELD_ANYACC | FIELD_NOLOCK | FIELD_PRESERVE) as u8,
        )?;

        /* Store (Arg0, Local0) */
        self.write_store()?;
        self.emit_byte(ARG0_OP)?;
        self.emit_byte(LOCAL0_OP)?;

        /* Store (Arg1, Local1) */
        self.write_store()?;
        self.emit_byte(ARG1_OP)?;
        self.emit_byte(LOCAL1_OP)?;

        /* ACPI SPEC requires to return at maximum 4KiB */
        /* If (LGreater (Local1, 0x1000)) */
        self.write_if()?;
        self.emit_byte(LGREATER_OP)?;
        self.emit_byte(LOCAL1_OP)?;
        self.write_integer(0x1000)?;

        /* Store (0x1000, Local1) */
        self.write_store()?;
        self.write_integer(0x1000)?;
        self.emit_byte(LOCAL1_OP)?;

        /* Pop if */
        self.pop_len();

        /* Store (Local1, Local3) */
        self.write_store()?;
        self.emit_byte(LOCAL1_OP)?;
        self.emit_byte(LOCAL3_OP)?;

        /* If (LGreater (Local0, length)) */
        self.write_if()?;
        self.emit_byte(LGREATER_OP)?;
        self.emit_byte(LOCAL0_OP)?;
        self.write_integer(length as u64)?;

        /* Return(Buffer(Local1){0}) */
        self.emit_byte(RETURN_OP)?;
        self.emit_byte(BUFFER_OP)?;
        self.write_len_f()?;
        self.emit_byte(LOCAL1_OP)?;
        self.emit_byte(0)?;
        self.pop_len();

        /* Pop if */
        self.pop_len();

        /* If (LGreater (Local0, length - 4096)) */
        self.write_if()?;
        self.emit_byte(LGREATER_OP)?;
        self.emit_byte(LOCAL0_OP)?;
        self.write_integer((length - 4096) as u64)?;

        /* Subtract (length, Local0, Local2) */
        self.emit_byte(SUBTRACT_OP)?;
        self.write_integer(length as u64)?;
        self.emit_byte(LOCAL0_OP)?;
        self.emit_byte(LOCAL2_OP)?;

        /* If (LGreater (Local1, Local2)) */
        self.write_if()?;
        self.emit_byte(LGREATER_OP)?;
        self.emit_byte(LOCAL1_OP)?;
        self.emit_byte(LOCAL2_OP)?;

        /* Store (Local2, Local1) */
        self.write_store()?;
        self.emit_byte(LOCAL2_OP)?;
        self.emit_byte(LOCAL1_OP)?;

        /* Pop if */
        self.pop_len();

        /* Pop if */
        self.pop_len();

        /* Name (ROM1, Buffer (Local3) {0}) */
        self.write_name("ROM1")?;
        self.emit_byte(BUFFER_OP)?;
        self.write_len_f()?;
        self.emit_byte(LOCAL3_OP)?;
        self.emit_byte(0)?;
        self.pop_len();

        /* Multiply (Local1, 0x08, Local1) */
        self.emit_byte(MULTIPLY_OP)?;
        self.emit_byte(LOCAL1_OP)?;
        self.write_integer(0x08)?;
        self.emit_byte(LOCAL1_OP)?;

        /* Multiply (Local0, 0x08, Local0) */
        self.emit_byte(MULTIPLY_OP)?;
        self.emit_byte(LOCAL0_OP)?;
        self.write_integer(0x08)?;
        self.emit_byte(LOCAL0_OP)?;

        /* CreateField (RBF0, Local0, Local1, TMPB) */
        self.emit_ext_op(CREATEFIELD_OP)?;
        self.emit_namestring("RBF0")?;
        self.emit_byte(LOCAL0_OP)?;
        self.emit_byte(LOCAL1_OP)?;
        self.emit_namestring("TMPB")?;

        /* Store (TMPB, ROM1) */
        self.write_store()?;
        self.emit_namestring("TMPB")?;
        self.emit_namestring("ROM1")?;

        /* Return (ROM1) */
        self.emit_byte(RETURN_OP)?;
        self.emit_namestring("ROM1")?;

        /* Pop method */
        self.pop_len();

        Ok(())
    }

    /// Helper functions for enabling/disabling Tx GPIOs based on the GPIO
    /// polarity. These functions end up calling acpigen_soc_{set,clear}_tx_gpio to
    /// make callbacks into SoC acpigen code.
    #[cfg(any(feature = "amd", feature = "intel"))]
    pub fn enable_tx_gpio(&mut self, gpio: &Gpio) -> Result<(), Error> {
        if gpio.active_low {
            self.soc_clear_tx_gpio(gpio.pins[0] as u32)
        } else {
            self.soc_set_tx_gpio(gpio.pins[0] as u32)
        }
    }

    #[cfg(any(feature = "amd", feature = "intel"))]
    pub fn disable_tx_gpio(&mut self, gpio: &Gpio) -> Result<(), Error> {
        if gpio.active_low {
            self.soc_set_tx_gpio(gpio.pins[0] as u32)
        } else {
            self.soc_clear_tx_gpio(gpio.pins[0] as u32)
        }
    }

    #[cfg(any(feature = "amd", feature = "intel"))]
    pub fn get_rx_gpio(&mut self, gpio: &Gpio) -> Result<(), Error> {
        self.soc_read_rx_gpio(gpio.pins[0] as u32)?;

        if gpio.active_low {
            self.write_xor(LOCAL0_OP, 1, LOCAL0_OP)?;
        }

        Ok(())
    }

    #[cfg(any(feature = "amd", feature = "intel"))]
    pub fn get_tx_gpio(&mut self, gpio: &Gpio) -> Result<(), Error> {
        self.soc_get_tx_gpio(gpio.pins[0] as u32)?;

        if gpio.active_low {
            self.write_xor(LOCAL0_OP, 1, LOCAL0_OP)?;
        }

        Ok(())
    }

    /// Refer to ACPI 6.4.3.5.3 Word Address Space Descriptor section for details
    pub fn resource_word(
        &mut self,
        res_type: u16,
        gen_flags: u16,
        type_flags: u16,
        gran: u16,
        range_min: u16,
        range_max: u16,
        translation: u16,
        length: u16,
    ) -> Result<(), Error> {
        self.emit_byte(0x88)?;
        /* Byte 1+2: length (0x000d) */
        self.emit_byte(0x0d)?;
        self.emit_byte(0x00)?;
        /* resource type */
        self.emit_byte(res_type as u8)?; // 0 - mem, 1 - io, 2 - bus
                                         /* general flags */
        self.emit_byte(gen_flags as u8)?;
        /* type flags */
        // refer to ACPI Table 6-234 (Memory), 6-235 (IO), 6-236 (Bus) for details
        self.emit_byte(type_flags as u8)?;
        /* granularity, min, max, translation, length */
        self.emit_word(gran as u32)?;
        self.emit_word(range_min as u32)?;
        self.emit_word(range_max as u32)?;
        self.emit_word(translation as u32)?;
        self.emit_word(length as u32)
    }

    /// Refer to ACPI 6.4.3.5.2 DWord Address Space Descriptor section for details
    pub fn resource_dword(
        &mut self,
        res_type: u16,
        gen_flags: u16,
        type_flags: u16,
        gran: u32,
        range_min: u32,
        range_max: u32,
        translation: u32,
        length: u32,
    ) -> Result<(), Error> {
        self.emit_byte(0x87)?;
        /* Byte 1+2: length (0023) */
        self.emit_byte(23)?;
        self.emit_byte(0x00)?;
        /* resource type */
        self.emit_byte(res_type as u8)?; // 0 - mem, 1 - io, 2 - bus
                                         /* general flags */
        self.emit_byte(gen_flags as u8)?;
        /* type flags */
        // refer to ACPI Table 6-234 (Memory), 6-235 (IO), 6-236 (Bus) for details
        self.emit_byte(type_flags as u8)?;
        /* granularity, min, max, translation, length */
        self.emit_dword(gran)?;
        self.emit_dword(range_min)?;
        self.emit_dword(range_max)?;
        self.emit_dword(translation)?;
        self.emit_dword(length)
    }

    pub fn emit_qword(&mut self, data: u64) -> Result<(), Error> {
        self.emit_dword((data & 0xffff_ffff) as u32)?;
        self.emit_dword(((data >> 32) & 0xffff_ffff) as u32)
    }

    /// Refer to ACPI 6.4.3.5.1 QWord Address Space Descriptor section for details
    pub fn resource_qword(
        &mut self,
        res_type: u16,
        gen_flags: u16,
        type_flags: u16,
        gran: u64,
        range_min: u64,
        range_max: u64,
        translation: u64,
        length: u64,
    ) -> Result<(), Error> {
        self.emit_byte(0x8a)?;
        /* Byte 1+2: length (0x002b) */
        self.emit_byte(0x2b)?;
        self.emit_byte(0x00)?;
        /* resource type */
        self.emit_byte(res_type as u8)?; // 0 - mem, 1 - io, 2 - bus
                                         /* general flags */
        self.emit_byte(gen_flags as u8)?;
        /* type flags */
        // refer to ACPI Table 6-234 (Memory), 6-235 (IO), 6-236 (Bus) for details
        self.emit_byte(type_flags as u8)?;
        /* granularity, min, max, translation, length */
        self.emit_qword(gran)?;
        self.emit_qword(range_min)?;
        self.emit_qword(range_max)?;
        self.emit_qword(translation)?;
        self.emit_qword(length)
    }

    pub fn write_adr(&mut self, adr: u64) -> Result<(), Error> {
        self.write_name_qword("_ADR", adr)
    }

    /// acpigen_write_ADR_soundwire_device() - SoundWire ACPI Device Address Encoding.
    /// @address: SoundWire device address properties.
    ///
    /// From SoundWire Discovery and Configuration Specification Version 1.0 Table 3.
    ///
    ///   63..52 - Reserved (0)
    ///   51..48 - Zero-based SoundWire Link ID, relative to the immediate parent.
    ///            Used when a Controller has multiple master devices, each producing a
    ///            separate SoundWire Link.  Set to 0 for single-link controllers.
    ///   47..0  - SoundWire Device ID Encoding from specification version 1.2 table 88
    ///   47..44 - SoundWire specification version that this device supports
    ///   43..40 - Unique ID for multiple devices
    ///   39..24 - MIPI standard manufacturer code
    ///   23..08 - Vendor defined part ID
    ///   07..00 - MIPI class encoding
    pub fn write_adr_soundwire_device(&mut self, address: &SoundwireAddress) -> Result<(), Error> {
        self.write_adr(
            ((address.link_id as u64 & 0xf) << 48)
                | ((address.version as u64 & 0xf) << 44)
                | ((address.unique_id as u64 & 0xf) << 40)
                | ((address.manufacturer_id as u64 & 0xffff) << 24)
                | ((address.part_id as u64 & 0xffff) << 8)
                | (address.class as u64 & 0xff),
        )
    }

    pub fn notify(&mut self, namestr: &str, value: i32) -> Result<(), Error> {
        self.emit_byte(NOTIFY_OP)?;
        self.emit_namestring(namestr)?;
        self.write_integer(value as u64)
    }

    fn _create_field(
        &mut self,
        aml_op: u8,
        srcop: u8,
        byte_offset: usize,
        name: &str,
    ) -> Result<(), Error> {
        self.emit_byte(aml_op)?;
        self.emit_byte(srcop)?;
        self.write_integer(byte_offset as u64)?;
        self.emit_namestring(name)
    }

    pub fn write_create_byte_field(
        &mut self,
        op: u8,
        byte_offset: usize,
        name: &str,
    ) -> Result<(), Error> {
        self._create_field(CREATE_BYTE_OP, op, byte_offset, name)
    }

    pub fn write_create_word_field(
        &mut self,
        op: u8,
        byte_offset: usize,
        name: &str,
    ) -> Result<(), Error> {
        self._create_field(CREATE_WORD_OP, op, byte_offset, name)
    }

    pub fn write_create_dword_field(
        &mut self,
        op: u8,
        byte_offset: usize,
        name: &str,
    ) -> Result<(), Error> {
        self._create_field(CREATE_DWORD_OP, op, byte_offset, name)
    }

    pub fn write_create_qword_field(
        &mut self,
        op: u8,
        byte_offset: usize,
        name: &str,
    ) -> Result<(), Error> {
        self._create_field(CREATE_QWORD_OP, op, byte_offset, name)
    }

    pub fn write_pct_package(
        &mut self,
        perf_ctrl: &AcpiAddr,
        perf_sts: &AcpiAddr,
    ) -> Result<(), Error> {
        self.write_name("_PCT")?;
        self.write_package(0x02)?;
        self.write_register_resource(perf_ctrl)?;
        self.write_register_resource(perf_sts)?;

        self.pop_len();

        Ok(())
    }

    pub fn write_xpss_package(&mut self, pstate_value: &XpssSwPstate) -> Result<(), Error> {
        self.write_package(0x08)?;
        self.write_dword(pstate_value.core_freq as u32)?;
        self.write_dword(pstate_value.power as u32)?;
        self.write_dword(pstate_value.transition_latency as u32)?;
        self.write_dword(pstate_value.bus_master_latency as u32)?;

        self.write_byte_buffer(&pstate_value.control_value.to_le_bytes())?;
        self.write_byte_buffer(&pstate_value.status_value.to_le_bytes())?;
        self.write_byte_buffer(&pstate_value.control_mask.to_le_bytes())?;
        self.write_byte_buffer(&pstate_value.status_mask.to_le_bytes())?;

        self.pop_len();

        Ok(())
    }

    pub fn write_xpss_object(&mut self, pstate_values: &[XpssSwPstate]) -> Result<(), Error> {
        self.write_name("XPSS")?;
        self.write_package(pstate_values.len() as u8)?;
        for pstate in pstate_values.iter() {
            self.write_xpss_package(pstate)?;
        }

        self.pop_len();

        Ok(())
    }

    pub fn write_delay_until_namestr_int(
        &mut self,
        wait_ms: u32,
        name: &str,
        value: u64,
    ) -> Result<(), Error> {
        let mut wait_ms_segment = 1;
        let mut segments = wait_ms;

        if wait_ms > 32 {
            wait_ms_segment = 16;
            segments = wait_ms / 16;
        }

        self.write_store_int_to_op(segments as u64, LOCAL7_OP)?;
        self.emit_byte(WHILE_OP)?;
        self.write_len_f()?;
        self.emit_byte(LGREATER_OP)?;
        self.emit_byte(LOCAL7_OP)?;
        self.emit_byte(ZERO_OP)?;

        /* If name is not provided then just delay in a loop. */
        if name != "" {
            self.write_if_lequal_namestr_int(name, value)?;
            self.emit_byte(BREAK_OP)?;
            self.pop_len(); /* If */
        }

        self.write_sleep(wait_ms_segment)?;
        self.emit_byte(DECREMENT_OP)?;
        self.emit_byte(LOCAL7_OP)?;
        self.pop_len(); /* While */

        Ok(())
    }
}

pub fn hex2bin(c: char) -> u8 {
    if c >= 'A' && c <= 'F' {
        c as u8 - b'A' + 10
    } else if c >= 'a' && c <= 'f' {
        c as u8 - b'a' + 10
    } else {
        c as u8 - b'0'
    }
}

impl GlobalSearch for AcpiGen {
    type Error = Error;
}
