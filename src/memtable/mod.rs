mod constant;
mod encoder;
mod skiplist;

use crate::errors::Error;
use std::usize;
use std::{marker::PhantomPinned, pin::Pin};

pub struct MemTables {
    mutable: *mut Chunk,
    queue: Vec<Chunk>,
    _marker: PhantomPinned,
}
impl Unpin for MemTables {}

// MemTables[#TODO] (should add some comments)
impl MemTables {
    pub fn new() -> Pin<Box<Self>> {
        let memtables = MemTables {
            mutable: std::ptr::null_mut(),
            queue: vec![Chunk::default()],
            _marker: PhantomPinned,
        };
        let mut pind_memtables = Box::pin(memtables);
        let mutable_ptr: *mut Chunk = &mut pind_memtables.as_mut().queue[0];
        unsafe {
            pind_memtables.as_mut().get_unchecked_mut().mutable = mutable_ptr;
        }
        pind_memtables
    }
    pub fn get(&self, key: &[u8]) -> Result<Vec<u8>, Error> {
        for memtable in self.queue.iter() {
            if let Ok(result) = memtable.get(key) {
                return Ok(result);
            }
        }

        Err(Error::KeyNotFound)
    }
    pub fn delete(&self, key: &[u8]) -> Result<(), Error> {
        unsafe { (*self.mutable).delete(key) }
    }

    pub fn add(&mut self, key: &[u8], value: &[u8]) -> Result<(), Error> {
        let size = key.len() + value.len();
        unsafe {
            if (*self.mutable).is_overflowed(size) {
                let _ = self.rotate();
            }
            (*self.mutable).insert(key, value)
        }
    }

    fn rotate(&mut self) -> Result<(), Error> {
        self.queue.insert(0, Chunk::default());
        self.mutable = &mut self.queue[0];
        Ok(())
    }
}

#[derive(Default)]
struct Chunk {
    store: skiplist::SkipList,
    total_size: usize,
    used_size: usize,
}

// MemTable[#TODO] (should add some comments)
impl Chunk {
    fn get(&self, key: &[u8]) -> Result<Vec<u8>, Error> {
        self.store.get(key)
    }

    fn delete(&mut self, key: &[u8]) -> Result<(), Error> {
        self.store.delete(key)
    }
    fn insert(&mut self, key: &[u8], value: &[u8]) -> Result<(), Error> {
        self.store.insert(key, value)
    }

    fn is_overflowed(&self, size: usize) -> bool {
        self.used_size + size >= self.total_size
    }
}
