mod meta;

use memmap2::{Mmap, MmapMut};
use std::fs::File;
use std::io::{Seek, SeekFrom, Write};
use std::{fs, usize};

use crate::bytes;

use self::meta::WalMeta;

const DEFAULT_WAL_FILE_NUMBER: usize = 8;
// default wal page size is 10M
const DEFAULT_WAL_PAGE_SIZE: u64 = 1024;

const WAL_END_MAGIC_NUMBER: u64 = 0xABCDABEF;

pub struct WalManager {
    wal: Option<Wal>,
    metadata: WalMeta,
    root_dir: String,
    page_size: u64,
    reader: Option<WalReader>,
}

// WalManager[#TODO] (should add some comments)
impl WalManager {
    pub fn new(root_dir: String, mut file: File, is_initial: bool) -> Self {
        if is_initial {
            file.seek(SeekFrom::Start(DEFAULT_WAL_PAGE_SIZE)).unwrap();
            file.write_all(&[0]).unwrap();
        }
        file.seek(SeekFrom::Start(0)).unwrap();
        let mmap = unsafe { memmap2::MmapMut::map_mut(&file).unwrap() };
        let mut metadata = WalMeta::new(mmap, is_initial);
        if !is_initial {
            metadata.reinitial();
        }

        WalManager {
            metadata,
            wal: None,
            root_dir,
            page_size: DEFAULT_WAL_PAGE_SIZE,
            reader: None,
        }
    }

    pub fn record(&mut self, key: &[u8], val: &[u8]) {
        if self.wal.is_none() || self.wal.as_mut().unwrap().is_overflow(key, val) {
            self.rotate();
        }
        if let Some(ref mut wal) = self.wal {
            wal.write(key, val);
        }
    }

    pub fn rotate(&mut self) {
        self.wal.take();
        /* if let Some(wal) = self.wal.take() {
        }; */
        let id = self.metadata.rotate();
        let wal_fname = format!("{}/wal/wal-{}", self.root_dir, id,);
        let wal = Wal::new_writer(&wal_fname, self.page_size);
        self.wal = Some(wal);
    }

    pub fn replay(&mut self) -> Option<(Vec<u8>, Vec<u8>)> {
        // iteral meta.cache
        if self.reader.is_none() {
            let id = self.metadata.itertor()?;
            let wal_file = format!("{}/wal/wal-{}", self.root_dir, id);
            self.reader = Some(WalReader::new_reader(&wal_file));
        }

        if self.reader.is_none() {
            self.metadata.reset();
            return None;
        }

        let item = if let Some(ref mut reader) = self.reader {
            reader.read()
        } else {
            None
        };
        if item.is_none() {
            self.reader.take();
            Some((vec![], vec![]))
        } else {
            item
        }
    }
}

pub struct Wal {
    mut_mmap: MmapMut,
    page_size: usize,
    next_offset: usize,
}

// Wal[#TODO] (should add some comments)
impl Wal {
    pub fn new_writer(m_file: &str, page_size: u64) -> Self {
        let mut file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(m_file)
            .unwrap();
        file.seek(SeekFrom::Start(page_size)).unwrap();
        file.write_all(&[0]).unwrap();
        file.seek(SeekFrom::Start(0)).unwrap();

        let mut_mmap = unsafe { memmap2::MmapMut::map_mut(&file).unwrap() };

        Wal {
            mut_mmap,
            page_size: page_size as usize,
            next_offset: 0,
        }
    }

    pub fn is_overflow(&mut self, key: &[u8], value: &[u8]) -> bool {
        // last 8 used to write magic number
        if self.next_offset + 8 + key.len() + 8 + value.len() >= self.page_size - 8 {
            self.mut_mmap[self.next_offset..self.next_offset + 8]
                .clone_from_slice(&u64::to_le_bytes(WAL_END_MAGIC_NUMBER));
            true
        } else {
            false
        }
    }

    pub fn write(&mut self, key: &[u8], value: &[u8]) {
        let mut offset = self.next_offset;
        self.mut_mmap[offset..offset + 8].clone_from_slice(&u64::to_le_bytes(key.len() as u64));
        offset += 8;

        self.mut_mmap[offset..offset + key.len()].clone_from_slice(key);
        offset += key.len();

        self.mut_mmap[offset..offset + 8].clone_from_slice(&u64::to_le_bytes(value.len() as u64));
        offset += 8;

        self.mut_mmap[offset..offset + value.len()].clone_from_slice(value);
        offset += value.len();

        self.next_offset = offset;
    }
}

// Drop[#TODO] (should add some comments)
impl Drop for Wal {
    fn drop(&mut self) {
        self.mut_mmap[self.next_offset..self.next_offset + 8]
            .clone_from_slice(&u64::to_le_bytes(WAL_END_MAGIC_NUMBER));
    }
}

pub struct WalReader {
    mmap: Mmap,
    next_offset: usize,
}

// Wal[#TODO] (should add some comments)
impl WalReader {
    pub fn new_reader(m_file: &str) -> Self {
        let mut file = fs::OpenOptions::new()
            .read(true)
            .write(false)
            .open(m_file)
            .unwrap();
        file.seek(SeekFrom::Start(0)).unwrap();

        let mmap = unsafe { memmap2::Mmap::map(&file).unwrap() };

        WalReader {
            mmap,
            next_offset: 0,
        }
    }

    pub fn read(&mut self) -> Option<(Vec<u8>, Vec<u8>)> {
        let mut offset = self.next_offset;
        let first_u64 = u64::from_le_bytes(self.mmap[offset..offset + 8].try_into().unwrap());
        offset += 8;
        if first_u64 == WAL_END_MAGIC_NUMBER {
            return None;
        }

        let key_len = first_u64 as usize;
        let key: Vec<u8> = self.mmap[offset..offset + key_len].into();
        offset += key_len;

        let value_len = u64::from_le_bytes(self.mmap[offset..offset + 8].try_into().unwrap());
        offset += 8;
        let value: Vec<u8> = self.mmap[offset..offset + value_len as usize].into();
        offset += value_len as usize;

        self.next_offset = offset;

        Some((key, value))
    }
}
