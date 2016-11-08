use super::dyn_table::DynTable;
use super::integers;
use super::huffman::Huffman;
use super::static_table::STATIC_TABLE;

use header::{HeaderList, HeaderEntry};

// private type for representing the result of decoding an entry
// ( number of bytes used, HeaderEntry )
type DecEntry<'a> = Result<(HeaderEntry<'a>, usize), &'static str>;

pub struct Decoder {
    dyn_table: DynTable,
    huffman: Huffman, // this might be temporary, since you should not need to initialize this each time
}

impl Decoder {

    // create a new DynTable with the default capacity
    // the number of entries is just an assumption
    pub fn new(max_size: usize, num_entries: usize) -> Self {
        Decoder { dyn_table: DynTable::new(max_size, num_entries) ,
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
    pub fn get_header_list<'a>(&'a mut self, hpack_block: &[u8]) -> Result<HeaderList<'a>, &'static str> {
        let hpack_block_len = hpack_block.len();
        let mut stride = 0;

        // just assuming 10 entries is enough for now
        let mut header_list = HeaderList::with_capacity(10);

        // loop though all the entries and determine the header representation
        // type in order to decode it properly
        //
        // hpack_block points to the first encoded entry, after each entry is decoded
        // must find out how much of the buffer has been consumed
        while stride < hpack_block_len {
            let entry: (HeaderEntry<'a>, usize); // DecEntry Ok Result
            if      hpack_block[stride] & 0x80 == 0x80 { // Indexed Field Representation
                entry = try!(self.indexed_header(&hpack_block[stride..]));
            }
            else if hpack_block[stride] & 0xC0 == 0x40 { // Literal Field Representation
                entry = try!(self.literal_header(&hpack_block[stride..]));
            }
            else if hpack_block[stride] & 0xF0 == 0x00 { // Without Indexing
                entry = try!(self.literal_header_unindexed(&hpack_block[stride..]));
            }
            else if hpack_block[stride] & 0xF0 == 0x10 { // Never Indexed
                entry = try!(self.literal_header_never_indexed(&hpack_block[stride..]));
            }
            else if hpack_block[stride] & 0xE0 == 0x20 { // Max Size Update
                let consumed = self.size_update(&hpack_block[stride..]);
                stride += consumed;
                continue;
            }
            else {
                return Err("Unrecognized block type");
            }
            header_list.add_entry(entry.0);
            stride += entry.1; // move over the consumed bytes
        }

        Ok(header_list)
    }

    // use the index to get the entry from the correct
    // table : static/dynamic
    // the index given starts at 1 (not 0)
    fn use_index<'a>(&'a self, index: usize) -> Result<HeaderEntry<'a>, &'static str> {
        // get the length of the dynamic table
        // to make sure indexing is in range
        let ne = self.dyn_table.num_entries() + 61;
        // pull result from static or dynamic table
        // or return error
        match index {
            0            => Err("hpack: index of 0 was found"),
            i @ 1 ... 62 => Ok(STATIC_TABLE[i - 1].into()),
            i if i < ne  => Ok(self.dyn_table.get_header_entry(i - 62)),
            _            => Err("hpack: index is out of range"),
        }
    }

    // be carful using this funciton as it is stateful, call it in the correct order
    fn consume_literal(&self, total_consumed: &mut usize, buf: &[u8]) -> Result<String, &'static str>{
        // get value length and huffman status
        let is_huffman = buf[*total_consumed] & 0x80 == 0x80;
        let (length, consumed) = try!(integers::decode_integer(&buf[*total_consumed..], 7));
        *total_consumed += consumed as usize;

        let value = self.huffman.decode(&buf[*total_consumed..length as usize]);
        *total_consumed += length as usize;

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

    fn indexed_header<'a>(&'a self, buf: &[u8]) -> DecEntry<'a> {
        let (index, consumed) = try!(integers::decode_integer(&buf, 7));
        let entry = try!(self.use_index(index as usize));
        Ok((entry, consumed as usize))
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

    fn literal_header<'a>(&'a mut self, buf: &[u8]) -> DecEntry<'a> {
        let mut total_consumed = 0usize;

        let (index, consumed) = try!(integers::decode_integer(&buf, 6));
        total_consumed += consumed as usize;

        if index == 0 { // must get name and value from literal
            let name = try!(self.consume_literal(&mut total_consumed, &buf));
            let value = try!(self.consume_literal(&mut total_consumed, &buf));
            self.dyn_table.add_entry_literal(name, value);
        }
        else { // have name via index
            let value = try!(self.consume_literal(&mut total_consumed, &buf));
            self.dyn_table.add_entry_id(index as usize, value);
        }

        // the entry to return will always be the latest added
        // entry in the dynamic table for this case
        let header_entry: HeaderEntry<'a> = self.dyn_table.get_header_entry(0);
        Ok((header_entry, total_consumed))
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

    fn literal_header_unindexed<'a>(&'a self, buf: &[u8]) -> DecEntry<'a> {
        unimplemented!();
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

    fn literal_header_never_indexed<'a>(&'a self, buf: &[u8]) -> DecEntry<'a> {
        unimplemented!();
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

    fn size_update(&self, buf: &[u8]) -> usize {
        unimplemented!();
    }
}

#[cfg(test)]
mod decoder_tests {

    use super::Decoder;
    //use header::HeaderList;

    #[test]
    fn tmp_decoder_test() {
        let decoder = Decoder::new(100, 10);

        let list = decoder.get_header_list(&[0x82, 0x84]).unwrap();

        for e in list.iter() {
            println!("{:?}", e);
        }

        assert_eq!(list.get_value_by_name(":method"), Some("GET"));
        assert_eq!(list.get_value_by_name(":path"), Some("/"));
    }
}
