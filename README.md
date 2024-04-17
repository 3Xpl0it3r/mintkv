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
[*] B+tree as engine
[*] Memtables
[ ] LE128 Code
[ ] Blocks(disk present)
[ ] Wal
[ ] tombstone
[ ] compaction

# Example

```rust
use mintkv::Mintkv;
use std::fs;

const TEST_COUNT: i32 = 100;
const DEFAULT_DATABASE: &str = "test.db";
fn main() {
    _ = fs::remove_file(DEFAULT_DATABASE);
    // new instance database
    let mut btree = Mintkv::new(DEFAULT_DATABASE);

    // add key-values
    for i in 1..=TEST_COUNT {
        let (key, value) = (format!("key{}", i), format!("value-{}", i));
        btree.insert(&key, &value);
    }

    // find keys
    for i in 1..=TEST_COUNT {
        let search_key = format!("key{}", i);
        let expected_value = format!("value-{}", i);
        if let Ok(item) = btree.find(&search_key) {
            let value = String::from_utf8(item.value.clone()).unwrap();
            if !value.eq(expected_value.as_str()) {
                println!("Cannot find key {}", search_key);
            }
        } else {
            println!("Find key {} Failed", search_key);
        }
    }

    // remove key-values
    for i in 1..=TEST_COUNT {
        let key = format!("key{}", i);
        let expected_value = format!("value-{}", i);
        if let Ok(item) = btree.delete(&key) {
            if !item.eq(expected_value.as_str()) {
                println!("Cannot Remove key {}", key);
            }
        } else {
            println!("Remove key {} failed", key);
        }
    }
    _ = fs::remove_file(DEFAULT_DATABASE);
}

```
