# Agent/API Boundary Notes

This document records what `kb-cli v0.4.8` actually supports. It is intentionally conservative.

## Implemented CLI commands

| Command | Purpose |
|---|---|
| `kb init [--force]` | Create the local LLM Wiki directory structure and generated `rules/` layer. |
| `kb ingest [--copy\|--move] [--recursive] [--dry-run\|--preview]` | Organize source files into `raw/` subfolders. |
| `kb bootstrap [--copy\|--move] [--dry-run\|--preview]` | Run `init + ingest + extract-metadata + build-wiki`. |
| `kb extract-metadata [--force]` | Extract PDF metadata from `raw/papers/`. |
| `kb build-wiki` | Generate Markdown wiki pages and indexes. |
| `kb status [--dry-run\|--preview] [--json] [--unprocessed]` | Refresh `processing/manifest.json`, print JSON summary, or list unprocessed raw files. |
| `kb prepare [--new\|--file PATH] [--dry-run\|--preview]` | Generate prepare queue/proposal files for future LLM wiki maintenance without calling an LLM. |
| `kb sync-wiki [--dry-run\|--preview] [--json]` | Link source front matter in `wiki/**/*.md` back to manifest entries. |
| `kb lint-static [--dry-run\|--preview\|--no-report] [--json] [--strict]` | Check broken WikiLinks, orphan pages, missing source front matter, empty pages, and duplicate titles. |
| `kb repl` | Start a simple interactive shell. |
| `kb list-models` | List configured LLM models. |
| `kb show-model` | Show the active model configuration. |
| `kb add-model ...` | Add a model configuration. |
| `kb switch-model <id>` | Switch the active model. |
| `kb delete-model <id>` | Delete a model configuration. |
| `kb validate-model [id]` | Validate local model configuration fields. |

## Three-layer LLM Wiki model

`kb init` creates the three basic layers:

```text
raw/    # original source materials; AI should read, not rewrite
wiki/   # AI-maintained Markdown encyclopedia
rules/  # schema, templates, and policy files for future AI maintainers
```

Generated rule files:

```text
rules/LLM_WIKI_SCHEMA.md
rules/PAPER_PAGE_TEMPLATE.md
rules/CONCEPT_PAGE_TEMPLATE.md
rules/QUERY_POLICY.md
rules/LINT_POLICY.md
```

Agents should read `rules/LLM_WIKI_SCHEMA.md` before making broad Wiki changes.

## Cross-platform bootstrap

The recommended machine-facing one-command workflow is:

```bash
kb --kb-path /path/to/kb bootstrap --copy
```

This is cross-platform because the workflow is implemented inside the Rust CLI, not inside a Windows-only batch file.


## Manifest / Status

`kb status` is implemented in v0.4.8:

```bash
kb --kb-path /path/to/kb status
kb --kb-path /path/to/kb status --json
kb --kb-path /path/to/kb status --unprocessed
kb --kb-path /path/to/kb status --dry-run
```

Current behavior:
- Scans files under `raw/`.
- Writes `processing/manifest.json` unless `--dry-run` is used.
- Tracks relative path, source kind, extension, size, SHA-256 content hash, status, first-seen timestamp, last-seen timestamp, and reserved `wiki_pages`.
- Prints human-readable summary output.

The manifest is not a stable external API yet; treat it as a local project file that future commands may evolve carefully.

## Prepare planning

`kb prepare` is implemented in v0.4.8 as a planning command only. It may write:

```text
processing/prepare_queue.json
processing/proposals/prepare_plan_<timestamp>.md
processing/proposals/prepare_agent_prompt_<timestamp>.md
```

It does not call an LLM, does not edit `wiki/`, and does not mark manifest entries as fully prepared. Treat the generated proposal as a review artifact for a human or future AI maintainer.

## Static wiki lint

`kb lint-static` is implemented in v0.4.8 as a deterministic local Markdown health check. It does not call an LLM and does not rewrite `wiki/`.

```bash
kb --kb-path /path/to/kb lint-static
kb --kb-path /path/to/kb lint-static --no-report
kb --kb-path /path/to/kb lint-static --json
kb --kb-path /path/to/kb lint-static --strict
```

Reports are written under `outputs/reports/` unless `--dry-run`, `--preview`, or `--no-report` is used. `--no-report` is specific to `lint-static`.

## Current non-goals

Do not assume the following exist in `v0.4.8`:

```text
global --format json
autonomous prepare execution
query
semantic LLM lint
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
kb --kb-path /path/to/kb bootstrap --dry-run
```

Then run the real command:

```bash
kb --kb-path /path/to/kb bootstrap --copy
```

## Output style

`kb status --json` provides a machine-readable summary. Other command outputs remain human-readable and should not be treated as a stable JSON API.


## sync-wiki

Use `kb sync-wiki --dry-run` or `kb sync-wiki --preview` after wiki edits to preview manifest updates, then `kb sync-wiki` to record source-to-wiki links.
