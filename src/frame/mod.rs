//! All frames begin with a fixed 9-octet header followed by a variable-length payload.
//!
//!  +-----------------------------------------------+
//!  |                 Length (24)                   |
//!  +---------------+---------------+---------------+
//!  |   Type (8)    |   Flags (8)   |
//!  +-+-------------+---------------+-------------------------------+
//!  |R|                 Stream Identifier (31)                      |
//!  +=+=============================================================+
//!  |                   Frame Payload (0...)                      ...
//!  +---------------------------------------------------------------+
//! Figure 1: Frame Layout
//!
//! The fields of the frame header are defined as:
//!
//! Length:
//! The length of the frame payload expressed as an unsigned 24-bit integer. Values greater than 214 (16,384) MUST NOT be sent unless the receiver has set a larger value for SETTINGS_MAX_FRAME_SIZE.
//!
//! The 9 octets of the frame header are not included in this value.
//!
//! Type:
//! The 8-bit type of the frame. The frame type determines the format and semantics of the frame. Implementations MUST ignore and discard any frame that has a type that is unknown.
//!
//! Flags:
//! An 8-bit field reserved for boolean flags specific to the frame type.
//!
//! Flags are assigned semantics specific to the indicated frame type. Flags that have no defined semantics for a particular frame type MUST be ignored and MUST be left unset (0x0) when sending.
//!
//! R:
//! A reserved 1-bit field. The semantics of this bit are undefined, and the bit MUST remain unset (0x0) when sending and MUST be ignored when receiving.
//!
//! Stream Identifier:
//! A stream identifier (see Section 5.1.1) expressed as an unsigned 31-bit integer. The value 0x0 is reserved for frames that are associated with the connection as a whole as opposed to an individual stream.
//!
//! The structure and content of the frame payload is dependent entirely on the frame type.
//!

use std::mem;

use buf::Buf;

mod ftype;
use self::ftype::Ftype;

pub mod old_header;

pub mod frame_types;

//use self::frame_types::GenericFrame;

// this is an old implementation of an http2 frame (will be deleted later)
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

/// The Basic methods defined for all types of HTTP2 Frames.
/// The types that define more specific Frames all implement this
/// and by extension must implement Buf.
///
/// Provides read and write for fields in the buffer as mapped in
/// in the HTTP2 Frame specification
pub trait Http2Frame<'obj, 'buf> : Buf<'obj, 'buf, u8> {

    // immutable functions for Http2Frame
    // =============================
    fn get_length(&'obj self) -> u32 {
        let buf = self.buf();
        u32::from_be( unsafe { mem::transmute([ 0u8, buf[0], buf[1], buf[2] ]) } )
    }

    fn get_type(&'obj self) -> u8 {
        self.buf()[3]
    }

    fn get_flags(&'obj self) -> u8 {
        self.buf()[4]
    }

    fn get_s_identifier(&'obj self) -> u32 {
        let buf = self.buf();
        u32::from_be( unsafe { mem::transmute([ buf[5] & 0x7F, buf[6], buf[7], buf[8] ]) } )
    }

    fn payload(&'obj self) -> &[u8] {
        &self.buf()[9..]
    }

    // mutable functions for Http2Frame
    // =============================
    fn set_length(&'obj mut self, len: u32) {
        let len_u8 : &[u8; 4] = unsafe { mem::transmute(&len.to_be()) };
        debug_assert_eq!(len_u8[0], 0);
        let buf = self.mut_buf();
        buf[0] = len_u8[1];
        buf[1] = len_u8[2];
        buf[2] = len_u8[3];
    }

    fn set_type(&'obj mut self, f_type: u8) {
        self.mut_buf()[3] = f_type;
    }

    fn set_flags(&'obj mut self, f_flags: u8) {
        self.mut_buf()[4] = f_flags;
    }

    fn set_s_identifier(&'obj mut self, s_identifier: u32) {
        let ident_u8 : &[u8; 4] = unsafe { mem::transmute(&s_identifier.to_be()) };
        debug_assert_eq!(ident_u8[0] & 0x80, 0);
        let buf = self.mut_buf();
        buf[5] = ident_u8[0];
        buf[6] = ident_u8[1];
        buf[7] = ident_u8[2];
        buf[8] = ident_u8[3];
    }

    fn mut_payload(&'obj mut self) -> &mut [u8] {
        &mut self.mut_buf()[9..]
    }
}

/// convenience macro to impl Http2Frame for listed types
/// automatically give Buf trait for u8 type and member name buf
/// all types that use this macro must then have that member
//macro_rules! impl_http2_frame{
//    ( $($typename:ty),+ ) => {
//        $(
//            impl_buf!( u8 : buf => $typename ; );
//            impl Http2Frame for $typename {}
//        )*
//    }
//}

#[cfg(test)]
mod http2_frame_tests {

    use buf::Buf;
    use super::Http2Frame;
    use super::frame_types::GenericFrame;

    // test frame with invalid payload and length
    // (just to check if fields are read and written properly)
    static TST_FRAME : &'static[u8] = &[0x00, 0x00, 0xEE, 0x01, 0x25, 0x00, 0x00, 0x00, 0x01, 0x80];

    #[test]
    fn read_frame_test() {
        let mut buf : Vec<u8> = Vec::with_capacity(TST_FRAME.len());

        for byte in TST_FRAME {
            buf.push(*byte);
        }

        let frame = GenericFrame::point_to(&mut buf);

        assert_eq!(frame.get_length(), 238);
        assert_eq!(frame.get_type(), 1);
        assert_eq!(frame.get_flags(), 0x25);
        assert_eq!(frame.get_s_identifier(), 1);
        assert_eq!(frame.payload()[..], TST_FRAME[9..]);
    }

    #[test]
    fn write_frame_test(){
        let mut buf : Vec<u8> = vec![0;10];

        let mut frame = GenericFrame::point_to(&mut buf);

        frame.set_length(238);
        frame.set_type(1);
        frame.set_flags(0x25);
        frame.set_s_identifier(1);
        frame.mut_payload()[0] = 0x80;

        assert_eq!(frame.buf()[..], TST_FRAME[..]);
    }
}
