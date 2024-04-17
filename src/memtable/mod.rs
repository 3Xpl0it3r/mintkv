mod constant;
mod encoder;

use crate::chunk::Chunk;
use crate::errors::Error;
use std::usize;

pub struct MemTables {
    // in memory ,read && writable
    mutable: *mut Chunk,
    // in memory ,but not modified
    warm_chunks: Vec<Chunk>,
    // in memory, but need to persistend to disk
    cold_chunks: Vec<Chunk>,
    /* _marker: PhantomPinned, */
    warm_num: usize,
}

const DEFAULT_WARM_CHUNKS_NUM: usize = 4;
// Default[#TODO] (should add some comments)
impl Default for MemTables {
    fn default() -> Self {
        let mut memtables = MemTables {
            mutable: std::ptr::null_mut(),
            warm_chunks: vec![Chunk::new()],
            cold_chunks: vec![],
            warm_num: DEFAULT_WARM_CHUNKS_NUM,
        };
        memtables.mutable = &mut memtables.warm_chunks[0];
        memtables
    }
}

// MemTables[#TODO] (should add some comments)
impl MemTables {
    pub fn get(&self, key: &[u8]) -> Result<Vec<u8>, Error> {
        for memtable in self.warm_chunks.iter() {
            if let Ok(result) = memtable.get(key) {
                return Ok(result);
            }
        }

        Err(Error::KeyNotFound)
    }
    pub fn delete(&self, key: &[u8]) -> Result<Vec<u8>, Error> {
        unsafe { (*self.mutable).delete(key) }
    }

    pub fn insert(&mut self, key: &[u8], value: &[u8]) -> Result<(), Error> {
        let size = key.len() + value.len();
        unsafe {
            if (*self.mutable).is_overflowed(size) {
                let _ = self.rotate();
            }
            (*self.mutable).insert(key, value)
        }
    }

    fn rotate(&mut self) -> Result<(), Error> {
        if self.warm_chunks.len() == self.warm_num {
            self.cold_chunks.push(self.warm_chunks.pop().unwrap());
        }
        self.warm_chunks.insert(0, Chunk::new());
        self.mutable = &mut self.warm_chunks[0];
        Ok(())
    }

    pub(crate) fn expired_chunks(&mut self) -> Option<Chunk> {
        self.cold_chunks.pop()
    }
}
