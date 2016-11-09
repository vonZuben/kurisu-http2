//! The decoded header list representation for
//! requests and responses. Uses borrowed strings
//! that belong in what ever decoding on encoding
//! context exists for the connection if possible,
//! Otherwise uses owed string. For that reason Cow
//! is used

use std::rc::Rc;
use std::slice::Iter;

/// Header list entry with owed or borrowed string
#[derive(Debug)]
pub struct HeaderEntry {
    name: Rc<String>,
    value: Rc<String>,
}

impl HeaderEntry {
    pub fn new(name: &Rc<String>, value: &Rc<String>) -> Self {
        HeaderEntry { name: name.clone(), value: value.clone() }
    }
}
// turn a tuple into a HeaderEntry from a &str
// to make testing easier
// Formate (name, value)
impl From<(&'static str, &'static str)> for HeaderEntry {

    fn from(obj: (&str, &str)) -> HeaderEntry {
        HeaderEntry { name: Rc::new(obj.0.to_string()), value: Rc::new(obj.1.to_string()) }
    }
}

// this is mostly for easy debug
impl PartialEq for HeaderEntry {
    fn eq(&self, other: &HeaderEntry) -> bool {
        *self.name == *other.name && *self.value == *other.value
    }
}
impl Eq for HeaderEntry {}

impl HeaderEntry {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn value(&self) -> &str {
        &self.value
    }
}

/// Header list to abstract the underlying memory management.
/// Once something is added to the HeaderList,
/// IN CAN NOT be modified
pub struct HeaderList (Vec<HeaderEntry>);

impl HeaderList {
    pub fn with_capacity(cap: usize) -> Self {
        HeaderList ( Vec::with_capacity(cap) )
    }

    pub fn add_entry(&mut self, entry: HeaderEntry) {
        self.0.push(entry);
    }

    // this function is useful for reading the headers that you expect
    // from a request
    pub fn get_value_by_name(&self, _name: &str) -> Option<&str> {
        for entry in &self.0 {
            if *entry.name == _name {
                return Some(&entry.value);
            }
        }
        None
    }

    // this function is useful when turning the HeaderList over into
    // an hpack representation for the response
    // NO ORDER guaranties
    pub fn iter(&self) -> Iter<HeaderEntry> {
        self.0.iter()
    }
}

#[cfg(test)]
mod header_list_tests {

    use std::rc::Rc;
    use super::{HeaderList, HeaderEntry};

    #[test]
    fn test_list_iter() {
        let mut list = HeaderList::with_capacity(10);

        list.add_entry(("host1", "local").into());
        list.add_entry(("host2", "local").into());
        list.add_entry(("host3", "local").into());
        list.add_entry(("host4", "local").into());

        assert_eq!(list.get_value_by_name("host3").unwrap(), "local");

        for entry in list.iter() {
            println!("{:?}", entry);
            assert_eq!(entry.value(), "local");
        }
    }
}
