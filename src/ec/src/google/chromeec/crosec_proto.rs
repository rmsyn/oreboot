/* SPDX-License-Identifier: GPL-2.0-only */

//#include <console/console.h>
//#include <stdint.h>
//#include <string.h>
//
//#include "ec.h"
//#include "ec_commands.h"
//#include "ec_message.h"

/* Common utilities */
use crate::google::chromeec::ec::*;

/* Dumps EC command / response data into debug output.
 *
 * @param name	Message prefix name.
 * @param cmd	Command code, or -1 to ignore cmd message.
 * @param data	Data buffer to print.
 * @param len	Length of data.
 */
pub fn cros_ec_dump_data(name: &str, cmd: i32, data: &[u8]) {
	println!("{}: ", name);
	if cmd != -1 {
		println!("cmd={:02x}: ", cmd);
	}
	for b in data {
		print!("{:02x}", b);
	}
	println!("");
}

/* Calculate a simple 8-bit checksum of a data block
 *
 * @param data	Data block to checksum
 * @param size	Size of data block in bytes
 * @return checksum value (0 to 255)
 */
pub fn cros_ec_calc_checksum(data: &[u8]) -> u8 {
	let mut csum = 0;
	for &b in data {
		csum += b;
	}
	csum & 0xff
}

/**
 * Create a request packet for protocol version 3.
 *
 * @param cec_command	Command description.
 * @param cmd		Packed command bit stream.
 * @return packet size in bytes, or <0 if error.
 */
pub fn create_proto3_request(cec_command: &ChromeECCommand) -> Result<ECCommandV3, Error> {
    let mut cmd = ECCommandV3::new();
    let out_bytes = cec_command.size_in() as usize + cmd.header().len();

	/* Fail if output size is too big */
    if out_bytes > cmd.len() {
        println!("{}: Cannot send {} bytes\n", "create_proto3_request", cec_command.size_in());
        return Err(Error::ECResRequestTruncated);
    }

    {
        let rq = cmd.header_mut();
	    /* Fill in request packet */
	    rq.set_checksum(0);
	    rq.set_command(cec_command.cmd_code());
	    rq.set_command_version(cec_command.cmd_version());
	    rq.set_reserved(0);
	    rq.set_data_len(cec_command.size_in());
    }

	/* Copy data after header */
    cmd.data_mut()[..cec_command.size_in() as usize].copy_from_slice(cec_command.data_in());
    let csum = cros_ec_calc_checksum(&cmd.as_bytes()[..out_bytes]);
	/* Write checksum field so the entire packet sums to 0 */
	cmd.header_mut().set_checksum(csum);

	cros_ec_dump_data("out", cmd.header().command() as i32, &cmd.as_bytes()[..out_bytes]);

	/* Return request packet */
	Ok(cmd)
}

/**
 * Prepare the device to receive a protocol version 3 response.
 *
 * @param cec_command	Command description.
 * @param resp		Response buffer.
 * @return maximum expected number of bytes in response, or <0 if error.
 */
pub fn prepare_proto3_response_buffer(cec_command: &ChromeECCommand, resp: &ECResponseV3) -> Result<usize, Error> {
	let in_bytes = cec_command.size_out() as usize + resp.header().len();

	/* Fail if input size is too big */
	if in_bytes > resp.len() {
		println!("{}: Cannot receive {} bytes\n", "prepare_proto3_response_buffer",
		       cec_command.size_out());
		return Err(Error::ECResResponseTooBig);
	}

	/* Return expected size of response packet */
	Ok(in_bytes)
}

/**
 * Handle a protocol version 3 response packet.
 *
 * The packet must already be stored in the response buffer.
 *
 * @param resp		Response buffer.
 * @param cec_command	Command structure to receive valid response.
 * @return number of bytes of response data, or <0 if error
 */
pub fn handle_proto3_response(resp: &ECResponseV3, cec_command: &mut ChromeECCommand) -> Result<usize, Error> {
	let rs = resp.header();

	cros_ec_dump_data("in-header", -1, &rs.as_bytes());

	/* Check input data */
	if rs.struct_version() != EC_HOST_RESPONSE_VERSION {
		println!("{}: EC response version mismatch", "handle_proto3_response");
		return Err(Error::ECResInvalidResponse);
	}

	if rs.reserved() != 0 {
		println!("{}: EC response reserved != 0", "handle_proto3_response");
		return Err(Error::ECResInvalidResponse);
	}

	if rs.data_len() as usize > resp.raw_data().len() ||
	    rs.data_len() > cec_command.size_out() {
		println!("{}: EC returned too much data\n", "handle_proto3_response");
		return Err(Error::ECResResponseTooBig);
	}

	cros_ec_dump_data("in-data", -1, resp.data());

	/* Update in_bytes to actual data size */
	let in_bytes = rs.len() + rs.data_len() as usize;

	/* Verify checksum */
	let csum = cros_ec_calc_checksum(&resp.as_bytes()[..in_bytes]);
	if csum != 0 {
		println!("{}: EC response checksum invalid: 0x{:02x}\n",
		       "handle_proto3_response", csum);
		return Err(Error::ECResInvalidChecksum);
	}

	/* Return raw response. */
	cec_command.set_cmd_code(rs.result());
	cec_command.set_size_out(rs.data_len());
	cec_command.data_out_mut().copy_from_slice(resp.data());

	/* Return error result, if any */
	if rs.result() != 0 {
		println!("{}: EC response with error code: {}\n",
		       "handle_proto3_response", rs.result());
		return Err(Error::ECResResponse(-(rs.result() as i32)));
	}

	Ok(rs.data_len() as usize)
}

pub fn send_command_proto3(cec_command: &mut ChromeECCommand, crosec_io: CrosECIO, context: ECContext) -> Result<usize, Error> {
    let resp = ECResponseV3::new();

	/* Create request packet */
	let req = create_proto3_request(cec_command)?;

	/* Prepare response buffer */
	let in_bytes = prepare_proto3_response_buffer(cec_command, &resp)?;

    let out_bytes = req.header().data_len() as usize;
	let rv = crosec_io(out_bytes, in_bytes, context);
	if rv != 0 {
		println!("{}: failed to complete I/O: Err = {:02x}.\n",
		       "send_command_proto3", rv);
		return Err(Error::ECResError);
	}

	/* Process the response */
	handle_proto3_response(&resp, cec_command)
}

pub fn crosec_command_proto_v3(cec_command: &mut ChromeECCommand, crosec_io: CrosECIO, context: ECContext) -> Result<usize, Error>
{
	send_command_proto3(cec_command, crosec_io, context)
}

pub fn crosec_command_proto(cec_command: &mut ChromeECCommand, crosec_io: CrosECIO, context: ECContext) -> Result<usize, Error>
{
	// TODO(hungte) Detect and fallback to v2 if we need.
	crosec_command_proto_v3(cec_command, crosec_io, context)
}
