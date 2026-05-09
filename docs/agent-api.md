# Agent/API Boundary Notes

This document records what `kb-cli v0.5.10` actually supports. It is intentionally conservative.

## Implemented CLI commands

| Command | Purpose |
|---|---|
| `kb init [--force]` | Create the local LLM Wiki directory structure and generated `rules/` layer. |
| `kb ingest [--copy\|--move] [--recursive] [--dry-run\|--preview]` | Organize source files into `raw/` subfolders. |
| `kb bootstrap [--copy\|--move] [--dry-run\|--preview]` | Run `init + ingest + extract-metadata + build-wiki`. |
| `kb extract-metadata [--force]` | Extract PDF metadata from `raw/papers/`. |
| `kb build-wiki` | Generate Markdown wiki pages and indexes. |
| `kb status [--dry-run\|--preview] [--json] [--unprocessed]` | Refresh `processing/manifest.json`, print JSON summary, or list unprocessed raw files. |
| `kb prepare [--new\|--file PATH] [--dry-run\|--preview]` | Generate prepare queue/proposal files for future LLM wiki maintenance without calling an LLM. |
| `kb sync-wiki [--dry-run\|--preview] [--json]` | Link source front matter in `wiki/**/*.md` back to manifest entries. |
| `kb lint-static [--dry-run\|--preview\|--no-report] [--json] [--strict]` | Check broken WikiLinks, orphan pages, missing source front matter, empty pages, and duplicate titles. |
| `kb query <terms...> [--limit N] [--snippets N] [--json] [--title-only]` | Search `wiki/**/*.md` with deterministic local keyword matching. |
| `kb grep <pattern> [--path PATH] [--regex] [--json]` | Search text-like files with Rust-native line-level matching. |
| `kb extract-text [--dry-run|--preview] [--force] [--json]` | Extract plain text from supported source files into `processing/text/` with deterministic best-effort methods. |
| `kb refs [--path PATH] [--citations] [--json]` | Scan extracted text for deterministic reference hints such as headings, bibliography entries, DOI values, and optional citation markers. |
| `kb tasks [--dry-run|--preview] [--json]` | Generate deferred human/LLM/agent task handoff lists under `LLM/tasks/`. |
| `kb memory --task-id ID --summary TEXT` | Append completed-task audit memory under `LLM/memory/completed_tasks.md`. |
| `kb repl` | Start a simple interactive shell. |
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

In `v0.5.10`, the existing `kb repl` command is treated as an early shell experiment. Ambiguous LLM-like command patterns are not parsed by the shell. See `docs/cli-shell-principle.md`.

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

## Cross-platform bootstrap

The recommended machine-facing one-command workflow is:

```bash
kb --kb-path /path/to/kb bootstrap --copy
```

This is cross-platform because the workflow is implemented inside the Rust CLI, not inside a Windows-only batch file.


## Manifest / Status

`kb status` is implemented:

```bash
kb --kb-path /path/to/kb status
kb --kb-path /path/to/kb status --json
kb --kb-path /path/to/kb status --unprocessed
kb --kb-path /path/to/kb status --dry-run
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
kb --kb-path /path/to/kb lint-static
kb --kb-path /path/to/kb lint-static --no-report
kb --kb-path /path/to/kb lint-static --json
kb --kb-path /path/to/kb lint-static --strict
```

Reports are written under `outputs/reports/` unless `--dry-run`, `--preview`, or `--no-report` is used. `--no-report` is specific to `lint-static`.


## Query skeleton

`kb query` is implemented in v0.5.0 as deterministic local keyword search over `wiki/**/*.md`.

```bash
kb --kb-path /path/to/kb query thermal cloak
kb --kb-path /path/to/kb query thermal cloak --limit 5
kb --kb-path /path/to/kb query thermal cloak --json
kb --kb-path /path/to/kb query thermal cloak --title-only
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

`ingest` and `bootstrap` default to copy mode unless `--move` is explicitly provided. Recursive mode skips generated/project folders such as:

```text
raw, wiki, processing, references, outputs, logs, .git, .obsidian, target, node_modules
```

For automation, prefer running a dry run first:

```bash
kb --kb-path /path/to/kb bootstrap --dry-run
```

Then run the real command:

```bash
kb --kb-path /path/to/kb bootstrap --copy
```

## Output style

`kb status --json` provides a machine-readable summary. Other command outputs remain human-readable and should not be treated as a stable JSON API.


## sync-wiki

Use `kb sync-wiki --dry-run` or `kb sync-wiki --preview` after wiki edits to preview manifest updates, then `kb sync-wiki` to record source-to-wiki links.


## Deferred task handoff

`kb tasks` is a deterministic management command. It gathers unresolved work from simple checks and emits a reviewable handoff list for a human manager or a future explicit LLM/agent skill.

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
