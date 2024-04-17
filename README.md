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
const DEFAULT_DATABASE: &str = "test.db";

fn main() {
    let mut db = MintKv::new();

    db.insert("key1", "value1").unwrap();

    if let Ok(result) = db.get("key1") {
        println!("Found key1: result: {:?}", result);
    }

    if let Ok(result) = db.delete("key1") {
        println!("Removed key1: {}", result);
    }

    let result = db.get("key1");
    println!("After Removed key1, then get key1: {:?}", result);
}

```
