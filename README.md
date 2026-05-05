# kb-cli

> A small Rust CLI for building and maintaining a local Markdown LLM Wiki.
>
> Current version: `v0.2.0`

`kb-cli` follows a Karpathy-style local knowledge workflow: keep source materials in `raw/`, let AI maintain Markdown pages under `wiki/`, and constrain future AI maintainers through a generated `rules/` layer.

## Current Scope

This version provides a conservative, file-based local LLM Wiki scaffold. It includes cross-platform bootstrap/ingest workflows and now generates the Wiki rule/schema layer. It still does **not** provide full RAG, vector search, JSON agent output, or autonomous LLM compilation.

## What Changed in v0.2.0

`kb init` now creates the third LLM Wiki layer:

```text
rules/
├── LLM_WIKI_SCHEMA.md
├── PAPER_PAGE_TEMPLATE.md
├── CONCEPT_PAGE_TEMPLATE.md
├── QUERY_POLICY.md
└── LINT_POLICY.md
```

This layer is the "operating contract" for future AI agents. It explains how to treat `raw/`, how to maintain `wiki/`, how to answer questions, and how to lint the Wiki.

## Installation

```bash
cargo build --release
cargo install --path . --force
```

The binary name is:

```bash
kb
```

## Cross-Platform Quick Start

Turn an existing literature folder into an LLM Wiki in one command:

```bash
kb --kb-path /path/to/your/literature-folder bootstrap --copy
```

Windows example:

```powershell
kb --kb-path "D:\github\LLM-wiki\quantum" bootstrap --copy
```

macOS/Linux example:

```bash
kb --kb-path "$HOME/github/LLM-wiki/quantum" bootstrap --copy
```

The bootstrap command runs:

```text
init -> ingest -> extract-metadata -> build-wiki
```

By default, `bootstrap` uses safe copy mode unless `--move` is explicitly provided.

## Manual Workflow

```bash
# 1. Create the local LLM Wiki directory structure and rules layer
kb --kb-path /path/to/kb init

# 2. Organize root-level source files into raw/ subfolders
kb --kb-path /path/to/kb ingest --copy

# 3. Extract metadata from PDFs under raw/papers/
kb --kb-path /path/to/kb extract-metadata

# 4. Generate Markdown wiki pages and indexes
kb --kb-path /path/to/kb build-wiki
```

Useful ingest modes:

```bash
# Safest: copy files into raw/
kb --kb-path /path/to/kb ingest --copy

# Move files into raw/
kb --kb-path /path/to/kb ingest --move

# Include subfolders while skipping raw/wiki/.git/.obsidian/target/etc.
kb --kb-path /path/to/kb ingest --copy --recursive

# Preview planned actions without copying or moving
kb --kb-path /path/to/kb ingest --copy --dry-run
```

Useful bootstrap modes:

```bash
kb --kb-path /path/to/kb bootstrap --copy
kb --kb-path /path/to/kb bootstrap --move
kb --kb-path /path/to/kb bootstrap --copy --recursive
kb --kb-path /path/to/kb bootstrap --copy --dry-run
kb --kb-path /path/to/kb bootstrap --copy --force-metadata
```

## Windows Helper Script

Windows users can also use the bundled wrapper script:

```bat
scripts\build_llm_wiki.bat "D:\github\LLM-wiki\quantum"
```

This script delegates the real work to the cross-platform Rust command:

```bat
kb --kb-path <target> bootstrap --copy
```

## Knowledge Base Layout

```text
KnowledgeBase/
├── raw/
│   ├── papers/       # Original PDF papers
│   ├── notes/        # Notes and documents
│   ├── images/       # Images and figures
│   ├── datasets/     # Data sets
│   ├── archives/     # Compressed source archives
│   ├── repos/        # Code/repository-like source files
│   └── other/        # Other source materials
├── wiki/
│   ├── papers/       # Generated paper pages
│   ├── notes/        # Generated note pages
│   ├── concepts/     # Concept pages
│   ├── topics/       # Topic pages
│   ├── people/       # People/institution pages
│   ├── methods/      # Method/protocol pages
│   ├── comparisons/  # Comparison pages
│   ├── timelines/    # Timeline pages
│   ├── questions/    # Saved Q&A pages
│   └── indexes/      # Navigation indexes
├── rules/            # LLM Wiki schema and AI-maintainer contract
├── outputs/          # Generated answers and reports
├── processing/       # Intermediate processing files
├── references/       # Templates and reference materials
└── logs/             # Metadata and logs
```

## Implemented Commands

| Command | Status | Purpose |
|---|---:|---|
| `kb init [--force]` | implemented | Create the local LLM Wiki structure and generated rules layer. |
| `kb ingest [--copy\|--move] [--recursive] [--dry-run]` | implemented | Organize source files into `raw/` subfolders. |
| `kb bootstrap [--copy\|--move]` | implemented | Run `init + ingest + extract-metadata + build-wiki`. |
| `kb extract-metadata [--force]` | implemented | Extract basic metadata from PDFs under `raw/papers/`. |
| `kb build-wiki` | implemented | Generate Markdown paper/note pages and index pages. |
| `kb repl` | implemented | Start a simple interactive shell. |
| `kb list-models` | implemented | List configured LLM models. |
| `kb show-model` | implemented | Show the current model configuration. |
| `kb add-model ...` | implemented | Add a model configuration. |
| `kb switch-model <id>` | implemented | Switch the active model. |
| `kb delete-model <id>` | implemented | Delete a model configuration. |
| `kb validate-model [id]` | implemented | Check local model configuration fields. |

## Not Yet Implemented

These are planned directions, not current features:

```text
kb compile       # AI compiles raw materials into wiki pages
kb query         # Query the compiled wiki and optionally save durable answers
kb lint          # Wiki health checks: broken links, orphan pages, duplicated concepts
processing/manifest.json tracking
vector search / RAG / JSON agent API
```

## File Classification Used by `ingest`

```text
.pdf                         -> raw/papers
.md/.markdown/.txt/.docx     -> raw/notes
.png/.jpg/.jpeg/.svg/etc.    -> raw/images
.csv/.xlsx/.json/.xml/etc.   -> raw/datasets
.zip/.rar/.7z/.tar/.gz/etc.  -> raw/archives
.rs/.py/.js/.ts/.go/etc.     -> raw/repos
other ordinary files         -> raw/other
```

`ingest --recursive` skips generated/project folders such as:

```text
raw, wiki, processing, references, outputs, logs, .git, .obsidian, target, node_modules
```

It also skips common executable/script/config files such as `.bat`, `.cmd`, `.ps1`, `.sh`, `.exe`, `.dll`, `README.md`, `Cargo.toml`, and `.gitignore`.

## Model Configuration

The real runtime config file is local-only:

```text
.model_config.json
```

It is ignored by Git. Start from the example file if needed:

```bash
cp .model_config.example.json .model_config.json
```

Model-switch request/response files are also ignored:

```text
.model_switch_input.json
.model_switch_output.json
```

Each new REPL LLM request writes a `request_id` to `.model_switch_input.json`; matching output should include the same `request_id` to avoid stale responses.

## Notes for Future Agents

`docs/agent-api.md` records the current boundary: this project is not yet a full machine-readable agent API. Do not assume `--format json`, vector search, `add-paper`, `compile`, `query`, `lint`, or full RAG exists in `v0.2.0`.

## License

MIT License - see [LICENSE](LICENSE).
