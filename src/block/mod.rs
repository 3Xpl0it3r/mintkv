mod encoder;
mod meta;
/* mod varint; */

// block is ask sstable

use std::fs::{self, File, OpenOptions};
use std::io::{ErrorKind, Read, Write};
use std::os::unix::fs::FileExt;
use std::{u8, usize};

use crate::btree::BTree;
use crate::bytes;
use crate::chunk::Chunk;
use crate::errors::Error;

// disk file layout
// blocks
//      b_000000001
//      b_000000002
//      b_000000003

pub(crate) struct Blocks {
    data_dir: String,
    metadata: meta::Metadata,
    segment: Option<Segment>,
    metafile: File,
}

// Blocks[#TODO] (should add some comments)
impl Blocks {
    pub(crate) fn open_or_create(root_dir: &str) -> Blocks {
        let block_dir = format!("{root_dir}/blocks");
        let block_meta = format!("{block_dir}/metadata.json");
        let mut metadata = meta::Metadata::new();
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

        Self {
            data_dir: block_dir.clone(),
            metadata,
            metafile: meta_file,
            segment: None,
        }
    }

    pub(crate) fn write_block(&mut self, chunk: Chunk) {
        let (key, value) = chunk.encode();
        if self.segment.is_none()
            || self
                .segment
                .as_ref()
                .unwrap()
                .is_overflow(key.len() + value.len())
        {
            self.rotate(&key);
        }
        self.segment.as_mut().unwrap().insert(key.clone(), value)
    }

    fn rotate(&mut self, key: &[u8]) {
        let new_sg_file = format!("{}/block-{}", self.data_dir, self.metadata.next_block_id);
        self.metadata.next_block_id += 1;
        let new_segment = Some(Segment::new(new_sg_file.as_str()));
        // drop previous segment
        let _ = self.segment.take();
        self.segment = new_segment;
        self.metadata.insert(key, new_sg_file.as_bytes());
        self.write_metadata();
    }

    pub fn remove(&self, key: &[u8]) -> Result<Vec<u8>, Error> {
        Err(Error::KeyNotFound)
    }

    pub fn get(&self, key: &[u8]) -> Result<Vec<u8>, Error> {
        if let Some((_, seg_file)) = self.metadata.get(key) {
            let path = String::from_utf8(seg_file).unwrap();
            let segment = Segment::reader(path.as_str());
            segment.search(key)
        } else {
            Err(Error::KeyNotFound)
        }
    }

    pub fn flush(&mut self) {
        if let Some(ref mut segment) = self.segment {
            segment.flush();
        }
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
    file_name: String,
    btree: BTree,
    used_size: usize,
    max_segment_size: usize,
    id: usize, // id according a filename 按照时间
}

// Segment[#TODO] (should add some comments)
impl Segment {
    fn reader(path: &str) -> Self {
        Segment {
            btree: BTree::reader(path),
            max_segment_size: 4096,
            used_size: 0,
            id: 0,
            file_name: path.into(),
        }
    }
    fn new(path: &str) -> Self {
        Segment {
            btree: BTree::new(path),
            max_segment_size: 4096 * 10,
            used_size: 0,
            id: 0,
            file_name: path.into(),
        }
    }

    fn insert(&mut self, key: Vec<u8>, value: Vec<u8>) {
        self.btree.insert(&key, &value);
        self.used_size += key.len() + value.len();
    }

    fn search(&self, key: &[u8]) -> Result<Vec<u8>, Error> {
        if let Ok(may_found_stable) = self.btree.fuzz_find(key) {

            let chunks = Chunk::decode(&may_found_stable.value).unwrap();

            for item in chunks.into_iter() {
                if bytes::compare(item.0.as_ref(), key) == std::cmp::Ordering::Equal {
                    return Ok(item.1);
                }
            }
            Err(Error::KeyNotFound)
        } else {
            Err(Error::KeyNotFound)
        }
    }

    fn is_overflow(&self, size: usize) -> bool {
        self.used_size + size > self.max_segment_size
    }

    fn flush(&mut self) {
        self.btree.flush();
    }
}
// Drop[#TODO] (should add some comments)
