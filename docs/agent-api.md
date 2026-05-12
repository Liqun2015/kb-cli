# Agent/API Boundary Notes

This document records what `kb-cli v0.7.7` actually supports. It is intentionally conservative.

## Implemented CLI commands

| Command | Purpose |
|---|---|
| `kb init [--force]` | Create the local LLM Wiki directory structure and generated `rules/` layer. |
| `kb [--source PATH] [--kb-path PATH] ingest [--copy\|--move] [--recursive] [--dry-run\|--preview]` | Organize source files from the source directory into `raw/` subfolders. |
| `kb --source <folder> setup [--recursive] [--copy\|--move] [--dry-run\|--preview]` | Create `<folder>/knowledgebase` by default, then run `init + ingest + extract-metadata + build-wiki`. `kb bootstrap` is a compatibility alias. |
| `kb extract-metadata [--force]` | Extract PDF metadata from `raw/papers/`. |
| `kb build-wiki` | Generate Markdown wiki pages and indexes. |
| `kb status [--dry-run\|--preview] [--json] [--unprocessed]` | Refresh `processing/manifest.json`, print JSON summary, or list unprocessed raw files. |
| `kb prepare [--new\|--file PATH] [--dry-run\|--preview]` | Generate prepare queue/proposal files for future LLM wiki maintenance without calling an LLM. |
| `kb sync-wiki [--dry-run\|--preview] [--json]` | Link source front matter in `wiki/**/*.md` back to manifest entries. |
| `kb lint-static [--dry-run\|--preview\|--no-report] [--json] [--strict]` | Check broken WikiLinks, orphan pages, missing source front matter, empty pages, and duplicate titles. |
| `kb query <terms...> [--limit N] [--snippets N] [--json] [--title-only]` | Search `wiki/**/*.md` with deterministic local keyword matching. |
| `kb links [--unresolved|--ambiguous|--resolved] [--json]` | Scan WikiLinks and deterministic link-resolution hints. |
| `kb grep <pattern> [--path PATH] [--regex] [--json]` | Search text-like files with Rust-native line-level matching. |
| `kb extract-text [--dry-run|--preview] [--force] [--json]` | Extract plain text from supported source files into `processing/text/` with deterministic best-effort methods. |
| `kb refs [--path PATH] [--citations] [--json]` | Scan extracted text for deterministic reference hints such as headings, bibliography entries, DOI values, and optional citation markers. |
| `kb refs-index [--path PATH] [--dry-run|--preview] [--json]` | Build bibliographic index relation candidates between extracted references and local papers. |
| `kb refs-graph [--json|--mermaid|--dot] [--dry-run]` | Export bibliographic relation candidates as graph data for third-party visualization. |
| `kb keywords [terms...] [--terms-file PATH] [--dry-run] [--json]` | Detect keyword/topic co-occurrence candidates among extracted text files. |
| `kb tasks [--dry-run|--preview] [--json] [--index-only|--no-index]` | Generate deferred human/LLM/agent handoffs, `LLM/tasks/index.md`, and bounded `LLM/tasks/items/<task_id>.md` files. |
| `kb memory --task-id ID --summary TEXT` | Append completed-task audit memory under `LLM/memory/completed_tasks.md`. |
| `kb health [--dry-run|--preview] [--json] [--strict]` | Summarize deterministic LLM Wiki and literature-relation health. |
| `kb shell` | Start deterministic interactive shell mode. |
| `kb list-models` | List configured LLM models. |
| `kb show-model` | Show the active model configuration. |
| `kb add-model ...` | Add a model configuration. |
| `kb switch-model <id>` | Switch the active model. |
| `kb delete-model <id>` | Delete a model configuration. |
| `kb validate-model [id]` | Validate local model configuration fields. |


## LLM command guide

When an LLM or Claude Code session uses this project, it should read `docs/llm-command-guide.md` before doing broad inspection or wiki maintenance. That guide explains how deterministic commands should be used as the first scouting pass before semantic LLM work begins.

Do not ask an LLM to infer file lists, keyword matches, reference hints, missing text conversions, broken links, or completed task history when a deterministic command can provide that evidence.

## CLI / shell / LLM boundary

`kb ...` is batch mode. The `kb>` prompt is interactive shell mode. Core knowledge-base commands should have a one-to-one deterministic mapping between the two modes.

`kb>` is not an LLM chat interface. It must not silently interpret free-form natural language as an LLM request. Future LLM behavior must be introduced only through a deliberately designed explicit interface. Do not reserve or advertise placeholder LLM command names inside the deterministic shell before that design exists.

In the current version, `kb shell` is the deterministic shell entry point. It delegates only known structured `kb` commands to batch-mode `kb` semantics and remains outside LLM chat behavior. Unknown input is a safe no-op. See `docs/cli-shell-principle.md` and `docs/shell.md`.

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

## Cross-platform setup

The recommended machine-facing one-command workflow is:

```bash
kb --source /path/to/literature-folder setup --recursive
```

This is cross-platform because the workflow is implemented inside the Rust CLI, not inside a Windows-only batch file.


## Manifest / Status

`kb status` is implemented:

```bash
kb --kb-path /path/to/literature-folder/knowledgebase status
kb --kb-path /path/to/literature-folder/knowledgebase status --json
kb --kb-path /path/to/literature-folder/knowledgebase status --unprocessed
kb --kb-path /path/to/literature-folder/knowledgebase status --dry-run
```

Current behavior:
- Scans files under `raw/`.
- Writes `processing/manifest.json` unless `--dry-run` is used.
- Tracks relative path, source kind, extension, size, SHA-256 content hash, status, first-seen timestamp, last-seen timestamp, and reserved `wiki_pages`.
- Prints human-readable summary output.

The manifest is not a stable external API yet; treat it as a local project file that future commands may evolve carefully.

## Prepare planning

`kb prepare` is implemented as a planning command only. It may write:

```text
processing/prepare_queue.json
processing/proposals/prepare_plan_<timestamp>.md
processing/proposals/prepare_agent_prompt_<timestamp>.md
```

It does not call an LLM, does not edit `wiki/`, and does not mark manifest entries as fully prepared. Treat the generated proposal as a review artifact for a human or future AI maintainer.

## Static wiki lint

`kb lint-static` is implemented as a deterministic local Markdown health check. It does not call an LLM and does not rewrite `wiki/`.

```bash
kb --kb-path /path/to/literature-folder/knowledgebase lint-static
kb --kb-path /path/to/literature-folder/knowledgebase lint-static --no-report
kb --kb-path /path/to/literature-folder/knowledgebase lint-static --json
kb --kb-path /path/to/literature-folder/knowledgebase lint-static --strict
```

Reports are written under `outputs/reports/` unless `--dry-run`, `--preview`, or `--no-report` is used. `--no-report` is specific to `lint-static`.


## Query skeleton

`kb query` is implemented in v0.5.0 as deterministic local keyword search over `wiki/**/*.md`.

```bash
kb --kb-path /path/to/literature-folder/knowledgebase query thermal cloak
kb --kb-path /path/to/literature-folder/knowledgebase query thermal cloak --limit 5
kb --kb-path /path/to/literature-folder/knowledgebase query thermal cloak --json
kb --kb-path /path/to/literature-folder/knowledgebase query thermal cloak --title-only
```

Current behavior:
- Scans Markdown files under `wiki/`.
- Matches query terms with AND semantics.
- Searches page titles, relative paths, and Markdown bodies by default.
- Prints ranked local results with snippets.
- Does not call an LLM, use embeddings, build a vector index, or perform RAG.



## Command classification

Every command must belong to a documented category. See `docs/command-classification.md`.

The current categories are:

1. Deterministic core commands
2. Deterministic search / inspection commands
3. Text conversion / source processing commands
4. Task preparation commands
5. Explicit future LLM / agent commands

Commands must not silently cross these boundaries. Deterministic commands should reduce unnecessary LLM work and must not hide LLM calls.

## Grep and extract-text

`kb grep` is Rust-native line-level text search. It does not call external `grep` or `rg`.

`kb extract-text` is deterministic best-effort text extraction. It does not call OCR or an LLM by default. It is reserved as a future PDF Text Conversion Agent entry point, but any OCR/LLM/agent behavior must be explicit and opt-in.


## Deferred LLM/agent skills

Some deterministic commands are intentionally designed to expose the boundary between mechanical processing and semantic work.

- `kb extract-text` may produce weak or empty text for scanned, image-based, or layout-broken PDFs. OCR, layout repair, and LLM cleanup are future explicit PDF Text Conversion Agent skills.
- `kb refs` finds reference hints but does not reconcile references, normalize metadata, or build a reliable citation graph. Those are future explicit Reference Reconciliation / Citation Graph Building skills.
- `kb grep` finds lines; it does not interpret them semantically.

See `docs/llm-agent-skills.md`. These skills must remain explicit and reviewable.

## Current non-goals

Do not assume the following exist in `v0.5.10`:

```text
global --format json
autonomous prepare execution
semantic LLM lint
vector search
full RAG
semantic retrieval
saved query answers
add-paper
list-papers
list-notes
networked agent API
long-running background daemon
```

## Safety behavior

`ingest` and `setup` default to copy mode unless `--move` is explicitly provided. Recursive mode skips generated/project folders such as:

```text
raw, wiki, processing, references, topics, outputs, logs, knowledgebase, .git, .obsidian, target, node_modules
```

For automation, prefer running a dry run first:

```bash
kb --source /path/to/literature-folder setup --recursive --dry-run
```

Then run the real command:

```bash
kb --source /path/to/literature-folder setup --recursive
```

## Output style

`kb status --json` provides a machine-readable summary. Other command outputs remain human-readable and should not be treated as a stable JSON API.


## sync-wiki

Use `kb sync-wiki --dry-run` or `kb sync-wiki --preview` after wiki edits to preview manifest updates, then `kb sync-wiki` to record source-to-wiki links.


## Deferred task handoff

`kb tasks` is a deterministic management command. It gathers unresolved work from simple checks and emits a reviewable handoff list, a Manager LLM dashboard at `LLM/tasks/index.md`, and bounded Worker task items under `LLM/tasks/items/`.

Each task group should include:

- target agent
- goal
- requirements
- file list
- evidence
- source command
- priority

It must not execute the deferred work. It must not call an LLM, run OCR, rewrite Wiki pages, build citation graphs, or invent missing source information.


## LLM role hierarchy

Future integrations should distinguish the Manager LLM from Worker LLMs.

- The Manager LLM is the highest-level LLM. It may invoke deterministic `kb` commands, inspect outputs, generate task handoffs, delegate bounded work, and record completion memory.
- Worker LLMs receive scoped tasks with explicit file lists and requirements. They should not decide global workflow or silently expand scope.
- Deterministic `kb` commands remain Rust-native tools and must not secretly call LLM APIs.

See `docs/llm-hierarchy.md`.


## WikiLink inspection handoff

Use `kb links` for deterministic WikiLink extraction and resolution hints. The command may return deferred task hints for unresolved or ambiguous links, but it must not repair links automatically or call an LLM. A Manager LLM may use this output to assign bounded work to a Wiki Link Repair Agent.


## `kb refs-index` output boundary

`kb refs-index` is available as a deterministic candidate builder for bibliographic index relations. Agent integrations should treat its output as evidence, not as a final citation graph. Candidate, ambiguous, and missing rows require review and may be routed through `LLM/tasks/` or a Manager LLM workflow.


## Third-party graph tools

Third-party graph skills and tools should use `docs/third-party-skills/` as the guidance source for relationship status, visual style, JSON fields, evidence preservation, and human-review requirements.

Confirmed bibliographic index relations should render as solid edges. Candidate and ambiguous relations should render as dashed edges. Missing or unresolved references should render as hollow nodes.


## `kb refs-graph`

`kb refs-graph` exports bibliographic index relation candidates into graph data for third-party skills. It is deterministic and must not call an LLM or confirm uncertain citation identities.


## `kb keywords`

`kb keywords` detects keyword/topic co-occurrence candidates among extracted text files. It is deterministic and must not call an LLM or infer scientific idea relations.

Manager LLMs may use its output to assign bounded Worker LLM tasks for semantic review. Worker LLMs must keep evidence lines and file lists intact.


## `kb health`

`kb health` provides a deterministic relationship-health dashboard for Manager LLM sessions and review tools. It reads existing project directories and reports, then emits status, counts, checks, and deferred task hints. It does not execute Worker tasks or call an LLM.


## Topic-specific relationship overlays

Global bibliographic index relations stay under `processing/refs/`. Topic-specific causal, method, evidence, idea, and importance relations belong under `topics/<topic>/`.

Do not ask a Worker LLM to store topic-level interpretation in `processing/refs/`. If a topic-specific relation is uncertain, create or update a bounded task under `topics/<topic>/tasks/` or `LLM/tasks/`, and record accepted decisions under `topics/<topic>/memory/` or `LLM/memory/`.

## Topic schema notes

As of `v0.6.3`, topic-specific relationship records should follow:

```text
docs/topic-relation-schema.md
docs/literature-importance-schema.md
docs/third-party-skills/topic-graph-schema.md
```

These documents are schema contracts for future Manager LLM, Worker LLM, and third-party skill integrations. They are not executable APIs yet.


## Static viewer boundary

`kb view` is a static display command. It renders existing Markdown/JSON outputs into `outputs/html/index.html` and opens that static file in the system default browser by default. `kb view --no-open` refreshes the file without opening a browser. It must not call an LLM, start a local server, execute shell commands from the browser, or modify source/wiki files. The `kb-view>` box inside the generated HTML is display navigation only.


## Topic workspace command

`kb topic init <topic>` creates deterministic topic workspace scaffolding under `topics/<topic>/`. It is safe for Manager LLM sessions to request this command when a concrete topic needs a local relationship overlay. It does not call an LLM, infer relations, rank literature, or modify global bibliographic index files under `processing/refs/`.


## Topic workspace inspection commands

`kb topic list` and `kb topic status <topic>` are deterministic structure checks for `topics/<topic>/`. They may be used by Manager LLMs before assigning Worker LLM topic-relation tasks. They must not call LLM APIs, infer relationships, rank literature, or modify topic files.


## Topic rank handoff

`kb topic rank <topic>` produces topic-local importance candidates under `topics/<topic>/importance/`. Agents must not treat these labels as final unless they also appear in `confirmed_importance.md` or have explicit human-review evidence.
