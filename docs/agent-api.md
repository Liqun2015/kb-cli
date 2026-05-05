# Agent API Notes

This document describes the **current v0.1.2 boundary** for agents using `kb-cli`.

## Current Command Pattern

```bash
kb [OPTIONS] <COMMAND> [ARGS]
```

Important option:

| Option | Description | Example |
|---|---|---|
| `--kb-path <path>` | Specify the knowledge-base directory. | `kb --kb-path /user/KnowledgeBase init` |

## Implemented Commands

```bash
kb init [--force]
kb extract-metadata [--force]
kb build-wiki
kb repl
kb list-models
kb show-model
kb add-model [OPTIONS]
kb switch-model <id>
kb delete-model <id>
kb validate-model [id]
```

## Not Implemented Yet

The following are design goals, **not current v0.1.2 features**:

```bash
kb --format json ...
kb add-paper ...
kb search ...
kb list-papers
kb list-notes
kb --use-llm search ...
```

Agents should not depend on those commands until they are explicitly implemented.

## Stable Local Workflow

1. Put PDFs into `KnowledgeBase/raw/papers/`.
2. Put notes/documents into `KnowledgeBase/raw/notes/`.
3. Run:

```bash
kb --kb-path KnowledgeBase extract-metadata
kb --kb-path KnowledgeBase build-wiki
```

4. Read generated Markdown under:

```text
KnowledgeBase/wiki/
KnowledgeBase/wiki/indexes/
```

## Model-Switch Bridge

The REPL can write file-based LLM requests:

```text
.model_switch_input.json
```

Each request contains:

```json
{
  "request_id": "kb-...",
  "prompt": "...",
  "context": "...",
  "timestamp": "..."
}
```

An external model-switch process may respond by writing:

```text
.model_switch_output.json
```

Recommended response shape:

```json
{
  "request_id": "same request_id from input",
  "response": "...",
  "model": "optional model name",
  "error": null
}
```

The CLI ignores or warns about responses without a matching `request_id` to avoid stale answers.

## Agent Safety Rules for This Version

- Prefer `kb --help` and command-specific `--help` before assuming a command exists.
- Treat stdout as human-readable text, not stable JSON.
- Do not assume vector search, RAG, or network calls.
- Do not write source files into `KnowledgeBase/`; keep the Rust tool and the data directory separate.
