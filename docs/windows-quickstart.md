# Windows Quick Start

This guide is written for PowerShell on Windows. The safest workflow is still the cross-platform Rust CLI; the batch file is only a convenience wrapper. For macOS/Linux, use `docs/unix-quickstart.md`.

Current version: `v0.6.0.4`

## 1. Install Rust once

Open PowerShell and run:

```powershell
winget install Rustlang.Rustup
```

Close PowerShell, open it again, then verify:

```powershell
rustc --version
cargo --version
```

## 2. Build and install `kb`

From the `kb-cli` project directory:

```powershell
cd D:\github\LLM-wiki\kb-cli
cargo install --path . --force
```

Verify that Windows can find the executable:

```powershell
kb --help
```

If `kb` is not recognized, close and reopen PowerShell. If it is still not found, check that Cargo's bin directory is on `PATH`:

```powershell
$env:USERPROFILE\.cargo\bin
```

## 3. Bootstrap a knowledge folder

Recommended safe command:

```powershell
kb --kb-path "D:\github\LLM-wiki\quantum" bootstrap --copy
```

This runs:

```text
init -> ingest -> extract-metadata -> build-wiki
```

It will create or ensure:

```text
raw\
wiki\
rules\
processing\
outputs\
references\
logs\
```

## 4. Preview before writing

For a dry run, use either `--dry-run` or its alias `--preview`:

```powershell
kb --kb-path "D:\github\LLM-wiki\quantum" ingest --copy --dry-run
kb --kb-path "D:\github\LLM-wiki\quantum" ingest --copy --preview
```

For linting only, the clearest wording is `--no-report`:

```powershell
kb --kb-path "D:\github\LLM-wiki\quantum" lint-static --no-report
```

Equivalent aliases still work:

```powershell
kb --kb-path "D:\github\LLM-wiki\quantum" lint-static --dry-run
kb --kb-path "D:\github\LLM-wiki\quantum" lint-static --preview
```

## 5. Recommended v0.5.10 verification flow

After bootstrapping or after an AI/human edits `wiki\`, run:

```powershell
kb --kb-path "D:\github\LLM-wiki\quantum" status
kb --kb-path "D:\github\LLM-wiki\quantum" sync-wiki --dry-run
kb --kb-path "D:\github\LLM-wiki\quantum" sync-wiki
kb --kb-path "D:\github\LLM-wiki\quantum" lint-static
kb --kb-path "D:\github\LLM-wiki\quantum" query thermal cloak
```

Open the generated lint report here:

```text
D:\github\LLM-wiki\quantum\outputs\reports\lint_static_<timestamp>.md
```

## 6. Build kb-cli itself when needed

From the project root, use the deterministic release-build helper:

```powershell
scripts\build_release.bat
```

This helper runs:

```text
cargo fmt --check
cargo test
cargo check
cargo build --release
```

It compiles `kb-cli` itself and writes the executable to:

```text
target\release\kb.exe
```

For knowledge-base workflows, prefer explicit `kb ...` commands instead of broad wrapper scripts.

## 7. Run tests before release

From the `kb-cli` project directory:

```powershell
cargo fmt --check
cargo test
cargo check
cargo build --release
```

The current test suite covers `build-wiki` and `lint-static`, especially the source-front-matter and report-writing behavior that protects the local Wiki loop.

## 8. Safety notes

- `--copy` is the default and safest mode.
- `--move` changes the folder layout and should be used only when intentional.
- `--dry-run` previews write-like actions.
- `--preview` is an alias for `--dry-run` on dry-run-capable commands.
- `lint-static --no-report` checks without writing `outputs\reports\lint_static_*.md`.
- Recursive ingest skips managed folders such as `raw`, `wiki`, `rules`, `processing`, `references`, `outputs`, `logs`, `.git`, `.obsidian`, `target`, and `node_modules`.

## 9. Troubleshooting

### `kb` is not recognized

Reopen PowerShell after `cargo install --path . --force`. Then check:

```powershell
where.exe kb
```

### The wrong folder was used

Always pass an explicit path:

```powershell
kb --kb-path "D:\github\LLM-wiki\quantum" status
```

### A path contains spaces

Quote it:

```powershell
kb --kb-path "D:\My Knowledge Bases\quantum" bootstrap --copy
```

### Lint reports too many orphan pages

Start with broken links and missing source front matter first. Orphan pages are often normal while the Wiki is young. Link useful pages from `wiki\Home.md`, index pages, or topic pages after the source links are clean.

## 10. Open in Obsidian

Open the knowledge folder itself, for example:

```text
D:\github\LLM-wiki\quantum
```

Useful files:

```text
wiki\Home.md
processing\manifest.json
rules\LLM_WIKI_SCHEMA.md
outputs\reports\
```
