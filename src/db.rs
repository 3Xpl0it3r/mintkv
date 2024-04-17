use crate::block::Blocks;
use crate::errors::Error;
use crate::memtable::MemTables;

// MintKv[#TODO] (shoule add some comments )
pub struct MintKv {
    _data_dir: String,
    memtables: MemTables,
    blocks: Blocks,
}

// MintKv[#TODO] (should add some comments)
impl MintKv {
    pub fn new(data_dir: &str) -> Self {
        Self {
            _data_dir: data_dir.into(),
            memtables: MemTables::default(),
            blocks: Blocks::initial(data_dir),
        }
    }
}

// MintKv[#TODO] (should add some comments)
impl MintKv {
    pub fn get(&self, key: &str) -> Result<String, Error> {
        if let Ok(result) = self.memtables.get(key.as_bytes()) {
            return Ok(String::from_utf8(result).unwrap());
        }
        if let Ok(result) = self.blocks.get(key.as_bytes()) {
            return Ok(String::from_utf8(result).unwrap());
        }

        Err(Error::KeyNotFound)
    }

    pub fn insert(&mut self, key: &str, value: &str) -> Result<(), Error> {
        self.flush_memtable().unwrap();
        self.memtables.insert(key.as_bytes(), value.as_bytes())
    }

    pub fn delete(&mut self, key: &str) -> Result<String, Error> {
        // first delete from memtables
        if let Ok(result) = self.memtables.delete(key.as_bytes()) {
            return Ok(String::from_utf8(result).unwrap());
        }
        // if key not existed in memtables, then search from sstable ,remove it if existed in
        // sstables;
        if let Ok(result) = self.blocks.remove(key.as_bytes()) {
            return Ok(String::from_utf8(result).unwrap());
        }

        Err(Error::KeyNotFound)
    }

    pub fn get_range(&self, limits: i32) {
        // range query
    }

    fn flush_memtable(&mut self) -> Result<(), Error> {
        while let Some(chunk) = self.memtables.expired_chunks() {
            self.blocks.write_block(chunk);
        }
        Ok(())
    }
}
