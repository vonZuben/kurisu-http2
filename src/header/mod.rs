//! The decoded header list representation for
//! requests and responses. Uses borrowed strings
//! that belong in what ever decoding on encoding
//! context exists for the connection if possible,
//! Otherwise uses owed string. For that reason Cow
//! is used

use std::borrow::{Cow, Borrow};
use std::collections::HashMap;
use std::collections::hash_map::Values;

/// Header list entry with owed or borrowed string
#[derive(Debug)]
pub struct HeaderEntry<'a> {
    name: Cow<'a, str>,
    value: Cow<'a, str>,
}

// turn a tuple into a HeaderEntry from any combination of
// owned and borrowed string data
// Formate (name, value)
impl<'a, T1, T2> From<(T1, T2)> for HeaderEntry<'a>
    where T1: Into<Cow<'a, str>>, T2: Into<Cow<'a, str>> {

    fn from(obj: (T1, T2)) -> HeaderEntry<'a> {
        HeaderEntry { name: obj.0.into(), value: obj.1.into() }
    }
}

impl<'a> HeaderEntry<'a> {
    pub fn name(&self) -> &str {
        self.name.borrow()
    }
    pub fn value(&self) -> &str {
        self.value.borrow()
    }
}

/// Header list to abstract the underlying memory management.
/// Once something is added to the HeaderList,
/// IN CAN NOT be modified
pub struct HeaderList<'a> (HashMap<&'a str, HeaderEntry<'a>>);

impl<'a> HeaderList<'a> {
    pub fn with_capacity(cap: usize) -> Self {
        HeaderList ( HashMap::with_capacity(cap) )
    }

    // the unsafe in this function is done to save space
    // since the HeaderEntry representation is
    // still useful and there is no reason to create
    // another owned key.
    //
    // This is safe as long as the 'entry' in the HashMap is
    // never exposed mutably
    pub fn add_entry(&mut self, entry: HeaderEntry<'a>) {
        let ptr = entry.name.borrow() as *const str;
        unsafe {
            self.0.insert(&*ptr, entry);
        }
    }

    // this function is useful for reading the headers that you expect
    // from a request
    pub fn get_value_by_name(&self, name: &str) -> Option<&str> {
        match self.0.get(name) {
            Some(entry) => Some(entry.value.borrow()),
            None        => None,
        }
    }

    // this function is useful when turning the HeaderList over into
    // an hpack representation for the response
    // NO ORDER guaranties
    pub fn iter(&self) -> Values<&'a str, HeaderEntry<'a>> {
        self.0.values()
    }
}

#[cfg(test)]
mod header_list_tests {

    use super::{HeaderList, HeaderEntry};

    #[test]
    fn test_list() {
        let mut list = HeaderList::with_capacity(10);

        // test with owned and borrowed data
        list.add_entry(("host", "local".to_string()).into());

        assert_eq!(list.get_value_by_name("host").unwrap(), "local");
        //assert_eq!(list.get_entry_by_name("host").unwrap().value, "local");
    }

    #[test]
    fn test_list_iter() {
        let mut list = HeaderList::with_capacity(10);

        list.add_entry(("host1", "local").into());
        list.add_entry(("host2", "local".to_string()).into());
        list.add_entry(("host3".to_string(), "local").into());
        list.add_entry(("host4".to_string(), "local".to_string()).into());

        for entry in list.iter() {
            println!("{:?}", entry);
            assert_eq!(entry.value(), "local");
        }
    }
}
