//! Every connection manages an instance of the hpack encoder/decoder
//! This is so that a dynamic table can be properly managed per connection

mod huffman;
mod integers;
mod table;
pub mod decoder;

static DEFAULT_SIZE: usize = 4096;

