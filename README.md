# Meson Contracts Solana

Execute the code below to run the unit tests:

```bash
cargo build-sbf
cargo test-sbf --test functional
RUST_LOG=error cargo test-sbf --test functional -- --nocapture  # See the logs from functional.rs
```