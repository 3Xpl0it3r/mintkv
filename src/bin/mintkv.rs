use mintkv::db::MintKv;
const TEST_COUNT: i32 = 1000;
const DEFAULT_DATABASE: &str = "test.db";

fn main() {
    let mut db = MintKv::default();
    for i in 0..1000 {
        let (key, value) = (format!("key-{}", i), format!("value-{}", i));
        db.insert(&key, &value).unwrap();
    }
}
