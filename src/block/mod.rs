mod encoder;
/* mod varint; */

// block is ask sstable

use std::collections::HashMap;
use std::fs::{self, read, File, OpenOptions};
use std::io::{BufReader, ErrorKind, Write};
use std::usize;

use serde::{Deserialize, Serialize};

use crate::btree::BTree;
use crate::chunk::Chunk;
use crate::errors::Error;

// disk file layout
// blocks
//      b_000000001
//      b_000000002
//      b_000000003

pub(crate) struct Blocks {
    data_dir: String,
    metadata: Meta,
    segment: Segment,
    blocks: Vec<Segment>,
}

// BlocksMeta[#TODO] (shoule add some comments )
#[derive(Serialize, Deserialize, Default, Clone, Copy)]
struct Meta {
    current_id: u32,
}

// Blocks[#TODO] (should add some comments)
impl Blocks {
    pub(crate) fn initial(root_dir: &str) -> Blocks {
        let block_dir = format!("{root_dir}/blocks");
        let block_meta = format!("{block_dir}/metadata.json");
        let mut meta = match OpenOptions::new()
            .write(true)
            .read(true)
            .open(block_meta.as_str())
        {
            Ok(file_ptr) => {
                let reader = BufReader::new(file_ptr);
                serde_json::from_reader(reader).unwrap()
            }
            Err(err) => {
                if err.kind() == ErrorKind::NotFound {
                    let _ = fs::create_dir_all(block_dir.clone());
                    let mut meta_file =
                        File::create_new(block_meta.as_str()).expect("crate new database failed");
                    let default_meta = Meta::default();
                    let json = serde_json::to_string(&default_meta).unwrap();
                    meta_file.write_all(json.as_bytes()).unwrap();
                    drop(meta_file);
                    default_meta
                } else {
                    panic!("open database failed");
                }
            }
        };

        meta.current_id += 1;
        Self {
            data_dir: block_dir.clone(),
            metadata: meta,
            segment: Segment::new(format!("{}/block-{}", block_dir, meta.current_id).as_str()),
            blocks: Vec::new(),
        }
    }

    pub(crate) fn write_block(&mut self, chunk: Chunk) {
        let (key, value) = chunk.serialize();
        if self.segment.used_size + key.len() + value.len() > self.segment.max_segment_size {
            self.roate();
        }
        self.segment.insert(key, value);
    }

    fn roate(&mut self) {
        self.metadata.current_id += 1;
        self.segment =
            Segment::new(format!("{}/block-{}", self.data_dir, self.metadata.current_id).as_str());
    }

    pub(crate) fn remove(&self, key: &[u8]) -> Result<Vec<u8>, Error> {
        Err(Error::KeyNotFound)
    }

    pub(crate) fn get(&self, key: &[u8]) -> Result<Vec<u8>, Error> {
        Err(Error::KeyNotFound)
    }
}

// Block[#TODO] (shoule add some comments )
pub(crate) struct Segment {
    btree: BTree,
    used_size: usize,
    max_segment_size: usize,
    id: usize, // id according a filename 按照时间
}

// Segment[#TODO] (should add some comments)
impl Segment {
    fn new(path: &str) -> Self {
        Segment {
            btree: BTree::new(path),
            max_segment_size: 4096 * 10,
            used_size: 0,
            id: 0,
        }
    }

    fn insert(&mut self, key: Vec<u8>, value: Vec<u8>) {
        self.btree.insert(&key, &value);
        self.used_size += key.len() + value.len();
    }
}
