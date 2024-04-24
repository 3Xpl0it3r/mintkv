use std::fs::{File, OpenOptions};
use std::io::ErrorKind;
use std::{fs, usize};

use crate::block::Blocks;
use crate::bytes;
use crate::checkpoint::CheckPoint;
use crate::errors::Error;
use crate::memtable::MemTables;
use crate::wal::WalManager;

// Options[#TODO] (shoule add some comments )
pub struct DBOptions {
    // chunk是memtable里面一块数据, chunk持久化到磁盘就是B树里面Leaf节点行一个value
    chunk_size: usize,
    // block是一个B树,一个page定义一个B树节点, page_size定义树节点能存储多少数据
    page_size: usize,
    // 一个block最大可以占多少磁盘
    block_size: usize,
}

// MintKv[#TODO] (shoule add some comments )
pub struct MintKv {
    memtables: MemTables,
    blocks: Blocks,
    wal_mg: WalManager,
    check_point: CheckPoint,
}

// MintKv[#TODO] (should add some comments)
impl MintKv {
    pub fn new(data_dir: &str) -> Self {
        let mut is_initial = true;
        if fs::metadata(data_dir).is_err() {
            fs::create_dir(data_dir).expect("Fatal: Create Data Dir failed");
            fs::create_dir(format!("{data_dir}/wal")).expect("Fatal: Create Data Dir failed");
            fs::create_dir(format!("{data_dir}/blocks")).expect("Fatal: Create Data Dir failed");
        }
        let wal_fp = match OpenOptions::new()
            .write(true)
            .read(true)
            .open(format!("{data_dir}/wal/metadata"))
        {
            Ok(file_ptr) => {
                is_initial = false;
                file_ptr
            }
            Err(err) => {
                if err.kind() == ErrorKind::NotFound {
                    File::create_new(format!("{data_dir}/wal/metadata"))
                        .expect("crate new database failed")
                } else {
                    panic!("open database failed");
                }
            }
        };
        let check_point = CheckPoint::get_or_create(data_dir);

        let mut db = MintKv {
            check_point,
            memtables: MemTables::default(),
            blocks: Blocks::open_or_create(data_dir),
            wal_mg: WalManager::new(data_dir.to_string(), wal_fp, is_initial),
        };

        if !is_initial {
            db.recover_wal();
        }

        db
    }
}

/// Get / Delete / Get
impl MintKv {
    pub fn get(&self, key: u64) -> Result<String, Error> {
        let key = bytes::Varint::encode_u64(key);
        if let Ok(result) = self.memtables.get(&key) {
            return Ok(String::from_utf8(result).unwrap());
        }
        if let Ok(result) = self.blocks.get(&key) {
            return Ok(String::from_utf8(result).unwrap());
        }

        Err(Error::KeyNotFound)
    }

    pub fn insert(&mut self, key: u64, value: &[u8]) -> Result<(), Error> {
        let key = bytes::Varint::encode_u64(key);
        self.wal_mg.record(&key, value);
        self.flush_memtable().unwrap();
        self.memtables.insert(&key, value)
    }

    pub fn delete(&mut self, key: u64) -> Result<String, Error> {
        // first delete from memtables
        let key = bytes::Varint::encode_u64(key);
        if let Ok(result) = self.memtables.delete(&key) {
            return Ok(String::from_utf8(result).unwrap());
        }
        // if key not existed in memtables, then search from sstable ,remove it if existed in
        // sstables;
        if let Ok(result) = self.blocks.remove(&key) {
            return Ok(String::from_utf8(result).unwrap());
        }

        Err(Error::KeyNotFound)
    }

}

// MintKv[#TODO] (should add some comments)
impl MintKv {
    fn flush_memtable(&mut self) -> Result<(), Error> {
        while let Some(chunk) = self.memtables.expired_chunks() {
            let key = bytes::Varint::read_u64(&chunk.last_key);
            self.check_point.record(key.1);
            self.blocks.write_block(chunk);
        }
        Ok(())
    }

    pub fn commit(&mut self) {
        self.blocks.flush();
    }

    fn recover_wal(&mut self) {
        while let Some(kv_item) = self.wal_mg.replay() {
            if kv_item.0.is_empty() {
                continue;
            }
            let key = bytes::Varint::read_u64(&kv_item.0).1;
            if key < self.check_point.last_key {
                continue;
            }
            self.memtables
                .insert(&kv_item.0, &kv_item.1)
                .expect("Replay wal log failed");
        }
    }
}
