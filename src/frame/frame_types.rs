//! All Frame Types detailed in the standard HTTP2 specification.
//! These all extend Http2Frame in order to map out the respective
//! frame types.

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

macro_rules! impl_debug_print {
    ( $($typename:ident),+ ) => {
        $(
            impl<'a> fmt::Debug for $typename<'a> {
                fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
                    write!(f, "length: {}, type: 0x{:02X}, flags: 0x{:02X}, s_ident: {}, payload {:?}",
                           self.get_length(), self.get_type(), self.get_flags(), self.get_stream_id(), self.payload())
                }
            }
        )*
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
            impl_debug_print!( $typename );
        )*
    }
}

impl_debug_print!( GenericFrame );

impl_buf_frame!( HeadersFrame, DataFrame, PriorityFrame, RstStreamFrame );

// ================================================
// the major header types are defined as follows
// ================================================

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

// helper function to get 32bit numbers from the big endian input stream
unsafe fn getu32_from_be(buf: &[u8]) -> u32 {
    use std::ptr;
    debug_assert_eq!(buf.len(), 4);
    let mut num : u32 = mem::uninitialized();
    ptr::copy(buf.as_ptr(), &mut num as *mut u32 as *mut u8, 4);
    u32::from_be(num)
}

// helper for HeadersFrame to determine the state of PADDED and PRIORITY flags
enum PadPrioState {
    PaddedOnly,
    PriorityOnly,
    Both,
    Neither,
}

/// A Map for buffers that contains frames of type HEADERS
pub struct HeadersFrame<'buf> {
    buf: &'buf mut [u8],
}

impl<'obj, 'buf> HeadersFrame<'buf> where HeadersFrame<'buf>: Http2Frame<'obj, 'buf>, 'buf: 'obj {

    // private utility functions
    // =============================

    // determine the specific combination of PADDED and PRIORITY flags present
    // to determine the memory layout
    fn pad_prio_flags(&'obj self) -> PadPrioState {
        use self::PadPrioState::*;
        const PAD_PRIO : u8 = PADDED | PRIORITY;
        let flags = self.get_flags() & PAD_PRIO;
        match flags {
            0        => Neither,
            PADDED   => PaddedOnly,
            PRIORITY => PriorityOnly,
            PAD_PRIO => Both,
            _        => unreachable!(),
        }
    }

    // immutable functions
    // =============================
    // Each of these functions first determines the memory layout then
    // and then pulls the correct info

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

/// ===============================
/// DATA
/// ===============================
/// DATA frames (type=0x0) convey arbitrary, variable-length sequences of octets associated with a stream. One or more DATA frames are used, for instance, to carry HTTP request or response payloads.
///
///  +---------------+
///  |Pad Length? (8)|
///  +---------------+-----------------------------------------------+
///  |                            Data (*)                         ...
///  +---------------------------------------------------------------+
///  |                           Padding (*)                       ...
///  +---------------------------------------------------------------+
/// Figure 6: DATA Frame Payload
///

pub struct DataFrame<'buf> {
    buf: &'buf mut [u8],
}

impl<'obj, 'buf> DataFrame<'buf> where DataFrame<'buf>: Http2Frame<'obj, 'buf>, 'buf: 'obj {

    fn padded(&'obj self) -> bool {
        self.get_flags() & PADDED != 0
    }

    pub fn get_data(&'obj self) -> &[u8] {
        match self.padded() {
            false => &self.payload()[0..],
            true  => {
                let end = self.payload().len() - self.payload()[0] as usize;
                &self.payload()[1..end]
            }
        }
    }

}

/// ===============================
/// PRIORITY
/// ===============================
/// The PRIORITY frame (type=0x2) specifies the sender-advised priority of a stream (Section 5.3). It can be sent in any stream state, including idle or closed streams.
///
///  +-+-------------------------------------------------------------+
///  |E|                  Stream Dependency (31)                     |
///  +-+-------------+-----------------------------------------------+
///  |   Weight (8)  |
///  +-+-------------+
/// Figure 8: PRIORITY Frame Payload

pub struct PriorityFrame<'buf> {
    buf: &'buf mut [u8],
}

impl<'obj, 'buf> PriorityFrame<'buf> where PriorityFrame<'buf>: Http2Frame<'obj, 'buf>, 'buf: 'obj {

    pub fn get_priority_info(&'obj self) -> (bool, u32, u8) {
        let buf = &self.payload()[..];
        let stream_dep = unsafe { getu32_from_be(&buf[0..4]) };
        let exclusive = stream_dep & 0x80000000 != 0;
        let weight = buf[4];
        (exclusive, stream_dep & 0x7FFFFFFF, weight)
    }
}

/// ===============================
/// RST_STREAM
/// ===============================
/// The RST_STREAM frame (type=0x3) allows for immediate termination of a stream. RST_STREAM is sent to request cancellation of a stream or to indicate that an error condition has occurred.
///
///  +---------------------------------------------------------------+
///  |                        Error Code (32)                        |
///  +---------------------------------------------------------------+
/// Figure 9: RST_STREAM Frame Payload

pub struct RstStreamFrame<'buf> {
    buf: &'buf mut [u8],
}

impl<'obj, 'buf> RstStreamFrame<'buf> where RstStreamFrame<'buf>: Http2Frame<'obj, 'buf>, 'buf: 'obj {

    pub fn get_error_code(&'obj self) -> u32 {
        let buf = &self.payload()[..];
        unsafe { getu32_from_be(&buf[0..4]) }
    }
}

#[cfg(test)]
mod frame_type_tests {

    use super::*;
    use buf::Buf;

    #[test]
    fn read_headers_test() { // TEST different PADDED/PRIORITY flag combinations
        //================================
        // Neither
        //================================
        let mut buf = vec![0x00, 0x00, 0xEE, 0x01, 0x00, 0x00, 0x00, 0x00, 0x01, 0x82, 0x41, 0x8A, 0xA0, 0xE4, 0x1D, 0x13, 0x9D, 0x09, 0xB8, 0xF0, 0x1E, 0x07, 0x87, 0x84, 0x40, 0x85, 0xAE, 0xC1, 0xCD, 0x48, 0xFF, 0x86, 0xA8, 0xEB, 0x10, 0x64, 0x9C, 0xBF, 0x58, 0x86, 0xA8, 0xEB, 0x10, 0x64, 0x9C, 0xBF, 0x40, 0x92, 0xB6, 0xB9, 0xAC, 0x1C, 0x85, 0x58, 0xD5];

        let bc = buf.clone();

        let headers : HeadersFrame = GenericFrame::point_to(&mut buf).into();

        assert_eq!(None, headers.get_pad_length());

        let data = headers.get_priority_info();
        assert_eq!(data, None);

        assert_eq!(headers.get_header_block_fragment()[..], bc[9..]);

        //================================
        // PaddedOnly
        //================================
        let mut buf = vec![0x00, 0x00, 0xEE, 0x01, 0x08, 0x00, 0x00, 0x00, 0x01, 0x0F, 0x82, 0x41, 0x8A, 0xA0, 0xE4, 0x1D, 0x13, 0x9D, 0x09, 0xB8, 0xF0, 0x1E, 0x07, 0x87, 0x84, 0x40, 0x85, 0xAE, 0xC1, 0xCD, 0x48, 0xFF, 0x86, 0xA8, 0xEB, 0x10, 0x64, 0x9C, 0xBF, 0x58, 0x86, 0xA8, 0xEB, 0x10, 0x64, 0x9C, 0xBF, 0x40, 0x92, 0xB6, 0xB9, 0xAC, 0x1C, 0x85, 0x58, 0xD5];

        let bc = buf.clone();

        let headers : HeadersFrame = GenericFrame::point_to(&mut buf).into();

        assert_eq!(Some(15), headers.get_pad_length());

        let data = headers.get_priority_info();
        assert_eq!(data, None);

        assert_eq!(headers.get_header_block_fragment()[..], bc[10..]);

        //================================
        // PriorityOnly
        //================================
        let mut buf = vec![0x00, 0x00, 0xEE, 0x01, 0x20, 0x00, 0x00, 0x00, 0x01, 0x80, 0x00, 0x00, 0x1F, 0xFF, 0x82, 0x41, 0x8A, 0xA0, 0xE4, 0x1D, 0x13, 0x9D, 0x09, 0xB8, 0xF0, 0x1E, 0x07, 0x87, 0x84, 0x40, 0x85, 0xAE, 0xC1, 0xCD, 0x48, 0xFF, 0x86, 0xA8, 0xEB, 0x10, 0x64, 0x9C, 0xBF, 0x58, 0x86, 0xA8, 0xEB, 0x10, 0x64, 0x9C, 0xBF, 0x40, 0x92, 0xB6, 0xB9, 0xAC, 0x1C, 0x85, 0x58, 0xD5];

        let bc = buf.clone();

        let headers : HeadersFrame = GenericFrame::point_to(&mut buf).into();

        assert_eq!(None, headers.get_pad_length());

        let (exclusive, dep, weight) = headers.get_priority_info().unwrap();
        assert_eq!(exclusive, true);
        assert_eq!(dep, 31);
        assert_eq!(weight, 255);

        assert_eq!(headers.get_header_block_fragment()[..], bc[14..]);

        //================================
        // Both
        //================================
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

    #[test]
    fn data_frame_tests() {
        let mut buf = vec![0x00, 0x00, 0x04, 0x00, 0x08, 0x00, 0x00, 0x00, 0x01, 0x01, 0xFF, 0xFF, 0x10];

        let bc = buf.clone();

        let data : DataFrame = GenericFrame::point_to(&mut buf).into();

        assert_eq!(data.get_data()[..], bc[10..12]);
    }

    #[test]
    fn priority_frame_tests() {
        let mut buf = vec![0x00, 0x00, 0x05, 0x02, 0x08, 0x00, 0x00, 0x00, 0x01, 0x80, 0x00, 0x00, 0x01, 0x05];

        let priority : PriorityFrame = GenericFrame::point_to(&mut buf).into();

        assert_eq!(priority.get_priority_info(), (true, 1, 5));
    }

    #[test]
    fn rst_stream_frame_tests() {
        let mut buf = vec![0x00, 0x00, 0x04, 0x03, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x05];

        let priority : RstStreamFrame = GenericFrame::point_to(&mut buf).into();

        assert_eq!(priority.get_error_code(), 5);
    }

}
