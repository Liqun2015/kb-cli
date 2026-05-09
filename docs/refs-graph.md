# `kb refs-graph`

Current version: `v0.6.2`

`kb refs-graph` exports bibliographic index relation candidates as graph data for third-party visualization skills and tools.

It is the first graph-oriented command in the literature-relationship workflow:

```text
kb extract-text
    ↓
kb refs
    ↓
kb refs-index
    ↓
kb refs-graph
```

## Purpose

`kb refs-graph` turns `processing/refs/refs_index_*.md` reports into a graph representation with nodes, edges, evidence, review flags, and visual-style hints.

It does **not** decide final bibliographic identity. It only exports the deterministic relation state found by `kb refs-index`.

## Default input and output

Default input:

```text
processing/refs/
```

The command reads the latest `refs_index_*.md` report unless `--path` is provided.

Default output:

```text
processing/refs/refs_graph_YYYYMMDD_HHMMSS.json
```

Optional output files:

```text
processing/refs/refs_graph_YYYYMMDD_HHMMSS.mmd
processing/refs/refs_graph_YYYYMMDD_HHMMSS.dot
```

## Usage

```bash
kb refs-graph
kb refs-graph --json
kb refs-graph --mermaid
kb refs-graph --dot
kb refs-graph --path processing/refs/refs_index_20260509_120000.md
kb refs-graph --dry-run
```

Use with a specific KB path:

```bash
kb --kb-path "D:\\github\\llm-wiki\\quantum" refs-graph --json
```

## Visual protocol

`kb refs-graph` follows `docs/third-party-skills/relation-graph-visual-protocol.md`:

```text
confirmed relation        -> solid edge
candidate / ambiguous     -> dashed edge
missing / unresolved node -> hollow node
needs human review        -> explicit evidence and review marker
```

## What it can do well

- Read deterministic `refs-index` Markdown reports.
- Convert relation rows into graph nodes and edges.
- Preserve relation status, score, evidence, and review requirements.
- Export JSON graph data for third-party tools.
- Optionally emit Mermaid or Graphviz DOT text.

## What it must not do

- It must not call an LLM.
- It must not query Crossref, Semantic Scholar, or online DOI services.
- It must not decide that a `candidate` relation is actually confirmed.
- It must not hide uncertain relations.
- It must not choose one target for an `ambiguous` relation.
- It must not rewrite wiki pages or reference reports.

## Deferred work

Uncertain graph edges remain tasks for a human reviewer, optionally prepared by a Manager LLM and Worker LLM.

The final guarantee for bibliographic identity remains human review.
