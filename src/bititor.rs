use borrow_iter::{BPeekable, BorrowPeekable};

/// Iterates over the bits of a buffer
pub struct BitItor<'a, I: Iterator + 'a> {
    buf: BPeekable<'a, I>,
    bit: u8,
}

// NOTE TO SELF -- this works if i just take mut ref to an already iterator
impl<'a, 'b, I> BitItor<'a, I>
    where 'b: 'a, I: Iterator<Item=&'b u8> {

    pub fn new(buf: &'a mut I) -> Self {
        BitItor {
            buf: buf.borrow_peekable(),
            bit: 0,
        }
    }
}

impl<'a, 'b, I> Iterator for BitItor<'a, I>
    where 'b: 'a, I: Iterator<Item=&'b u8> {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        // is this the end of the buffer
        if self.buf.bpeek().is_none() {
            return None;
        }

        // get is_set
        let is_set: bool;
        {
            let byte = self.buf.bpeek().unwrap();
            let mask = 0x80 >> self.bit;
            is_set = *byte & mask > 0;
        }

        // iterate
        self.bit += 1;
        if self.bit > 7 {
            self.bit = 0;
            self.buf.next();
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

        let mut biter = buf.iter();

        let mut bi = BitItor::new(&mut biter);

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
