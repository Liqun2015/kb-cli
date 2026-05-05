# claude.md — Development Guardrails for kb-cli

## Current version

`kb-cli v0.1.4`

## Core goal

This project is a small Rust CLI for a local Karpathy-style Markdown knowledge base. It should remain simple, file-based, and easy to inspect.

## v0.1.4 design decision

The Windows batch quick-start was useful, but the real workflow must be cross-platform. Therefore the core onboarding workflow now lives in Rust:

```bash
kb bootstrap --copy
```

This command runs:

```text
init -> ingest -> extract-metadata -> build-wiki
```

The Windows script `scripts/build_llm_wiki.bat` should remain only a wrapper around `kb bootstrap`.

## Implemented commands

```text
init
ingest
bootstrap
extract-metadata
build-wiki
repl
list-models
show-model
add-model
switch-model
delete-model
validate-model
```

## Do not invent undocumented features

Do not claim or implement large new systems unless explicitly requested. In particular, do not add:

```text
full RAG
vector database
embedding pipeline
SQLite/Postgres storage
network server
web UI
JSON agent API
background daemon
cloud sync
```

## Safety rules for ingest/bootstrap

- `init` must not move files.
- `ingest` must not default to destructive behavior.
- `bootstrap` must be safe by default.
- `--copy` is the safe path.
- `--move` is allowed only when explicitly provided.
- `--dry-run` should remain available for inspection.
- Recursive ingest must skip generated/project directories:

```text
raw
wiki
processing
references
outputs
logs
.git
.obsidian
target
node_modules
```

## File classification boundary

Keep classification simple and extension-based:

```text
PDF -> raw/papers
notes/documents -> raw/notes
images -> raw/images
data files -> raw/datasets
archives -> raw/archives
code-like files -> raw/repos
other -> raw/other
```

Do not add content classification, OCR, PDF parsing beyond metadata, or LLM-based sorting in this version line.

## Documentation rule

README and docs must describe only implemented behavior. Do not list future commands such as `add-paper`, `search`, `list-papers`, or `--format json` unless they are actually implemented.

## Versioning rule

Small compatible improvements should increment the patch version:

```text
0.1.4 -> 0.1.5
```

Feature clusters can increment the minor version later:

```text
0.1.x -> 0.2.0
```
