# kb-cli

> A small Rust CLI for building and maintaining a local Markdown knowledge base.
>
> Current version: `v0.1.2`

`kb-cli` follows a Karpathy-style local knowledge workflow: put source materials in `raw/`, generate Markdown pages under `wiki/`, and use Obsidian as the reading/editing frontend.

## Current Scope

This version is intentionally conservative. It provides a reliable local scaffold and a few file-based operations. It does **not** yet provide full RAG, vector search, JSON agent output, or a networked agent API.

## Installation

```bash
# Build
cargo build --release

# Install locally
cargo install --path .
```

The binary name is:

```bash
kb
```

## Quick Start

```bash
# Initialize the default KnowledgeBase directory
kb init

# Re-create missing bootstrap directories/files when needed
kb init --force

# Extract metadata from PDFs in KnowledgeBase/raw/papers/
kb extract-metadata

# Overwrite existing metadata
kb extract-metadata --force

# Build Markdown wiki pages and indexes
kb build-wiki

# Enter interactive REPL mode
kb repl
```

Use a custom knowledge-base path with:

```bash
kb --kb-path /path/to/KnowledgeBase init
kb --kb-path /path/to/KnowledgeBase extract-metadata
kb --kb-path /path/to/KnowledgeBase build-wiki
```

## Knowledge Base Layout

```text
KnowledgeBase/
├── raw/
│   ├── papers/       # Original PDF papers
│   ├── notes/        # Notes and documents
│   ├── images/       # Images and figures
│   ├── datasets/     # Data sets
│   └── repos/        # Code repositories
├── wiki/
│   ├── papers/       # Generated paper pages
│   ├── notes/        # Generated note pages
│   ├── concepts/     # Concept pages
│   ├── topics/       # Topic pages
│   └── indexes/      # Navigation indexes
├── outputs/          # Generated answers and reports
└── logs/             # Metadata and logs
```

## Implemented Commands

| Command | Status | Purpose |
|---|---:|---|
| `kb init [--force]` | implemented | Create the local knowledge-base structure. |
| `kb extract-metadata [--force]` | implemented | Extract basic metadata from PDFs under `raw/papers/`. |
| `kb build-wiki` | implemented | Generate Markdown paper/note pages and index pages. |
| `kb repl` | implemented | Start a simple interactive shell. |
| `kb list-models` | implemented | List configured LLM models. |
| `kb show-model` | implemented | Show the current model configuration. |
| `kb add-model ...` | implemented | Add a model configuration. |
| `kb switch-model <id>` | implemented | Switch the active model. |
| `kb delete-model <id>` | implemented | Delete a model configuration. |
| `kb validate-model [id]` | implemented | Check local model configuration fields. |

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

`docs/agent-api.md` records the current boundary: this project is not yet a full machine-readable agent API. Do not assume `--format json`, vector search, `add-paper`, or full RAG exists in `v0.1.2`.

## License

MIT License - see [LICENSE](LICENSE).
