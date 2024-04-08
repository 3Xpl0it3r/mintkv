use std::fmt::Debug;
use std::ops::Div;
use std::{i32, usize};

use crate::constant::DEFAULT_PAGE_SIZE;
use crate::error::Error;

const DEFAULT_MAX_KEY_SIZE: usize = 24;
const DEFAULT_MAX_VALUE_SIZE: usize = 24;

#[inline]
fn max_kvs() -> usize {
    DEFAULT_PAGE_SIZE.div(DEFAULT_MAX_KEY_SIZE + DEFAULT_MAX_VALUE_SIZE)
}

#[derive(Clone, Debug)]
pub struct Item {
    pub key: Vec<u8>,
    pub value: Vec<u8>,
}

// Item[#TODO] (should add some comments)
impl Item {
    pub fn new(key: Vec<u8>, value: Vec<u8>) -> Self {
        Item { key, value }
    }
    pub fn size(&self) -> usize {
        self.key.len() + self.value.len()
    }
}

#[derive(Default)]
pub struct Node {
    pub page_number: u64, // cost 8B
    pub items: Vec<Item>,
    pub children: Vec<u64>,
}

// Debug[#TODO] (should add some comments)
impl Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut keys = vec![];
        for item in self.items.iter() {
            keys.push(item.key.clone());
        }
        write!(f, "Keys: {:?}", keys)
    }
}

/// node format
/// |-----------------------------------------------------------------------------------|
/// | node type | key number  |   .............. Data ....                              |
/// |  1B       |  2B         |   ........................                              |
/// |-----------------------------------------------------------------------------------|
/// |                          DATA                                                     |
/// | c0 | len(k0) |  k0 | len(v0)  | v0 |  c1 |  len(k1) | k2 | len(v2) | v2 | ........|
/// | 8B |  1B     | xx  |  1B      | xx |  8B |  1B      | x  |  1B     | xxx| ........|
/// |-----------------------------------------------------------------------------------|
impl Node {
    pub fn display(&self) {
        let mut keys = vec![];
        for item in self.items.iter() {
            keys.push(item.key.clone());
        }
        println!(
            "PN:{} Key:{:?} Value: {:?}",
            self.page_number, self.items, self.children
        );
    }

    pub fn new_empty_node(page_number: u64) -> Self {
        Node {
            page_number,
            items: vec![],
            children: vec![],
        }
    }

    pub fn serialize(&self, buf: &mut [u8]) {
        let node_type = self.is_leaf();
        buf[0] = node_type;

        let kv_cnt = self.items.len() as u16;
        buf[1..3].clone_from_slice(u16::to_le_bytes(kv_cnt).as_ref());

        let mut pos = 3;
        for i in 0..kv_cnt {
            let item = self.items.get(i as usize).unwrap();
            let klen = item.key.len();
            let vlen = item.value.len();
            if node_type == 0 {
                // 0 means this node is internal node  3-11
                buf[pos..pos + 8]
                    .clone_from_slice(u64::to_le_bytes(self.children[i as usize]).as_ref());
                pos += 8;
            }
            // write key size
            buf[pos] = klen as u8; // 19
            pos += 1;

            // write key
            buf[pos..pos + klen].clone_from_slice(&item.key); // 20-23
            pos += klen;
            // write value size
            buf[pos] = vlen as u8;
            pos += 1;
            // write value
            buf[pos..pos + vlen].clone_from_slice(&item.value);
            pos += vlen;
        }
        if node_type == 0 && self.children.len() > self.items.len() {
            buf[pos..pos + 8]
                .clone_from_slice(u64::to_le_bytes(self.children[kv_cnt as usize]).as_ref());
        }
    }

    pub fn deserialize(&mut self, buf: &[u8]) {
        // get node type
        let is_leaf = buf[0] == 0;
        // get the number of key-value pairs
        let kv_size = u16::from_le_bytes(buf[1..3].try_into().unwrap());
        // get child and key-value
        let mut offset = 3;
        for _ in 0..kv_size {
            if is_leaf {
                let child = u64::from_le_bytes(buf[offset..offset + 8].try_into().unwrap());
                offset += 8;
                self.children.push(child);
            }
            // get the length of key
            let key_len = buf[offset] as usize;
            offset += 1;
            let mut key = vec![0; key_len];
            key.clone_from_slice(buf[offset..offset + key_len].try_into().unwrap());
            offset += key_len;

            let value_len = buf[offset] as usize;
            offset += 1;
            let mut value = vec![0; value_len];
            value.clone_from_slice(buf[offset..offset + value_len].try_into().unwrap());
            offset += value_len;

            self.items.push(Item::new(key, value));
        }
        if is_leaf {
            let last_child = u64::from_le_bytes(buf[offset..offset + 8].try_into().unwrap());
            self.children.push(last_child);
        }
    }

    pub fn split(&mut self) -> Result<(Item, Node), Error> {
        let splited_index = self.get_split_index();
        if splited_index == -1 {
            return Err(Error::Generic);
        }
        let splited_index = splited_index as usize;

        let middle_item = self.items[splited_index].clone();
        let mut new_node = Node::default();
        if self.is_leaf() == 1 {
            new_node
                .items
                .extend_from_slice(&self.items[splited_index + 1..]);
            self.items.drain(splited_index..self.items.len());
        } else {
            new_node
                .items
                .extend_from_slice(&self.items[splited_index + 1..]);
            self.items.drain(splited_index..self.items.len());
            new_node
                .children
                .extend_from_slice(&self.children[splited_index + 1..]);
            self.children.drain(splited_index + 1..);
        }
        Ok((middle_item, new_node))
    }

    pub fn can_spare_element(&self) -> bool {
        self.items.len() > 1
    }

    fn get_split_index(&self) -> i32 {
        if self.items.len() > max_kvs() {
            return max_kvs().div(2) as i32;
        }
        -1
    }

    pub fn find_key(&self, key: &str) -> (bool, usize) {
        let key = String::from_utf8(key.into()).unwrap();
        for (idx, elem) in self.items.iter().enumerate() {
            let cur_key = String::from_utf8(elem.key.clone()).unwrap();
            let cmp_result = cur_key.cmp(&key);
            match cmp_result {
                std::cmp::Ordering::Equal => {
                    return (true, idx);
                }
                std::cmp::Ordering::Greater => {
                    return (false, idx);
                }
                std::cmp::Ordering::Less => {}
            }
        }
        (false, self.items.len())
    }

    pub fn is_leaf(&self) -> u8 {
        if self.children.is_empty() {
            1
        } else {
            0
        }
    }

    pub fn is_underflow(&self) -> bool {
        self.items.len() < max_kvs().div(2)
    }

    pub fn is_overflow(&self) -> bool {
        if self.items.len() > max_kvs() {
            return true;
        }
        false
    }
}
