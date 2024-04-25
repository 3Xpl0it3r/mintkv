use crate::bytes::{self, VarintCodec};
use std::usize;

// BlocksMeta[#TODO] (shoule add some comments )
type Key = Vec<u8>;
type Value = Vec<u8>;

// 用medata文件来记录所有block信息,以及每个block 第一个key
// 可以根据查询的key来快速定位存储在哪个block里面
// 每一个block信息是一个B+树, leaf节点存储了chunk
#[derive(Debug)]
pub(super) struct Metadata {
    // the max blocked id
    pub next_block_id: u64,
    // 每个block及其对应的第一个key, 方便二分查询快速定位到某一个具体block去执行查询
    pub indices: Vec<(Key, Value)>,
}

// Meta[#TODO] (should add some comments)
impl Metadata {
    pub(super) fn new() -> Self {
        Self {
            next_block_id: 0,
            indices: Vec::new(),
        }
    }
}
// metadata format
//  默认用一个4096页面来保存metatadata, key + value 最大限制124B + 4B ,
//  一个页面最小可以保存30个block, 每个block是一个B+tree, 最为短期存储已经足够了
// |--------------------------------------------------------------------------------------|
// |  next_id   |blocks_num|  k1_size | k1  | v1_size | v1 | ............DATA ........................|
// |--------------------------------------------------------------------------------------|
// |   8B       |  8B      |   8B     | xB  |  8B     | yB | .........................................|
// |--------------------------------------------------------------------------------------|

// Meta[#TODO] (should add some comments)
impl Metadata {
    pub(super) fn serialize(&self, buffer: &mut [u8]) {
        let mut offset = 0;
        buffer[offset..offset + 8].clone_from_slice(&u64::to_le_bytes(self.next_block_id));
        offset += 8;

        let key_num = self.indices.len();
        buffer[offset..offset + 8].clone_from_slice(&u64::to_le_bytes(key_num as u64));
        offset += 8;

        for (key, value) in self.indices.iter() {
            let key_size = key.len();
            buffer[offset..offset + 8].clone_from_slice(&u64::to_le_bytes(key_size as u64));
            offset += 8;

            buffer[offset..offset + key_size].clone_from_slice(key);
            offset += key_size;

            let value_size = value.len();
            buffer[offset..offset + 8].clone_from_slice(&u64::to_le_bytes(value_size as u64));
            offset += 8;

            buffer[offset..offset + value_size].clone_from_slice(value);
            offset += value_size;
        }
    }

    pub(super) fn deserial(&mut self, buffer: &[u8]) {
        let mut offset = 0;
        self.next_block_id = u64::from_le_bytes(buffer[offset..offset + 8].try_into().unwrap());
        offset += 8;

        let key_num = u64::from_le_bytes(buffer[offset..offset + 8].try_into().unwrap());
        offset += 8;

        for _ in 0..key_num {
            let key_size = u64::from_le_bytes(buffer[offset..offset + 8].try_into().unwrap());
            offset += 8;

            let key: Vec<u8> = buffer[offset..offset + key_size as usize].into();
            offset += key_size as usize;

            let value_size = u64::from_le_bytes(buffer[offset..offset + 8].try_into().unwrap());
            offset += 8;

            let value: Vec<u8> = buffer[offset..offset + value_size as usize].into();
            offset += value_size as usize;
            self.indices.push((key, value));
        }
    }

    pub(super) fn insert(&mut self, key: &[u8], value: &[u8]) {
        let mut insert_index = 0;
        let mut debug = Vec::new();
        for (_, elem) in self.indices.iter().enumerate() {
            debug.push(u64::varint_decode(&elem.0).1);
        }
        let mut found = false;
        for (idx, elem) in self.indices.iter().enumerate() {
            match bytes::compare(&elem.0, key) {
                std::cmp::Ordering::Less => {}
                std::cmp::Ordering::Equal => {
                    found = true;
                    insert_index = idx;
                    break;
                }
                std::cmp::Ordering::Greater => {
                    found = false;
                    insert_index = idx;
                    break;
                }
            }
        }
        if !found && insert_index == 0 {
            self.indices.push((key.into(), value.into()));
        } else {
            self.indices
                .insert(insert_index, (key.into(), value.into()))
        }
    }

    pub(super) fn get(&self, key: &[u8]) -> Option<(Vec<u8>, Vec<u8>)> {
        let mut index = 0;
        for (idx, elem) in self.indices.iter().enumerate() {
            match bytes::compare(&elem.0, key) {
                std::cmp::Ordering::Less => index = idx,
                std::cmp::Ordering::Equal => break,
                std::cmp::Ordering::Greater => break,
            }
        }
        if index == self.indices.len() {
            None
        } else {
            Some(self.indices[index].clone())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let metadata = Metadata::new();
        assert_eq!(metadata.next_block_id, 0);
        assert!(metadata.indices.is_empty());
    }

    #[test]
    fn test_insert() {
        let mut metadata = Metadata::new();
        let key = b"key1".to_vec();
        let value = b"value1".to_vec();

        metadata.insert(&key, &value);

        assert_eq!(metadata.indices.len(), 1);
        assert_eq!(metadata.indices[0].0, key);
        assert_eq!(metadata.indices[0].1, value);
    }

    #[test]
    fn test_serialize_and_deserial() {
        let mut metadata = Metadata::new();
        metadata.next_block_id = 1;

        let key1 = b"key1".to_vec();
        let value1 = b"value1".to_vec();
        let key2 = b"key2".to_vec();
        let value2 = b"value2".to_vec();

        metadata.insert(&key1, &value1);
        metadata.insert(&key2, &value2);

        let mut buffer = vec![0u8; 128]; // 假设我们有足够的空间来序列化metadata
        metadata.serialize(&mut buffer);

        // 创建一个新的Metadata实例来反序列化
        let mut deserialized_metadata = Metadata::new();
        deserialized_metadata.deserial(&buffer);

        // 验证反序列化后的Metadata是否与原始的一致
        assert_eq!(deserialized_metadata.next_block_id, metadata.next_block_id);
        assert_eq!(deserialized_metadata.indices.len(), metadata.indices.len());

        for (idx, (deserialized_key, deserialized_value)) in
            deserialized_metadata.indices.iter().enumerate()
        {
            let (key, value) = &metadata.indices[idx];
            assert_eq!(deserialized_key, key);
            assert_eq!(deserialized_value, value);
        }
    }
}
