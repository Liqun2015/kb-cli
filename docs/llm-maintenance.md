# LLM Maintenance Workflow

This document defines how humans, Manager LLMs, Worker LLMs, and `kb-cli` should cooperate when maintaining a local Markdown-based LLM Wiki.

The central rule is simple:

> deterministic commands should scout, extract, index, check, and report; humans or explicit LLM workers should handle bounded semantic work under review.

An "LLM-maintained wiki" does **not** mean that an LLM freely edits original files or silently decides the whole workflow. It means that LLM work is broken into explicit, reviewable tasks supported by deterministic command outputs.

## Roles

### Human reviewer

The human reviewer owns the final decision. The reviewer accepts, rejects, or revises proposed Markdown edits, relationship records, topic-local importance judgments, and memory records.

### Manager LLM

The Manager LLM coordinates work. It should first use deterministic `kb-cli` commands to inspect the local knowledge base, gather evidence, detect missing work, and create bounded task plans.

The Manager LLM should not skip directly to rewriting the wiki from memory.

### Worker LLM

The Worker LLM completes one assigned task at a time. It should receive a clear task contract, explicit file list, evidence lines, expected output, and uncertainty rules.

The Worker LLM should not redefine the project direction or silently broaden the task.

### kb-cli command layer

The Rust CLI performs deterministic work such as ingestion, manifest tracking, text extraction, keyword search, reference scanning, graph export, static linting, task handoff, and memory logging.

By default, `kb-cli` should not hide LLM calls inside ordinary commands.

## Layered File Model

The wiki is maintained through separated layers:

```text
raw/          original source materials; read-only records
processing/   deterministic extraction, metadata, reference hints, and candidate relations
wiki/         reviewable Markdown knowledge pages and relationship explanations
topics/       topic-specific relation overlays and topic-local importance records
LLM/tasks/    bounded work orders for future human/LLM/agent handling
LLM/memory/   completed-task records and accepted decisions
rules/        operating contract for human and AI maintainers
outputs/      generated reports and static viewer output
```

This separation matters. For example, a source PDF should not be rewritten. A candidate relation should not be treated as reviewed knowledge. A topic-local interpretation should not be mixed into the global bibliographic index.

## Intended Maintenance Loop

```text
raw material enters raw/
        ↓
kb records the file in processing/manifest.json
        ↓
Manager LLM runs deterministic kb-cli commands to scout the material
        ↓
kb-cli performs what it can: search, extract, scan, index, lint, and report
        ↓
kb-cli writes unresolved or semantic work into LLM/tasks/
        ↓
Manager LLM assigns bounded tasks to Worker LLMs or human reviewers
        ↓
Worker LLMs propose Markdown updates or relationship decisions under explicit requirements
        ↓
humans review changes through normal file diff / Git diff
        ↓
accepted changes become wiki pages, topic relation records, or LLM/memory/ entries
```

## Manager LLM First-pass Commands

A Manager LLM should use structured commands before doing semantic work:

```bash
kb status
kb query <terms...>
kb grep <pattern>
kb extract-text
kb refs
kb refs-index
kb refs-graph
kb keywords
kb links
kb lint-static
kb health
kb tasks
```

The purpose is to ground future edits in local files, extracted evidence, and generated reports.

## Worker Task Contract

A Worker LLM should receive a bounded task with the following fields:

```text
goal
requirements
file list
evidence lines
expected output
review rules
uncertainty rules
forbidden actions
```

A good Worker task is narrow enough that its output can be reviewed with a normal Git diff.

## What Worker LLMs May Do

Worker LLMs may propose:

- source-specific paper summaries under `wiki/papers/`
- concept pages under `wiki/concepts/`
- method pages under `wiki/methods/`
- topic pages under `wiki/topics/`
- topic-local relation records under `topics/<topic>/`
- review notes under `LLM/memory/`
- index updates under `wiki/indexes/`

They should preserve uncertainty and cite the provided evidence.

## What Worker LLMs Must Not Do

Worker LLMs must not:

- modify files under `raw/`
- silently rewrite large parts of the wiki without a bounded task
- invent citations, DOIs, source identities, or evidence lines
- treat candidate relations as reviewed facts
- merge global bibliographic relations and topic-specific interpretive relations
- trigger hidden LLM behavior inside deterministic commands
- redefine the project architecture without explicit human approval

## Relationship Review Rules

LLM Wiki maintains an evidence-backed relationship network among references. The relationship network has three levels.

### 1. Bibliographic index relations

Examples:

- citation
- reference-list match
- DOI match
- title / author / journal / volume / page / date match

Rust-native commands may extract candidates. Human review is the final guarantee when identity is uncertain.

### 2. Keyword / topic relations

Examples:

- shared terms
- methods
- models
- devices
- research topics

Rust-native commands may detect co-occurrence candidates. Meaning should be reviewed before it is treated as knowledge.

### 3. Scientific idea relations

Examples:

- method transfer
- mechanism similarity
- theoretical inheritance
- research gap
- cross-domain inspiration

These should be handled through explicit Manager LLM / Worker LLM workflows with evidence and review.

## Topic-specific Overlay Rule

Global bibliographic index relations belong under:

```text
processing/refs/
```

Topic-specific interpretive relations belong under:

```text
topics/<topic>/
```

A citation relation such as `Paper A cites Paper B` is global. But relations such as `Paper A motivates Paper B for thermal metamaterials`, `Paper A improves Paper B under this topic`, or `Paper A is core literature for this topic` are topic-dependent.

Do not mix these two layers.

## Example: New Paper Enters raw/

One new paper may lead to multiple possible updates:

```text
wiki/papers/<paper-title>.md
wiki/concepts/<key-concept>.md
wiki/methods/<method-name>.md
wiki/topics/<research-topic>.md
wiki/indexes/papers_index.md
topics/<topic>/relations/
topics/<topic>/importance/
LLM/memory/completed_tasks.md
```

A safe process is:

```bash
kb status
kb extract-text --path raw/papers/example.pdf
kb refs --path processing/text
kb refs-index
kb keywords thermal metamaterial cloak
kb query thermal metamaterial
kb tasks
```

Then the Manager LLM creates a bounded task for a Worker LLM or human reviewer.

## Example Worker Task

```text
goal:
Create a reviewable Markdown summary page for one source paper.

requirements:
- Do not modify raw/.
- Use only the provided extracted text and reference hints.
- Preserve uncertainty when the text is incomplete.
- Add source_files front matter.
- Do not infer topic-level importance unless explicitly requested.

file list:
- raw/papers/example.pdf
- processing/text/example.txt
- processing/refs/refs_index_latest.md

expected output:
- wiki/papers/example.md
- short note for LLM/memory/completed_tasks.md

review rules:
- Human reviewer must inspect the Git diff before acceptance.
```

## CLI / Shell / LLM Boundary

`kb` batch commands and `kb shell` should share deterministic command semantics.

```text
kb --kb-path ./quantum query thermal cloak
≈
kb> use ./quantum
kb> query thermal cloak
```

LLM-assisted behavior must not be triggered implicitly from free-form text in `kb>`. Future LLM behavior should be introduced only through a deliberately designed, explicit top-level interface.

## Completion Memory

Accepted work should be recorded under:

```text
LLM/memory/completed_tasks.md
```

Use `kb memory` where possible so completed work can be inspected later without relying on chat history.

## Review Checklist

Before accepting LLM-generated wiki maintenance work, check:

- Does the task have a bounded goal?
- Are source files and evidence lines explicit?
- Were original files under `raw/` preserved?
- Are candidate relations labeled as candidates?
- Are topic-specific claims stored under `topics/<topic>/`?
- Does the Git diff show small, reviewable changes?
- Was accepted work recorded under `LLM/memory/`?

## Non-goals

The maintenance workflow is not intended to provide:

- hidden LLM execution
- automatic scientific correctness
- automatic literature review replacement
- unreviewed citation-graph interpretation
- vector search or RAG by default
- autonomous large-scale wiki rewriting

The goal is a traceable human/AI maintenance loop, not a black-box knowledge system.
