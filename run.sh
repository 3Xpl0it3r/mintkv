rm test.db
rm -rf data
rm -rf target
cargo build --release
time RUST_BACKTRACE=1 ./target/release/mintkv
# hexdump -C test.db
