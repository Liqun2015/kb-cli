# claude.md — Development Guardrails for kb-cli

## Current version

`kb-cli v0.4.1`

## Core goal

This project is a small Rust CLI for a local Karpathy-style LLM Wiki. It should remain simple, file-based, inspectable, and cross-platform.

The intended three-layer model is:

```text
raw/    original source materials; AI reads but does not rewrite
wiki/   AI-maintained Markdown encyclopedia
rules/  schema/contract layer that constrains AI maintenance behavior
```

## v0.4.0 design decision

This release establishes manifest tracking. `kb status` must scan `raw/` and maintain:

```text
processing/manifest.json
```

The manifest is the bridge between the source-material layer and future AI compile operations. It should track raw files by relative path, source kind, size, SHA-256 content hash, status, first-seen timestamp, and last-seen timestamp; missing raw files should remain as `raw_missing`.

The rule/schema layer added in v0.2.0 remains required. `kb init` must still generate:

```text
rules/LLM_WIKI_SCHEMA.md
rules/PAPER_PAGE_TEMPLATE.md
rules/CONCEPT_PAGE_TEMPLATE.md
rules/QUERY_POLICY.md
rules/LINT_POLICY.md
```

## Implemented commands

```text
init
ingest
bootstrap
extract-metadata
build-wiki
status
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
full JSON agent API
background daemon
cloud sync
```

`compile` is implemented only as a review-first planning command in v0.4.1. It generates `processing/compile_queue.json` and `processing/proposals/compile_plan_*.md`; it does not call an LLM or edit `wiki/`. `query` and `lint` are still planned but not implemented.

## Safety rules for init/ingest/bootstrap

- `init` must not move files.
- `init` may create missing generated rule files.
- `init --force` may overwrite generated rule files, but it must not rewrite user source materials.
- `ingest` must not default to destructive behavior.
- `bootstrap` must be safe by default.
- `--copy` is the safe path.
- `--move` is allowed only when explicitly provided.
- `--dry-run` should remain available for inspection.
- Recursive ingest must skip generated/project directories:

```text
raw
wiki
rules
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

## Manifest rules

- `processing/manifest.json` is generated metadata; it describes `raw/` but must not alter raw files.
- The manifest should remain plain JSON and human-inspectable.
- Use stable relative paths with `/` separators.
- Use content hashes to detect changed raw files.
- Do not treat the manifest as a database or introduce SQLite in this version line.

## Documentation rule

README and docs must describe only implemented behavior. Do not list future commands as available unless they are actually implemented.

## Versioning rule

Compile planning work starts in v0.4.x. Small compatible improvements should increment the patch version:

```text
0.4.0 -> 0.4.1
```

The next major implementation phase should be query/lint scaffolding after compile planning is stable.
