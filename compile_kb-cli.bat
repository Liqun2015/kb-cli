REM cd D:\github\LLM-wiki\kb-cli

cargo check
cargo build --release
cargo install --path . --force

kb --help
kb --kb-path "D:\github\LLM-wiki\quantum" status
kb --kb-path "D:\github\LLM-wiki\quantum" compile --new --dry-run
kb --kb-path "D:\github\LLM-wiki\quantum" compile --new