fn is_hex_digit(c: char) -> bool {
    (c >= 'A' && c <= 'F') || (c >= 'a' && c <= 'f')
}

fn to_lower_hex(c: char) -> char {
    assert!(is_hex_digit(c));
    match c {
        'A' => 'a',
        'B' => 'b',
        'C' => 'c',
        'D' => 'd',
        'E' => 'e',
        'F' => 'f',
        _ => unreachable!("invalid hex digit"),
    }
}

pub fn hexstrtobin(string: &str, buf: &mut [u8]) -> usize {
    let mut byte = 0;
    let mut count = 0;
    let mut ptr = 0;

    for c in string.chars() {
        let mut b = c as u8;
        if !is_hex_digit(c) {
            continue;
        }
        if b.is_ascii_digit() {
            b -= b'0';
        } else {
            b = to_lower_hex(c) as u8 - b'a' + 10;
        }

        byte <<= 4;
        byte |= b;

        count += 1;

        if count > 1 {
            if ptr >= buf.len() {
                return ptr;
            }
            buf[ptr] = byte;
            ptr += 1;
            byte = 0;
            count = 0;
        }
    }

    return ptr;
}
