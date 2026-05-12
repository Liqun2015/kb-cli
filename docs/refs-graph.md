# `kb refs-graph`

Current version: `v0.7.7`

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
literature importance     -> node size
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

## V2 JSON fields

`v0.6.3` exports a conservative V2 graph shape. The goal is to make graph data easier for third-party skills and Manager LLM workflows to consume without pretending that the global bibliographic graph contains topic-specific causal knowledge.

Graph-level fields include:

```text
schema_version: refs-graph.v2.1
graph_kind: global_bibliographic_index_graph
relation_layer: processing/refs
direction_model: directed_edges
```

Node records include a lightweight `weight` object and visual size placeholders:

```json
{
  "weight": {
    "importance_level": "unknown",
    "importance_score": 0.0,
    "weight_source": "not_ranked_by_refs_graph",
    "notes": ["bibliographic graph node"]
  },
  "visual": {
    "node_style": "filled",
    "node_size": "default",
    "node_size_value": "20"
  }
}
```

`v0.6.3.1` reserves node circle size for literature importance. `kb refs-graph` does not rank literature yet; it only preserves the graph interface for future `kb rank` and topic-local importance data.

This is only a placeholder for future ranking commands. `kb refs-graph` does not decide whether a paper is core, important, background, or peripheral.

Edge records include directed relation fields:

```json
{
  "direction": "source_cites_or_mentions_target",
  "relation_layer": "global_bibliographic_index",
  "relation_type": "cites_or_references",
  "relation_label": "bibliographic index candidate",
  "status": "candidate",
  "confidence": 0.72,
  "needs_human_review": true,
  "human_final_guarantee_required": true
}
```

For confirmed DOI-level matches, `human_final_guarantee_required` may be false. For candidate, ambiguous, missing, or needs-human cases, it must be true.

## What V2 deliberately does not mean

The V2 graph export does not mean LLM Wiki has a universal causal graph.

```text
Global refs graph: A cites / references / mentions B.
Topic graph: A motivates / improves / contradicts / supports B within a specific topic.
```

Topic-specific causal, method, evidence, idea, and importance relations belong under `topics/<topic>/`, not in `processing/refs/`.
