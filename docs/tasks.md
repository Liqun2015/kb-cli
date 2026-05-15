# `kb tasks`

`kb tasks` generates a deterministic handoff list for work that should be deferred to a human reviewer or a future explicit LLM/agent skill.

It does **not** call an LLM. It does **not** execute the deferred work. It only organizes task groups with goals, requirements, file lists, and evidence.

## Purpose

Many `kb-cli` commands should do one simple thing well. When a task crosses the boundary into OCR, semantic cleanup, reference reconciliation, citation graph reasoning, or Wiki drafting, the command should not pretend to solve it. It should identify the unresolved work and produce a reviewable task handoff.

`kb tasks` is the first shared handoff command for that pattern.

## Usage

```bash
kb tasks
kb tasks --dry-run
kb tasks --preview
kb tasks --json
kb tasks --limit 2
kb tasks --index-only
kb tasks --no-index
```

With an explicit knowledge-base path:

```bash
kb --kb-path ./quantum tasks
kb --kb-path "D:\\github\\llm-wiki\\quantum" tasks
```


## Why `LLM/tasks/` instead of `outputs/tasks/`

`kb-cli` deterministic commands are the first scouting pass. They search, scan, inspect, and organize what they can do reliably. In that sense, they are the front line that maps the terrain and does the simple work first.

The hard work that remains—OCR, layout repair, reference reconciliation, citation graph reasoning, link repair, or Wiki drafting—belongs to a later human/LLM/agent layer. For that reason, handoff lists are written under `LLM/tasks/`, not under the generic `outputs/` tree.

`LLM/tasks/` is a workbench for future explicit LLM/agent skills. It is not an automatic execution directory, and `kb tasks` does not run those agents.

## Output

By default, `kb tasks` now writes three layers:

```text
LLM/tasks/index.md                 # Manager LLM task dashboard
LLM/tasks/items/<task_id>.md        # bounded Worker LLM / human task files
LLM/tasks/llm_tasks_YYYYMMDD_HHMMSS.md  # timestamped audit snapshot
```

The timestamped report contains task groups with:

- target agent
- goal
- requirements
- file list
- evidence
- source command
- priority

`LLM/tasks/index.md` is the normal intake point for the Manager LLM.

`LLM/tasks/items/<task_id>.md` files are the normal intake point for Worker LLMs.

Use `--dry-run` or `--preview` to print results without writing files.


## Manager LLM task index

Starting in `v0.7.3`, `kb tasks` makes the Manager LLM task list explicit.

The Manager LLM should start from:

```text
LLM/tasks/index.md
```

The index summarizes:

- pending Worker task items
- topic review queues
- older handoff reports and audit-review tasks
- recommended Manager → Worker → human review loop

A Worker LLM should not scan the whole task directory looking for work. It should receive one bounded item file from:

```text
LLM/tasks/items/<task_id>.md
```

See `docs/llm-task-index.md`.

### Index-only refresh

```bash
kb tasks --index-only
```

This refreshes `LLM/tasks/index.md` and `LLM/tasks/items/<task_id>.md` without writing a new timestamped handoff snapshot.

### Snapshot-only mode

```bash
kb tasks --no-index
```

This preserves the old report-only behavior and does not touch the Manager LLM dashboard.

## Current deterministic task groups

The current skeleton can detect several task groups:

| Task group | Trigger | Deferred target |
|---|---|---|
| `pdf-text-conversion-001` | PDFs under `raw/papers/` without usable extracted text under `processing/text/` | PDF Text Conversion Agent |
| `reference-reconciliation-001` | Reference headings, DOI values, or bibliography entries found in `processing/text/` | Reference Reconciliation Agent |
| `bibliographic-index-review-001` | `refs_index_*.md` reports contain `candidate`, `ambiguous`, or `missing` relations | Human Reference Index Reviewer |
| `keyword-topic-review-001` | `keywords_*.md` reports contain keyword/topic candidate relations | Keyword Topic Relation Worker |
| `wiki-source-traceability-001` | Source-derived Wiki pages under `wiki/papers/` or `wiki/notes/` without `source_files` or `source_ids` | Wiki Source Traceability Agent |
| `wiki-link-repair-001` | Broken WikiLinks under `wiki/` | Wiki Link Repair Agent |

## Design boundary

`kb tasks` only reports deferred work. It must not silently perform that work.

It does not:

- run OCR
- call an LLM
- rewrite Wiki pages
- build a citation graph
- merge reference identities
- create missing pages
- fix broken links

Future commands should follow the same pattern: perform a narrow deterministic ability first, then return unresolved work as explicit task groups for a higher-level manager or explicit LLM/agent skill.

## After task completion

When a task from `LLM/tasks/` is completed by a human, LLM, or future sub-agent, record the result with `kb memory`:

```bash
kb memory --task-id pdf-text-conversion-001 --summary "Converted missing PDF text files."
```

Completed task memories are appended to:

```text
LLM/memory/completed_tasks.md
```

This keeps completed work inspectable without relying on chat history or hidden model state.

## Paper section extraction tasks

Starting in `v0.7.4`, `kb tasks` also checks whether extracted paper text has corresponding section files under:

```text
processing/sections/<source-key>/introduction.txt
processing/sections/<source-key>/references.txt
```

If these files are missing or suspiciously short, `kb tasks` adds a `paper-section-extraction-001` task for a `Paper Section OCR/LLM Worker`. The Worker should recover the two sections from text, OCR, or multimodal LLM reading, but the work must remain explicit and reviewable.


## Topic-local task handoff

Starting in `v0.7.12`, global `kb tasks` is complemented by a topic-local task command:

```bash
kb topic tasks <topic>
kb topic task <topic>   # alias
```

Global tasks stay under:

```text
LLM/tasks/
```

Topic-local tasks stay under:

```text
topics/<topic>/tasks/index.md
topics/<topic>/tasks/items/<task_id>.md
```

Use `kb topic tasks <topic>` when the remaining work depends on the topic boundary, topic literature list, topic-local importance candidates, directed relation rows, or the topic graph. It does not call an LLM, infer relations, or accept scholarly claims. It only creates bounded handoff files for a Manager LLM or human maintainer.


## Generic agent handoff

Starting in `v0.7.13`, task files are complemented by generic external-agent handoff files. Use:

```bash
kb handoff --topic <topic>
kb topic handoff <topic>
kb handoff --agent claude-code
kb handoff --all-agents
```

The handoff files define Manager/Worker boundaries, forbidden edits, evidence rules, and start prompts. Task files remain the bounded units of work. Claude Code, OpenCode, OpenClaw, and Codex are adapters; `AGENTS.md` is the generic contract.
