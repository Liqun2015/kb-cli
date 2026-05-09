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

By default, `kb tasks` writes a Markdown handoff report to:

```text
LLM/tasks/llm_tasks_YYYYMMDD_HHMMSS.md
```

The report contains task groups with:

- target agent
- goal
- requirements
- file list
- evidence
- source command
- priority

Use `--dry-run` or `--preview` to print results without writing a report.

## Current deterministic task groups

The current skeleton can detect several task groups:

| Task group | Trigger | Deferred target |
|---|---|---|
| `pdf-text-conversion-001` | PDFs under `raw/papers/` without usable extracted text under `processing/text/` | PDF Text Conversion Agent |
| `reference-reconciliation-001` | Reference headings, DOI values, or bibliography entries found in `processing/text/` | Reference Reconciliation Agent |
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
