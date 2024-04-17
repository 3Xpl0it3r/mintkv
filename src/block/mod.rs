mod encoder;
/* mod varint; */

// block is ask sstable

use std::usize;

use crate::btree::BTree;
use crate::errors::Error;

// disk file layout
// blocks
//      b_000000001
//      b_000000002
//      b_000000003

// Blocks[#TODO] (shoule add some comments )
#[derive(Default)]
pub(crate) struct Blocks {
    blocks: Vec<Segment>,
}

// Blocks[#TODO] (should add some comments)
impl Blocks {
    fn initial() -> Blocks {
        let mut should_init = false;
        if let Ok(curr_dir) = std::env::current_dir() {}

        todo!()
    }

    pub(crate) fn remove(&self, key: &[u8]) -> Result<Vec<u8>, Error> {
        Err(Error::KeyNotFound)
    }

    pub(crate) fn get(&self, key: &[u8]) -> Result<Vec<u8>, Error> {
        Err(Error::KeyNotFound)
    }
}

// Block[#TODO] (shoule add some comments )
struct Segment {
    btree: BTree,
    max_segment_size: usize,
    id: usize, // id according a filename 按照时间
}

// Segment[#TODO] (should add some comments)
impl Segment {}
