use memmap2::MmapMut;

const DEFAULT_WAL_LEN: u8 = 10;
pub struct WalMeta {
    pub start_index: u8,
    pub wal_len: u8,
    pub cache: Vec<u8>,
    mmap: MmapMut,

    // helper fields
    cur_index: u8,
}

// WalMeta[#TODO] (should add some comments)
impl WalMeta {
    pub fn new(mmap: MmapMut, is_initial: bool) -> Self {
        let cache = Vec::new();
        let mut wal = WalMeta {
            start_index: 0,
            cache,
            mmap,
            wal_len: 0,
            cur_index: 0,
        };
        if is_initial {
            wal.mmap[0] = wal.wal_len;
            wal.mmap[1] = wal.start_index;
        }
        wal
    }
}

// WalMeta[#TODO] (should add some comments)
impl WalMeta {
    pub fn reset(&mut self) {
        self.cache.clear();
        self.wal_len = 0;
        self.start_index = 0;
        self.mmap[0] = self.wal_len;
        self.mmap[1] = self.start_index;
    }
    pub fn reinitial(&mut self) {
        let mut offset = 0;
        self.wal_len = self.mmap[offset];
        offset += 1;
        self.start_index = self.mmap[offset];
        self.cur_index = self.start_index;
        offset += 1;
        if self.wal_len < DEFAULT_WAL_LEN {
            for i in 0..self.wal_len {
                self.cache.insert(0, i);
            }
        } else {
            for i in self.cur_index..DEFAULT_WAL_LEN {
                self.cache.push(i);
            }
            for i in 0..self.cur_index {
                self.cache.push(i);
            }
        }
        _ = offset;
    }

    pub fn itertor(&mut self) -> Option<u8> {
        if self.cache.is_empty() {
            return None;
        }
        self.wal_len -= 1;
        Some(self.cache.remove(0))
    }

    pub fn rotate(&mut self) -> u8 {
        if self.cache.len() < DEFAULT_WAL_LEN as usize {
            self.cache.push(self.start_index);
            self.wal_len += 1;
        }
        let allocated = self.start_index;
        self.start_index = (self.start_index + 1) % DEFAULT_WAL_LEN;

        self.mmap[0] = self.wal_len;
        self.mmap[1] = self.start_index;
        allocated
    }
}
