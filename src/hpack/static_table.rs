
/// Static table definition for all decoding contexts
pub static STATIC_TABLE: &'static [(&'static str, &'static str)] = &[
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
