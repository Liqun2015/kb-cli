# LLM Command Guide

Current version: `v0.7.11`

This document is the operating guide for the top-level Manager LLM, Claude Code sessions, and human operators who use `kb-cli` as a structured command layer.

The LLM that uses these commands is the highest-level coordinator. It is not the same role as a lower-level Worker LLM that receives a bounded task from `LLM/tasks/`. See `docs/llm-hierarchy.md`.

For the canonical command map and lifecycle vocabulary, see `docs/commands.md` and `docs/task-lifecycle.md`.

The key idea is simple:

```text
Rust-native kb commands scout first.
They search, inspect, extract, count, and report what can be handled deterministically.
Worker LLM or agent work starts only after those commands have narrowed the problem and produced file lists, evidence, and task handoff notes.
```

In other words, `kb-cli` commands are the first pass. They do the fast, cheap, repeatable scouting work. The Manager LLM should use that evidence to decide what to delegate. Worker LLMs should spend their effort only on bounded hard parts: semantic repair, synthesis, reference reconciliation, concept writing, and wiki maintenance.

## Required operating principle

Before delegating work to a Worker LLM, the Manager LLM should run the relevant deterministic command first whenever possible.

Do not ask an LLM to guess:

- which files exist;
- which pages have broken links;
- which pages lack `source_files`;
- which files contain a keyword;
- which extracted texts contain reference hints;
- which PDFs still need text conversion;
- which completed tasks have already been recorded.

Use `kb-cli` commands to collect that evidence, then hand bounded evidence to the Worker LLM.

## Command map for LLM operators

| Command | Main ability | Reads | Writes | Use before asking LLM to... | Deferred work |
|---|---|---|---|---|---|
| `kb init` | Create project directories | project root | `raw/`, `wiki/`, `processing/`, `rules/`, `LLM/` | start a new local wiki | none |
| `kb ingest` | Register/copy raw files | source files, `raw/` | `raw/`, `processing/manifest.json` | summarize newly added sources | source interpretation |
| `kb prepare` | Generate reviewable writing/task materials | manifest, raw records | `processing/proposals/` | write or update wiki pages | actual wiki writing |
| `kb build-wiki` | Generate deterministic wiki skeleton pages | `raw/`, manifest | `wiki/` | draft richer paper/concept pages | semantic summaries |
| `kb sync-wiki` | Sync wiki front matter back to manifest | `wiki/` | `processing/manifest.json` | audit source traceability | source judgment |
| `kb lint-static` | Find static wiki problems | `wiki/` | `outputs/reports/` | repair links, sources, isolated pages | semantic link repair |
| `kb query` | Find relevant wiki pages | `wiki/**/*.md` | stdout / JSON | answer a topic-level question | answer synthesis |
| `kb grep` | Find exact lines and structure traces | selected text paths | stdout / JSON | inspect keywords, fields, DOI traces, TODOs | interpretation |
| `kb extract-text` | Extract direct text where possible | `raw/papers/` or chosen files | `processing/text/` | read PDF contents | OCR/layout repair |
| `kb refs` | Scan reference hints | `processing/text/` | stdout / JSON | reconcile references or build citation maps | semantic reconciliation |
| `kb refs-index` | Build bibliographic index relation candidates | `processing/text/`, `raw/papers/`, metadata | `processing/refs/` | review citation/index relations | human identity confirmation |
| `kb refs-graph` | Export bibliographic relation candidates as graph data | `processing/refs/refs_index_*.md` | `processing/refs/refs_graph_*.json` / `.mmd` / `.dot` | visualize relation certainty | human review of uncertain edges |
| `kb keywords` | Detect keyword/topic co-occurrence candidates | `processing/text/` | `processing/keywords/keywords_*.md` / JSON | find topic-level evidence | semantic meaning / idea relations |
| `kb tasks` | Generate deferred work lists and task index | project state | `LLM/tasks/index.md`, `LLM/tasks/items/` | assign bounded work to Manager/Worker LLMs | execution of tasks |
| `kb topic tasks <topic>` | Generate topic-local handoff tasks | `topics/<topic>/` | `topics/<topic>/tasks/index.md`, `topics/<topic>/tasks/items/` | assign bounded topic-local Worker LLM tasks | execution of tasks / scholarly judgment |
| `kb memory` | Record completed task outcomes | operator input | `LLM/memory/` | inspect project history | validation of quality |

## Standard Manager-led maintenance loop

Use this loop when a human operator or Manager LLM is asked to improve the wiki.

```text
1. Run deterministic commands to inspect the project.
2. Generate or read task handoff notes under LLM/tasks/.
3. Delegate only the named semantic task to the appropriate Worker LLM.
4. Review changes with git diff.
5. Run deterministic checks again.
6. Record completion with kb memory.
```

Example:

```bash
kb tasks
# Manager LLM starts from LLM/tasks/index.md
# Worker LLM receives one bounded LLM/tasks/items/<task_id>.md file
# Perform one task only, under Git review
kb lint-static
kb memory --task-id wiki-link-repair-001 --summary "Repaired broken WikiLinks found by lint-static." --source-task LLM/tasks/items/<task_id>.md
```

## Use `--json` when the LLM must parse output

Human-facing output is useful for inspection. LLM/agent workflows should prefer machine-readable output when available.

Examples:

```bash
kb query thermal cloak --json
kb grep "source_files:" --path wiki --json
kb refs --json
kb tasks --json --dry-run
```

The LLM should treat JSON output as evidence, not as final interpretation.

## Typical workflows

### 1. Find topic-relevant wiki pages

```bash
kb query thermal cloak --limit 10
kb query thermal cloak --json
```

Use the result list as the candidate file set. The LLM may then read those Markdown pages and synthesize an answer or propose edits.

Do not ask the LLM to search the entire repository manually when `kb query` can produce candidates first.

### 2. Inspect exact keyword or structure traces

```bash
kb grep "source_files:" --path wiki
kb grep "\[\[.*\]\]" --regex --path wiki
kb grep "DOI" --path processing/text
```

Use this for exact evidence: line numbers, file paths, and matching lines.

The LLM may interpret the matches afterward, but it should not invent matches that were not returned.

### 3. Convert PDF text before asking for summaries

```bash
kb extract-text --dry-run
kb extract-text
kb grep "References" --path processing/text
```

If a PDF has no extractable text, do not force `extract-text` to become OCR or LLM cleanup. Let `kb tasks` create a handoff for future PDF Text Conversion Agent work.

### 4. Prepare reference reconciliation work

```bash
kb refs --path processing/text
kb refs --path processing/text --citations --json
kb tasks
```

The LLM may reconcile duplicate references, normalize citation formats, or infer a citation graph only after `kb refs` has produced deterministic hints.

### 5. Repair wiki structure

```bash
kb lint-static
kb tasks
```

Use lint output and task files to identify broken links, missing source metadata, and isolated pages. The LLM may propose repairs, but the deterministic command should provide the initial evidence.

### 6. Record completed work

```bash
kb memory \
  --task-id reference-reconciliation-001 \
  --title "Normalize reference hints" \
  --summary "Reviewed refs output and normalized DOI mentions in wiki/papers/example.md." \
  --file wiki/papers/example.md \
  --source-task LLM/tasks/items/<task_id>.md \
  --follow-up "Citation graph still requires explicit agent design."
```

Completed task memory is project-local audit material. It is not hidden model memory.

## What the Manager LLM should not do

A Manager LLM or human maintainer should not:

- bypass `kb grep`, `kb query`, `kb refs`, or `kb lint-static` for mechanical inspection;
- make broad edits without a file list;
- modify `raw/` source materials;
- claim a PDF was unreadable without first checking `kb extract-text` output or task handoff evidence;
- invent `source_files` or `source_ids` values;
- treat `kb>` shell mode as a natural-language chat interface;
- add hidden LLM calls to deterministic commands;
- close tasks without recording a completion memory when the work is meaningful;
- let Worker LLMs redefine project scope or silently expand file lists.

## What belongs in `LLM/tasks/`

`LLM/tasks/` is the handoff area for unresolved work that deterministic commands intentionally do not solve.

Good task handoff items include:

- target agent or human role;
- goal;
- requirements;
- file list;
- evidence;
- source command;
- priority;
- explicit non-goals.

These files are meant for a human manager or Manager LLM to read before delegating hard work to a Worker LLM or agent skill.

## What belongs in `LLM/memory/`

`LLM/memory/` records completed task outcomes so future maintainers can inspect what was done.

Good memory records include:

- task id;
- title;
- summary;
- source task file;
- files touched;
- remaining follow-up issues;
- timestamp.

Do not rely on chat history alone.

## One-command-one-ability rule

Each command should do one clear thing well. If it reaches work that requires semantic judgment, it should report or hand off that work rather than silently becoming smarter.

```text
extract-text -> get direct text if possible; defer OCR/layout repair.
refs         -> find reference hints; defer semantic reconciliation.
grep         -> find lines; defer interpretation.
tasks        -> write task handoff; defer execution.
memory       -> record completion; defer quality judgment.
```

This rule keeps the project fast, auditable, and LLM-efficient.


## Link inspection

Manager LLMs should run `kb links`, `kb links --unresolved`, and `kb links --ambiguous` before assigning link-repair work to a Worker LLM. Worker LLMs should receive the concrete source pages, line numbers, candidate targets, and requirements instead of guessing link problems from memory.


## Using `kb refs-index` and `kb refs-graph`

Before asking a Worker LLM to reason about a literature network, the Manager LLM should run:

```bash
kb extract-text
kb refs
kb refs-index
kb refs-graph
```

The Manager LLM should treat `confirmed` rows as high-confidence evidence and route `candidate`, `ambiguous`, and `missing` rows to human review or bounded Worker LLM preparation. Worker LLMs may normalize strings and organize evidence, but they must not be treated as the final guarantee for bibliographic identity.


## Third-party graph handoff

When the Manager LLM prepares data for graph visualization or third-party skill generation, it should preserve relationship status and evidence. Use `docs/third-party-skills/` as the visual and JSON guidance source.


## Using `kb keywords`

Use `kb keywords` after `kb extract-text` when the Manager LLM needs second-level literature relationship evidence: shared terms, devices, methods, models, or topics.

```bash
kb keywords
kb keywords thermal cloak metamaterial
kb keywords --terms-file references/thermal_terms.txt
kb keywords --json
```

The command returns candidate keyword/topic relations only. A Worker LLM may review whether the shared terms are scientifically meaningful, but it must preserve file lists and evidence lines and must not promote candidates into scientific idea relations without review.


## Topic relationship navigation

When a user asks about a specific research topic, the Manager LLM should not treat global reference indexes as topic-level conclusions.

Use this order:

```text
1. Identify the topic slug.
2. Inspect topics/<topic>/ if it exists.
3. Read scope.md, literature.md, importance.md, relations/, tasks/, and memory/ if present.
4. Use global commands such as kb query, kb grep, kb refs-index, kb refs-graph, kb keywords, and kb health for supporting evidence.
5. Generate bounded tasks when Worker LLM or human review is required. Use `kb topic tasks <topic>` for topic-local handoffs and `kb tasks` for global handoffs.
```

`processing/refs/` remains the global bibliographic layer. `topics/<topic>/` is the topic-local interpretation layer.

## Topic schema rule added in v0.6.3

When the Manager LLM is working inside a concrete topic, it should read these files before assigning Worker LLM tasks:

```text
docs/topic-relationships.md
docs/topic-relation-schema.md
docs/literature-importance-schema.md
docs/topic-v2-roadmap.md
```

The Manager LLM should then inspect:

```text
topics/<topic>/scope.md
topics/<topic>/literature.md
topics/<topic>/importance.md
topics/<topic>/relations/
```

Worker LLMs should not invent new relation types unless the Manager LLM explicitly updates the schema. Use `unknown` with evidence when unsure.


## Static viewer boundary

`kb view` is a static display command. It may render existing Markdown/JSON outputs into `outputs/html/index.html`, but it must not call an LLM, execute shell commands from the browser, or modify source/wiki files. The `kb-view>` box inside the generated HTML is display navigation only.


## Topic workspace initialization

When a topic-specific relationship overlay is needed, Manager LLM may ask the user to run:

```bash
kb topic init <topic>
```

After initialization, Manager LLM should inspect `topics/<topic>/scope.md` and `topics/<topic>/literature.md` before assigning Worker LLM tasks.


## Topic workspace inspection

Manager LLM may use:

```bash
kb topic list
kb topic status <topic>
```

before assigning topic-local Worker LLM tasks. These commands only inspect the topic workspace structure. They do not validate the truth of topic-local relations.

When scope, literature, importance, relations, or graph artifacts are ready for delegation, run:

```bash
kb topic tasks <topic>
```

This writes `topics/<topic>/tasks/index.md` and bounded task files under `topics/<topic>/tasks/items/`. The command is a handoff generator only; it does not execute the tasks.


## Topic importance workflow

Manager LLM may run `kb topic rank <topic> --dry-run` or inspect its generated report to decide which papers need human review. Worker LLMs must treat rank output as candidate evidence only, not as final authority.
