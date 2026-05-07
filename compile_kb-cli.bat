REM cd D:\github\LLM-wiki\kb-cli

cargo check
cargo build --release
cargo install --path . --force

kb --help
kb --kb-path "D:\github\LLM-wiki\quantum" status
kb prepare --help
kb --kb-path "D:\github\LLM-wiki\quantum" prepare --new --dry-run
kb --kb-path "D:\github\LLM-wiki\quantum" prepare --new
kb lint-static --help
kb --kb-path "D:\github\LLM-wiki\quantum" lint-static --dry-run
kb --kb-path "D:\github\LLM-wiki\quantum" lint-static