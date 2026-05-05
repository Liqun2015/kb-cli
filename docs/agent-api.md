# Agent/API Boundary Notes

This document records what `kb-cli v0.1.4` actually supports. It is intentionally conservative.

## Implemented CLI commands

| Command | Purpose |
|---|---|
| `kb init [--force]` | Create the local knowledge-base directory structure. |
| `kb ingest [--copy\|--move] [--recursive] [--dry-run]` | Organize source files into `raw/` subfolders. |
| `kb bootstrap [--copy\|--move]` | Run `init + ingest + extract-metadata + build-wiki`. |
| `kb extract-metadata [--force]` | Extract PDF metadata from `raw/papers/`. |
| `kb build-wiki` | Generate Markdown wiki pages and indexes. |
| `kb repl` | Start a simple interactive shell. |
| `kb list-models` | List configured LLM models. |
| `kb show-model` | Show the active model configuration. |
| `kb add-model ...` | Add a model configuration. |
| `kb switch-model <id>` | Switch the active model. |
| `kb delete-model <id>` | Delete a model configuration. |
| `kb validate-model [id]` | Validate local model configuration fields. |

## Cross-platform bootstrap

The recommended machine-facing one-command workflow is now:

```bash
kb --kb-path /path/to/kb bootstrap --copy
```

This is cross-platform because the workflow is implemented inside the Rust CLI, not inside a Windows-only batch file.

## Current non-goals

Do not assume the following exist in `v0.1.4`:

```text
--format json
vector search
full RAG
semantic retrieval
add-paper
list-papers
list-notes
networked agent API
long-running background daemon
```

## Safety behavior

`ingest` and `bootstrap` default to copy mode unless `--move` is explicitly provided. Recursive mode skips generated/project folders such as:

```text
raw, wiki, processing, references, outputs, logs, .git, .obsidian, target, node_modules
```

For automation, prefer running a dry run first:

```bash
kb --kb-path /path/to/kb bootstrap --copy --dry-run
```

Then run the real command:

```bash
kb --kb-path /path/to/kb bootstrap --copy
```

## Output style

The current output is human-readable text. It is not a stable JSON API. Future versions may add a dedicated machine-readable output mode, but agents must not rely on that in `v0.1.4`.
