use std::mem;
use std::fmt;
use buf::Buf;
use super::Http2Frame;

/// Type used to read initial data from peer.
/// Used to determine type of frame for further specialization
pub struct GenericFrame<'buf> {
    buf: &'buf mut [u8],
}

impl_buf!( u8 : buf => GenericFrame; );
impl<'obj, 'buf> Http2Frame<'obj, 'buf> for GenericFrame<'buf> where 'buf: 'obj {}

impl<'a> fmt::Debug for GenericFrame<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "length: {}, type: 0x{:02X}, flags: 0x{:02X}, s_ident: {}, payload {:?}",
               self.get_length(), self.get_type(), self.get_flags(), self.get_s_identifier(), self.payload())
    }
}

macro_rules! impl_frame_type {
    ( $typename:ident ) => {
        impl<'a> Into<$typename<'a>> for GenericFrame<'a> {
            fn into(mut self) -> $typename<'a> {
                $typename { buf: mem::replace(&mut self.buf, &mut []) }
            }
        }
    }
}

macro_rules! impl_buf_frame {
    ( $($typename:ident),+ ) => {
        $(
            impl_buf!( u8 : buf => $typename; );
            impl<'obj, 'buf> Http2Frame<'obj, 'buf> for $typename<'buf> where 'buf: 'obj {}
            impl_frame_type!( $typename );
        )*
    }
}

impl_buf_frame!( HeadersFrame );

/// ===============================
/// HEADER FLAGS
/// ===============================
///
/// END_STREAM (0x1):
/// When set, bit 0 indicates that the header block (Section 4.3) is the last that the endpoint will send for the identified stream.
///
/// A HEADERS frame carries the END_STREAM flag that signals the end of a stream. However, a HEADERS frame with the END_STREAM flag set can be followed by CONTINUATION frames on the same stream. Logically, the CONTINUATION frames are part of the HEADERS frame.
///
/// END_HEADERS (0x4):
/// When set, bit 2 indicates that this frame contains an entire header block (Section 4.3) and is not followed by any CONTINUATION frames.
///
/// A HEADERS frame without the END_HEADERS flag set MUST be followed by a CONTINUATION frame for the same stream. A receiver MUST treat the receipt of any other type of frame or a frame on a different stream as a connection error (Section 5.4.1) of type PROTOCOL_ERROR.
///
/// PADDED (0x8):
/// When set, bit 3 indicates that the Pad Length field and any padding that it describes are present.
///
/// PRIORITY (0x20):
/// When set, bit 5 indicates that the Exclusive Flag (E), Stream Dependency, and Weight fields are present; see Section 5.3.

const END_STREAM : u8 = 0x1;
const END_HEADERS : u8 = 0x4;
const PADDED : u8 = 0x8;
const PRIORITY : u8 = 0x20;

/// ===============================
/// HEADERS
/// ===============================
///
/// +---------------+
///  |Pad Length? (8)|
///  +-+-------------+-----------------------------------------------+
///  |E|                 Stream Dependency? (31)                     |
///  +-+-------------+-----------------------------------------------+
///  |  Weight? (8)  |
///  +-+-------------+-----------------------------------------------+
///  |                   Header Block Fragment (*)                 ...
///  +---------------------------------------------------------------+
///  |                           Padding (*)                       ...
///  +---------------------------------------------------------------+
/// Figure 7: HEADERS Frame Payload
///
/// The HEADERS frame payload has the following fields:
///
/// Pad Length:
/// An 8-bit field containing the length of the frame padding in units of octets. This field is only present if the PADDED flag is set.
/// E:
/// A single-bit flag indicating that the stream dependency is exclusive (see Section 5.3). This field is only present if the PRIORITY flag is set.
/// Stream Dependency:
/// A 31-bit stream identifier for the stream that this stream depends on (see Section 5.3). This field is only present if the PRIORITY flag is set.
/// Weight:
/// An unsigned 8-bit integer representing a priority weight for the stream (see Section 5.3). Add one to the value to obtain a weight between 1 and 256. This field is only present if the PRIORITY flag is set.
/// Header Block Fragment:
/// A header block fragment (Section 4.3).
/// Padding:
/// Padding octets.

unsafe fn getu32_from_be(buf: &[u8]) -> u32 {
    use std::ptr;
    debug_assert_eq!(buf.len(), 4);
    let mut num : u32 = 0;
    ptr::copy(buf.as_ptr(), &mut num as *mut u32 as *mut u8, 4);
    u32::from_be(num)
}

enum PadPrioState {
    PaddedOnly,
    PriorityOnly,
    Both,
    Neither,
}

pub struct HeadersFrame<'buf> {
    buf: &'buf mut [u8],
}

impl<'obj, 'buf> HeadersFrame<'buf> where HeadersFrame<'buf>: Http2Frame<'obj, 'buf>, 'buf: 'obj{

    // private utility functions
    // =============================
    fn pad_prio_flags(&'obj self) -> PadPrioState {
        use self::PadPrioState::*;
        const PAD_PRIO : u8 = PADDED | PRIORITY;
        let flags = self.get_flags() & PAD_PRIO;
        match flags {
            0        => Neither,
            PADDED   => PaddedOnly,
            PRIORITY => PriorityOnly,
            PAD_PRIO => Both,
            _        => panic!("impossible pad_prio_state"),
        }
    }

    // immutable functions
    // =============================
    pub fn get_pad_length(&'obj self) -> Option<u8> {
        use self::PadPrioState::*;
        match self.pad_prio_flags() {
            PaddedOnly | Both => Some(self.payload()[0]),
            _                 => None,
        }
    }
    pub fn get_priority_info(&'obj self) -> Option<(bool, u32, u8)> {
        use self::PadPrioState::*;
        let buf = match self.pad_prio_flags() {
            PriorityOnly => &self.payload()[0..5],
            Both         => &self.payload()[1..6],
            _            => return None,
        };
        let stream_dep = unsafe { getu32_from_be(&buf[0..4]) };
        let exclusive = stream_dep & 0x80000000 != 0;
        let weight = buf[4];
        Some((exclusive, stream_dep & 0x7FFFFFFF, weight))
    }
    pub fn get_header_block_fragment(&'obj self) -> &[u8] {
        use self::PadPrioState::*;
        match self.pad_prio_flags() {
            Neither      => &self.payload()[0..],
            PaddedOnly   => &self.payload()[1..],
            PriorityOnly => &self.payload()[5..],
            Both         => &self.payload()[6..],
        }
    }
}

#[cfg(test)]
mod frame_type_tests {

    use super::GenericFrame;
    use super::HeadersFrame;
    use buf::Buf;

    #[test]
    fn tmp_test() {
        let mut buf = vec![0x00, 0x00, 0xEE, 0x01, 0x2D, 0x00, 0x00, 0x00, 0x01, 0x0F, 0x80, 0x00, 0x00, 0x1F, 0xFF, 0x82, 0x41, 0x8A, 0xA0, 0xE4, 0x1D, 0x13, 0x9D, 0x09, 0xB8, 0xF0, 0x1E, 0x07, 0x87, 0x84, 0x40, 0x85, 0xAE, 0xC1, 0xCD, 0x48, 0xFF, 0x86, 0xA8, 0xEB, 0x10, 0x64, 0x9C, 0xBF, 0x58, 0x86, 0xA8, 0xEB, 0x10, 0x64, 0x9C, 0xBF, 0x40, 0x92, 0xB6, 0xB9, 0xAC, 0x1C, 0x85, 0x58, 0xD5];

        let bc = buf.clone();

        let headers : HeadersFrame = GenericFrame::point_to(&mut buf).into();

        assert_eq!(Some(15), headers.get_pad_length());

        let (exclusive, dep, weight) = headers.get_priority_info().unwrap();
        assert_eq!(exclusive, true);
        assert_eq!(dep, 31);
        assert_eq!(weight, 255);

        assert_eq!(headers.get_header_block_fragment()[..], bc[15..]);
    }
}
