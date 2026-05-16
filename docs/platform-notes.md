# Platform Notes

`kb-cli` is designed so the Rust CLI stays cross-platform and the platform-specific differences stay in documentation and small wrapper scripts.

Current version: `v0.7.27`

## Related architecture document

The batch-mode / shell-mode boundary is defined in `docs/cli-shell-principle.md`. Platform wrapper scripts must not bypass that boundary.

## Core policy

The core commands are the same on Windows, macOS, and Linux:

```bash
kb init
kb ingest
kb build-wiki
kb status
kb build <topic>
kb sync-wiki
kb lint-static
```

Platform differences should normally be limited to:

- shell syntax;
- path syntax;
- executable lookup through `PATH`;
- wrapper scripts such as `.bat` and `.sh`.

Do not add large platform-specific branches to the Rust core unless a real bug requires it.

## Path examples

Windows PowerShell:

```powershell
kb --lib "D:\github\LLM-wiki\quantum" status
```

macOS/Linux:

```bash
kb --lib "$HOME/github/LLM-wiki/quantum" status
```

Cross-platform relative path:

```bash
kb --lib ./quantum status
```

## Wrapper scripts

The `scripts/` directory should remain small and deterministic. In v0.5.10 it contains only:

```text
scripts/build_release.bat   Windows Rust release build helper
scripts/build_release.sh    macOS/Linux Rust release build helper
scripts/git_safe_push.bat   Windows review-first Git helper
scripts/git_safe_push.sh    macOS/Linux review-first Git helper
scripts/README.md           Script policy and usage notes
```

Knowledge-base workflows should be documented as explicit `kb ...` command sequences rather than hidden inside broad wrapper scripts. This avoids confusing "build the Rust executable" with "build or initialize a knowledge base."

## Release check

Before tagging a release, run the Rust checks on at least one platform:

```bash
cargo fmt --check
cargo test
cargo check
cargo build --release
```

When possible, also smoke-test the quick-start examples on Windows PowerShell and a Unix-like shell.

## Git helper scripts

Review-first Git helper scripts are platform-specific wrappers around the same manual workflow:

```text
scripts/git_safe_push.bat   Windows CMD/PowerShell
scripts/git_safe_push.sh    macOS/Linux bash or zsh
```

These scripts are intentionally thin. They should not hide diffs, skip review pauses, rewrite history, or force-push.
