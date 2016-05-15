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

pub mod headers;

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
