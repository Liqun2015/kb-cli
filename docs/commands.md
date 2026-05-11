# Command Overview

This document is the command map for `kb-cli`. It is meant for human maintainers, Manager LLM sessions, Worker LLM tasks, and future third-party skills.

The guiding rule is simple:

> Every command must have one clear ability. When a command cannot safely finish a semantic task, it should report evidence and defer the hard part to `LLM/tasks/` or human review.

## Command categories

| Category | Commands | Default LLM behavior | Purpose |
|---|---|---:|---|
| Deterministic core | `init`, `ingest`, `bootstrap`, `status`, `sync-wiki`, `lint-static`, `build-wiki` | No LLM | Maintain the file structure, manifest, wiki skeleton, and static checks. |
| Search / inspection | `query`, `grep`, `links` | No LLM | Locate pages, lines, and WikiLink relations for later review. |
| Text / reference processing | `extract-text`, `extract-sections`, `refs`, `refs-index`, `refs-graph`, `keywords`, `extract-metadata` | No LLM | Convert source traces into searchable text, section-level evidence, bibliographic relation candidates, and keyword/topic relation evidence. |
| Handoff / memory | `prepare`, `tasks`, `memory` | No LLM | Prepare reviewable work items and record completed work. |
| Health / reporting | `health` | No LLM | Summarize relationship-network completeness, review backlog, and pipeline health. |
| Proof audit | `audit-wiki` | Explicit LLM handoff | Check whether the folder has become a reviewable LLM Wiki and write an LLM audit prompt. |
| Static viewing | `view` | No LLM | Generate a local read-only HTML dashboard from Markdown/JSON outputs. |
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

### `kb extract-sections`

- **Ability:** slice `Introduction` and `References` from extracted paper text.
- **Primary input:** `processing/text/` or a text file passed with `--path`.
- **Primary output:** `processing/sections/<source-key>/introduction.txt`, `references.txt`, and `section_manifest.md`.
- **Allows LLM:** no by default.
- **Deferred work:** scanned/image-based PDFs, bad OCR, missing headings, and ambiguous section boundaries should become explicit `Paper Section OCR/LLM Worker` tasks.
- **Why it matters:** Introduction and References are high-value inputs for building literature relationships among papers.

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

- **Ability:** export bibliographic index relation candidates as directed V2 graph data for third-party visualization.
- **Primary input:** `processing/refs/refs_index_*.md`.
- **Primary output:** `processing/refs/refs_graph_*.json`, optionally `.mmd` or `.dot`; JSON includes `graph_kind`, `direction_model`, directed edge fields, evidence, review flags, and placeholder node weights.
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

- **Ability:** consolidate deterministic findings into Manager/Worker handoff lists and refresh the Manager LLM task dashboard.
- **Primary input:** `raw/`, `processing/text/`, `processing/sections/`, `processing/refs/`, `wiki/`, `topics/*/review/`, `processing/proposals/`, static checks.
- **Primary output:** `LLM/tasks/index.md`, `LLM/tasks/items/<task_id>.md`, and `LLM/tasks/llm_tasks_*.md`.
- **Allows LLM:** no. It prepares tasks for future human/LLM execution.
- **Deferred work:** all generated task groups should include target agent, goal, requirements, file list, and evidence. The Manager LLM starts from `LLM/tasks/index.md`; Worker LLMs receive bounded files under `LLM/tasks/items/`.
- **Useful modes:** `--index-only` refreshes the Manager/Worker task dashboard without writing a new timestamped snapshot; `--no-index` preserves the older report-only behavior.

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
- **Deferred work:** missing extraction, missing Introduction/References sections, uncertain bibliographic index relations, keyword/topic review, and initial wiki drafting become bounded handoff hints. Use `kb tasks` for consolidated `LLM/tasks/` files.

### `kb audit-wiki`

- **Ability:** audit whether the local folder has moved from a material pile to a reviewable, human/AI-maintainable Markdown LLM Wiki.
- **Primary input:** `raw/`, `rules/`, `processing/manifest.json`, `processing/text/`, `processing/refs/`, `wiki/`, `topics/`, `LLM/tasks/`, `LLM/memory/`, and `outputs/`.
- **Primary output:** `outputs/reports/wiki_audit_*.md`; unless `--no-llm-prompt` is used, also `LLM/tasks/wiki_audit_llm_review_*.md`.
- **Allows LLM:** not as a hidden call. It generates an explicit bounded LLM audit prompt.
- **Deferred work:** a Manager LLM or human reviewer should use the generated prompt to independently evaluate the proof chain.

## Interactive shell and model commands

### `kb view` / `kb view --relations`

- **Ability:** generate static local HTML viewers for generated LLM Wiki artifacts.
- **Regular dashboard:** `kb view` writes `outputs/html/index.html`.
- **Relationship review mode:** `kb view --relations` writes `outputs/html/relationship_viewer.html` and `outputs/html/relationship_data.json`.
- **Topic focus:** `kb view --relations --topic <topic>` keeps global data but sets the requested topic as the default focus when available.
- **Data-only export:** `kb view --relations --data-only` writes only `relationship_data.json`.
- **Primary input:** `wiki/`, `processing/refs/`, `processing/keywords/`, `outputs/reports/`, `LLM/tasks/`, `LLM/memory/`, and `topics/`.
- **Allows LLM:** no.
- **Deferred work:** relationship edges marked candidate, ambiguous, missing, or `needs_llm_review` are review inputs for Manager/Worker LLM workflows. The viewer itself is display-only and must not execute local commands or interpret natural language.
- **Boundary:** do not add a separate `kb view-relations` command; relationship graph viewing belongs under `kb view --relations`.

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


## Topic relationship overlay roadmap

`topics/<topic>/` is reserved for topic-specific literature relationship overlays. This is not a replacement for `processing/refs/`.

```text
processing/refs/  global bibliographic index relations
topics/<topic>/   topic-local causal, method, evidence, idea, and importance relations
```

Before adding topic-specific commands, read `docs/topic-relationships.md`. `kb topic init` is implemented as deterministic scaffolding. Future commands such as `kb topic graph` should remain deterministic commands that help Manager LLMs and humans maintain topic overlays without silently making causal or scientific-idea claims.

## Topic schema foundation added in v0.6.3

`v0.6.3` does not add a new executable command. It adds schema documents that constrain future topic-level commands and Worker LLM tasks:

```text
docs/topic-relation-schema.md
docs/literature-importance-schema.md
docs/topic-v2-roadmap.md
docs/third-party-skills/topic-graph-schema.md
```

Future commands such as `kb topic`, `kb relation-review`, or topic graph exporters should follow these schemas instead of inventing new relation fields ad hoc.


## `kb topic`

- **Category:** topic-specific relationship workspace command.
- **Current subcommands:** `kb topic init <topic>`, `kb topic list`, `kb topic status <topic>`, `kb topic rank <topic>`, and `kb topic review <topic>`.
- **Primary input:** topic name or slug.
- **Primary output:** `topics/<topic>/` workspace files, topic-local reports under `topics/<topic>/importance/`, and review queues under `topics/<topic>/review/`.
- **LLM allowed:** no. This command only creates deterministic scaffolding.
- **Deferred work:** Manager LLM / Worker LLM / human reviewers fill and review topic-local importance and relation records later. `kb topic rank` produces candidates only; `kb topic review` turns those candidates into a queue only. Neither command decides final importance.

Example:

```bash
kb topic init thermal-metamaterials
kb topic init thermal-metamaterials --title "Thermal Metamaterials"
kb topic list
kb topic status thermal-metamaterials
kb topic rank thermal-metamaterials --dry-run
kb topic review thermal-metamaterials --dry-run
```

### `kb topic rank <topic>`

- **Ability:** generate deterministic topic-local literature importance candidates.
- **Primary input:** `topics/<topic>/literature.md`, topic-local Markdown files, and weak global references from `processing/refs/refs_index_*.md`.
- **Primary output:** `topics/<topic>/importance/importance_candidates_*.md`, terminal summary, or JSON.
- **Allows LLM:** no.
- **Deferred work:** core/important labels require human review before they influence graph node size or research conclusions.
- **Layer rule:** importance is topic-specific, so rank/review outputs belong under `topics/<topic>/`, not under global `processing/` folders.


### `kb topic review <topic>`

- **Ability:** build a deterministic review queue from topic-local importance candidates.
- **Primary input:** candidate files under `topics/<topic>/importance/`.
- **Primary output:** `topics/<topic>/review/review_queue.md` and `topics/<topic>/review/review_summary.md`.
- **Allows LLM:** no.
- **Overwrite rule:** existing queue and summary files are not overwritten unless `--force` is used.
- **Layer rule:** reviewed decisions belong under `topics/<topic>/reviewed/`; this command does not write accepted/rejected/deferred decisions.

Example:

```bash
kb topic review thermal-metamaterials
kb topic review thermal-metamaterials --dry-run
kb topic review thermal-metamaterials --json
kb topic review thermal-metamaterials --force
```

`kb topic review` does not perform scientific judgment. Every generated row remains in `candidate` status.
