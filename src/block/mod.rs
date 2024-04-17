mod encoder;
/* mod varint; */

// block is ask sstable

use std::fs::{self, File, OpenOptions};
use std::io::{ErrorKind, Read, Write};
use std::os::unix::fs::FileExt;
use std::usize;

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
    _blocks: Vec<Segment>,
    metafile: File,
}

// BlocksMeta[#TODO] (shoule add some comments )
#[derive(Debug)]
struct Meta {
    // the max blocked id
    block_id: u64,
}

// Meta[#TODO] (should add some comments)
impl Meta {
    fn new() -> Self {
        Self { block_id: 1 }
    }
}

// Blocks[#TODO] (should add some comments)
impl Blocks {
    pub(crate) fn initial(root_dir: &str) -> Blocks {
        let block_dir = format!("{root_dir}/blocks");
        let block_meta = format!("{block_dir}/metadata.json");
        let mut metadata = Meta::new();
        let meta_file = match OpenOptions::new()
            .write(true)
            .read(true)
            .open(block_meta.as_str())
        {
            Ok(mut file_ptr) => {
                let mut buffer = [0u8; 4096];
                file_ptr.read_exact(&mut buffer).unwrap();
                metadata.deserial(buffer.as_slice());
                file_ptr
            }
            Err(err) => {
                if err.kind() == ErrorKind::NotFound {
                    let _ = fs::create_dir_all(block_dir.clone());
                    let meta_file =
                        File::create_new(block_meta.as_str()).expect("crate new database failed");
                    let mut buffer = [0u8; 4096];
                    metadata.serialize(&mut buffer);
                    meta_file
                        .write_at(&buffer, 0)
                        .expect("Persistent Metadata Initial data failed");
                    meta_file
                } else {
                    panic!("open database failed");
                }
            }
        };

        let segment = Segment::new(format!("{}/block-{}", block_dir, &metadata.block_id).as_str());
        Self {
            data_dir: block_dir.clone(),
            metadata,
            metafile: meta_file,
            segment,
            _blocks: Vec::new(),
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
        self.metadata.block_id += 1;
        self.segment =
            Segment::new(format!("{}/block-{}", self.data_dir, self.metadata.block_id).as_str());
        self.write_metadata();
    }

    pub(crate) fn remove(&self, key: &[u8]) -> Result<Vec<u8>, Error> {
        Err(Error::KeyNotFound)
    }

    pub(crate) fn get(&self, key: &[u8]) -> Result<Vec<u8>, Error> {
        Err(Error::KeyNotFound)
    }

    #[inline]
    fn write_metadata(&self) {
        let mut buffer = [0u8; 4096];
        self.metadata.serialize(&mut buffer);
        self.metafile
            .write_at(&buffer, 0)
            .expect("Persistent Metadata Initial data failed");
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

// Meta[#TODO] (should add some comments)
impl Meta {
    fn serialize(&self, buffer: &mut [u8]) {
        let mut offset = 0;

        buffer[offset..offset + 8].clone_from_slice(self.block_id.to_le_bytes().as_ref());
        offset += 8;
    }

    fn deserial(&mut self, buffer: &[u8]) {
        let mut offset = 0;

        let current_id = u64::from_le_bytes(buffer[offset..offset + 8].try_into().unwrap());
        offset += 8;
    }
}
