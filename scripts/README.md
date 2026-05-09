# Scripts

This directory intentionally keeps only small, deterministic developer helpers.

## Build helpers

```bat
scripts\build_release.bat
```

```bash
scripts/build_release.sh
```

These run the local Rust formatting/verification/build flow:

```text
cargo fmt
cargo fmt --check
cargo test
cargo check
cargo build --release
```

`cargo fmt` is run first on purpose. Formatting is deterministic mechanical maintenance. After the script succeeds, review any formatting-only edits with `git diff` before committing.

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

## Lockfile behavior

The release scripts run Cargo normally. If `Cargo.lock` is absent, Cargo will generate it locally. Review the generated lockfile before committing.
