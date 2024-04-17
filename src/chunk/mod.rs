use std::result;

use crate::errors::Error;

mod skiplist;

pub(crate) struct Chunk {
    store: skiplist::SkipList,
    total_size: usize,
    used_size: usize,
    key_nums: usize,
    first_key: Vec<u8>,
}

const DEFAULT_MAX_CHUNK_SIZE: usize = 32;

// Default[#TODO] (should add some comments)
impl Default for Chunk {
    fn default() -> Self {
        Chunk {
            store: skiplist::SkipList::default(),
            total_size: DEFAULT_MAX_CHUNK_SIZE,
            used_size: 0,
            key_nums: 0,
            first_key: Vec::default(),
        }
    }
}
// Chunk[#TODO] (should add some comments)
impl Chunk {
    // chunk disk layout
    // |----------------------------------------------------------------------------------|
    // | key_num |  1st off | .| end off |k1_size|k1 | v1_size | v1 | ....................|
    // |----------------------------------------------------------------------------------|
    // |  8B     |  2B      | .|  2B     |  2B   |x  |  2B     | x  |  ......   ....      |
    // |--------------------------------------------------------------|-------------------|
    pub(crate) fn serialize(self) -> (Vec<u8>, Vec<u8>) {
        let mut buffer = vec![0u8; self.used_size * (8 + 2 + 2) + 8];
        let mut offset = 0;
        buffer[offset..offset + 8].clone_from_slice(self.key_nums.to_le_bytes().as_ref());
        offset += 8;

        let mut ptr_pos = offset;
        offset += 2 * self.key_nums;

        let store_iter = self.store.into_iter();
        for (key, value) in store_iter {
            buffer[ptr_pos..ptr_pos + 8].clone_from_slice(offset.to_le_bytes().as_ref());
            ptr_pos += 8;

            buffer[offset..offset + 8].clone_from_slice(key.len().to_le_bytes().as_ref());
            offset += 8;

            buffer[offset..offset + key.len()].clone_from_slice(&key);
            offset += key.len();

            buffer[offset..offset + 8].clone_from_slice(value.len().to_le_bytes().as_ref());
            offset += 8;

            buffer[offset..offset + value.len()].clone_from_slice(&value);
            offset += value.len();
        }

        (self.first_key, buffer)
    }
}

// MemTable[#TODO] (should add some comments)
impl Chunk {
    pub(crate) fn get(&self, key: &[u8]) -> Result<Vec<u8>, Error> {
        self.store.get(key)
    }

    pub(crate) fn delete(&mut self, key: &[u8]) -> Result<Vec<u8>, Error> {
        match self.store.delete(key) {
            Ok(removed) => {
                self.key_nums -= 1;
                self.used_size -= key.len() + removed.len();
                Ok(removed)
            }
            Err(_) => todo!(),
        }
    }
    pub(crate) fn insert(&mut self, key: &[u8], value: &[u8]) -> Result<(), Error> {
        self.used_size += key.len() + value.len();
        self.key_nums += 1;
        self.store.insert(key, value)
    }

    pub(crate) fn is_overflowed(&self, size: usize) -> bool {
        self.used_size + size >= self.total_size
    }
}
