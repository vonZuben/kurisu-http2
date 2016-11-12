use std::rc::Rc;
use std::ops::Index;

use header::*;

// Rc is used to wrap the strings because
// different entries can refer to each other
pub struct TableEntry (pub EntryInner, pub EntryInner);

impl TableEntry {
    pub fn new<A, B>(name: A, value: B) -> Self
        where A: Into<EntryInner>, B: Into<EntryInner> {
        TableEntry ( name.into(), value.into() )
    }
}

impl Clone for TableEntry {
    fn clone(&self) -> TableEntry {
        TableEntry ( self.0.clone(), self.1.clone() )
    }
}

impl From<TableEntry> for HeaderEntry {
    fn from(entry: TableEntry) -> HeaderEntry {
        HeaderEntry::new(entry.0, entry.1)
    }
}

// I use this type because it is easier if the HeaderEntry type
// only has to deal with owned strings
pub struct StaticTable (Vec<TableEntry>);

impl StaticTable {
    pub fn new() -> Self {
        let mut vec = Vec::with_capacity(STATIC_TABLE.len());
        for i in STATIC_TABLE {
            vec.push(TableEntry (i.0.into(), i.1.into()));
        }
        StaticTable ( vec )
    }
}

impl Index<usize> for StaticTable {
    type Output = TableEntry;

    fn index<'a>(&'a self, _index: usize) -> &'a TableEntry {
        &self.0[_index]
    }
}

/// Static table definition for all decoding contexts
static STATIC_TABLE: &'static [(&'static str, &'static str)] = &[
    ( ":authority",                   "" ),
    ( ":method", 	                  "GET" ),
    ( ":method", 	                  "POST" ),
    ( ":path", 	                      "/" ),
    ( ":path", 	                      "/index.html" ),
    ( ":scheme", 	                  "http" ),
    ( ":scheme", 	                  "https" ),
    ( ":status", 	                  "200" ),
    ( ":status", 	                  "204" ),
    ( ":status", 	                  "206" ),
    ( ":status", 	                  "304" ),
    ( ":status", 	                  "400" ),
    ( ":status", 	                  "404" ),
    ( ":status", 	                  "500" ),
    ( "accept-charset",               "" ),
    ( "accept-encoding", 	          "gzip, deflate" ),
    ( "accept-language",              "" ),
    ( "accept-ranges",                "" ),
    ( "accept",                       "" ),
    ( "access-control-allow-origin",  "" ),
    ( "age",                          "" ),
    ( "allow",                        "" ),
    ( "authorization",                "" ),
    ( "cache-control",                "" ),
    ( "content-disposition",          "" ),
    ( "content-encoding",             "" ),
    ( "content-language",             "" ),
    ( "content-length",               "" ),
    ( "content-location",             "" ),
    ( "content-range",                "" ),
    ( "content-type",                 "" ),
    ( "cookie",                       "" ),
    ( "date",                         "" ),
    ( "etag",                         "" ),
    ( "expect",                       "" ),
    ( "expires",                      "" ),
    ( "from",                         "" ),
    ( "host",                         "" ),
    ( "if-match",                     "" ),
    ( "if-modified-since",            "" ),
    ( "if-none-match",                "" ),
    ( "if-range",                     "" ),
    ( "if-unmodified-since",          "" ),
    ( "last-modified",                "" ),
    ( "link",                         "" ),
    ( "location",                     "" ),
    ( "max-forwards",                 "" ),
    ( "proxy-authenticate",           "" ),
    ( "proxy-authorization",          "" ),
    ( "range",                        "" ),
    ( "referer",                      "" ),
    ( "refresh",                      "" ),
    ( "retry-after",                  "" ),
    ( "server",                       "" ),
    ( "set-cookie",                   "" ),
    ( "strict-transport-security",    "" ),
    ( "transfer-encoding",            "" ),
    ( "user-agent",                   "" ),
    ( "vary",                         "" ),
    ( "via",                          "" ),
    ( "www-authenticate",             "" ),
    ];

#[cfg(test)]
mod static_table_tests {

    use super::STATIC_TABLE;

    #[test]
    fn valid_static_table() {
        assert_eq!(STATIC_TABLE.len(), 61);
    }
}
