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

