//! All Frame Types detailed in the standard HTTP2 specification.
//! These all extend Http2Frame in order to map out the respective
//! frame types.
//! Internet Engineering Task Force (IETF)
//! Request for Comments: 7540

use std::mem;
use std::fmt;
use buf::Buf;
use super::Http2Frame;

use self::flags::*;

// This mod is just used to organize all the flags used by the frames
pub mod flags {
    pub const END_STREAM : u8 = 0x1;
    pub const END_HEADERS : u8 = 0x4;
    pub const PADDED : u8 = 0x8;
    pub const PRIORITY : u8 = 0x20;
}

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

impl_debug_print!( GenericFrame );

macro_rules! impl_into_type {
    ( $typename:ident ) => {
        impl<'a> Into<$typename<'a>> for GenericFrame<'a> {
            fn into(mut self) -> $typename<'a> {
                $typename { buf: mem::replace(&mut self.buf, &mut []) }
            }
        }
    }
}

macro_rules! create_frame_type {
    { $name:ident $code:tt } => {
        impl_buf!( u8 : buf => $name; );
        impl<'obj, 'buf> Http2Frame<'obj, 'buf> for $name<'buf> where 'buf: 'obj {}
        impl_into_type!( $name );
        impl_debug_print!( $name );

        pub struct $name<'buf> {
            buf: &'buf mut [u8],
        }

        impl<'obj, 'buf> $name<'buf> where $name<'buf>: Http2Frame<'obj, 'buf>, 'buf: 'obj
            $code
    }
}
// ==============================================================
// These functions are used to read numbers from the input stream
// ==============================================================

// helper function to get 32bit numbers from the big endian input stream
unsafe fn getu32_from_be(buf: &[u8]) -> u32 {
    use std::ptr;
    debug_assert_eq!(buf.len(), 4);
    let mut num : u32 = mem::uninitialized();
    ptr::copy_nonoverlapping(buf.as_ptr(), &mut num as *mut u32 as *mut u8, 4);
    u32::from_be(num)
}

// helper function to get 16bit numbers from the big endian input stream
unsafe fn getu16_from_be(buf: &[u8]) -> u16 {
    use std::ptr;
    debug_assert_eq!(buf.len(), 2);
    let mut num : u16 = mem::uninitialized();
    ptr::copy_nonoverlapping(buf.as_ptr(), &mut num as *mut u16 as *mut u8, 2);
    u16::from_be(num)
}

// ================================================
// the major header types are defined as follows
// ================================================

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

// helper for HeadersFrame to determine the state of PADDED and PRIORITY flags
enum PadPrioState {
    PaddedOnly,
    PriorityOnly,
    Both,
    Neither,
}

// All the data that can be expected to be in a Header frame
pub struct HeaderData<'obj> {
    pub padding: Option<u8>,
    pub priority_data: Option<(bool, u32, u8)>, // exclusive, stream dep, weight
    pub header_block_fragment: &'obj [u8],
}

create_frame_type!{
    HeadersFrame {

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

    pub fn get_header_data(&'obj self) -> HeaderData<'obj> {
        let buf = &self.payload();

        use self::PadPrioState::*;
        match self.pad_prio_flags() {

            Neither      =>
                HeaderData {
                    padding: None,
                    priority_data: None,
                    header_block_fragment: &buf[0..],
                },

            PaddedOnly   =>
                HeaderData {
                    padding: Some(buf[0]),
                    priority_data: None,
                    header_block_fragment: &buf[1..],
                },

            PriorityOnly => {
                let stream_dep = unsafe { getu32_from_be(&buf[0..4]) };
                let exclusive = stream_dep & 0x80000000 != 0;
                let weight = buf[4];

                HeaderData {
                    padding: None,
                    priority_data: Some((exclusive, stream_dep & 0x7FFFFFFF, weight)),
                    header_block_fragment: &buf[5..],
                }
            },

            Both         => {
                let stream_dep = unsafe { getu32_from_be(&buf[1..5]) };
                let exclusive = stream_dep & 0x80000000 != 0;
                let weight = buf[5];

                HeaderData {
                    padding: Some(buf[0]),
                    priority_data: Some((exclusive, stream_dep & 0x7FFFFFFF, weight)),
                    header_block_fragment: &buf[6..],
                }
            },

        }
    }
} }

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

create_frame_type!{
    DataFrame {

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

} }

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

create_frame_type! {
    PriorityFrame {

    pub fn get_priority_info(&'obj self) -> (bool, u32, u8) {
        let buf = &self.payload()[..];
        let stream_dep = unsafe { getu32_from_be(&buf[0..4]) };
        let exclusive = stream_dep & 0x80000000 != 0;
        let weight = buf[4];
        (exclusive, stream_dep & 0x7FFFFFFF, weight)
    }
} }

/// ===============================
/// RST_STREAM
/// ===============================
/// The RST_STREAM frame (type=0x3) allows for immediate termination of a stream. RST_STREAM is sent to request cancellation of a stream or to indicate that an error condition has occurred.
///
///  +---------------------------------------------------------------+
///  |                        Error Code (32)                        |
///  +---------------------------------------------------------------+
/// Figure 9: RST_STREAM Frame Payload

create_frame_type! {
    RstStreamFrame {

    pub fn get_error_code(&'obj self) -> u32 {
        let buf = &self.payload()[..];
        unsafe { getu32_from_be(&buf[0..4]) }
    }
} }

/// ===============================
/// SETTINGS
/// ===============================
/// The SETTINGS frame (type=0x4) conveys configuration parameters that affect how endpoints communicate, such as preferences and constraints on peer behavior. The SETTINGS frame is also used to acknowledge the receipt of those parameters. Individually, a SETTINGS parameter can also be referred to as a "setting".
///
/// 6.5.1 SETTINGS Format
///
/// The payload of a SETTINGS frame consists of zero or more parameters, each consisting of an unsigned 16-bit setting identifier and an unsigned 32-bit value.
///
///  +-------------------------------+
///  |       Identifier (16)         |
///  +-------------------------------+-------------------------------+
///  |                        Value (32)                             |
///  +---------------------------------------------------------------+
/// Figure 10: Setting Format

pub struct Settings<'obj> {
    s_buf: &'obj [u8],
}

impl<'obj> Iterator for Settings<'obj> {
    type Item = (u16, u32); // id / value

    fn next(&mut self) -> Option<Self::Item> {
        let buf : &[u8] = &self.s_buf;
        if buf.len() == 0 {
            None
        }
        else {
            let id = unsafe { getu16_from_be(&buf[0..2]) };
            let value = unsafe { getu32_from_be(&buf[2..6]) };
            self.s_buf = &buf[6..];
            Some((id, value))
        }
    }
}

create_frame_type! {
    SettingsFrame {

    // return an array filled with the setting parameters from the frame
    pub fn get_settings_paramaters(&'obj self) -> Settings {
        let length = self.get_length();
        debug_assert!(length % 6 == 0); // should probably make this a hard check and return an error
        // actually just note here that a lot more error checking should be done
        Settings { s_buf: &self.payload()[..] }
    }
} }

/// ===============================
/// PUSH_PROMISE
/// ===============================
/// The PUSH_PROMISE frame (type=0x5) is used to notify the peer endpoint in advance of streams the sender intends to initiate. The PUSH_PROMISE frame includes the unsigned 31-bit identifier of the stream the endpoint plans to create along with a set of headers that provide additional context for the stream. Section 8.2 contains a thorough description of the use of PUSH_PROMISE frames.
///
///  +---------------+
///  |Pad Length? (8)|
///  +-+-------------+-----------------------------------------------+
///  |R|                  Promised Stream ID (31)                    |
///  +-+-----------------------------+-------------------------------+
///  |                   Header Block Fragment (*)                 ...
///  +---------------------------------------------------------------+
///  |                           Padding (*)                       ...
///  +---------------------------------------------------------------+
/// Figure 11: PUSH_PROMISE Payload Format

create_frame_type! {
    PushPromiseFrame {

    fn padded(&'obj self) -> bool {
        self.get_flags() & PADDED != 0
    }

    // return the stream id for the push and a ref to the header block fragment
    pub fn get_push_data(&'obj self) -> (u32, &[u8]) {
        let (padding, buf) = match self.padded() {
            true  => {
                (self.payload()[0], &self.payload()[1..])
            },
            false => {
                (0, &self.payload()[0..])
            },
        };
        let id = unsafe { getu32_from_be(&buf[..4]) };
        let end = buf.len() - padding as usize;
        (id & 0x7FFFFFFF, &buf[4..end])
    }
} }

/// ===============================
/// PING
/// ===============================
/// The PING frame (type=0x6) is a mechanism for measuring a minimal round-trip time from the sender, as well as determining whether an idle connection is still functional. PING frames can be sent from any endpoint.
///
///  +---------------------------------------------------------------+
///  |                                                               |
///  |                      Opaque Data (64)                         |
///  |                                                               |
///  +---------------------------------------------------------------+
/// Figure 12: PING Payload Format

create_frame_type! {
    PingFrame {

    // returns reg to that data - equivelent to the payload function but checks for valid size
    pub fn get_ping_data(&'obj self) -> &'obj [u8] {
        let buf = &self.payload();
        debug_assert_eq!(buf.len(), 8);
        buf
    }
} }

/// ===============================
/// GOAWAY
/// ===============================
/// The GOAWAY frame (type=0x7) is used to initiate shutdown of a connection or to signal serious error conditions. GOAWAY allows an endpoint to gracefully stop accepting new streams while still finishing processing of previously established streams. This enables administrative actions, like server maintenance.
///
///  +-+-------------------------------------------------------------+
///  |R|                  Last-Stream-ID (31)                        |
///  +-+-------------------------------------------------------------+
///  |                      Error Code (32)                          |
///  +---------------------------------------------------------------+
///  |                  Additional Debug Data (*)                    |
///  +---------------------------------------------------------------+
/// Figure 13: GOAWAY Payload Format

create_frame_type! {
    GoAwayFrame {

    pub fn get_go_away_info(&'obj self) -> (u32, u32, &'obj [u8]) {
        let buf = &self.payload();
        let last_stread_id = unsafe { getu32_from_be(&buf[0..4]) & 0x7FFFFFFF };
        let error_code = unsafe { getu32_from_be(&buf[4..8]) };
        (last_stread_id, error_code, &buf[8..])
    }
} }

/// ===============================
/// WINDOW_UPDATE
/// ===============================
/// The WINDOW_UPDATE frame (type=0x8) is used to implement flow control; see Section 5.2 for an overview.
///
///  +-+-------------------------------------------------------------+
///  |R|              Window Size Increment (31)                     |
///  +-+-------------------------------------------------------------+
/// Figure 14: WINDOW_UPDATE Payload Format

create_frame_type! {
    WindowUpdateFrame {

    pub fn get_window_update(&'obj self) -> u32 {
        let buf = &self.payload()[..];
        debug_assert_eq!(buf.len(), 4);
        unsafe { getu32_from_be(buf) }
    }
} }

/// ===============================
/// CONTINUATION
/// ===============================
/// The CONTINUATION frame (type=0x9) is used to continue a sequence of header block fragments (Section 4.3). Any number of CONTINUATION frames can be sent, as long as the preceding frame is on the same stream and is a HEADERS, PUSH_PROMISE, or CONTINUATION frame without the END_HEADERS flag set.
///
///  +---------------------------------------------------------------+
///  |                   Header Block Fragment (*)                 ...
///  +---------------------------------------------------------------+
/// Figure 15: CONTINUATION Frame Payload

create_frame_type! {
    ContinuationFrame {

    pub fn get_contuniation(&'obj self) -> &'obj [u8] {
        &self.payload()[..]
    }
} }

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

        let h_data = headers.get_header_data();

        assert_eq!(None, h_data.padding);
        assert_eq!(None, h_data.priority_data);
        assert_eq!(h_data.header_block_fragment[..], bc[9..]);

        //================================
        // PaddedOnly
        //================================
        let mut buf = vec![0x00, 0x00, 0xEE, 0x01, 0x08, 0x00, 0x00, 0x00, 0x01, 0x0F, 0x82, 0x41, 0x8A, 0xA0, 0xE4, 0x1D, 0x13, 0x9D, 0x09, 0xB8, 0xF0, 0x1E, 0x07, 0x87, 0x84, 0x40, 0x85, 0xAE, 0xC1, 0xCD, 0x48, 0xFF, 0x86, 0xA8, 0xEB, 0x10, 0x64, 0x9C, 0xBF, 0x58, 0x86, 0xA8, 0xEB, 0x10, 0x64, 0x9C, 0xBF, 0x40, 0x92, 0xB6, 0xB9, 0xAC, 0x1C, 0x85, 0x58, 0xD5];

        let bc = buf.clone();

        let headers : HeadersFrame = GenericFrame::point_to(&mut buf).into();

        let h_data = headers.get_header_data();

        assert_eq!(Some(15), h_data.padding);
        assert_eq!(None, h_data.priority_data);
        assert_eq!(h_data.header_block_fragment[..], bc[10..]);

        //================================
        // PriorityOnly
        //================================
        let mut buf = vec![0x00, 0x00, 0xEE, 0x01, 0x20, 0x00, 0x00, 0x00, 0x01, 0x80, 0x00, 0x00, 0x1F, 0xFF, 0x82, 0x41, 0x8A, 0xA0, 0xE4, 0x1D, 0x13, 0x9D, 0x09, 0xB8, 0xF0, 0x1E, 0x07, 0x87, 0x84, 0x40, 0x85, 0xAE, 0xC1, 0xCD, 0x48, 0xFF, 0x86, 0xA8, 0xEB, 0x10, 0x64, 0x9C, 0xBF, 0x58, 0x86, 0xA8, 0xEB, 0x10, 0x64, 0x9C, 0xBF, 0x40, 0x92, 0xB6, 0xB9, 0xAC, 0x1C, 0x85, 0x58, 0xD5];

        let bc = buf.clone();

        let headers : HeadersFrame = GenericFrame::point_to(&mut buf).into();

        let h_data = headers.get_header_data();

        assert_eq!(None, h_data.padding);
        assert_eq!(Some((true, 31, 255)), h_data.priority_data);
        assert_eq!(h_data.header_block_fragment[..], bc[14..]);

        //================================
        // Both
        //================================
        let mut buf = vec![0x00, 0x00, 0xEE, 0x01, 0x2D, 0x00, 0x00, 0x00, 0x01, 0x0F, 0x80, 0x00, 0x00, 0x1F, 0xFF, 0x82, 0x41, 0x8A, 0xA0, 0xE4, 0x1D, 0x13, 0x9D, 0x09, 0xB8, 0xF0, 0x1E, 0x07, 0x87, 0x84, 0x40, 0x85, 0xAE, 0xC1, 0xCD, 0x48, 0xFF, 0x86, 0xA8, 0xEB, 0x10, 0x64, 0x9C, 0xBF, 0x58, 0x86, 0xA8, 0xEB, 0x10, 0x64, 0x9C, 0xBF, 0x40, 0x92, 0xB6, 0xB9, 0xAC, 0x1C, 0x85, 0x58, 0xD5];

        let bc = buf.clone();

        let headers : HeadersFrame = GenericFrame::point_to(&mut buf).into();

        let h_data = headers.get_header_data();

        assert_eq!(Some(15), h_data.padding);
        assert_eq!(Some((true, 31, 255)), h_data.priority_data);
        assert_eq!(h_data.header_block_fragment[..], bc[15..]);
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

    #[test]
    fn settings_frame_tests() {
        let mut buf = vec![0x00, 0x00, 0x0C, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x03, 0x00, 0x02, 0x00, 0x00, 0x00, 0x05];

        let sframe : SettingsFrame = GenericFrame::point_to(&mut buf).into();

        let mut params = sframe.get_settings_paramaters();

        assert_eq!(params.next(), Some((1, 3)));
        assert_eq!(params.next(), Some((2, 5)));
        assert_eq!(params.next(), None);
    }

    #[test]
    fn push_promise_frame_tests() {
        let mut buf = vec![0x00, 0x00, 0x0C, 0x05, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x07, 0x00, 0x03, 0x00, 0x02, 0x00, 0x00, 0x00, 0x05];

        let bc = buf.clone();

        let push_frame : PushPromiseFrame = GenericFrame::point_to(&mut buf).into();

        assert_eq!(push_frame.get_push_data(), (7, &bc[13..]));
    }

    #[test]
    fn ping_frame_tests() {
        let mut buf = vec![0x00, 0x00, 0x08, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x00, 0x02, 0x00, 0x00, 0x00, 0x05];

        let bc = buf.clone();

        let ping_frame : PingFrame = GenericFrame::point_to(&mut buf).into();

        assert_eq!(ping_frame.get_ping_data(), &bc[9..]);
    }

    #[test]
    fn go_away_frame_tests() {
        let mut buf = vec![0x00, 0x00, 0x0C, 0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x05, 0x30, 0x33];

        let go_away_frame : GoAwayFrame = GenericFrame::point_to(&mut buf).into();

        assert_eq!(go_away_frame.get_go_away_info(), (2, 5, &b"03"[..]));
    }

    #[test]
    fn window_update_frame_tests() {
        let mut buf = vec![0x00, 0x00, 0x0C, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x90];

        let window_update_frame : WindowUpdateFrame = GenericFrame::point_to(&mut buf).into();

        assert_eq!(window_update_frame.get_window_update(), 400);
    }

    #[test]
    fn continuation_frame_tests() {
        let mut buf = vec![0x00, 0x00, 0x0C, 0x09, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x01, 0x90, 0xFF];

        let bc = buf.clone();

        let continuation : ContinuationFrame = GenericFrame::point_to(&mut buf).into();

        assert_eq!(continuation.get_contuniation(), &bc[9..]);
    }
}
