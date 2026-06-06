# Command Overview

This document is the command map for `kb-cli`. It is meant for human maintainers, Manager LLM sessions, Worker LLM tasks, and future third-party skills.

The guiding rule is simple:

> Every command must have one clear ability. When a command cannot safely finish a semantic task, it should report evidence and defer the hard part to `LLM/tasks/` or human review.

## Interface Directory Boundary

`interfaces/` is the single directory for generated, non-source interface artifacts used for human-machine review, machine-machine handoff, diagnostics, and generated review pages. It is not the durable source of truth. See `docs/interface-boundary.md`.


## Command categories

| Category | Commands | Default LLM behavior | Purpose |
|---|---|---:|---|
| Deterministic core | `init`, `ingest`, `setup`, `build`, `status`, `sync-wiki`, `lint-static`, `build-wiki` | No LLM | Maintain the file structure, manifest, wiki skeleton, topic preparation pipeline, and static checks. |
| Search / inspection | `query`, `grep`, `links` | No LLM | Locate pages, lines, and WikiLink relations for later review. |
| Text / reference processing | `extract-text`, `extract-sections`, `refs`, `refs-index`, `refs-graph`, `keywords`, `extract-metadata` | No LLM | Convert source traces into searchable text, section-level evidence, bibliographic relation candidates, and keyword/topic relation evidence. |
| Handoff / task closure | `tasks`, `task`, `prompt`, `batch`, `handoff`, `memory` | No LLM | Prepare reviewable work items, mark task progress, print bounded Worker prompts, create small batches, and record accepted work. |
| Health / reporting | `health` | No LLM | Summarize relationship-network completeness, review backlog, and pipeline health. |
| Proof audit | `audit-wiki` | Explicit LLM handoff | Check whether the folder has become a reviewable LLM Wiki and write an LLM audit prompt. |
| System check | `check` | No LLM | Generate local read-only HTML check dashboards from Markdown/JSON outputs. |
| User knowledge portal | `view` | No LLM | Generate the reader-facing LLM Wiki portal from `wiki/` pages. |
| Review writing | `review` | Reserved | Reserved entry for the future literature-review writing workflow. |
| Interactive shell | `shell` | No hidden LLM | Deterministic interactive command shell; not a chat interface. |
| Model configuration | `list-models`, `show-model`, `add-model`, `switch-model`, `delete-model`, `validate-model` | Configuration only | Manage model configuration without turning deterministic commands into implicit LLM calls. |

## Core commands

### `llm-wiki.toml`

- **Role:** root marker file for an initialized local LLM Wiki workspace.
- **Created by:** `kb init` and `kb setup`.
- **Typical location:** `<source>/knowledgebase/llm-wiki.toml`.
- **Purpose:** records `type = "llm-wiki"`, layout directories, source boundary, and the default policy that LLM agents should edit inside the workspace only.


### `kb init`

- **Ability:** create or ensure the local knowledge-base directory structure.
- **Primary input:** knowledge-base path.
- **Primary output:** `llm-wiki.toml`, `raw/`, `wiki/`, `processing/`, `rules/`, `LLM/tasks/`, `LLM/memory/`, and related directories.
- **Allows LLM:** no.
- **Deferred work:** none. Directory creation is deterministic.

### `kb ingest`

- **Ability:** organize source files into `raw/` subfolders and register them for later processing.
- **Primary input:** files under the source directory; with `--source`, files may live outside the knowledge base workspace.
- **Primary output:** `raw/papers/`, `raw/notes/`, `raw/images/`, `raw/datasets/`, `processing/manifest.json`.
- **Allows LLM:** no.
- **Deferred work:** ambiguous file meaning, paper interpretation, and semantic classification should be deferred to later tasks.

### `kb setup`

- **Ability:** run the safe one-command setup path: `init -> ingest -> extract-metadata -> build-wiki`.
- **Primary input:** a source-material directory; by default `kb --source <folder> setup` creates `<folder>/knowledgebase` as the workspace.
- **Primary output:** a `knowledgebase/` workspace containing `llm-wiki.toml`, initialized folders, organized `raw/` materials, refreshed metadata, and wiki skeleton pages.
- **Allows LLM:** no.
- **Deferred work:** semantic synthesis, citation reconciliation, OCR/layout repair, and relationship judgment remain explicit LLM/human review tasks.
- **Compatibility:** `kb bootstrap` remains available as an older alias, but documentation should prefer `kb setup`.

### `kb create --wiki <A> --from <B> --about <topic>`

- **Ability:** turn an existing literature/source-material folder into a complete, named, agent-ready LLM Wiki database in one command.
- **Equivalent stages:** `kb --wiki <A> --source <B> setup --recursive --topic <topic> -> kb --wiki <A> build <topic>`.
- **Primary input:** an existing literature folder supplied with `--from <B>`.
- **Primary output:** the target LLM Wiki database supplied with `--wiki <A>`, with source files organized under `raw/`, deterministic evidence under `processing/`, topic workbench files under `topics/<topic>/`, agent handoff files, and human/agent interface files under `interfaces/`.
- **Allows LLM:** no. This command still only runs deterministic setup/build stages.
- **Deferred work:** OCR repair, paper-card semantic completion, relation confirmation, concept-page writing, and synthesis remain external-agent or human-review tasks.
- **Useful options:** `--wiki`, `--from`, `--about`, `--move`, `--force`, `--dry-run`, `--limit`.

Examples:

```bash
kb create --wiki D:\LLM-wiki\thermal --from D:\Literatures\thermal-metamaterials --about thermal-metamaterials
kb create --wiki D:\LLM-wiki\thermal --from D:\Literatures\thermal-metamaterials --about thermal-metamaterials --dry-run
```


### `kb build <topic>`

- **Ability:** run the post-setup deterministic preparation pipeline for one topic.
- **Equivalent stages:** `extract-text -> extract-sections -> refs -> refs-index -> refs-graph -> topic build <topic> -> topic review <topic> -> topic tasks <topic> -> handoff --all-agents -> topic handoff <topic> --all-agents`.
- **Primary input:** an initialized or setup LLM Wiki workspace with source materials under `raw/`.
- **Primary output:** `processing/text/`, `processing/sections/`, `processing/refs/`, `topics/<topic>/importance/`, `topics/<topic>/review/`, `topics/<topic>/tasks/`, `AGENTS.md`, `obsidian_main.md`, `LLM/handoff/`, and topic-local handoff files.
- **Allows LLM:** no. This is a convenience pipeline, not an embedded LLM runner.
- **Deferred work:** scholarly confirmation, semantic synthesis, paper-card drafting, concept-page writing, and OCR/layout repair remain explicit external-agent or human tasks.
- **Useful options:** `--dry-run`, `--force`, `--skip-extract`, `--skip-refs`, `--skip-review`, `--skip-tasks`, `--skip-handoff`, `--limit`, `--title`, `--no-init`.
- **Alias:** `kb assemble <topic>`.

Example:

```bash
kb build thermal-metamaterials
kb build thermal-metamaterials --force
kb build thermal-metamaterials --dry-run
```

### `kb status`

- **Ability:** inspect `raw/` and manifest status.
- **Primary input:** `raw/`, `processing/manifest.json`.
- **Primary output:** terminal summary or JSON.
- **Allows LLM:** no.
- **Deferred work:** unprocessed raw files can later be turned into explicit `tasks` or topic task items.

### `kb build-wiki`

- **Ability:** build deterministic wiki skeleton pages from local metadata and raw materials.
- **Primary input:** `raw/`, `interfaces/logs/papers_metadata.json`, `processing/manifest.json`.
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
- **Primary output:** terminal summary, JSON, or `interfaces/reports/lint_static_*.md`.
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
- **Primary output:** metadata artifacts.
- **Allows LLM:** no.
- **Deferred work:** metadata cleanup, title normalization, and identity reconciliation may become human/LLM tasks.

## Handoff and memory commands

### `kb tasks`

- **Ability:** consolidate deterministic findings into Manager/Worker handoff lists and refresh the Manager LLM task dashboard.
- **Primary input:** `raw/`, `processing/text/`, `processing/sections/`, `processing/refs/`, `wiki/`, `topics/*/review/`, static checks.
- **Primary output:** `LLM/tasks/index.md`, `LLM/tasks/items/<task_id>.md`, and `LLM/tasks/llm_tasks_*.md`.
- **Allows LLM:** no. It prepares tasks for future human/LLM execution.
- **Deferred work:** all generated task groups should include target agent, goal, requirements, file list, and evidence. The Manager LLM starts from `LLM/tasks/index.md`; Worker LLMs receive bounded files under `LLM/tasks/items/`.
- **Useful modes:** `--index-only` refreshes the Manager/Worker task dashboard without writing a new timestamped snapshot; `--no-index` preserves the older report-only behavior.

### `kb task`

- **Ability:** list, show, and mark Worker/Human task progress so a Manager LLM can resume after restart.
- **Primary output:** `LLM/tasks/state/<task_id>.json` and `LLM/tasks/status_events.jsonl`.
- **Typical commands:**

```bash
kb --wiki <A> task list
kb --wiki <A> task show paper-key-info-extraction-001
kb --wiki <A> task status paper-key-info-extraction-001 --mark assigned --assignee openclaw
kb --wiki <A> task status paper-key-info-extraction-001 --mark completed --note "accepted after review"
```

### `kb prompt`

- **Ability:** print reusable prompt templates for common Worker/Human tasks.
- **Typical commands:**

```bash
kb --wiki <A> prompt paper-profile --task <task-id> --topic <topic>
kb --wiki <A> prompt topic-narrative --topic <topic>
kb --wiki <A> prompt relation-review --task <task-id>
kb --wiki <A> prompt human-review --task <task-id>
```

### `kb batch`

- **Ability:** create a bounded small Worker batch instead of dispatching many papers one by one.
- **Primary output:** `LLM/tasks/batches/<batch-id>.md`.
- **Typical command:**

```bash
kb --wiki <A> batch paper-profile --topic <topic> --limit 5
```

### `kb memory`

- **Ability:** append completed task records for later inspection.
- **Primary input:** explicit user-provided task id, summary, files, source task, and follow-up notes.
- **Primary output:** `LLM/memory/completed_tasks.md`.
- **Allows LLM:** no.
- **Deferred work:** it does not validate whether work is truly correct. Human review remains necessary.

## Health and reporting commands

### `kb health`

- **Ability:** summarize deterministic LLM Wiki and literature-relation health.
- **Primary input:** `raw/`, `processing/text/`, `processing/refs/`, `processing/keywords/`, `wiki/`, `LLM/tasks/`, `LLM/memory/`, `interfaces/reports/`.
- **Primary output:** `interfaces/reports/health_*.md`, terminal summary, or JSON.
- **Allows LLM:** no. It is a dashboard/scouting command for Manager LLM sessions.
- **Deferred work:** missing extraction, missing Introduction/References sections, uncertain bibliographic index relations, keyword/topic review, and initial wiki drafting become bounded handoff hints. Use `kb tasks` for consolidated `LLM/tasks/` files.

### `kb audit-wiki`

- **Ability:** audit whether the local folder has moved from a material pile to a reviewable, human/AI-maintainable Markdown LLM Wiki.
- **Primary input:** `raw/`, `rules/`, `processing/manifest.json`, `processing/text/`, `processing/refs/`, `wiki/`, `topics/`, `LLM/tasks/`, `LLM/memory/`, and `interfaces/`.
- **Primary output:** `interfaces/reports/wiki_audit_*.md`; unless `--no-llm-prompt` is used, also `LLM/tasks/wiki_audit_llm_review_*.md`.
- **Allows LLM:** not as a hidden call. It generates an explicit bounded LLM audit prompt.
- **Deferred work:** a Manager LLM or human reviewer should use the generated prompt to independently evaluate the proof chain.

## Interactive shell and model commands

### `kb check` / `kb check --relations`

- **Ability:** generate static local HTML viewers for generated LLM Wiki artifacts.
- **Regular dashboard:** `kb check` writes `interfaces/html/index.html`.
- **Topic overview mode:** `kb check --relations` writes `interfaces/html/relationship_viewer.html` and `interfaces/html/relationship_data.json` as a topic index for all workspaces under `topics/`.
- **Single-topic graph mode:** `kb check --relations --topic <topic>` writes a filtered relation graph for only the requested topic. It does not keep unrelated global/topic data in the JSON.
- **Data-only export:** `kb check --relations --data-only` writes only `relationship_data.json`.
- **Primary input:** regular dashboard reads `wiki/`, `processing/refs/`, `processing/keywords/`, `interfaces/reports/`, `LLM/tasks/`, and `LLM/memory/`; relation mode reads `topics/` and, when `--topic` is provided, the selected topic workspace.
- **Allows LLM:** no hidden call. The regular dashboard includes a safe **About launching LLM** launch panel with copyable external-agent commands/prompts, but the viewer itself never starts an LLM, executes shell commands, or modifies files.
- **Deferred work:** relationship edges marked candidate, ambiguous, missing, or `needs_llm_review` are review inputs for Manager/Worker LLM workflows. The viewer itself is display-only and must not execute local commands or interpret natural language.
- **Boundary:** do not add a separate `kb check-relations` command; topic relationship viewing belongs under `kb check --relations`.


### `kb view`

- **Ability:** generate the user-facing LLM Wiki knowledge portal.
- **Output:** `interfaces/html/browse.html`.
- **Primary input:** reviewed or draft Markdown pages under `wiki/`, including WikiLinks and image references.
- **Allows LLM:** no hidden call. It only renders existing knowledge pages.
- **Boundary:** do not show low-level task status, JSON, agent handoff details, or workflow internals here. Those belong to `kb check`.

### `kb review`

- **Ability:** reserved command for the future literature-review writing workflow.
- **Current behavior:** prints a placeholder and writes no files.
- **Boundary:** do not silently synthesize a review article until the review workflow has an explicit schema, inputs, outputs, and human-review gates.

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
- **Current subcommands:** `kb topic init <topic>`, `kb topic list`, `kb topic status <topic>`, `kb topic rank <topic>`, `kb topic relations <topic>`, `kb topic build <topic>`, `kb topic review <topic>`, `kb topic tasks <topic>`, and `kb topic handoff <topic>`.
- **Primary input:** topic name or slug.
- **Primary output:** `topics/<topic>/` workspace files, topic-local reports under `topics/<topic>/importance/`, topic graph artifacts under `topics/<topic>/graph/`, review queues under `topics/<topic>/review/`, and topic task files under `topics/<topic>/tasks/` and topic handoff files under `topics/<topic>/handoff/`.
- **LLM allowed:** no. This command only creates deterministic scaffolding.
- **Deferred work:** Manager LLM / Worker LLM / human reviewers fill and review topic-local importance and relation records later. `kb topic rank` produces importance candidates, `kb topic relations` compiles directed relation records into graph artifacts, `kb topic build` runs both, `kb topic review` turns candidates into a queue, `kb topic tasks` writes bounded topic-local task files, and `kb topic handoff` writes generic external-agent entry files plus optional adapters. None of these commands decides final scholarly claims.

Example:

```bash
kb topic init thermal-metamaterials
kb topic init thermal-metamaterials --title "Thermal Metamaterials"
kb topic list
kb topic status thermal-metamaterials
kb topic rank thermal-metamaterials --dry-run
kb topic relations thermal-metamaterials --dry-run
kb topic build thermal-metamaterials --dry-run
kb topic review thermal-metamaterials --dry-run
kb topic tasks thermal-metamaterials --dry-run
kb topic handoff thermal-metamaterials --dry-run
```

### `kb topic rank <topic>`

- **Ability:** generate deterministic topic-local literature importance candidates.
- **Primary input:** `topics/<topic>/literature.md`, topic-local Markdown files, and weak global references from `processing/refs/refs_index_*.md`.
- **Primary output:** `topics/<topic>/importance/importance_candidates_*.md`, terminal summary, or JSON.
- **Allows LLM:** no.
- **Deferred work:** core/important labels require human review before they influence graph node size or research conclusions.
- **Layer rule:** importance is topic-specific, so rank/review outputs belong under `topics/<topic>/`, not under global `processing/` folders.

### `kb topic relations <topic>`

- **Ability:** compile topic-local directed relation records into graph artifacts.
- **Primary input:** `topics/<topic>/literature.md`, `topics/<topic>/importance/*.md`, and `topics/<topic>/relations/*.md`.
- **Primary output:** `topics/<topic>/graph/topic_graph.json` and `topics/<topic>/graph/topic_graph.md`.
- **Allows LLM:** no.
- **Layer rule:** topic-local paper-to-paper relation rows stay under `topics/<topic>/relations/`; the generated graph stays under `topics/<topic>/graph/`.

Example:

```bash
kb topic relations thermal-metamaterials
kb topic relations thermal-metamaterials --dry-run
kb topic relations thermal-metamaterials --json
```

### `kb topic build <topic>`

- **Ability:** run `kb topic rank <topic>` and `kb topic relations <topic>` as one topic-build command.
- **Primary input:** the topic workspace under `topics/<topic>/`.
- **Primary output:** importance candidate reports plus topic graph artifacts.
- **Allows LLM:** no.
- **Default behavior:** safely initializes a missing topic workspace without overwriting existing files. Use `--no-init` to require a pre-existing workspace.

Example:

```bash
kb topic build thermal-metamaterials
kb topic build thermal-metamaterials --limit 25
kb topic build thermal-metamaterials --dry-run
kb topic build thermal-metamaterials --json
kb check --relations --topic thermal-metamaterials
```



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


### `kb topic tasks <topic>`

- **Ability:** generate topic-local Manager/Worker LLM handoff tasks.
- **Visible alias:** `kb topic task <topic>`.
- **Primary input:** one topic workspace under `topics/<topic>/`.
- **Primary output:** `topics/<topic>/tasks/index.md` and `topics/<topic>/tasks/items/<task_id>.md`.
- **Allows LLM:** no. It prepares bounded task files for later explicit LLM/human execution.
- **Detected work:** missing scope definition, incomplete literature curation, missing importance generation, missing importance review queue, missing relation authoring, uncertain or weak-evidence relation rows, missing graph exports, and synthesis opportunities.
- **Boundary:** generated tasks are routing artifacts. They do not execute agents, infer relations, accept importance labels, or rewrite wiki pages.

Example:

```bash
kb topic tasks thermal-metamaterials
kb topic task thermal-metamaterials
kb topic tasks thermal-metamaterials --dry-run
kb topic tasks thermal-metamaterials --json
kb topic tasks thermal-metamaterials --limit 3
```


## `kb handoff`

Generate generic external-agent handoff files, with optional agent-specific adapters. The default result is tool-neutral and starts from `AGENTS.md`; Obsidian users start from `obsidian_main.md`; Manager agents then route through `LLM/handoff/current.md`.

```bash
kb handoff
kb handoff --topic thermal-metamaterials
kb handoff --all-topics
kb handoff --agent claude-code
kb handoff --agent opencode
kb handoff --agent openclaw
kb handoff --agent codex
kb handoff --all-agents
kb handoff --topic thermal-metamaterials --force
```

Default project/global files:

```text
AGENTS.md
obsidian_main.md
LLM/handoff/current.md
LLM/handoff/index.md
LLM/handoff/protocol.md
LLM/handoff/manager.md
LLM/handoff/worker.md
LLM/handoff/task_schema.md
LLM/handoff/safety.md
LLM/handoff/handoff.json
```

When a topic is provided, also writes generic topic files:

```text
topics/<topic>/index.md
topics/<topic>/handoff/AGENTS.md
topics/<topic>/handoff/protocol.md
topics/<topic>/handoff/manager.md
topics/<topic>/handoff/worker_task_template.md
topics/<topic>/handoff/start_prompt.md
topics/<topic>/handoff/handoff.json
```

Adapter files are generated only when requested:

```text
CLAUDE.md
LLM/handoff/adapters/claude-code.md
LLM/handoff/adapters/opencode.md
LLM/handoff/adapters/openclaw.md
LLM/handoff/adapters/codex.md
topics/<topic>/handoff/adapters/<agent>.md
```

## `kb topic handoff <topic>`

Generate topic-local generic handoff files without regenerating the project-level files. Use `--agent` or `--all-agents` to add topic-specific adapters.

```bash
kb topic handoff thermal-metamaterials
kb topic handoff thermal-metamaterials --agent claude-code
kb topic handoff thermal-metamaterials --agent opencode
kb topic handoff thermal-metamaterials --agent openclaw
kb topic handoff thermal-metamaterials --agent codex
kb topic handoff thermal-metamaterials --all-agents
kb topic handoff thermal-metamaterials --force
kb topic handoff thermal-metamaterials --dry-run
```

## Topic defaults during build

When converting a literature directory:

```bash
kb --source /path/to/literatures setup --recursive
```

If `--topic` is omitted, the source directory name becomes the default topic.

When initializing a blank database:

```bash
kb --wiki MyKnowledgeBase init
```

If `--topic` is omitted, the database directory name becomes the default topic. Running `kb init` without `--wiki` now exits with a prompt instead of silently creating `knowledgebase/`.
