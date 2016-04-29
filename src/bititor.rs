use std::iter::Iterator;

/// Iterates over the bits of a buffer
pub struct BitItor<'a> {
    buf: &'a [u8],
    index: usize,
    bit: u8,
}

impl<'a> BitItor<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        BitItor {
            buf: buf,
            index: 0,
            bit: 0,
        }
    }
}

impl<'a> Iterator for BitItor<'a> {
    type Item = bool;
    fn next(&mut self) -> Option<Self::Item> {
        // is this the end of the buffer
        if self.index == self.buf.len() {
            return None;
        }

        // get is_set
        let byte = self.buf[self.index];
        let mask = 0x80 >> self.bit;
        let is_set: bool = byte & mask > 0;

        // iterate
        self.bit += 1;
        if self.bit > 7 {
            self.bit = 0;
            self.index += 1;
        }

        Some(is_set)
    }
}

#[cfg(test)]
mod bit_iter_tests {
    use super::BitItor;

    #[test]
    fn bit_iterator() {
        // test via iterating through a random buffer and
        // reconstructing it with the iterator results
        let buf = [0xf3, 0x21, 0x75, 0x21];
        println!("{:?}", buf);
        let bi = BitItor::new(&buf);

        let mut tbuf = [0u8; 4];
        let mut index = 0;
        let mut bit = 0;
        for b in bi {
            println!("{:?}", b);
            tbuf[index] <<= 1;
            if b {
                tbuf[index] |= 0x1;
            }

            println!("{:?}", tbuf);

            bit += 1;
            if bit > 7 {
                bit = 0;
                index += 1;
            }
        }

        assert_eq!(buf, tbuf);
    }
}
