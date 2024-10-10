TRACE_FILE=trace.log RUST_LOG=info cargo run --release -- ../code/std/core/sys ../code/std/core ../code/std
cargo prove trace --elf ./elf/validator-sp1-elf --trace trace.log
rm trace.log