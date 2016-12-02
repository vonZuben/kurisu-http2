use std::collections::VecDeque;

use header::*;

mod static_table;
use self::static_table::{StaticTable, TableEntry};

/// the dynamic table used during an HTTP2
/// hpack encryption context
pub struct Table {
    dyn_table: VecDeque<TableEntry>,
    static_table: StaticTable,
    current_size: usize,
    max_size: usize,
}

impl Table {

    // the table allocates a VecDeque with an estimated number of entries
    // pre allocated to save reallocations
    //
    // the max_size is the hpack spec size calculated as the sum of octets in
    // the name and value of each entry plus 32
    pub fn new(max_size: usize, num_entries: usize) -> Self {
        Table {
            dyn_table: VecDeque::with_capacity(num_entries),
            static_table: StaticTable::new(),
            current_size: 0,
            max_size: max_size,
        }
    }

    //=========================================
    // adding entries to the dynamic table
    //=========================================
    // must first check that there is room and do eviction if needed
    //
    // add an entry using a name entry that already exists in the table
    pub fn add_entry_id(&mut self, name_id: usize, value: String) -> Result<(), &'static str> {
        let name_rc;
        {
            let entry = try!(self.get_entry(name_id));
            name_rc = entry.0.clone();
        }
        let new_entry = TableEntry::new(name_rc, value);
        self.add(new_entry);
        Ok(())
    }

    // add a completely new entry
    pub fn add_entry_literal(&mut self, name: String, value: String) {
        let new_entry = TableEntry::new(name, value);
        self.add(new_entry);
    }
    //=========================================

    // Get a header entry with the information
    // at the given index
    //
    // This function takes the local index, so the global
    // entry would be 62 but you would pass 0 as the index
    pub fn get_header_entry(&self, index: usize) -> Result<HeaderEntry, &'static str> {
        let entry = try!(self.get_entry(index));
        Ok(entry.clone().into())
    }

    // quicker way to get the latest entry put into the dynamic table
    // useful when adding literals to the table that are going to
    // be used straight away in a header list
    pub fn get_dyn_front(&self) -> HeaderEntry {
        debug_assert!(self.num_dyn_entries() > 0);
        let entry = &self.dyn_table[0];
        entry.clone().into()
    }

    // this is usefull for the functions that construct a header
    // with out modifing the dyn_table
    pub fn get_name_rc(&self, index: usize) -> Result<EntryInner, &'static str> {
        let entry = try!(self.get_entry(index));
        Ok(entry.0.clone())
    }

    pub fn max_size_update(&mut self, new_max_size: usize) {
        self.max_size = new_max_size;
        // run evict without intention of adding a new entry
        self.evict(0);
    }

    pub fn num_dyn_entries(&self) -> usize {
        self.dyn_table.len()
    }

    //=========================================
    // private utility fn
    //=========================================
    // use the index to get the entry from the correct
    // table : static/dynamic
    // the index given starts at 1 (not 0)
    fn get_entry(&self, index: usize) -> Result<&TableEntry, &'static str> {
        // get the length of the dynamic table
        // to make sure indexing is in range
        let ne = self.dyn_table.len() + 62;
        // pull result from static or dynamic table
        // or return error
        match index {
            0            => Err("hpack: index of 0 was found"),
            i @ 1 ... 61 => Ok(&self.static_table[i - 1]),
            i if i < ne  => Ok(&self.dyn_table[i - 62]),
            _            => Err("hpack: index is out of range"),
        }
    }

    // evict entries until size can fit into the table
    // call this before adding as a check because
    // eviction only occurs if it is needed
    fn evict(&mut self, size: usize) {
        while self.current_size + size > self.max_size {
            let old_entry = self.dyn_table.pop_back();
            match old_entry {
                Some(ref e) => {
                    self.current_size -= Self::size_of_entry(e);
                },
                None => break, // if there are no more entries don't keep trying to make room
            }
        }
    }

    fn add(&mut self, entry: TableEntry) {
        let entry_size = Self::size_of_entry(&entry);
        // first make sure there is room
        self.evict(entry_size);

        // still need to check if there is room to add the entry
        // after eviction. If not then leave the table empty
        // as this is the spec's intended behaviour
        if self.current_size + entry_size <= self.max_size {
            self.current_size += entry_size;
            self.dyn_table.push_front(entry);
        }
        // if there is no room even after emptying the
        // entire table, then the add results in an empty table
    }

    // calculate size according to spec
    fn size_of_entry(entry: &TableEntry) -> usize {
        entry.0.len() + entry.1.len() + 32
    }
}

#[allow(unused_variables)]
#[cfg(test)]
mod dyn_table_tests {

    use super::Table;

    #[test]
    fn test_add() {
        let mut table = Table::new(100, 10);

        table.add_entry_literal("name1".to_string(), "value1".to_string());
        table.add_entry_id(1, "value2".to_string()).unwrap();

        assert_eq!(table.num_dyn_entries(), 2);
        assert_eq!(table.get_header_entry(62).unwrap(), (":authority", "value2").into());
        assert_eq!(table.get_header_entry(63).unwrap(), ("name1", "value1").into());
    }

    #[test]
    #[should_panic]
    fn test_evictions() {
        let mut table = Table::new(37, 10); // test add

        table.add_entry_literal("nm".to_string(), "val".to_string());
        assert_eq!(table.get_header_entry(62).unwrap(), ("nm", "val").into());

        table.add_entry_id(62, "ttt".to_string()).unwrap(); // will evict the first entry but Rc should still be valid
        assert_eq!(table.num_dyn_entries(), 1);
        assert_eq!(table.get_header_entry(62).unwrap(), ("nm", "ttt").into());

        table.add_entry_id(62, "XXXX".to_string()).unwrap(); // will evict and not be enough room to add
        assert_eq!(table.num_dyn_entries(), 0);
        let entry = table.get_header_entry(62).unwrap(); // panic here
    }

    #[test]
    #[should_panic]
    fn test_max_size_set() {
        let mut table = Table::new(200, 10);

        table.add_entry_literal("n".to_string(), "v".to_string());
        table.add_entry_id(62, "z".to_string()).unwrap();

        assert_eq!(table.get_header_entry(62).unwrap(), ("n", "z").into());
        assert_eq!(table.get_header_entry(63).unwrap(), ("n", "v").into());

        table.max_size_update(10); // should evict
        assert_eq!(table.num_dyn_entries(), 0);
        let entry = table.get_header_entry(62).unwrap(); // panic here
    }
}
