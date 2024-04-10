use mintkv::Mintkv;
use std::fs;

const TEST_COUNT: i32 = 2000;
const DEFAULT_DATABASE: &str = "test.db";

fn main() {
    _ = fs::remove_file(DEFAULT_DATABASE);
    let mut db = Mintkv::new(DEFAULT_DATABASE);
    // add key-values
    for i in 1..=TEST_COUNT {
        let (key, value) = (format!("key{}", i), format!("value-{}", i));
        println!("Inseted :{}", key);
        db.insert(&key, &value);
    }

    for i in 1..=TEST_COUNT {
        let key = format!("key{}", i);
        db.find(&key).expect("Find key {} Failed");
    }


    for i in 1..=TEST_COUNT {
        let key = format!("key{}", i);
        let res = db.delete(&key);
        println!(
            "Removed: {}, Result: {:?}\n-----------------------------",
            key, res
        );
    }
}
