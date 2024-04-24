# cargo test -- --nocapture
rm -rf target
cargo build --release
time RUST_BACKTRACE=1 ./target/release/mintkv
