use std::fs::{File, OpenOptions};
use std::io::{ErrorKind, Seek, SeekFrom, Write};

use memmap2::MmapMut;

//checkpoints 用来继续当前wal 里面记录的数据有多少已经持久化到磁盘
//
// 每一个chunk对应一个mmap文件
// 每当一个cold chunk 被持久化到磁盘，就迭代一个新mmap
//
//
//
// CheckPoint[#TODO] (shoule add some comments )
pub struct CheckPoint {
    pub last_key: u64,
    mmap: MmapMut,
}

// CheckPoint[#TODO] (should add some comments)
impl CheckPoint {
    pub fn get_or_create(data_dir: &str) -> Self {
        let mut is_initial = true;
        let mut file = match OpenOptions::new()
            .write(true)
            .read(true)
            .open(format!("{data_dir}/checkpoint"))
        {
            Ok(file_ptr) => {
                is_initial = false;
                file_ptr
            }
            Err(err) => {
                if err.kind() == ErrorKind::NotFound {
                    File::create_new(format!("{data_dir}/checkpoint"))
                        .expect("crate new database failed")
                } else {
                    panic!("open database failed");
                }
            }
        };
        if is_initial {
            file.seek(SeekFrom::Start(4096)).unwrap();
            file.write_all(&[0]).unwrap();
        }
        file.seek(SeekFrom::Start(0)).unwrap();
        let mmap = unsafe { memmap2::MmapMut::map_mut(&file).unwrap() };
        let mut checkpoint = CheckPoint { last_key: 0, mmap };
        if !is_initial {
            checkpoint.last_key = u64::from_le_bytes(checkpoint.mmap[0..8].try_into().unwrap());
        }
        checkpoint
    }

    pub fn record(&mut self, key: u64) {
        self.last_key = key;
        self.mmap[0..8].clone_from_slice(u64::to_le_bytes(self.last_key).as_ref());
    }
}
