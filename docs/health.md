# `kb health`

`kb health` is a deterministic relationship-health and workflow-health summary command.

It is designed for Manager LLM sessions, human maintainers, and future review dashboards. It does not call an LLM. It reads the current LLM Wiki filesystem state and summarizes whether the relationship-maintenance pipeline has enough evidence, reports, and review handoffs to continue safely.

## Purpose

LLM Wiki maintains relationships among references. `kb health` checks whether the supporting pipeline is in place:

```text
raw/papers/
    -> processing/text/
    -> processing/refs/
    -> processing/refs/refs_index_*.md
    -> processing/refs/refs_graph_*.json/.mmd/.dot
    -> processing/keywords/keywords_*.md
    -> wiki/
    -> LLM/tasks/
    -> LLM/memory/
```

The command is intentionally not a semantic judge. It reports what is present, what is missing, and what should be handed off.

## Usage

```bash
kb health
kb health --dry-run
kb health --json
kb health --strict
```

With an explicit knowledge base path:

```bash
kb --kb-path "D:\github\llm-wiki\quantum" health
```

## Output

By default, `kb health` writes:

```text
outputs/reports/health_YYYYMMDD_HHMMSS.md
```

The report includes:

```text
status
score
counts
checks
deferred task hints
```

`--dry-run` / `--preview` prints the summary without writing a report file.

`--json` prints a machine-readable health report for third-party dashboards or Manager LLM workflows.

## What it checks

Current checks include:

```text
raw_papers
extracted_text_files
empty_text_files
wiki_pages
refs_reports
refs_index_reports
refs_graph_json / mermaid / dot
keyword_reports
llm_task_reports
llm_memory_files
lint_reports
uncertain_bibliographic_relation_hints
keyword_topic_relation_hints
third-party skill docs
```

## Deferred work

When health checks reveal incomplete or uncertain work, the command emits bounded deferred task hints, for example:

```text
health-pdf-text-conversion-001
health-bibliographic-index-review-001
health-keyword-topic-review-001
health-wiki-drafting-001
```

These are hints for the Manager LLM. They do not execute Worker tasks. Use `kb tasks` to generate a consolidated `LLM/tasks/` handoff list.

## Boundaries

`kb health` does not:

- call an LLM;
- run OCR;
- repair files;
- confirm bibliographic identity;
- create wiki pages;
- decide scientific idea relations;
- generate graph viewers.

It only summarizes deterministic evidence and review backlog.

## Manager / Worker role

The Manager LLM should use `kb health` as a dashboard command before assigning work:

```text
Manager LLM runs kb health
    -> sees missing text extraction / uncertain relations / missing graph export
    -> runs more specific commands when needed
    -> uses kb tasks to create handoff lists
    -> assigns bounded work to Worker LLMs or humans
    -> records accepted work with kb memory
```

Humans remain the final guarantee for uncertain bibliographic index relations.
