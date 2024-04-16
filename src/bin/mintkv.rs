use mintkv::btree;
use mintkv::memtable;
use std::fs;

const TEST_COUNT: i32 = 2;
const DEFAULT_DATABASE: &str = "test.db";

fn main() {
    let mut memtables = memtable::MemTables::new();
    for i in 0..10 {
        let (key, value) = (format!("key-{}", i), format!("value-{}", i));
        let result = memtables.add(key.as_bytes(), value.as_bytes());
        println!("input : {}, result: {:?}", key, result);
    }

    let result = memtables.delete("key-2".as_bytes());
    println!("delete : Key-2, result: {:?}", result);

    for i in 0..10 {
        let key = format!("key-{}", i);
        let result = memtables.get(key.as_bytes());
        println!("search {} , result: {:?}", key, result);
    }
}

fn main1() {
    _ = fs::remove_file(DEFAULT_DATABASE);
    let mut db = btree::BTree::new(DEFAULT_DATABASE);
    // add key-values
    for i in 1..=TEST_COUNT {
        let (key, value) = (format!("key{}", i), format!("value-{}", i));
        println!("Inseted :{}", key);
        db.insert(key.as_bytes(), value.as_bytes());
    }

    for i in 1..=TEST_COUNT {
        let key = format!("key{}", i);
        db.find(key.as_bytes()).expect("Find key {} Failed");
    }

    for i in 1..=TEST_COUNT {
        let key = format!("key{}", i);
        let res = db.delete(key.as_bytes());
        println!(
            "Removed: {}, Result: {:?}\n-----------------------------",
            key, res
        );
    }
}
