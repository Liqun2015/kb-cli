#!/usr/bin/env bash
set -euo pipefail

# Deterministic Rust build helper for macOS/Linux.
# Run from the project root or call as scripts/build_release.sh.

echo
echo "===== Checking Rust formatting ====="
cargo fmt --check

echo
echo "===== Running tests ====="
cargo test

echo
echo "===== Checking project ====="
cargo check

echo
echo "===== Building release executable ====="
cargo build --release

echo
echo "Build succeeded."
echo "Executable: target/release/kb"
