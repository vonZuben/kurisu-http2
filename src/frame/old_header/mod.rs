use std::mem;
use std::borrow::Cow;

use frame::Frame;

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
/// MAIN HEADER DEFINITION
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

#[derive(Debug)]
pub struct HeaderFrame<'a>{
    pub pad_l: Option<u8>,
    pub exclusive: Option<bool>,
    pub stream_dep: Option<u32>,
    pub weight: Option<u8>,
    pub header_frag: &'a[u8],
}

impl<'a> HeaderFrame<'a> {
    pub fn new(frame: &Frame<'a>) -> Self {
        let buf = frame.payload;

        const PAD_PRIO : u8 = PADDED | PRIORITY;
        let flags = frame.f_flags & PAD_PRIO;

        // check what flags are set in order to determine which fields are present in the header
        match flags {
            PAD_PRIO => HeaderFrame { // All fields are present
                pad_l: Some(buf[0]),
                exclusive: Some(buf[1] & 0x80 != 0x00),
                stream_dep: Some(u32::from_le( unsafe { mem::transmute([ buf[4], buf[3], buf[2], buf[1] & 0x7F ]) } )),
                weight: Some(buf[5]),
                header_frag: &buf[6..],
            },
            PADDED => HeaderFrame { // Only the Padding field is present
                pad_l: Some(buf[0]),
                exclusive: None,
                stream_dep: None,
                weight: None,
                header_frag: &buf[1..],
            },
            PRIORITY => HeaderFrame { // Only the Priority fields are present
                pad_l: None,
                exclusive: Some(buf[0] & 0x80 != 0x00),
                stream_dep: Some(u32::from_le( unsafe { mem::transmute([ buf[3], buf[2], buf[3], buf[0] & 0x7F ]) } )),
                weight: Some(buf[4]),
                header_frag: &buf[5..],
            },
            _ => HeaderFrame { // neither the Padding or Priority fields are present
                pad_l: None,
                exclusive: None,
                stream_dep: None,
                weight: None,
                header_frag: &buf[0..],
            },
        }
    }
}

fn process_header<'a>(header: &'a HeaderFrame) {

}

/// ===============================
/// HEADER FRAGMENT FORMATS
/// ===============================
///
/// 6.1 Indexed Header Field Representation
/// An indexed header field representation identifies an entry in either the static table or the dynamic table (see Section 2.3).
///
/// An indexed header field representation causes a header field to be added to the decoded header list, as described in Section 3.2.
///
///   0   1   2   3   4   5   6   7
/// +---+---+---+---+---+---+---+---+
/// | 1 |        Index (7+)         |
/// +---+---------------------------+
/// Figure 5: Indexed Header Field
///
/// An indexed header field starts with the '1' 1-bit pattern, followed by the index of the matching header field, represented as an integer with a 7-bit prefix (see Section 5.1).
///
/// The index value of 0 is not used. It MUST be treated as a decoding error if found in an indexed header field representation.
///


/// 6.2 Literal Header Field Representation
/// A literal header field representation contains a literal header field value. Header field names are provided either as a literal or by reference to an existing table entry, either from the static table or the dynamic table (see Section 2.3).
///
/// This specification defines three forms of literal header field representations: with indexing, without indexing, and never indexed.
///
/// 6.2.1 Literal Header Field with Incremental Indexing
///
/// A literal header field with incremental indexing representation results in appending a header field to the decoded header list and inserting it as a new entry into the dynamic table.
///
///   0   1   2   3   4   5   6   7
/// +---+---+---+---+---+---+---+---+
/// | 0 | 1 |      Index (6+)       |
/// +---+---+-----------------------+
/// | H |     Value Length (7+)     |
/// +---+---------------------------+
/// | Value String (Length octets)  |
/// +-------------------------------+
/// Figure 6: Literal Header Field with Incremental Indexing — Indexed Name
///
///   0   1   2   3   4   5   6   7
/// +---+---+---+---+---+---+---+---+
/// | 0 | 1 |           0           |
/// +---+---+-----------------------+
/// | H |     Name Length (7+)      |
/// +---+---------------------------+
/// |  Name String (Length octets)  |
/// +---+---------------------------+
/// | H |     Value Length (7+)     |
/// +---+---------------------------+
/// | Value String (Length octets)  |
/// +-------------------------------+
/// Figure 7: Literal Header Field with Incremental Indexing — New Name
///
/// A literal header field with incremental indexing representation starts with the '01' 2-bit pattern.
///
/// If the header field name matches the header field name of an entry stored in the static table or the dynamic table, the header field name can be represented using the index of that entry. In this case, the index of the entry is represented as an integer with a 6-bit prefix (see Section 5.1). This value is always non-zero.
///
/// Otherwise, the header field name is represented as a string literal (see Section 5.2). A value 0 is used in place of the 6-bit index, followed by the header field name.
///
/// Either form of header field name representation is followed by the header field value represented as a string literal (see Section 5.2).
///
/// 6.2.2 Literal Header Field without Indexing
///
/// A literal header field without indexing representation results in appending a header field to the decoded header list without altering the dynamic table.
///
///   0   1   2   3   4   5   6   7
/// +---+---+---+---+---+---+---+---+
/// | 0 | 0 | 0 | 0 |  Index (4+)   |
/// +---+---+-----------------------+
/// | H |     Value Length (7+)     |
/// +---+---------------------------+
/// | Value String (Length octets)  |
/// +-------------------------------+
/// Figure 8: Literal Header Field without Indexing — Indexed Name
///
///   0   1   2   3   4   5   6   7
/// +---+---+---+---+---+---+---+---+
/// | 0 | 0 | 0 | 0 |       0       |
/// +---+---+-----------------------+
/// | H |     Name Length (7+)      |
/// +---+---------------------------+
/// |  Name String (Length octets)  |
/// +---+---------------------------+
/// | H |     Value Length (7+)     |
/// +---+---------------------------+
/// | Value String (Length octets)  |
/// +-------------------------------+
/// Figure 9: Literal Header Field without Indexing — New Name
///
/// A literal header field without indexing representation starts with the '0000' 4-bit pattern.
///
/// If the header field name matches the header field name of an entry stored in the static table or the dynamic table, the header field name can be represented using the index of that entry. In this case, the index of the entry is represented as an integer with a 4-bit prefix (see Section 5.1). This value is always non-zero.
///
/// Otherwise, the header field name is represented as a string literal (see Section 5.2). A value 0 is used in place of the 4-bit index, followed by the header field name.
///
/// Either form of header field name representation is followed by the header field value represented as a string literal (see Section 5.2).
///
/// 6.2.3 Literal Header Field Never Indexed
///
/// A literal header field never-indexed representation results in appending a header field to the decoded header list without altering the dynamic table. Intermediaries MUST use the same representation for encoding this header field.
///
///   0   1   2   3   4   5   6   7
/// +---+---+---+---+---+---+---+---+
/// | 0 | 0 | 0 | 1 |  Index (4+)   |
/// +---+---+-----------------------+
/// | H |     Value Length (7+)     |
/// +---+---------------------------+
/// | Value String (Length octets)  |
/// +-------------------------------+
/// Figure 10: Literal Header Field Never Indexed — Indexed Name
///
///   0   1   2   3   4   5   6   7
/// +---+---+---+---+---+---+---+---+
/// | 0 | 0 | 0 | 1 |       0       |
/// +---+---+-----------------------+
/// | H |     Name Length (7+)      |
/// +---+---------------------------+
/// |  Name String (Length octets)  |
/// +---+---------------------------+
/// | H |     Value Length (7+)     |
/// +---+---------------------------+
/// | Value String (Length octets)  |
/// +-------------------------------+
/// Figure 11: Literal Header Field Never Indexed — New Name
///
/// A literal header field never-indexed representation starts with the '0001' 4-bit pattern.
///
/// When a header field is represented as a literal header field never indexed, it MUST always be encoded with this specific literal representation. In particular, when a peer sends a header field that it received represented as a literal header field never indexed, it MUST use the same representation to forward this header field.
///
/// This representation is intended for protecting header field values that are not to be put at risk by compressing them (see Section 7.1 for more details).
///
/// The encoding of the representation is identical to the literal header field without indexing (see Section 6.2.2).
///
/// 6.3 Dynamic Table Size Update
/// A dynamic table size update signals a change to the size of the dynamic table.
///
///   0   1   2   3   4   5   6   7
/// +---+---+---+---+---+---+---+---+
/// | 0 | 0 | 1 |   Max size (5+)   |
/// +---+---------------------------+
/// Figure 12: Maximum Dynamic Table Size Change
///
/// A dynamic table size update starts with the '001' 3-bit pattern, followed by the new maximum size, represented as an integer with a 5-bit prefix (see Section 5.1).
///
/// The new maximum size MUST be lower than or equal to the limit determined by the protocol using HPACK. A value that exceeds this limit MUST be treated as a decoding error. In HTTP/2, this limit is the last value of the SETTINGS_HEADER_TABLE_SIZE parameter (see Section 6.5.2 of [HTTP2]) received from the decoder and acknowledged by the encoder (see Section 6.5.3 of [HTTP2]).
///
/// Reducing the maximum size of the dynamic table can cause entries to be evicted (see Section 4.3).

/// Header Definition for static and dynamic tables
struct HeaderEntry<'a> {
    name: Cow<'a, str>,
    value: Cow<'a, str>,
}

static STATIC_TABLE: &'static [HeaderEntry<'static>] = &[
    HeaderEntry { name: Cow::Borrowed(":authority"),                  value: Cow::Borrowed("") },
    HeaderEntry { name: Cow::Borrowed(":method"), 	                  value: Cow::Borrowed("GET") },
    HeaderEntry { name: Cow::Borrowed(":method"), 	                  value: Cow::Borrowed("POST") },
    HeaderEntry { name: Cow::Borrowed(":path"), 	                  value: Cow::Borrowed("/") },
    HeaderEntry { name: Cow::Borrowed(":path"), 	                  value: Cow::Borrowed("/index.html") },
    HeaderEntry { name: Cow::Borrowed(":scheme"), 	                  value: Cow::Borrowed("http") },
    HeaderEntry { name: Cow::Borrowed(":scheme"), 	                  value: Cow::Borrowed("https") },
    HeaderEntry { name: Cow::Borrowed(":status"), 	                  value: Cow::Borrowed("200") },
    HeaderEntry { name: Cow::Borrowed(":status"), 	                  value: Cow::Borrowed("204") },
    HeaderEntry { name: Cow::Borrowed(":status"), 	                  value: Cow::Borrowed("206") },
    HeaderEntry { name: Cow::Borrowed(":status"), 	                  value: Cow::Borrowed("304") },
    HeaderEntry { name: Cow::Borrowed(":status"), 	                  value: Cow::Borrowed("400") },
    HeaderEntry { name: Cow::Borrowed(":status"), 	                  value: Cow::Borrowed("404") },
    HeaderEntry { name: Cow::Borrowed(":status"), 	                  value: Cow::Borrowed("500") },
    HeaderEntry { name: Cow::Borrowed("accept-charset"),              value: Cow::Borrowed("") },
    HeaderEntry { name: Cow::Borrowed("accept-encoding"), 	          value: Cow::Borrowed("gzip, deflate") },
    HeaderEntry { name: Cow::Borrowed("accept-language"),             value: Cow::Borrowed("") },
    HeaderEntry { name: Cow::Borrowed("accept-ranges"),               value: Cow::Borrowed("") },
    HeaderEntry { name: Cow::Borrowed("accept"),                      value: Cow::Borrowed("") },
    HeaderEntry { name: Cow::Borrowed("access-control-allow-origin"), value: Cow::Borrowed("") },
    HeaderEntry { name: Cow::Borrowed("age"),                         value: Cow::Borrowed("") },
    HeaderEntry { name: Cow::Borrowed("allow"),                       value: Cow::Borrowed("") },
    HeaderEntry { name: Cow::Borrowed("authorization"),               value: Cow::Borrowed("") },
    HeaderEntry { name: Cow::Borrowed("cache-control"),               value: Cow::Borrowed("") },
    HeaderEntry { name: Cow::Borrowed("content-disposition"),         value: Cow::Borrowed("") },
    HeaderEntry { name: Cow::Borrowed("content-encoding"),            value: Cow::Borrowed("") },
    HeaderEntry { name: Cow::Borrowed("content-language"),            value: Cow::Borrowed("") },
    HeaderEntry { name: Cow::Borrowed("content-length"),              value: Cow::Borrowed("") },
    HeaderEntry { name: Cow::Borrowed("content-location"),            value: Cow::Borrowed("") },
    HeaderEntry { name: Cow::Borrowed("content-range"),               value: Cow::Borrowed("") },
    HeaderEntry { name: Cow::Borrowed("content-type"),                value: Cow::Borrowed("") },
    HeaderEntry { name: Cow::Borrowed("cookie"),                      value: Cow::Borrowed("") },
    HeaderEntry { name: Cow::Borrowed("date"),                        value: Cow::Borrowed("") },
    HeaderEntry { name: Cow::Borrowed("etag"),                        value: Cow::Borrowed("") },
    HeaderEntry { name: Cow::Borrowed("expect"),                      value: Cow::Borrowed("") },
    HeaderEntry { name: Cow::Borrowed("expires"),                     value: Cow::Borrowed("") },
    HeaderEntry { name: Cow::Borrowed("from"),                        value: Cow::Borrowed("") },
    HeaderEntry { name: Cow::Borrowed("host"),                        value: Cow::Borrowed("") },
    HeaderEntry { name: Cow::Borrowed("if-match"),                    value: Cow::Borrowed("") },
    HeaderEntry { name: Cow::Borrowed("if-modified-since"),           value: Cow::Borrowed("") },
    HeaderEntry { name: Cow::Borrowed("if-none-match"),               value: Cow::Borrowed("") },
    HeaderEntry { name: Cow::Borrowed("if-range"),                    value: Cow::Borrowed("") },
    HeaderEntry { name: Cow::Borrowed("if-unmodified-since"),         value: Cow::Borrowed("") },
    HeaderEntry { name: Cow::Borrowed("last-modified"),               value: Cow::Borrowed("") },
    HeaderEntry { name: Cow::Borrowed("link"),                        value: Cow::Borrowed("") },
    HeaderEntry { name: Cow::Borrowed("location"),                    value: Cow::Borrowed("") },
    HeaderEntry { name: Cow::Borrowed("max-forwards"),                value: Cow::Borrowed("") },
    HeaderEntry { name: Cow::Borrowed("proxy-authenticate"),          value: Cow::Borrowed("") },
    HeaderEntry { name: Cow::Borrowed("proxy-authorization"),         value: Cow::Borrowed("") },
    HeaderEntry { name: Cow::Borrowed("range"),                       value: Cow::Borrowed("") },
    HeaderEntry { name: Cow::Borrowed("referer"),                     value: Cow::Borrowed("") },
    HeaderEntry { name: Cow::Borrowed("refresh"),                     value: Cow::Borrowed("") },
    HeaderEntry { name: Cow::Borrowed("retry-after"),                 value: Cow::Borrowed("") },
    HeaderEntry { name: Cow::Borrowed("server"),                      value: Cow::Borrowed("") },
    HeaderEntry { name: Cow::Borrowed("set-cookie"),                  value: Cow::Borrowed("") },
    HeaderEntry { name: Cow::Borrowed("strict-transport-security"),   value: Cow::Borrowed("") },
    HeaderEntry { name: Cow::Borrowed("transfer-encoding"),           value: Cow::Borrowed("") },
    HeaderEntry { name: Cow::Borrowed("user-agent"),                  value: Cow::Borrowed("") },
    HeaderEntry { name: Cow::Borrowed("vary"),                        value: Cow::Borrowed("") },
    HeaderEntry { name: Cow::Borrowed("via"),                         value: Cow::Borrowed("") },
    HeaderEntry { name: Cow::Borrowed("www-authenticate"),            value: Cow::Borrowed("") },
    ];

#[cfg(test)]
mod header_tests {
    use frame::Frame;
    use super::HeaderFrame;

    #[test]
    fn new_header_unpadded() {
        let tst_frame : &[u8] = &[0x00, 0x00, 0xEE, 0x01, 0x25, 0x00, 0x00, 0x00, 0x01, 0x80, 0x00, 0x00, 0x00, 0xFF, 0x82, 0x41, 0x8A, 0xA0, 0xE4, 0x1D, 0x13, 0x9D, 0x09, 0xB8, 0xF0, 0x1E, 0x07, 0x87, 0x84, 0x40, 0x85, 0xAE, 0xC1, 0xCD, 0x48, 0xFF, 0x86, 0xA8, 0xEB, 0x10, 0x64, 0x9C, 0xBF, 0x58, 0x86, 0xA8, 0xEB, 0x10, 0x64, 0x9C, 0xBF, 0x40, 0x92, 0xB6, 0xB9, 0xAC, 0x1C, 0x85, 0x58, 0xD5, 0x20, 0xA4, 0xB6, 0xC2, 0xAD, 0x61, 0x7B, 0x5A, 0x54, 0x25, 0x1F, 0x01, 0x31, 0x7A, 0xD1, 0xD0, 0x7F, 0x66, 0xA2, 0x81, 0xB0, 0xDA, 0xE0, 0x53, 0xFA, 0xFC, 0x08, 0x7E, 0xD4, 0xCE, 0x6A, 0xAD, 0xF2, 0xA7, 0x97, 0x9C, 0x89, 0xC6, 0xBF, 0xB5, 0x21, 0xAE, 0xBA, 0x0B, 0xC8, 0xB1, 0xE6, 0x32, 0x58, 0x6D, 0x97, 0x57, 0x65, 0xC5, 0x3F, 0xAC, 0xD8, 0xF7, 0xE8, 0xCF, 0xF4, 0xA5, 0x06, 0xEA, 0x55, 0x31, 0x14, 0x9D, 0x4F, 0xFD, 0xA9, 0x7A, 0x7B, 0x0F, 0x49, 0x58, 0x6D, 0x95, 0xC0, 0xB8, 0x9D, 0x79, 0xB5, 0xC2, 0xD3, 0x2A, 0x6E, 0x1C, 0xA3, 0xB0, 0xCC, 0x36, 0xCB, 0xAB, 0xB2, 0xE7, 0x53, 0xB8, 0x49, 0x7C, 0xA5, 0x89, 0xD3, 0x4D, 0x1F, 0x43, 0xAE, 0xBA, 0x0C, 0x41, 0xA4, 0xC7, 0xA9, 0x8F, 0x33, 0xA6, 0x9A, 0x3F, 0xDF, 0x9A, 0x68, 0xFA, 0x1D, 0x75, 0xD0, 0x62, 0x0D, 0x26, 0x3D, 0x4C, 0x79, 0xA6, 0x8F, 0xBE, 0xD0, 0x01, 0x77, 0xFE, 0x8D, 0x48, 0xE6, 0x2B, 0x1E, 0x0B, 0x1D, 0x7F, 0x5F, 0x2C, 0x7C, 0xFD, 0xF6, 0x80, 0x0B, 0xBD, 0x50, 0x92, 0x9B, 0xD9, 0xAB, 0xFA, 0x52, 0x42, 0xCB, 0x40, 0xD2, 0x5F, 0xA5, 0x11, 0x21, 0x27, 0xFA, 0x52, 0x3B, 0x3F, 0x51, 0x8B, 0x2D, 0x4B, 0x70, 0xDD, 0xF4, 0x5A, 0xBE, 0xFB, 0x40, 0x05, 0xDE];

        let frame = Frame::new(&tst_frame);

        let header = HeaderFrame::new(&frame);

        assert_eq!(header.pad_l, None);
        assert_eq!(header.exclusive, Some(true));
        assert_eq!(header.stream_dep, Some(0));
        assert_eq!(header.weight, Some(255));
    }
}
