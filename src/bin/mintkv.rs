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
