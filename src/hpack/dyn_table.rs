use std::collections::VecDeque;
use std::rc::Rc;

use header::HeaderEntry;

// Rc is used to wrap the strings because
// different entries can refer to each other
struct DynTableEntry (Rc<String>, String);

/// the dynamic table used during an HTTP2
/// hpack encryption context
pub struct DynTable (VecDeque<DynTableEntry>);

impl DynTable {

    pub fn with_capacity(size: usize) -> Self {
        DynTable ( VecDeque::with_capacity(size) )
    }

    // add an entry using a name entry that already exists in the table
    pub fn add_entry_id_name(&mut self, name_id: usize, value: String) {
        let name: Rc<String>;
        { // clone the Rc for the name
            let entry = self.get_entry(name_id);
            name = entry.0.clone();
        }
        self.0.push_front( DynTableEntry (name, value) );
    }

    // add a completely new entry
    pub fn add_entry_literal(&mut self, name: String, value: String) {
        self.0.push_front( DynTableEntry (Rc::new(name), value) );
    }

    // Get a header entry with the information
    // at the given index
    //
    // hpack dynamic table index starts at 1
    pub fn get_header_entry<'a>(&'a self, index: usize) -> HeaderEntry<'a> {
        let entry = self.get_entry(index);
        (entry.0.as_str(), entry.1.as_str()).into()
    }

    // private utility fn
    fn get_entry(&self, index: usize) -> &DynTableEntry {
        debug_assert!(index > 0, "index for dynamic table start at 1");
        debug_assert!(index < self.0.len() + 1, "index is out of range for dyn_table");
        &self.0[index - 1]
    }
}

#[cfg(test)]
mod dyn_table_tests {
    use super::DynTable;
    use header::HeaderEntry;

    #[test]
    fn test_add() {
        let mut table = DynTable::with_capacity(10);

        table.add_entry_literal("name1".to_string(), "value1".to_string());
        table.add_entry_literal("name2".to_string(), "value2".to_string());

        assert_eq!(table.get_header_entry(1), ("name2", "value2").into());
    }
}
