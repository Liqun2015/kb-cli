# macOS / Linux Quick Start

This guide is for macOS, Linux, and other Unix-like shells such as `bash` and `zsh`.

Current version: `v0.5.1`

## 1. Install Rust once

Install Rust with `rustup`:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Reload your shell, then verify:

```bash
rustc --version
cargo --version
```

If the commands are not found, make sure Cargo's bin directory is on `PATH`:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

Add that line to `~/.zshrc`, `~/.bashrc`, or `~/.profile` if needed.

## 2. Build and install `kb`

From the `kb-cli` project directory:

```bash
cargo install --path . --force
```

Verify:

```bash
kb --help
which kb
```

## 3. Bootstrap a knowledge folder

Recommended safe command:

```bash
kb --kb-path "$HOME/github/LLM-wiki/quantum" bootstrap --copy
```

This runs the deterministic local workflow:

```text
init -> ingest -> extract-metadata -> build-wiki
```

It will create or ensure:

```text
raw/
wiki/
rules/
processing/
outputs/
references/
logs/
```

## 4. Preview before writing

Use `--dry-run` or its alias `--preview`:

```bash
kb --kb-path "$HOME/github/LLM-wiki/quantum" ingest --copy --dry-run
kb --kb-path "$HOME/github/LLM-wiki/quantum" ingest --copy --preview
```

For linting without writing a report, use:

```bash
kb --kb-path "$HOME/github/LLM-wiki/quantum" lint-static --no-report
```

## 5. Prepare AI/human handoff artifacts

`prepare` creates reviewable planning artifacts. It does not call an LLM and does not edit `wiki/`.

```bash
kb --kb-path "$HOME/github/LLM-wiki/quantum" status
kb --kb-path "$HOME/github/LLM-wiki/quantum" prepare --new --dry-run
kb --kb-path "$HOME/github/LLM-wiki/quantum" prepare --new
```

Generated files:

```text
processing/prepare_queue.json
processing/proposals/prepare_plan_<timestamp>.md
processing/proposals/prepare_agent_prompt_<timestamp>.md
```

## 6. Run the shell helper

From the project root:

```bash
chmod +x scripts/build_llm_wiki.sh
scripts/build_llm_wiki.sh "$HOME/github/LLM-wiki/quantum"
```

Useful flags:

```bash
scripts/build_llm_wiki.sh "$HOME/github/LLM-wiki/quantum" --copy
scripts/build_llm_wiki.sh "$HOME/github/LLM-wiki/quantum" --move
scripts/build_llm_wiki.sh "$HOME/github/LLM-wiki/quantum" --recursive
scripts/build_llm_wiki.sh "$HOME/github/LLM-wiki/quantum" --dry-run
scripts/build_llm_wiki.sh "$HOME/github/LLM-wiki/quantum" --no-install
```

The helper delegates to:

```bash
kb --kb-path <target> bootstrap --copy
```

## 7. Recommended verification flow

After bootstrapping or after a human/AI edits `wiki/`, run:

```bash
kb --kb-path "$HOME/github/LLM-wiki/quantum" status
kb --kb-path "$HOME/github/LLM-wiki/quantum" sync-wiki --dry-run
kb --kb-path "$HOME/github/LLM-wiki/quantum" sync-wiki
kb --kb-path "$HOME/github/LLM-wiki/quantum" lint-static
kb --kb-path "$HOME/github/LLM-wiki/quantum" query thermal cloak
```

Open the generated lint report under:

```text
$HOME/github/LLM-wiki/quantum/outputs/reports/
```

## 8. Run tests before release

From the `kb-cli` project directory:

```bash
cargo fmt --check
cargo test
cargo check
cargo build --release
```

## 9. Safety notes

- `--copy` is the default and safest mode.
- `--move` changes the folder layout and should be used only when intentional.
- `--dry-run` previews write-like actions.
- `--preview` is an alias for `--dry-run` on dry-run-capable commands.
- `lint-static --no-report` checks without writing `outputs/reports/lint_static_*.md`.
- Recursive ingest skips managed folders such as `raw`, `wiki`, `rules`, `processing`, `references`, `outputs`, `logs`, `.git`, `.obsidian`, `target`, and `node_modules`.

## 10. Troubleshooting

### `kb` is not found

Check:

```bash
which kb
echo "$PATH"
```

Then add Cargo's bin directory:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

### A path contains spaces

Quote it:

```bash
kb --kb-path "$HOME/My Knowledge Bases/quantum" bootstrap --copy
```

### Permission denied when running the helper

Run:

```bash
chmod +x scripts/build_llm_wiki.sh
```

## 11. Open in Obsidian

Open the knowledge folder itself:

```text
$HOME/github/LLM-wiki/quantum
```

Useful files:

```text
wiki/Home.md
processing/manifest.json
rules/LLM_WIKI_SCHEMA.md
outputs/reports/
```
