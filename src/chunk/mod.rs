use crate::errors::Error;

mod skiplist;

pub(crate) struct Chunk {
    store: skiplist::SkipList,
    total_size: usize,
    used_size: usize,
    key_nums: usize,
}

const DEFAULT_MAX_CHUNK_SIZE: usize = 1024;

// Chunk[#TODO] (should add some comments)
impl Chunk {
    pub(crate) fn new() -> Self {
        Chunk {
            store: skiplist::SkipList::default(),
            total_size: DEFAULT_MAX_CHUNK_SIZE,
            used_size: 0,
            key_nums: 0,
        }
    }
}

/* // Default[#TODO] (should add some comments)
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
} */
// Chunk[#TODO] (should add some comments)
impl Chunk {
    // chunk disk layout
    // |----------------------------------------------------------------------------------|
    // | key_num |  1st off | .| end off |k1_size|k1 | v1_size | v1 | ....................|
    // |----------------------------------------------------------------------------------|
    // |  8B     |  2B      | .|  2B     |  2B   |x  |  2B     | x  |  ......   ....      |
    // |--------------------------------------------------------------|-------------------|
    pub(crate) fn serialize(self) -> (Vec<u8>, Vec<u8>) {
        let mut buffer = vec![0u8; self.key_nums * (8 + 8 + 8) + 8 + self.used_size];
        let mut offset = 0;
        buffer[offset..offset + 8].clone_from_slice(self.key_nums.to_le_bytes().as_ref());
        offset += 8;

        let mut ptr_pos = offset;
        offset += 8 * self.key_nums;

        let mut fist_key = None;

        let store_iter = self.store.into_iter();
        for (key, value) in store_iter {
            if fist_key.is_none() {
                fist_key = Some(key.clone());
            }
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

        (fist_key.unwrap(), buffer)
    }

    /// for debug
    fn deserialize(buffer: &[u8]) -> Result<Vec<(String, String)>, Error> {
        let mut offset = 0;
        let key_num = u64::from_le_bytes(buffer[offset..offset + 8].try_into().unwrap());
        offset += 8;
        let mut ordered_list = Vec::new();

        offset += 8 * key_num as usize;

        for _ in 0..key_num {
            let key_size = u64::from_le_bytes(buffer[offset..offset + 8].try_into().unwrap());
            offset += 8;

            let key = String::from_utf8(buffer[offset..offset + key_size as usize].into()).unwrap();
            offset += key_size as usize;

            let value_size = u64::from_le_bytes(buffer[offset..offset + 8].try_into().unwrap());
            offset += 8;

            let value =
                String::from_utf8(buffer[offset..offset + value_size as usize].into()).unwrap();
            offset += value_size as usize;
            ordered_list.push((key, value));
        }
        Ok(ordered_list)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_new() {
        let chunk = Chunk::new();

        // Add assertions here to validate the initialization of the chunk
        // For example:
        assert_eq!(chunk.total_size, DEFAULT_MAX_CHUNK_SIZE);
        assert_eq!(chunk.used_size, 0);
        assert_eq!(chunk.key_nums, 0);
    }

    #[test]
    fn test_chunk_serialize() {
        let mut chunk = Chunk::new();
        let key1 = b"key1".to_vec();
        let value1 = b"value1".to_vec();
        let key2 = b"key2".to_vec();
        let value2 = b"value2".to_vec();

        chunk.insert(&key1, &value1).unwrap();
        chunk.insert(&key2, &value2).unwrap();
        let key_nums = chunk.key_nums;
        let used_size = chunk.used_size;

        let (first_key, buffer) = chunk.serialize();

        let expected_buffer_length = key_nums * (8 + 8 + 8) + 8 + used_size;
        // Add assertions here to validate the serialization result
        // For example:
        assert_eq!(first_key, key1);
        assert_eq!(buffer.len(), expected_buffer_length);
    }

    #[test]
    fn test_chunk_deserialize() {
        let mut chunk = Chunk::new();
        let key1 = b"key1".to_vec();
        let value1 = b"value1".to_vec();
        let key2 = b"key2".to_vec();
        let value2 = b"value2".to_vec();
        chunk.insert(&key1, &value1).unwrap();
        chunk.insert(&key2, &value2).unwrap();

        // Add code here to populate the buffer with serialized data

        let (_, buffer) = chunk.serialize();
        let ordered_list = Chunk::deserialize(&buffer).unwrap();

        // Add assertions here to validate the deserialization result
        // For example:

        assert_eq!(ordered_list.len(), 2);
        assert_eq!(ordered_list[0], ("key1".to_string(), "value1".to_string()));
        assert_eq!(ordered_list[1], ("key2".to_string(), "value2".to_string()));
    }

    #[test]
    fn test_chunk_get() {
        let mut chunk = Chunk::new();
        let key1 = b"key1".to_vec();
        let value1 = b"value1".to_vec();
        let key2 = b"key2".to_vec();
        let value2 = b"value2".to_vec();

        chunk.insert(&key1, &value1).unwrap();
        chunk.insert(&key2, &value2).unwrap();

        let retrieved_value1 = chunk.get(&key1).unwrap();
        let retrieved_value2 = chunk.get(&key2).unwrap();

        // Add assertions here to validate the retrieved values
        // For example:
        assert_eq!(retrieved_value1, value1);
        assert_eq!(retrieved_value2, value2);
    }

    #[test]
    fn test_chunk_delete() {
        let mut chunk = Chunk::new();
        let key1 = b"key1".to_vec();
        let value1 = b"value1".to_vec();

        chunk.insert(&key1, &value1).unwrap();

        let removed_value = chunk.delete(&key1).unwrap();

        // Add assertions here to validate the removed value
        // For example:
        assert_eq!(removed_value, value1);
    }

    // Add more unit tests as needed
}
