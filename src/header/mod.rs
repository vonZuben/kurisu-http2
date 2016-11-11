//! The decoded header list representation for
//! requests and responses. Uses borrowed strings
//! that belong in what ever decoding on encoding
//! context exists for the connection if possible,
//! Otherwise uses owed string. For that reason Cow
//! is used

use std::rc::Rc;
use std::slice::Iter;
use std::ops::Deref;

// internal type to manage entries from the shared
// static table and the connection private dynamic table
#[derive(Debug)]
pub enum EntryInner {
    R(&'static str),
    C(Rc<String>),
}

impl AsRef<str> for EntryInner {
    fn as_ref(&self) -> &str {
        use self::EntryInner::*;
        match self {
            &R(ref v) => v,
            &C(ref v) => v.as_ref(),
        }
    }
}

impl Clone for EntryInner {
    fn clone(&self) -> EntryInner {
        use self::EntryInner::*;
        match self {
            &R(ref v) => R(v),
            &C(ref v) => C(v.clone()),
        }
    }
}

impl Deref for EntryInner {
    type Target = str;

    fn deref(&self) -> &str {
        self.as_ref()
    }
}

impl From<&'static str> for EntryInner {
    fn from(r: &'static str) -> EntryInner {
        EntryInner::R(r)
    }
}

impl From<Rc<String>> for EntryInner {
    fn from(c: Rc<String>) -> EntryInner {
        EntryInner::C(c)
    }
}

/// Header list entry with owed or borrowed string
#[derive(Debug)]
pub struct HeaderEntry {
    name: EntryInner,
    value: EntryInner,
}

impl HeaderEntry {
    pub fn new<A, B>(name: A, value: B) -> Self
        where A: Into<EntryInner>, B: Into<EntryInner> {
        HeaderEntry { name: name.into(), value: value.into() }
    }
}
// turn a tuple into a HeaderEntry from a &str
// to make testing easier
// Formate (name, value)
impl<A, B>  From<(A, B)> for HeaderEntry
    where A: Into<EntryInner>, B: Into<EntryInner> {

    fn from(obj: (A, B)) -> HeaderEntry {
        HeaderEntry { name: obj.0.into(), value: obj.1.into() }
    }
}

// this is mostly for easy debug
impl PartialEq for HeaderEntry {
    fn eq(&self, other: &HeaderEntry) -> bool {
        self.name() == other.name() && self.value() == other.value()
    }
}
impl Eq for HeaderEntry {}

impl HeaderEntry {
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }
    pub fn value(&self) -> &str {
        self.value.as_ref()
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
            if entry.name() == _name {
                return Some(entry.value.as_ref());
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
