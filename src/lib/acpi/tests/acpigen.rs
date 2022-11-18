use acpi::acpigen::*;

const ACPIGEN_TEST_BUFFER_SZ: usize = 16 * 1024;

/// Returns AML package length. Works with normal and extended packages.
/// This implementation is independent from acpigen.c implementation of package length.
pub fn decode_package_length(ptr: &str) -> usize {
    let aml = ptr.as_bytes();
    let offset = if aml[0] == EXT_OP_PREFIX { 2 } else { 1 };
    let mut byte_zero_mask = 0x3f; /* Bits [0:5] */
    let mut byte_count = aml[offset] >> 6;
    let mut package_length = 0;

    while byte_count != 0 {
        package_length |= aml[offset + byte_count as usize] << ((byte_count << 3) - 4);
        byte_zero_mask = 0x0f; /* Use bits [0:3] of byte 0 */
        byte_count -= 1;
    }

    package_length |= aml[offset] & byte_zero_mask;

    package_length as usize
}

pub fn get_current_block_length(acpigen: &AcpiGen, base: &str) -> usize {
    let offset = if base.as_bytes()[0] == EXT_OP_PREFIX {
        2
    } else {
        1
    };
    acpigen.get_current().len() - base[offset..].len()
}

pub fn setup_acpigen() -> [u8; ACPIGEN_TEST_BUFFER_SZ] {
    [0u8; ACPIGEN_TEST_BUFFER_SZ]
}

pub fn test_acpigen_single_if(acpigen: &mut AcpiGen, state: &mut [u8]) -> Result<(), Error> {
    acpigen.set_current(&std::str::from_utf8(&state).unwrap())?;

    /* Create dummy AML */
    acpigen.write_if_lequal_op_int(LOCAL0_OP, 64)?;

    for _i in 0..20 {
        acpigen.write_store_ops(ZERO_OP, LOCAL1_OP)?;
    }

    /* Close if */
    acpigen.write_if_end()?;

    let if_package_length = decode_package_length(acpigen.get_stack_current());
    let block_length = get_current_block_length(acpigen, acpigen.get_current());
    assert_eq!(if_package_length, block_length);

    Ok(())
}

pub fn create_nested_ifs_recursive(
    acpigen: &mut AcpiGen,
    stack_start: &mut [String],
    stack_end: &mut [String],
    i: usize,
    n: usize,
) -> Result<(), Error> {
    if i >= n {
        return Ok(());
    }

    stack_start[i] = String::from(acpigen.get_current());
    acpigen.write_if_and(LOCAL0_OP, ZERO_OP)?;

    for _k in 0..3 {
        acpigen.write_store_ops(ZERO_OP, LOCAL1_OP)?;
    }

    create_nested_ifs_recursive(acpigen, stack_start, stack_end, i + 1, n)?;

    acpigen.pop_len();

    stack_end[i] = String::from(acpigen.get_current());

    Ok(())
}

pub fn test_acpigen_nested_ifs(acpigen: &mut AcpiGen, state: &mut [u8]) -> Result<(), Error> {
    let acpigen_buf = std::str::from_utf8(state).unwrap();
    let nesting_level = 8;

    let mut block_start = [
        String::from(""),
        String::from(""),
        String::from(""),
        String::from(""),
        String::from(""),
        String::from(""),
        String::from(""),
        String::from(""),
    ];
    let mut block_end = [
        String::from(""),
        String::from(""),
        String::from(""),
        String::from(""),
        String::from(""),
        String::from(""),
        String::from(""),
        String::from(""),
    ];

    acpigen.set_current(&acpigen_buf)?;

    create_nested_ifs_recursive(acpigen, &mut block_start, &mut block_end, 0, nesting_level)?;

    for i in 0..nesting_level as usize {
        assert_eq!(
            decode_package_length(&block_start[i]),
            block_end[i].len() - block_start[i].len() - 1
        );
    }

    Ok(())
}

fn test_acpigen_write_package(acpigen: &mut AcpiGen, state: &mut [u8]) -> Result<(), Error> {
    let acpigen_buf = std::str::from_utf8(state).unwrap();

    acpigen.set_current(&acpigen_buf)?;
    acpigen.write_package(3)?;

    acpigen.write_return_singleton_buffer(0xa)?;
    acpigen.write_return_singleton_buffer(0x7)?;
    acpigen.write_return_singleton_buffer(0xf)?;

    acpigen.pop_len();

    let package_length = decode_package_length(acpigen.get_stack_current());
    let block_length = get_current_block_length(acpigen, acpigen.get_current());

    assert_eq!(package_length, block_length);

    Ok(())
}

fn test_acpigen_scope_with_contents(acpigen: &mut AcpiGen, state: &mut [u8]) -> Result<(), Error> {
    let acpigen_buf = std::str::from_utf8(state).unwrap();
    let mut block_start = [
        String::from(""),
        String::from(""),
        String::from(""),
        String::from(""),
        String::from(""),
        String::from(""),
        String::from(""),
        String::from(""),
    ];
    let mut block_counter = 0;

    acpigen.set_current(&acpigen_buf)?;

	/* Scope("\_SB") { */
    block_start[block_counter] = String::from(acpigen.get_current());
    block_counter += 1;
    acpigen.write_scope("\\_SB")?;

	/* Device("PCI0") { */
    block_start[block_counter] = String::from(acpigen.get_current());
    block_counter += 1;
    acpigen.write_device("PCI0")?;

	/* Name(INT1, 0x1234) */
    acpigen.write_name_integer("INT1", 0x1234)?;

	/* Name (_HID, EisaId ("PNP0A08")) // PCI Express Bus */
    acpigen.write_name("_HID")?;
    acpigen.emit_eisaid("PNP0A08")?;

	/* Method(^BN00, 0, NotSerialized) { */
    block_start[block_counter] = String::from(acpigen.get_current());
    block_counter += 1;
    acpigen.write_method("^BN00", 0)?;

	/* Return( 0x12 + ^PCI0.INT1 ) */
    acpigen.write_return_op(AND_OP)?;
    acpigen.write_byte(0x12)?;
    acpigen.emit_namestring("^PCI0.INT1")?;

	/* } */
    acpigen.pop_len();
    block_counter -= 1;
    let mut package_length = decode_package_length(&block_start[block_counter]);
    let mut block_length = get_current_block_length(acpigen, &block_start[block_counter]);
    assert_eq!(package_length, block_length);

	/* Method (_BBN, 0, NotSerialized) { */
    block_start[block_counter] = String::from(acpigen.get_current());
    block_counter += 1;
    acpigen.write_method("_BBN", 0)?;

	/* Return (BN00 ()) */
    acpigen.write_return_namestr("BN00")?;
    acpigen.emit_byte(0x0a)?;

	/* } */
    acpigen.pop_len();
    block_counter -= 1;
    package_length = decode_package_length(&block_start[block_counter]);
    block_length = get_current_block_length(acpigen, &block_start[block_counter]);
    assert_eq!(package_length, block_length);

	/* } */
    acpigen.pop_len();
    block_counter -= 1;
    package_length = decode_package_length(&block_start[block_counter]);
    block_length = get_current_block_length(acpigen, &block_start[block_counter]);
    assert_eq!(package_length, block_length);

	/* } */
    acpigen.pop_len();
    block_counter -= 1;
    package_length = decode_package_length(&block_start[block_counter]);
    block_length = get_current_block_length(acpigen, &block_start[block_counter]);
    assert_eq!(package_length, block_length);

    Ok(())
}

#[test]
fn test_single_if() -> Result<(), Error> {
    let mut acpigen = AcpiGen::new();
    let mut state = setup_acpigen();

    test_acpigen_single_if(&mut acpigen, &mut state)?;

    Ok(())
}

#[test]
fn test_nested_ifs() -> Result<(), Error> {
    let mut acpigen = AcpiGen::new();
    let mut state = setup_acpigen();

    test_acpigen_nested_ifs(&mut acpigen, &mut state)?;

    Ok(())
}

#[test]
fn test_write_package() -> Result<(), Error> {
    let mut acpigen = AcpiGen::new();
    let mut state = setup_acpigen();

    test_acpigen_write_package(&mut acpigen, &mut state)?;

    Ok(())
}

#[test]
fn test_scope_with_contents() -> Result<(), Error> {
    let mut acpigen = AcpiGen::new();
    let mut state = setup_acpigen();

    test_acpigen_scope_with_contents(&mut acpigen, &mut state)?;

    Ok(())
}
