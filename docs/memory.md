# `kb memory`

Current version: `v0.7.10`

`kb memory` records completed task outcomes under the LLM workbench:

```text
LLM/memory/completed_tasks.md
```

It is a deterministic audit-memory command. It does **not** call an LLM, execute an agent, rewrite Wiki pages, or decide whether the task was scientifically correct. It simply records what a human, LLM, or future sub-agent reports as completed so the work can be inspected later.

## Why `LLM/memory/`

`LLM/tasks/` contains handoff lists for hard work that deterministic commands intentionally defer.

`LLM/memory/` records what happened after those tasks were completed. This keeps the workflow inspectable:

```text
Rust deterministic commands
    -> LLM/tasks/       tasks to hand off
    -> human/LLM/agent  explicit completion work
    -> LLM/memory/      completed-task memory for later inspection
```

This memory is project-local and file-based. It is not hidden model memory, not a database, and not an automatic execution log.

## Usage

```bash
kb memory --task-id pdf-text-conversion-001 --summary "Converted missing PDF text files."
```

With a title, files, and source task report:

```bash
kb memory \
  --task-id pdf-text-conversion-001 \
  --title "Convert missing PDF text" \
  --summary "Converted three text-based PDFs into processing/text/. One scanned PDF remains unresolved." \
  --file processing/text/paper_a.txt \
  --file processing/text/paper_b.txt \
  --source-task LLM/tasks/llm_tasks_20260509_120000.md \
  --follow-up "raw/papers/scanned_example.pdf still requires explicit OCR."
```

Preview without writing:

```bash
kb memory --task-id wiki-link-repair-001 --summary "Reviewed broken links." --dry-run
```

JSON output:

```bash
kb memory --task-id reference-reconciliation-001 --summary "Built draft reference table." --json
```

## Required fields

`kb memory` requires:

```text
--task-id
--summary
```

The task id should usually come from `LLM/tasks/*.md`.

## Record format

The command appends entries to:

```text
LLM/memory/completed_tasks.md
```

Each entry includes:

```text
timestamp
task id
title
source task report
files
summary
follow-up / unresolved issues
```

## Boundary

`kb memory` only records completed work.

It must not:

- call an LLM;
- validate semantic correctness;
- rewrite source files;
- auto-close task reports;
- infer completion from file changes;
- hide unresolved issues.

If a completed task reveals new hard work, record it as follow-up. A later deterministic command or human manager can convert that follow-up into a new `LLM/tasks/` handoff.
