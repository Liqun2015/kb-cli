# Relation Graph JSON Schema Guidance

Current version: `v0.7.7`

This document defines a recommended JSON shape for third-party visualization skills and tools. It is a guidance schema, not yet a formal stable API.

## Node fields

Recommended node object:

```json
{
  "id": "paper_a",
  "label": "Paper A title",
  "node_type": "paper",
  "status": "local",
  "source_path": "raw/papers/paper_a.pdf",
  "visual": {
    "node_style": "filled",
    "node_size": "default",
    "node_size_value": "20"
  }
}
```

For unresolved references:

```json
{
  "id": "unresolved_reference_001",
  "label": "Unresolved reference entry",
  "node_type": "unresolved_reference",
  "status": "missing",
  "evidence": [
    "reference entry found in processing/text/paper_a.txt:327"
  ],
  "visual": {
    "node_style": "hollow",
    "node_size": "default",
    "node_size_value": "20"
  },
  "needs_human_review": true
}
```

## Edge fields

Recommended edge object:

```json
{
  "source": "paper_a",
  "target": "paper_b",
  "relation_type": "bibliographic_index",
  "status": "confirmed",
  "confidence": 1.0,
  "evidence": [
    "DOI exact match: 10.xxxx/example"
  ],
  "visual": {
    "edge_style": "solid",
    "source_node_style": "filled",
    "target_node_style": "filled"
  },
  "needs_human_review": false
}
```

Candidate edge:

```json
{
  "source": "paper_a",
  "target": "paper_b_candidate",
  "relation_type": "bibliographic_index",
  "status": "candidate",
  "confidence": 0.72,
  "evidence": [
    "title token overlap",
    "same first author",
    "same year"
  ],
  "visual": {
    "edge_style": "dashed",
    "source_node_style": "filled",
    "target_node_style": "filled"
  },
  "needs_human_review": true
}
```

Missing edge to hollow node:

```json
{
  "source": "paper_a",
  "target": "unresolved_reference_001",
  "relation_type": "bibliographic_index",
  "status": "missing",
  "confidence": 0.0,
  "evidence": [
    "reference entry found, no local match"
  ],
  "visual": {
    "edge_style": "dashed",
    "target_node_style": "hollow"
  },
  "needs_human_review": true
}
```

## Required preservation rule

Third-party skills and tools must preserve these fields when importing/exporting relationship data:

- `source`
- `target`
- `relation_type`
- `status`
- `evidence`
- `needs_human_review`
- `visual.edge_style`
- `visual.node_style` or target/source node style fields
- `visual.node_size` and `visual.node_size_value` when available

A visualization tool may add layout positions, colors, icons, or grouping metadata, but it must not drop evidence or review fields.

## V2 directed bibliographic graph fields

`v0.6.3` adds conservative V2 fields to the global bibliographic graph export.

Graph-level fields:

```json
{
  "schema_version": "refs-graph.v2.1",
  "graph_kind": "global_bibliographic_index_graph",
  "relation_layer": "processing/refs",
  "direction_model": "directed_edges"
}
```

The direction is intentionally narrow: the edge points from the source paper or extracted source text to the referenced paper candidate or unresolved reference node.

Edge-level fields:

```json
{
  "id": "edge::paper_a::paper_b::327",
  "source": "paper_a",
  "target": "paper_b",
  "direction": "source_cites_or_mentions_target",
  "relation_layer": "global_bibliographic_index",
  "relation_type": "cites_or_references",
  "relation_label": "bibliographic index candidate",
  "status": "candidate",
  "confidence": 0.72,
  "evidence": ["processing/text/paper_a.txt:327"],
  "needs_human_review": true,
  "human_final_guarantee_required": true,
  "visual": {
    "edge_style": "dashed",
    "edge_arrow": "directed",
    "source_node_style": "filled",
    "target_node_style": "filled",
    "target_node_marker": "candidate",
    "review_marker": "needs_human_review"
  }
}
```

Node-level weight and visual-size placeholder:

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

`v0.6.3.1` reserves node size for literature importance. This is a placeholder for future ranking commands. Third-party tools must not treat `unknown` / `default` as a real literature importance judgment.

Recommended future mapping:

```text
importance_level: core        -> node_size: large,  node_size_value: 36
importance_level: important   -> node_size: medium, node_size_value: 28
importance_level: background  -> node_size: small,  node_size_value: 20
importance_level: peripheral  -> node_size: tiny,   node_size_value: 14
importance_level: unknown     -> node_size: default,node_size_value: 20
```

## Layer separation rule

Third-party tools must keep these layers separate:

```text
processing/refs/refs_graph_*.json       global bibliographic index graph
topics/<topic>/graph/topic_graph.json   topic-specific causal / method / evidence / idea overlay
```

Do not merge topic-specific causal or method relations into the global bibliographic graph.
