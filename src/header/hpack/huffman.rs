
use std::collections::HashMap;
use std::slice;

use bititor::BitItor;

// huffman layout array of (huffman code, length of code)
type HuffmanTable = [(u32, u8)];

/// Decodes Huffman encoded strings
/// Optimized specialized for http2 Huffman encoded strings
pub struct Huffman {
    decode_table: &'static HashMap<(u32, u8), u8>,
    encode_table: &'static HuffmanTable,
}

lazy_static! {
    static ref D_TABLE: HashMap<(u32, u8), u8> = {
        let len = HUFFMAN_TABLE.len();

        let mut hash_map = HashMap::with_capacity(len);

        for i in 0..len {
            hash_map.insert(HUFFMAN_TABLE[i], i as u8);
        }

        drun!({ // checking the memory efficiency of the huffman encoder/decoder
            use std::mem;
            println!("huffman static HUFFMAN_TABLE len: {}", len);
            println!("huffman HUFFMAN_TABLE hasmap:\nmain size bytes {} :: Table size bytes {} :: Capacity {}",
                     mem::size_of::<Huffman>(),
                     mem::size_of_val(&hash_map.entry((0x1ff8, 13))) * hash_map.capacity(),
                     hash_map.capacity());
        });

        hash_map
    };
}

impl Huffman {
    pub fn new() -> Self {
        Huffman {
            decode_table: &D_TABLE,
            encode_table: HUFFMAN_TABLE,
        }
    }

    pub fn decode<'a, 'b, B: IntoIterator<Item=&'b u8>>(&self, buf: B) -> Vec<u8>
        where <B as ::std::iter::IntoIterator>::IntoIter: 'a {
        // create vec with enough space for most of the decoded buf
        // some reallocation will probably happen with current implementation

        let mut bts = buf.into_iter();

        //let bts: &mut B::IntoIter = &mut itr;

        let decode_size: usize = f32::ceil(bts.size_hint().0 as f32 * 1.5) as usize;
        let mut decoded = Vec::with_capacity(decode_size);

        // drun!{{
        //     println!("pre fill capacity: {}", decoded.capacity());
        //     for b in buf {
        //         print!("{:02X}", b);
        //     }
        //     println!("");
        // }}

        let bits = BitItor::new(&mut bts);

        // the encoded bits
        let mut code = 0u32;
        // the number of encoded bits
        let mut size = 0u8;
        for bit in bits {
            // get the bit
            code <<= 1;
            if bit {
                code |= 0x1;
            }
            size += 1;

            // check if the curently read bits are a valid huffman code
            match self.decode_table.get(&(code, size)) {
                Some(val)   => {
                    decoded.push(*val);
                    code = 0;
                    size = 0;
                },
                None        => {},
            }
        }

        drun!( {
            let len = decoded.len();
            let cap = decoded.capacity();

            println!("decoded len: {} AND decoded capacity {}", len, cap);
            println!("len capacity ratio: {}", len as f32 / cap as f32);
        } );

        decoded
    }

    // write the encoded result to dest and return the length of result
    pub fn encode(&self, src: &[u8], dest: &mut [u8]) -> usize {
        let mut dest_i = 0; // byte index
        let mut offset = 0;

        for i in src { // for each char in src as index
            let mut code_i = 0;

            let encodeing = self.encode_table[*i as usize];
            let mut code = encodeing.0;
            let mut code_len = encodeing.1 as i8;

            let shift = 32 - code_len - offset;

            code = code << shift;

            let be_code = u32::to_be(code);

            let code_buf: &[u8] = unsafe { slice::from_raw_parts(&be_code as *const u32 as *const u8, 4) };

            debug_assert!(offset >= 0);
            // this deal with what happens when a dest byte is
            // only partial filled by the previous huff code
            // because the codes need to be tightly packed
            if offset > 0 {
                dest[dest_i] |= code_buf[code_i];

                let t_offset = code_len + offset;
                if t_offset < 8 {
                    offset = t_offset;
                    continue;
                }

                dest_i += 1;
                code_len -= 8 - offset;
                if code_len <= 0 {
                    offset = code_len;
                    continue;
                }

                code_i += 1;
                offset = 0;
            }

            // deal with writing each part of the code to dest
            loop {
                dest[dest_i] = code_buf[code_i];

                code_len -= 8;
                if code_len > 0 {
                    dest_i += 1;
                }
                else if code_len == 0 {
                    dest_i += 1;
                    break;
                }
                else {
                    offset = 8 + code_len;
                    break;
                }

                code_i += 1;
            }
        }

        // write the 1's that "pad" the last dest byte if it
        // is not completely filled
        if offset != 0 {
            let end_bits = 8 - offset;
            debug_assert!(end_bits < 8 && end_bits > 0);
            let mut bits = 0;
            for _ in 0..end_bits {
                bits = ( bits << 1) | 1;
            }
            dest[dest_i] |= bits;
        }

        dest_i + 1
    }
}

/// Huffman table specialized for http2 headers
static HUFFMAN_TABLE: &'static HuffmanTable = &[
    (0x1ff8, 13),
    (0x7fffd8, 23),
    (0xfffffe2, 28),
    (0xfffffe3, 28),
    (0xfffffe4, 28),
    (0xfffffe5, 28),
    (0xfffffe6, 28),
    (0xfffffe7, 28),
    (0xfffffe8, 28),
    (0xffffea, 24),
    (0x3ffffffc, 30),
    (0xfffffe9, 28),
    (0xfffffea, 28),
    (0x3ffffffd, 30),
    (0xfffffeb, 28),
    (0xfffffec, 28),
    (0xfffffed, 28),
    (0xfffffee, 28),
    (0xfffffef, 28),
    (0xffffff0, 28),
    (0xffffff1, 28),
    (0xffffff2, 28),
    (0x3ffffffe, 30),
    (0xffffff3, 28),
    (0xffffff4, 28),
    (0xffffff5, 28),
    (0xffffff6, 28),
    (0xffffff7, 28),
    (0xffffff8, 28),
    (0xffffff9, 28),
    (0xffffffa, 28),
    (0xffffffb, 28),
    (0x14, 6),
    (0x3f8, 10),
    (0x3f9, 10),
    (0xffa, 12),
    (0x1ff9, 13),
    (0x15, 6),
    (0xf8, 8),
    (0x7fa, 11),
    (0x3fa, 10),
    (0x3fb, 10),
    (0xf9, 8),
    (0x7fb, 11),
    (0xfa, 8),
    (0x16, 6),
    (0x17, 6),
    (0x18, 6),
    (0x0, 5),
    (0x1, 5),
    (0x2, 5),
    (0x19, 6),
    (0x1a, 6),
    (0x1b, 6),
    (0x1c, 6),
    (0x1d, 6),
    (0x1e, 6),
    (0x1f, 6),
    (0x5c, 7),
    (0xfb, 8),
    (0x7ffc, 15),
    (0x20, 6),
    (0xffb, 12),
    (0x3fc, 10),
    (0x1ffa, 13),
    (0x21, 6),
    (0x5d, 7),
    (0x5e, 7),
    (0x5f, 7),
    (0x60, 7),
    (0x61, 7),
    (0x62, 7),
    (0x63, 7),
    (0x64, 7),
    (0x65, 7),
    (0x66, 7),
    (0x67, 7),
    (0x68, 7),
    (0x69, 7),
    (0x6a, 7),
    (0x6b, 7),
    (0x6c, 7),
    (0x6d, 7),
    (0x6e, 7),
    (0x6f, 7),
    (0x70, 7),
    (0x71, 7),
    (0x72, 7),
    (0xfc, 8),
    (0x73, 7),
    (0xfd, 8),
    (0x1ffb, 13),
    (0x7fff0, 19),
    (0x1ffc, 13),
    (0x3ffc, 14),
    (0x22, 6),
    (0x7ffd, 15),
    (0x3, 5),
    (0x23, 6),
    (0x4, 5),
    (0x24, 6),
    (0x5, 5),
    (0x25, 6),
    (0x26, 6),
    (0x27, 6),
    (0x6, 5),
    (0x74, 7),
    (0x75, 7),
    (0x28, 6),
    (0x29, 6),
    (0x2a, 6),
    (0x7, 5),
    (0x2b, 6),
    (0x76, 7),
    (0x2c, 6),
    (0x8, 5),
    (0x9, 5),
    (0x2d, 6),
    (0x77, 7),
    (0x78, 7),
    (0x79, 7),
    (0x7a, 7),
    (0x7b, 7),
    (0x7ffe, 15),
    (0x7fc, 11),
    (0x3ffd, 14),
    (0x1ffd, 13),
    (0xffffffc, 28),
    (0xfffe6, 20),
    (0x3fffd2, 22),
    (0xfffe7, 20),
    (0xfffe8, 20),
    (0x3fffd3, 22),
    (0x3fffd4, 22),
    (0x3fffd5, 22),
    (0x7fffd9, 23),
    (0x3fffd6, 22),
    (0x7fffda, 23),
    (0x7fffdb, 23),
    (0x7fffdc, 23),
    (0x7fffdd, 23),
    (0x7fffde, 23),
    (0xffffeb, 24),
    (0x7fffdf, 23),
    (0xffffec, 24),
    (0xffffed, 24),
    (0x3fffd7, 22),
    (0x7fffe0, 23),
    (0xffffee, 24),
    (0x7fffe1, 23),
    (0x7fffe2, 23),
    (0x7fffe3, 23),
    (0x7fffe4, 23),
    (0x1fffdc, 21),
    (0x3fffd8, 22),
    (0x7fffe5, 23),
    (0x3fffd9, 22),
    (0x7fffe6, 23),
    (0x7fffe7, 23),
    (0xffffef, 24),
    (0x3fffda, 22),
    (0x1fffdd, 21),
    (0xfffe9, 20),
    (0x3fffdb, 22),
    (0x3fffdc, 22),
    (0x7fffe8, 23),
    (0x7fffe9, 23),
    (0x1fffde, 21),
    (0x7fffea, 23),
    (0x3fffdd, 22),
    (0x3fffde, 22),
    (0xfffff0, 24),
    (0x1fffdf, 21),
    (0x3fffdf, 22),
    (0x7fffeb, 23),
    (0x7fffec, 23),
    (0x1fffe0, 21),
    (0x1fffe1, 21),
    (0x3fffe0, 22),
    (0x1fffe2, 21),
    (0x7fffed, 23),
    (0x3fffe1, 22),
    (0x7fffee, 23),
    (0x7fffef, 23),
    (0xfffea, 20),
    (0x3fffe2, 22),
    (0x3fffe3, 22),
    (0x3fffe4, 22),
    (0x7ffff0, 23),
    (0x3fffe5, 22),
    (0x3fffe6, 22),
    (0x7ffff1, 23),
    (0x3ffffe0, 26),
    (0x3ffffe1, 26),
    (0xfffeb, 20),
    (0x7fff1, 19),
    (0x3fffe7, 22),
    (0x7ffff2, 23),
    (0x3fffe8, 22),
    (0x1ffffec, 25),
    (0x3ffffe2, 26),
    (0x3ffffe3, 26),
    (0x3ffffe4, 26),
    (0x7ffffde, 27),
    (0x7ffffdf, 27),
    (0x3ffffe5, 26),
    (0xfffff1, 24),
    (0x1ffffed, 25),
    (0x7fff2, 19),
    (0x1fffe3, 21),
    (0x3ffffe6, 26),
    (0x7ffffe0, 27),
    (0x7ffffe1, 27),
    (0x3ffffe7, 26),
    (0x7ffffe2, 27),
    (0xfffff2, 24),
    (0x1fffe4, 21),
    (0x1fffe5, 21),
    (0x3ffffe8, 26),
    (0x3ffffe9, 26),
    (0xffffffd, 28),
    (0x7ffffe3, 27),
    (0x7ffffe4, 27),
    (0x7ffffe5, 27),
    (0xfffec, 20),
    (0xfffff3, 24),
    (0xfffed, 20),
    (0x1fffe6, 21),
    (0x3fffe9, 22),
    (0x1fffe7, 21),
    (0x1fffe8, 21),
    (0x7ffff3, 23),
    (0x3fffea, 22),
    (0x3fffeb, 22),
    (0x1ffffee, 25),
    (0x1ffffef, 25),
    (0xfffff4, 24),
    (0xfffff5, 24),
    (0x3ffffea, 26),
    (0x7ffff4, 23),
    (0x3ffffeb, 26),
    (0x7ffffe6, 27),
    (0x3ffffec, 26),
    (0x3ffffed, 26),
    (0x7ffffe7, 27),
    (0x7ffffe8, 27),
    (0x7ffffe9, 27),
    (0x7ffffea, 27),
    (0x7ffffeb, 27),
    (0xffffffe, 28),
    (0x7ffffec, 27),
    (0x7ffffed, 27),
    (0x7ffffee, 27),
    (0x7ffffef, 27),
    (0x7fffff0, 27),
    (0x3ffffee, 26),
    (0x3fffffff, 30),
    ];

#[cfg(test)]
mod huffman_tests {
    use super::Huffman;
    use std::str;

    #[test]
    fn decode_test1() {
        let encoded = [0x08, 0x9D, 0x5C, 0x0B, 0x81, 0x70, 0xDC, 0x78, 0x0F, 0x03];

        let huff = Huffman::new();
        let decoded = huff.decode(&encoded);

        println!("decoded value: {}", str::from_utf8(&decoded).unwrap());

        assert_eq!(decoded, b"127.0.0.1:8080");
    }

    #[test]
    fn decode_test2() {
        let encoded = [0xA0, 0xE4, 0x1D, 0x13, 0x9D, 0x09, 0xB8, 0xF0, 0x1E, 0x07];

        let huff = Huffman::new();
        let decoded = huff.decode(&encoded);

        println!("decoded value: {}", str::from_utf8(&decoded).unwrap());

        assert_eq!(decoded, b"localhost:8080");
    }

    #[test]
    fn encode_test() {
        let mut v = Vec::with_capacity(20);
        unsafe { v.set_len(20) };

        let s = b"localhost:8080";

        encode(s, &mut v);

        let encoded = [0xA0, 0xE4, 0x1D, 0x13, 0x9D, 0x09, 0xB8, 0xF0, 0x1E, 0x07];
        assert_eq!(encoded, v[..]);

        // longer test string
        let s = b"Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/53.0.2785.116 Safari/537.36";

        let mut v = Vec::with_capacity(100);
        unsafe { v.set_len(100) };
        encode(s, &mut v);

        let encoded = [0xD0, 0x7F, 0x66, 0xA2, 0x81, 0xB0, 0xDA, 0xE0, 0x53, 0xFA, 0xFC, 0x08, 0x7E, 0xD4, 0xCE, 0x6A, 0xAD, 0xF2, 0xA7, 0x97, 0x9C, 0x89, 0xC6, 0xBF, 0xB5, 0x21, 0xAE, 0xBA, 0x0B, 0xC8, 0xB1, 0xE6, 0x32, 0x58, 0x6D, 0x97, 0x57, 0x65, 0xC5, 0x3F, 0xAC, 0xD8, 0xF7, 0xE8, 0xCF, 0xF4, 0xA5, 0x06, 0xEA, 0x55, 0x31, 0x14, 0x9D, 0x4F, 0xFD, 0xA9, 0x7A, 0x7B, 0x0F, 0x49, 0x58, 0x6D, 0x95, 0xC0, 0xB8, 0x9D, 0x79, 0xB5, 0xC2, 0x17, 0x14, 0xDC, 0x39, 0x47, 0x61, 0x98, 0x6D, 0x97, 0x57, 0x65, 0xCF];

        for (x, y) in v.iter().zip(encoded.iter()) {
            assert_eq!(x, y);
        }

        // another test string
        let s = b"text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8";

        let mut v = Vec::with_capacity(100);
        unsafe { v.set_len(100) };
        encode(s, &mut v);

        let encoded = [0x49, 0x7C, 0xA5, 0x89, 0xD3, 0x4D, 0x1F, 0x43, 0xAE, 0xBA, 0x0C, 0x41, 0xA4, 0xC7, 0xA9, 0x8F, 0x33, 0xA6, 0x9A, 0x3F, 0xDF, 0x9A, 0x68, 0xFA, 0x1D, 0x75, 0xD0, 0x62, 0x0D, 0x26, 0x3D, 0x4C, 0x79, 0xA6, 0x8F, 0xBE, 0xD0, 0x01, 0x77, 0xFE, 0x8D, 0x48, 0xE6, 0x2B, 0x1E, 0x0B, 0x1D, 0x7F, 0x5F, 0x2C, 0x7C, 0xFD, 0xF6, 0x80, 0x0B, 0xBD];

        for (x, y) in v.iter().zip(encoded.iter()) {
            assert_eq!(x, y);
        }
    }

    fn encode(src: &[u8], dest: &mut Vec<u8>) {
        let huff = Huffman::new();
        let size = huff.encode(src, dest);

        unsafe { dest.set_len(size) };

        for b in dest {
            print!("{:02X}", b);
        }
        println!("");
    }
}
