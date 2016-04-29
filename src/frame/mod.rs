use std::mem;

#[derive(Debug)]
pub struct Frame<'a> {
    pub length: u32,
    pub f_type: u8,
    pub f_flags: u8,
    pub s_identifier: u32,
    pub payload: &'a[u8],
}

impl<'a> Frame<'a> {
    pub fn new(buf: &'a[u8]) -> Self {
        Frame {
            length: u32::from_le( unsafe { mem::transmute([ buf[2], buf[1], buf[0], 0u8 ]) } ),
            f_type: buf[3],
            f_flags: buf[4],
            s_identifier: u32::from_le( unsafe { mem::transmute([ buf[8], buf[7], buf[6], buf[5] & 0x7F ]) } ),
            payload: &buf[9..],
        }
    }
}
