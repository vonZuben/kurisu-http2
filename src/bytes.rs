use std::io;
use std::io::{Read, Write};

pub struct Bytes<'buf> {
    buf: &'buf [u8],
    pos: usize,
}

impl<'buf> Bytes<'buf> {

    pub fn new(buf: &'buf [u8]) -> Self {
        Self { buf, pos: 0 }
    }
}

pub struct BytesMut<'buf> {
    buf: &'buf mut [u8],
    pos: usize,
}

impl<'buf> BytesMut<'buf> {

    pub fn new(buf: &'buf mut [u8]) -> Self {
        Self { buf, pos: 0 }
    }
}

impl<'buf> Read for Bytes<'buf> {

    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        use std::cmp;
        let max = cmp::min(self.buf.len(), buf.len());
        buf.copy_from_slice(&self.buf[self.pos..max + self.pos]);
        self.pos += max;
        Ok(max)
    }
}

impl<'buf> Write for BytesMut<'buf> {

    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        use std::cmp;
        let max = cmp::min(self.buf.len() - self.pos, buf.len());
        self.buf[self.pos..self.pos + max].copy_from_slice(&buf[..max]);
        self.pos += max;
        Ok(max)
    }

    fn flush(&mut self) -> io::Result<()>{
        Ok(())
    }
}

impl<'buf> Iterator for Bytes<'buf> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buf.len() > self.pos {
            let tmp = self.buf[self.pos];
            self.pos += 1;
            Some(tmp)
        }
        else {
            None
        }
    }
}

impl<'buf> Iterator for BytesMut<'buf> {
    type Item = &'buf mut u8;

    fn next(&mut self) -> Option<&'buf mut u8> {
        if self.buf.len() > self.pos {
            let ret;
            unsafe {
                let tmp = self.buf.get_mut(self.pos).unwrap() as *mut u8;
                ret = tmp.as_mut().unwrap();
            }
            self.pos += 1;
            Some(ret)
        }
        else {
            None
        }
    }
}

impl<'buf> From<&'buf [u8]> for Bytes<'buf> {

    fn from(o: &'buf [u8]) -> Self {
        Bytes::new(o)
    }
}

impl<'buf> From<&'buf mut [u8]> for BytesMut<'buf> {

    fn from(o: &'buf mut [u8]) -> Self {
        BytesMut::new(o)
    }
}

// impl<'buf> std::iter::Peekable for Bytes<'buf> {
//     type Item = u8;

// //    fn peek(&self) ->
// }

#[cfg(test)]
mod bytes_test {

    use super::{Bytes, BytesMut};

     #[test]
    fn bytes_iterate() {
        let mut buf = [0u8, 1,2,3,4,5,6,7,8,9];

        let buf2 = buf.clone();

        let bts = Bytes::new(&mut buf);

        let mut i = 0;
        for b in bts {
            assert_eq!(b, buf2[i]);
            i += 1;
        }
    }

    #[test]
    fn read_test() {

        use std::io::Read;

        let mut buf = [34u8, 1,5,9,42,66,21,68,43,233];

        let mut t1 = Vec::new();
        let mut t2 = Vec::new();

        t1.extend_from_slice(&buf[..5]);
        t2.extend_from_slice(&buf[5..]);

        let mut b1 = Bytes::new(&mut buf);

        let mut read_to1 = [0;5];
        let mut read_to2 = [0;5];

        b1.read(&mut read_to1).unwrap();
        b1.read(&mut read_to2).unwrap();

        assert_eq!(&t1, &read_to1);
        assert_eq!(&t2, &read_to2);
    }

    #[test]
    fn write_test() {
        use std::io::Write;

        let buf = [123u8, 15, 51, 75, 93, 20, 13, 13, 45, 12];

        let mut t1 = Vec::new();
        let mut t2 = Vec::new();
        let mut t3 = Vec::new();

        t1.extend_from_slice(&buf[..5]);
        t2.extend_from_slice(&buf[5..7]);
        t3.extend_from_slice(&buf[7..]);

        let mut write_to = [0;10];

        {
            let mut w1 = BytesMut::new(&mut write_to);

            w1.write(&t1).unwrap();
            w1.write(&t2).unwrap();
            w1.write(&t3).unwrap();
        }

        assert_eq!(&buf, &write_to);
    }
}
