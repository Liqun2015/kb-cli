# Scripts

This directory intentionally keeps only small, deterministic developer helpers.

## Build helpers

```bat
scripts\build_release.bat
```

```bash
scripts/build_release.sh
```

These run the local Rust verification/build flow:

```text
cargo fmt --check
cargo test
cargo check
cargo build --release
```

They are for compiling `kb-cli` itself. They do not initialize or modify a knowledge base.

## Git helper

```bat
scripts\git_safe_push.bat "commit message"
```

```bash
scripts/git_safe_push.sh "commit message"
```

These show `git status`, `git diff`, then `git diff --cached` before committing and pushing.

## Policy

Do not add broad workflow automation here unless it is clearly deterministic, easy to audit, and unlikely to be confused with building the Rust executable.
Knowledge-base workflows should be documented as explicit `kb ...` command sequences rather than hidden inside large wrapper scripts.
