
export RUSTFLAGS="-Zsanitizer=address"
ASAN_OPTIONS=detect_leaks=1 cargo run --target aarch64-apple-darwin