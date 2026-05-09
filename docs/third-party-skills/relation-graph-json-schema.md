# Relation Graph JSON Schema Guidance

Current version: `v0.5.10.2`

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
    "node_style": "filled"
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
    "node_style": "hollow"
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

A visualization tool may add layout positions, colors, icons, or grouping metadata, but it must not drop evidence or review fields.
