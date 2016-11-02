use std::collections::VecDeque;
use std::rc::Rc;

use header::HeaderEntry;

static DEFAULT_SIZE: usize = 4096;

// Rc is used to wrap the strings because
// different entries can refer to each other
struct DynTableEntry (Rc<String>, String);

/// the dynamic table used during an HTTP2
/// hpack encryption context
pub struct DynTable {
    table: VecDeque<DynTableEntry>,
    current_size: usize,
    max_size: usize,
}

impl DynTable {

    // the table allocates a VecDeque with an estimated number of entries
    // pre allocated to save reallocations
    //
    // the max_size is the hpack spec size calculated as the sum of octets in
    // the name and value of each entry plus 32
    pub fn with_capacity(size: usize) -> Self {
        DynTable {
            table: VecDeque::with_capacity(size),
            current_size: 0,
            max_size: DEFAULT_SIZE,
        }
    }

    //=========================================
    // adding entries to the dynamic table
    //=========================================
    // must first check that there is room and do eviction if needed
    //
    // add an entry using a name entry that already exists in the table
    pub fn add_entry_id(&mut self, name_id: usize, value: String) {
        let name_rc: Rc<String>;
        { // clone the Rc for the name
            let entry = self.get_entry(name_id);
            name_rc = entry.0.clone();
        }
        let new_entry = DynTableEntry (name_rc, value);
        self.add(new_entry);
    }

    // add a completely new entry
    pub fn add_entry_literal(&mut self, name: String, value: String) {
        let new_entry = DynTableEntry (Rc::new(name), value);
        self.add(new_entry);
    }
    //=========================================

    // Get a header entry with the information
    // at the given index
    //
    // hpack dynamic table index starts at 1
    pub fn get_header_entry<'a>(&'a self, index: usize) -> HeaderEntry<'a> {
        let entry = self.get_entry(index);
        (entry.0.as_str(), entry.1.as_str()).into()
    }

    pub fn max_size_update(&mut self, new_max_size: usize) {
        self.max_size = new_max_size;
        // run evict without intention of adding a new entry
        self.evict(0);
    }

    //=========================================
    // private utility fn
    //=========================================
    fn get_entry(&self, index: usize) -> &DynTableEntry {
        debug_assert!(index < self.table.len(), "index is out of range for dyn_table");
        &self.table[index]
    }

    // evict entries until size can fit into the table
    // call this before adding as a check because
    // eviction only occurs if it is needed
    fn evict(&mut self, size: usize) {
        while self.current_size + size > self.max_size {
            let old_entry = self.table.pop_back();
            match old_entry {
                Some(ref e) => {
                    self.current_size -= Self::size_of_entry(e);
                },
                None => break, // if there are no more entries don't keep trying to make room
            }
        }
    }

    fn add(&mut self, entry: DynTableEntry) {
        let entry_size = Self::size_of_entry(&entry);
        // first make sure there is room
        self.evict(entry_size);

        // still need to check if there is room to add the entry
        // after eviction. If not then leave the table empty
        // as this is the spec's intended behaviour
        if self.current_size + entry_size <= self.max_size {
            self.current_size += entry_size;
            self.table.push_front(entry);
        }
    }

    fn size_of_entry(entry: &DynTableEntry) -> usize {
        entry.0.len() + entry.1.len() + 32
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

        assert_eq!(table.get_header_entry(0), ("name2", "value2").into());
    }
}
