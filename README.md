# mintkv
A simple KV database in rust is based on btree, written as a learning project.

# File system layout
```txt
data
|----blocks
|      b_0000001
          data
          meta.json
          index.json
          tombstone
|      b_0000002
          data
          meta.json
          index.json
          tombstone
|-----memtables (mmap)
|         m_0001
             data 
             tombstone

```

# TODO
- [x] B+tree as engine
- [x] Memtables
- [ ] LE128 Code
- [ ] Blocks(disk present)
- [ ] Wal
- [ ] tombstone
- [ ] compaction

# Example

```rust
use mintkv::db::MintKv;
const TEST_COUNT: i32 = 1000;

fn main() {
    let mut db = MintKv::new("./data");
    for i in 0..TEST_COUNT {
        let (key, value) = (format!("key-{}", i), format!("value-{}", i));
        db.insert(&key, &value).unwrap();
    }

    for i in 0..TEST_COUNT {
        let key = format!("key-{}", i);
        let result = db.get(&key);
        println!("Search For: {}, Reesult: {:?}", key, result);
    }
}
```
