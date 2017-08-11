use super::table::Table;
use super::integers;
use super::huffman::Huffman;

use std::iter::Peekable;

use borrow_iter::BorrowTake;

use header::*;

pub struct Decoder {
    table: Table,
    huffman: Huffman,
}

impl Decoder {

    // create a new DynTable with the default capacity
    // the number of entries is just an assumption
    pub fn new(max_size: usize, num_entries: usize) -> Self {
        Decoder { table: Table::new(max_size, num_entries),
            huffman: Huffman::new() }
    }

    /// function that takes the hpack block part of the header
    /// and creates a header list from it.
    ///
    /// This must take a complete block and not just a fragment
    /// ie. Until the END_HEADERS flag is passed
    ///
    /// Needs the dynamic table to be managed by the connection
    /// because it is a stateful list used for the entire connection
    pub fn get_header_list(&mut self, hpack_block: &[u8]) -> Result<HeaderList, &'static str> {

        let mut bts = hpack_block.iter().peekable();

        // just assuming 10 entries is enough for now
        let mut header_list = HeaderList::with_capacity(10);

        // loop though all the entries and determine the header representation
        // type in order to decode it properly
        //
        // hpack_block points to the first encoded entry, after each entry is decoded
        // must find out how much of the buffer has been consumed

        while bts.peek().is_some() {
            //let val = *bts.peek().unwrap();
            let entry;

            match *bts.peek().unwrap() {
                val if val & 0x80 == 0x80 => entry = try!(self.indexed_header(&mut bts)),
                val if val & 0xC0 == 0x40 => entry = try!(self.literal_header(&mut bts)),
                val if val & 0xF0 == 0x00 => entry = try!(self.literal_header_unindexed(&mut bts)),
                val if val & 0xF0 == 0x10 => entry = try!(self.literal_header_never_indexed(&mut bts)),
                val if val & 0xE0 == 0x20 =>       { try!(self.size_update(&mut bts)); continue; },
                _ => return Err("Unrecognized block type"),
            }
            header_list.add_entry(entry);
        }

        Ok(header_list)
    }


    // be carful using this funciton as it is stateful, call it in the correct order
    fn consume_literal<'a, I: Iterator<Item=&'a u8>>(&self, bts: &mut Peekable<I>) -> Result<String, &'static str> {
        // get value length and huffman status
        let is_huffman = *bts.peek().unwrap() & 0x80 == 0x80;
        let length = try!(integers::decode_integer(bts, 7)) as usize;

        let value;
        if is_huffman {
            value = self.huffman.decode(bts.borrow_take(length));
        }
        else {
            value = bts.borrow_take(length).map(|x|*x).collect();
        }

        unsafe { Ok(String::from_utf8_unchecked(value)) }
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
    /// An indexed header field starts with the '1' 1-bit pattern, followed by the index of the matching header field,
    /// represented as an integer with a 7-bit prefix (see Section 5.1).
    ///
    /// The index value of 0 is not used. It MUST be treated as a decoding error if found in an indexed header field representation.
    ///

    fn indexed_header<'a, I: Iterator<Item=&'a u8>>(&self, bts: &mut I) -> Result<HeaderEntry, &'static str> {
        let index = try!(integers::decode_integer(bts, 7));
        let entry = try!(self.table.get_header_entry(index as usize));
        Ok(entry)
    }

    /// 6.2 Literal Header Field Representation
    /// A literal header field representation contains a literal header field value.
    /// Header field names are provided either as a literal or by reference to an
    /// existing table entry, either from the static table or the dynamic table (see Section 2.3).
    ///
    /// This specification defines three forms of literal header
    /// field representations: with indexing, without indexing, and never indexed.
    ///
    /// 6.2.1 Literal Header Field with Incremental Indexing
    ///
    /// A literal header field with incremental indexing representation results in
    /// appending a header field to the decoded header list and inserting it as a new entry into the dynamic table.
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
    /// If the header field name matches the header field name of an entry stored in the static
    /// table or the dynamic table, the header field name can be represented using the index of
    /// that entry. In this case, the index of the entry is represented as an integer with a
    /// 6-bit prefix (see Section 5.1). This value is always non-zero.
    ///
    /// Otherwise, the header field name is represented as a string literal (see Section 5.2).
    /// A value 0 is used in place of the 6-bit index, followed by the header field name.
    ///
    /// Either form of header field name representation is followed by the header field value
    /// represented as a string literal (see Section 5.2).
    ///

    fn literal_header<'a, I: Iterator<Item=&'a u8>>(&mut self, bts: &mut Peekable<I>) -> Result<HeaderEntry, &'static str> {

        let index = try!(integers::decode_integer(bts, 6));

        if index == 0 { // must get name and value from literal
            let name = try!(self.consume_literal(bts));
            let value = try!(self.consume_literal(bts));
            self.table.add_entry_literal(name, value);
        }
        else { // have name via index
            let value = try!(self.consume_literal(bts));
            try!(self.table.add_entry_id(index as usize, value));
        }

        // the entry to return will always be the latest added
        // entry in the dynamic table for this case
        let header_entry = self.table.get_dyn_front();
        Ok(header_entry)
    }

    ///
    /// 6.2.2 Literal Header Field without Indexing
    ///
    /// A literal header field without indexing representation results in appending a header
    /// field to the decoded header list without altering the dynamic table.
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
    /// If the header field name matches the header field name of an entry stored in the static table
    /// or the dynamic table, the header field name can be represented using the index of that entry.
    /// In this case, the index of the entry is represented as an integer with a 4-bit prefix
    /// (see Section 5.1). This value is always non-zero.
    ///
    /// Otherwise, the header field name is represented as a string literal (see Section 5.2).
    /// A value 0 is used in place of the 4-bit index, followed by the header field name.
    ///
    /// Either form of header field name representation is followed by the header field value
    /// represented as a string literal (see Section 5.2).

    fn literal_header_unindexed<'a, I: Iterator<Item=&'a u8>>(&self, bts: &mut Peekable<I>) -> Result<HeaderEntry, &'static str> {
        // this function is more useful for intermediaries which
        // this library does not care about at the moment
        // so it will be treated the same as never indexed
        self.literal_header_never_indexed(bts)
    }

    ///
    /// 6.2.3 Literal Header Field Never Indexed
    ///
    /// A literal header field never-indexed representation results in appending a header field to the
    /// decoded header list without altering the dynamic table. Intermediaries MUST use the same
    /// representation for encoding this header field.
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
    /// When a header field is represented as a literal header field never indexed, it MUST always
    /// be encoded with this specific literal representation. In particular, when a peer sends a header
    /// field that it received represented as a literal header field never indexed, it MUST use the same
    /// representation to forward this header field.
    ///
    /// This representation is intended for protecting header field values that are not to be put at
    /// risk by compressing them (see Section 7.1 for more details).
    ///
    /// The encoding of the representation is identical to the literal header field without indexing (see Section 6.2.2).

    fn literal_header_never_indexed<'a, I: Iterator<Item=&'a u8>>(&self, bts: &mut Peekable<I>) -> Result<HeaderEntry, &'static str> {

        let index = try!(integers::decode_integer(bts, 4));

        let header_entry: HeaderEntry;
        if index == 0 { // must get name and value from literal
            let name = try!(self.consume_literal(bts));
            let value = try!(self.consume_literal(bts));
            header_entry = HeaderEntry::new(name, value);
        }
        else { // have name via index
            let name_rc = try!(self.table.get_name_rc(index as usize));
            let value = try!(self.consume_literal(bts));
            header_entry = HeaderEntry::new(name_rc, value);
        }

        Ok(header_entry)
    }

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
    /// A dynamic table size update starts with the '001' 3-bit pattern, followed by the new
    /// maximum size, represented as an integer with a 5-bit prefix (see Section 5.1).
    ///
    /// The new maximum size MUST be lower than or equal to the limit determined by the protocol using HPACK.
    /// A value that exceeds this limit MUST be treated as a decoding error. In HTTP/2, this limit is the last
    /// value of the SETTINGS_HEADER_TABLE_SIZE parameter (see Section 6.5.2 of [HTTP2]) received from the
    /// decoder and acknowledged by the encoder (see Section 6.5.3 of [HTTP2]).
    ///
    /// Reducing the maximum size of the dynamic table can cause entries to be evicted (see Section 4.3).

    fn size_update<'a, I: Iterator<Item=&'a u8>>(&mut self, bts: &mut I) -> Result<(), &'static str> {
        let size = try!(integers::decode_integer(bts, 5));
        self.table.max_size_update(size as usize);
        Ok(())
    }
}

#[cfg(test)]
mod decoder_tests {

    use super::Decoder;

    #[test]
    fn tmp_decoder_test() {
        let mut decoder = Decoder::new(100, 10);

        let list = decoder.get_header_list(&[0x82, 0x84, 0x48, 0x03, 0x35, 0x30, 0x30, 0x0F, 0x00, 0x01, 0x31]).unwrap();

        for e in list.iter() {
            println!("{:?}", e);
        }

        assert_eq!(list.get_value_by_name(":method"), Some("GET"));
        assert_eq!(list.get_value_by_name(":path"), Some("/"));
        assert_eq!(list.get_value_by_name(":status"), Some("500"));
        assert_eq!(list.get_value_by_name("accept-charset"), Some("1"));
    }

    #[test]
    fn comp_decoder_test() {
        let mut decoder = Decoder::new(4096, 10);

        let list = decoder.get_header_list(&[
            0x82, 0x41, 0x8A, 0xA0, 0xE4, 0x1D, 0x13, 0x9D, 0x09, 0xB8, 0xF0, 0x1E, 0x07, 0x87, 0x84, 0x40, 0x92, 0xB6, 0xB9, 0xAC, 0x1C, 0x85, 0x58, 0xD5, 0x20, 0xA4, 0xB6, 0xC2, 0xAD, 0x61, 0x7B, 0x5A, 0x54, 0x25, 0x1F, 0x01, 0x31, 0x7A, 0xD1, 0xD0, 0x7F, 0x66, 0xA2, 0x81, 0xB0, 0xDA, 0xE0, 0x53, 0xFA, 0xFC, 0x08, 0x7E, 0xD4, 0xCE, 0x6A, 0xAD, 0xF2, 0xA7, 0x97, 0x9C, 0x89, 0xC6, 0xBF, 0xB5, 0x21, 0xAE, 0xBA, 0x0B, 0xC8, 0xB1, 0xE6, 0x32, 0x58, 0x6D, 0x97, 0x57, 0x65, 0xC5, 0x3F, 0xAC, 0xD8, 0xF7, 0xE8, 0xCF, 0xF4, 0xA5, 0x06, 0xEA, 0x55, 0x31, 0x14, 0x9D, 0x4F, 0xFD, 0xA9, 0x7A, 0x7B, 0x0F, 0x49, 0x58, 0x6D, 0xF5, 0xC0, 0xBB, 0x20, 0x74, 0x2B, 0x84, 0x0D, 0x29, 0xB8, 0x72, 0x8E, 0xC3, 0x30, 0xDB, 0x2E, 0xAE, 0xCB, 0x9F, 0x53, 0xC0, 0x49, 0x7C, 0xA5, 0x89, 0xD3, 0x4D, 0x1F, 0x43, 0xAE, 0xBA, 0x0C, 0x41, 0xA4, 0xC7, 0xA9, 0x8F, 0x33, 0xA6, 0x9A, 0x3F, 0xDF, 0x9A, 0x68, 0xFA, 0x1D, 0x75, 0xD0, 0x62, 0x0D, 0x26, 0x3D, 0x4C, 0x79, 0xA6, 0x8F, 0xBE, 0xD0, 0x01, 0x77, 0xFE, 0x8D, 0x48, 0xE6, 0x2B, 0x1E, 0x0B, 0x1D, 0x7F, 0x46, 0xA4, 0x73, 0x15, 0x81, 0xD7, 0x54, 0xDF, 0x5F, 0x2C, 0x7C, 0xFD, 0xF6, 0x80, 0x0B, 0xBD, 0x50, 0x8D, 0x9B, 0xD9, 0xAB, 0xFA, 0x52, 0x42, 0xCB, 0x40, 0xD2, 0x5F, 0xA5, 0x23, 0xB3, 0x51, 0x8B, 0x2D, 0x4B, 0x70, 0xDD, 0xF4, 0x5A, 0xBE, 0xFB, 0x40, 0x05, 0xDE
        ]).unwrap();

        for e in list.iter() {
            println!("{:?}", e);
        }

        assert_eq!(list.get_value_by_name(":method"), Some("GET"));
        assert_eq!(list.get_value_by_name(":authority"), Some("localhost:8080"));
        assert_eq!(list.get_value_by_name(":scheme"), Some("https"));
        assert_eq!(list.get_value_by_name(":path"), Some("/"));
        assert_eq!(list.get_value_by_name("upgrade-insecure-requests"), Some("1"));
        assert_eq!(list.get_value_by_name("user-agent"), Some("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/59.0.3071.104 Safari/537.36"));
        assert_eq!(list.get_value_by_name("accept"), Some("text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,image/apng,*/*;q=0.8"));
        assert_eq!(list.get_value_by_name("accept-encoding"), Some("gzip, deflate, br"));
        assert_eq!(list.get_value_by_name("accept-language"), Some("en-US,en;q=0.8"));
    }
}
