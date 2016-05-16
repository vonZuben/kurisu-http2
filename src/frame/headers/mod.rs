//! +---------------+
//!  |Pad Length? (8)|
//!  +-+-------------+-----------------------------------------------+
//!  |E|                 Stream Dependency? (31)                     |
//!  +-+-------------+-----------------------------------------------+
//!  |  Weight? (8)  |
//!  +-+-------------+-----------------------------------------------+
//!  |                   Header Block Fragment (*)                 ...
//!  +---------------------------------------------------------------+
//!  |                           Padding (*)                       ...
//!  +---------------------------------------------------------------+
//! Figure 7: HEADERS Frame Payload
//!
//! The HEADERS frame payload has the following fields:
//!
//! Pad Length:
//! An 8-bit field containing the length of the frame padding in units of octets. This field is only present if the PADDED flag is set.
//! E:
//! A single-bit flag indicating that the stream dependency is exclusive (see Section 5.3). This field is only present if the PRIORITY flag is set.
//! Stream Dependency:
//! A 31-bit stream identifier for the stream that this stream depends on (see Section 5.3). This field is only present if the PRIORITY flag is set.
//! Weight:
//! An unsigned 8-bit integer representing a priority weight for the stream (see Section 5.3). Add one to the value to obtain a weight between 1 and 256. This field is only present if the PRIORITY flag is set.
//! Header Block Fragment:
//! A header block fragment (Section 4.3).
//! Padding:
//! Padding octets.
//!

use std::mem;
use std::borrow::Cow;

pub mod huffman;

mod integers;

#[derive(Debug)]
pub struct Header<'a>{
    pub pad_l: u8,
    pub exclusive: bool,
    pub stream_dep: u32,
    pub weight: u8,
    pub header_frag: &'a[u8],
}

impl<'a> Header<'a> {
    pub fn new(buf: &'a[u8]) -> Self {
        Header {
            pad_l: buf[0],
            exclusive: buf[1] & 0x80 != 0x0,
            stream_dep: u32::from_le( unsafe { mem::transmute([ buf[4], buf[3], buf[2], buf[1] & 0x7F ]) } ),
            weight: buf[5],
            header_frag: &buf[6..],
        }
    }
}

//struct HeaderList {
//    list:
//}

fn process_header<'a>(header: &'a Header) {

}

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
