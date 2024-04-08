rm test.db
rm -rf target
RUST_BACKTRACE=1 cargo run --bin mintkv 
hexdump -C test.db
