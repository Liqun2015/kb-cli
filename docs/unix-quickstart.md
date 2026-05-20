# macOS / Linux Quick Start

This guide is for macOS, Linux, and other Unix-like shells such as `bash` and `zsh`.

Current version: `v0.7.38`

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

## 3. Set up a knowledge folder

Recommended safe command:

```bash
kb --source "$HOME/github/LLM-wiki/quantum" setup --recursive
```

This runs the deterministic local workflow:

```text
init -> ingest -> extract-metadata -> build-wiki
```

It will create or ensure these folders under `knowledgebase/`:

```text
knowledgebase/raw/
knowledgebase/wiki/
knowledgebase/rules/
knowledgebase/processing/
knowledgebase/interfaces/
knowledgebase/references/
knowledgebase/interfaces/logs/
```

## 4. Preview before writing

Use `--dry-run` or its alias `--preview`:

```bash
kb --source "$HOME/github/LLM-wiki/quantum" --wiki "$HOME/github/LLM-wiki/quantum/knowledgebase" ingest --copy --recursive --dry-run
kb --source "$HOME/github/LLM-wiki/quantum" --wiki "$HOME/github/LLM-wiki/quantum/knowledgebase" ingest --copy --recursive --preview
```

For linting without writing a report, use:

```bash
kb --wiki "$HOME/github/LLM-wiki/quantum/knowledgebase" lint-static --no-report
```

## 5. Build AI/human handoff artifacts

`kb build <topic>` creates reviewable planning, task, and handoff artifacts. It does not call an LLM and does not edit final scholarly claims.

```bash
kb --wiki "$HOME/github/LLM-wiki/quantum/knowledgebase" status
kb --wiki "$HOME/github/LLM-wiki/quantum/knowledgebase" build <topic> --dry-run
kb --wiki "$HOME/github/LLM-wiki/quantum/knowledgebase" build <topic>
```

Generated files:

```text
processing/text/
processing/sections/
processing/refs/
topics/<topic>/tasks/
topics/<topic>/handoff/
LLM/handoff/current.md
obsidian_main.md
```

## 6. Build kb-cli itself when needed

From the project root, use the deterministic release-build helper:

```bash
chmod +x scripts/build_release.sh
scripts/build_release.sh
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
target/release/kb
```

For knowledge-base workflows, prefer explicit `kb ...` commands instead of broad wrapper scripts.

## 7. Recommended verification flow

After setup or after a human/AI edits `wiki/`, run:

```bash
kb --wiki "$HOME/github/LLM-wiki/quantum/knowledgebase" status
kb --wiki "$HOME/github/LLM-wiki/quantum/knowledgebase" sync-wiki --dry-run
kb --wiki "$HOME/github/LLM-wiki/quantum/knowledgebase" sync-wiki
kb --wiki "$HOME/github/LLM-wiki/quantum/knowledgebase" lint-static
kb --wiki "$HOME/github/LLM-wiki/quantum/knowledgebase" query thermal cloak
```

Open the generated lint report under:

```text
$HOME/github/LLM-wiki/quantum/interfaces/reports/
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
- `lint-static --no-report` checks without writing `interfaces/reports/lint_static_*.md`.
- Recursive ingest skips managed folders such as `raw`, `wiki`, `rules`, `processing`, `references`, `topics`, `interfaces`, `.git`, `.obsidian`, `target`, and `node_modules`.

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
kb --source "$HOME/My Knowledge Bases/quantum" setup --recursive
```

### Permission denied when running a script

Run:

```bash
chmod +x scripts/build_release.sh scripts/git_safe_push.sh
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
interfaces/reports/
```
