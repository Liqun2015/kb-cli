REM cd D:\github\LLM-wiki\kb-cli

cargo fmt --check
cargo test
cargo check
cargo build --release
cargo install --path . --force

kb --help
kb prepare --help
kb lint-static --help