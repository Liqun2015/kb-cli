# Command Overview

This document is the command map for `kb-cli`. It is meant for human maintainers, Manager LLM sessions, Worker LLM tasks, and future third-party skills.

The guiding rule is simple:

> Every command must have one clear ability. When a command cannot safely finish a semantic task, it should report evidence and defer the hard part to `LLM/tasks/` or human review.

## Command categories

| Category | Commands | Default LLM behavior | Purpose |
|---|---|---:|---|
| Deterministic core | `init`, `ingest`, `bootstrap`, `status`, `sync-wiki`, `lint-static`, `build-wiki` | No LLM | Maintain the file structure, manifest, wiki skeleton, and static checks. |
| Search / inspection | `query`, `grep`, `links` | No LLM | Locate pages, lines, and WikiLink relations for later review. |
| Text / reference processing | `extract-text`, `refs`, `refs-index`, `refs-graph`, `keywords`, `extract-metadata` | No LLM | Convert source traces into searchable text, bibliographic relation candidates, and keyword/topic relation evidence. |
| Handoff / memory | `prepare`, `tasks`, `memory` | No LLM | Prepare reviewable work items and record completed work. |
| Health / reporting | `health` | No LLM | Summarize relationship-network completeness, review backlog, and pipeline health. |
| Interactive shell | `shell` | No hidden LLM | Deterministic interactive command shell; not a chat interface. |
| Model configuration | `list-models`, `show-model`, `add-model`, `switch-model`, `delete-model`, `validate-model` | Configuration only | Manage model configuration without turning deterministic commands into implicit LLM calls. |

## Core commands

### `kb init`

- **Ability:** create or ensure the local knowledge-base directory structure.
- **Primary input:** knowledge-base path.
- **Primary output:** `raw/`, `wiki/`, `processing/`, `rules/`, `LLM/tasks/`, `LLM/memory/`, and related directories.
- **Allows LLM:** no.
- **Deferred work:** none. Directory creation is deterministic.

### `kb ingest`

- **Ability:** organize source files into `raw/` subfolders and register them for later processing.
- **Primary input:** files under the KB root or selected search scope.
- **Primary output:** `raw/papers/`, `raw/notes/`, `raw/images/`, `raw/datasets/`, `processing/manifest.json`.
- **Allows LLM:** no.
- **Deferred work:** ambiguous file meaning, paper interpretation, and semantic classification should be deferred to later tasks.

### `kb status`

- **Ability:** inspect `raw/` and manifest status.
- **Primary input:** `raw/`, `processing/manifest.json`.
- **Primary output:** terminal summary or JSON.
- **Allows LLM:** no.
- **Deferred work:** unprocessed raw files can later be turned into `prepare` or `tasks` items.

### `kb build-wiki`

- **Ability:** build deterministic wiki skeleton pages from local metadata and raw materials.
- **Primary input:** `raw/`, `logs/papers_metadata.json`, `processing/manifest.json`.
- **Primary output:** `wiki/` skeleton pages and indexes.
- **Allows LLM:** no.
- **Deferred work:** actual scientific synthesis and relationship interpretation belong to Manager/Worker LLM workflows.

### `kb sync-wiki`

- **Ability:** scan wiki front matter and update manifest links to source files.
- **Primary input:** `wiki/`, `processing/manifest.json`.
- **Primary output:** updated `processing/manifest.json`.
- **Allows LLM:** no.
- **Deferred work:** source traceability issues can be handed to `kb tasks`.

### `kb lint-static`

- **Ability:** detect static wiki issues such as broken WikiLinks, orphan pages, empty pages, duplicate titles, and missing source front matter.
- **Primary input:** `wiki/`.
- **Primary output:** terminal summary, JSON, or `outputs/reports/lint_static_*.md`.
- **Allows LLM:** no.
- **Deferred work:** link repair, source traceability repair, and concept cleanup should become `LLM/tasks/` items.

## Search and inspection commands

### `kb query`

- **Ability:** page-level local keyword search over `wiki/**/*.md`.
- **Primary input:** `wiki/`.
- **Primary output:** ranked pages and snippets.
- **Allows LLM:** no.
- **Deferred work:** semantic interpretation of results belongs to the Manager LLM.

### `kb grep`

- **Ability:** line-level Rust-native text search over a selected directory or file.
- **Primary input:** `wiki/`, `processing/text/`, or a specified path.
- **Primary output:** path, line number, and matching line.
- **Allows LLM:** no.
- **Deferred work:** meaning, summarization, or relationship synthesis from hits belongs to LLM tasks.

### `kb links`

- **Ability:** extract and deterministically resolve WikiLinks.
- **Primary input:** `wiki/`.
- **Primary output:** resolved, unresolved, and ambiguous link records.
- **Allows LLM:** no.
- **Deferred work:** ambiguous or unresolved semantic link repair should be handed to a Wiki Link Repair Worker.

## Text and reference processing commands

### `kb extract-text`

- **Ability:** best-effort deterministic text extraction from supported source files.
- **Primary input:** `raw/papers/` or a specified file/directory.
- **Primary output:** `processing/text/`.
- **Allows LLM:** no by default.
- **Deferred work:** OCR, layout repair, garbled text repair, table reconstruction, and LLM cleanup must be explicit future modes or Worker tasks.

### `kb refs`

- **Ability:** scan extracted text for reference headings, bibliography entries, DOI values, and optional citation markers.
- **Primary input:** `processing/text/`.
- **Primary output:** terminal or JSON reference hints.
- **Allows LLM:** no.
- **Deferred work:** reference normalization, citation reconciliation, and graph interpretation belong to future human/LLM review.

### `kb refs-index`

- **Ability:** build bibliographic index relation candidates between extracted reference entries and local papers.
- **Primary input:** `processing/text/`, `raw/papers/`, and metadata files.
- **Primary output:** `processing/refs/refs_index_*.md`, terminal summary, or JSON.
- **Allows LLM:** no.
- **Deferred work:** candidate, ambiguous, and missing relations require human review; LLMs may help prepare evidence but are not the final guarantee.

### `kb refs-graph`

- **Ability:** export bibliographic index relation candidates as graph data for third-party visualization.
- **Primary input:** `processing/refs/refs_index_*.md`.
- **Primary output:** `processing/refs/refs_graph_*.json`, optionally `.mmd` or `.dot`.
- **Allows LLM:** no.
- **Deferred work:** candidate / ambiguous / missing edges require human review; LLMs may help prepare evidence, but they do not guarantee bibliographic identity.

### `kb keywords`

- **Ability:** detect keyword/topic co-occurrence candidates among extracted text files.
- **Primary input:** `processing/text/`, explicit command-line terms, or a newline-separated terms file.
- **Primary output:** `processing/keywords/keywords_*.md`, terminal summary, or JSON.
- **Allows LLM:** no.
- **Deferred work:** deciding whether shared keywords are scientifically meaningful belongs to a Manager/Worker LLM workflow and should be recorded with `kb memory` if accepted.

### `kb extract-metadata`

- **Ability:** extract basic PDF metadata.
- **Primary input:** `raw/papers/`.
- **Primary output:** metadata logs.
- **Allows LLM:** no.
- **Deferred work:** metadata cleanup, title normalization, and identity reconciliation may become human/LLM tasks.

## Handoff and memory commands

### `kb prepare`

- **Ability:** generate reviewable task materials for wiki work without calling an LLM.
- **Primary input:** `raw/`, `processing/manifest.json`.
- **Primary output:** `processing/prepare_queue.json`, `processing/proposals/prepare_plan_*.md`, and `processing/proposals/prepare_agent_prompt_*.md`.
- **Allows LLM:** no. It prepares materials for external review or later LLM use.
- **Deferred work:** writing scientific summaries and concept pages remains a human/LLM task under Git review.

### `kb tasks`

- **Ability:** consolidate deterministic findings into Manager/Worker handoff lists.
- **Primary input:** `raw/`, `processing/text/`, `processing/refs/`, `wiki/`, static checks.
- **Primary output:** `LLM/tasks/llm_tasks_*.md`.
- **Allows LLM:** no. It prepares tasks for future human/LLM execution.
- **Deferred work:** all generated task groups should include target agent, goal, requirements, file list, and evidence.

### `kb memory`

- **Ability:** append completed task records for later inspection.
- **Primary input:** explicit user-provided task id, summary, files, source task, and follow-up notes.
- **Primary output:** `LLM/memory/completed_tasks.md`.
- **Allows LLM:** no.
- **Deferred work:** it does not validate whether work is truly correct. Human review remains necessary.

## Health and reporting commands

### `kb health`

- **Ability:** summarize deterministic LLM Wiki and literature-relation health.
- **Primary input:** `raw/`, `processing/text/`, `processing/refs/`, `processing/keywords/`, `wiki/`, `LLM/tasks/`, `LLM/memory/`, `outputs/reports/`.
- **Primary output:** `outputs/reports/health_*.md`, terminal summary, or JSON.
- **Allows LLM:** no. It is a dashboard/scouting command for Manager LLM sessions.
- **Deferred work:** missing extraction, uncertain bibliographic index relations, keyword/topic review, and initial wiki drafting become bounded handoff hints. Use `kb tasks` for consolidated `LLM/tasks/` files.

## Interactive shell and model commands

### `kb shell`

- **Ability:** deterministic interactive command shell that repeatedly delegates only known structured commands to batch-mode `kb` semantics. Unknown input is a safe no-op.
- **Primary input:** typed shell commands.
- **Primary output:** terminal results from deterministic commands.
- **Allows LLM:** no hidden LLM behavior.
- **Session commands:** `use`, `pwd`, `clear`, `help`, `exit`.
- **Deferred work:** natural-language chat behavior must be introduced only through a future explicit command or explicit task handoff, not by silently changing `kb>` semantics.
- **Manager LLM role:** use this shell to scout the knowledge base with `query`, `grep`, `refs-index`, `refs-graph`, `keywords`, `health`, and `tasks` before assigning bounded Worker LLM tasks.

### Model configuration commands

- **Commands:** `list-models`, `show-model`, `add-model`, `switch-model`, `delete-model`, `validate-model`.
- **Ability:** manage model settings.
- **Allows LLM:** configuration only. These commands must not make deterministic operations call an LLM implicitly.

## Future command slots

The following command areas are reserved but not implemented in this document:

| Future command | Purpose | Default rule |
|---|---|---|
| explicit LLM / agent commands | Run bounded Worker tasks. | Must be explicit and auditable. |
