/// 5.1 Integer Representation
/// Integers are used to represent name indexes, header field indexes, or string lengths. An integer representation can start anywhere within an octet. To allow for optimized processing, an integer representation always finishes at the end of an octet.
///
/// An integer is represented in two parts: a prefix that fills the current octet and an optional list of octets that are used if the integer value does not fit within the prefix. The number of bits of the prefix (called N) is a parameter of the integer representation.
///
/// If the integer value is small enough, i.e., strictly less than 2N-1, it is encoded within the N-bit prefix.
///
///   0   1   2   3   4   5   6   7
/// +---+---+---+---+---+---+---+---+
/// | ? | ? | ? |       Value       |
/// +---+---+---+-------------------+
/// Figure 2: Integer Value Encoded within the Prefix (Shown for N = 5)
///
/// Otherwise, all the bits of the prefix are set to 1, and the value, decreased by 2N-1, is encoded using a list of one or more octets. The most significant bit of each octet is used as a continuation flag: its value is set to 1 except for the last octet in the list. The remaining bits of the octets are used to encode the decreased value.
///
///   0   1   2   3   4   5   6   7
/// +---+---+---+---+---+---+---+---+
/// | ? | ? | ? | 1   1   1   1   1 |
/// +---+---+---+-------------------+
/// | 1 |    Value-(2^N-1) LSB      |
/// +---+---------------------------+
///                ...
/// +---+---------------------------+
/// | 0 |    Value-(2^N-1) MSB      |
/// +---+---------------------------+
/// Figure 3: Integer Value Encoded after the Prefix (Shown for N = 5)
///
/// Decoding the integer value from the list of octets starts by reversing the order of the octets in the list. Then, for each octet, its most significant bit is removed. The remaining bits of the octets are concatenated, and the resulting value is increased by 2N-1 to obtain the integer value.
///
/// The prefix size, N, is always between 1 and 8 bits. An integer starting at an octet boundary will have an 8-bit prefix.
///

pub fn decode_integer(buf: &[u8], prefix_size: u8) -> Result<(u32, u8), &'static str> {
    use std::num::Wrapping;

    if prefix_size < 1 || prefix_size > 8 {
        return Err("hpack integer: invalid prefix");
    }
    if buf.len() < 1 {
        return Err("hpack integer: not enough octets (0)");
    }

    // Make sure there's no overflow in the shift operation
    let Wrapping(mask) = if prefix_size == 8 {
        Wrapping(0xFF)
    } else {
        Wrapping(1u8 << prefix_size) - Wrapping(1)
    };
    let mut value = (buf[0] & mask) as u32;
    if value < (mask as u32) {
        // Value fits in the prefix bits.
        return Ok((value, 1));
    }

    // The value does not fit into the prefix bits, so we read as many following
    // bytes as necessary to decode the integer.
    // Already one byte used (the prefix)
    let mut total = 1;
    let mut m = 0;
    // The octet limit is chosen such that the maximum allowed *value* can
    // never overflow an unsigned 32-bit integer. The maximum value of any
    // integer that can be encoded with 5 octets is ~2^28
    let octet_limit = 5;

    for &b in buf[1..].iter() {
        total += 1;
        value += ((b & 127) as u32) * (1 << m);
        m += 7;

        if b & 128 != 128 {
            // Most significant bit is not set => no more continuation bytes
            return Ok((value, total));
        }

        if total == octet_limit {
            // The spec tells us that we MUST treat situations where the
            // encoded representation is too long (in octets) as an error.
            return Err("hpack integer: to many octets");
        }
    }

    // If we have reached here, it means the buffer has been exhausted without
    // hitting the termination condition.
    Err("hpack integer: not enough octets")
}

#[cfg(test)]
mod tests {
    use super::decode_integer;

    #[test]
    fn decode_test() {
        // simple tst
        let tst_num = vec![0x41];
        let num = decode_integer(&tst_num, 8).unwrap();
        assert_eq!(num, (65, 1));

        // complex number
        let tst_num = vec![0xFF, 0x05];
        let num = decode_integer(&tst_num, 8).unwrap();
        assert_eq!(num, (260, 2));

        // more complex number
        let tst_num = vec![0xFF, 0x9A, 0x0A];
        let num = decode_integer(&tst_num, 5).unwrap();
        assert_eq!(num, (1337, 3));
    }
}

// decode I from the next N bits
// if I < 2^N - 1, return I
// else
//     M = 0
//     repeat
//         B = next octet
//         I = I + (B & 127) * 2^M
//         M = M + 7
//     while B & 128 == 128
//     return I

