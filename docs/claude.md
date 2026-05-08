# claude.md — Development Guardrails for kb-cli

## Current version

`kb-cli v0.5.4`

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

The manifest is the bridge between the source-material layer and future AI prepare operations. It should track raw files by relative path, source kind, size, SHA-256 content hash, status, first-seen timestamp, and last-seen timestamp; missing raw files should remain as `raw_missing`.

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
prepare
sync-wiki
lint-static
query
repl
list-models
show-model
add-model
switch-model
delete-model
validate-model
```

## CLI / shell / LLM boundary

Preserve the deterministic boundary between batch mode, shell mode, and future LLM modes.

Required principles:

- `kb ...` is batch mode.
- `kb>` is interactive shell mode.
- Core knowledge-base commands must map one-to-one across batch and shell modes.
- `kb>` is not an LLM chat interface.
- Do not interpret free-form natural language inside `kb>` as an LLM request.
- Do not call any LLM API implicitly from `kb>`.
- Future LLM behavior must be introduced only through a deliberately designed explicit interface; do not advertise placeholder LLM command names before that design exists.
- Do not write a separate business-logic implementation for shell mode; shell mode should dispatch to the same command handlers used by batch mode.

See `docs/cli-shell-principle.md`.

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

`prepare` is implemented only as a review-first planning command. It generates `processing/prepare_queue.json` and `processing/proposals/prepare_plan_*.md`; it does not call an LLM or edit `wiki/`. `query` is implemented as deterministic local keyword search only; `lint-static` is implemented as a deterministic local health check.

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

Prepare planning work starts in v0.4.x. Small compatible improvements should increment the patch version:

```text
0.4.0 -> 0.4.1 -> 0.4.2 -> 0.4.3 -> 0.4.4 -> 0.4.5 -> 0.4.6 -> 0.4.7 -> 0.4.8 -> 0.5.0 -> 0.5.1 -> 0.5.2 -> 0.5.3 -> 0.5.4
```

The next major implementation phase after v0.5.4 should refine query ergonomics carefully, then consider saved queries or semantic LLM linting only after the deterministic path is stable.
