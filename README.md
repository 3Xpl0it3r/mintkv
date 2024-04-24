# mintkv
A simple tsdb database in rust is based on btree, written as a learning project. key must be u64, and value can be anything;

# File system layout
```txt
➜  mintkv git:(master) ✗ tree -L 3 data
data
├── blocks
│   ├── block-0
│   └── metadata.json
├── checkpoint
└── wal
    ├── metadata
    ├── wal-0
    ├── wal-1
    ├── wal-2
    ├── wal-3
    ├── wal-4
    ├── wal-5
    ├── wal-6
    ├── wal-7
    ├── wal-8
    └── wal-9

3 directories, 14 files
➜  mintkv git:(master) ✗
```

# TODO
- [x] B+tree as engine
- [x] Memtables
- [x] LE128 Code
- [x] Blocks(disk present)
- [x] Wal
- [ ] tombstone
- [ ] compaction

# Example

```rust
use mintkv::db::MintKv;
const TEST_COUNT: u64 = 1000;
fn main() {
    let mut db = MintKv::new("./data");
    for i in 0..TEST_COUNT {
        let value = format!("value-{}", i);
        db.insert(i, value.as_bytes())
            .unwrap()
    }
    db.commit();

    for i in 0..TEST_COUNT {
        let result = db.get(i);
        println!("Search {:?}, Result: {:?}", i, result);
    }
}

```
