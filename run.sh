rm test.db
rm -rf target
cargo build
time RUST_BACKTRACE=1 ./target/debug/mintkv
# hexdump -C test.db
