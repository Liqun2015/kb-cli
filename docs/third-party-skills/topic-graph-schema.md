# Topic Graph Schema for Third-Party Skills

Current version: `v0.7.30`

This document extends the global relation graph protocol with topic-specific overlays.

Third-party tools should treat global bibliographic graphs and topic graphs as related but separate layers.

## Global graph

Source:

```text
processing/refs/refs_graph_*.json
```

Typical edge:

```text
Paper A cites Paper B
```

This relation is topic-independent.

## Topic graph

Source:

```text
topics/<topic>/graph/topic_graph.json
```

Typical edge:

```text
Within this topic, Paper A improves Paper B.
Within this topic, Paper A supports Paper B.
Within this topic, Paper A motivates Paper B.
```

Topic graph edges must include a `topic` field.

## Minimal node fields

```json
{
  "id": "wiki/papers/example.md",
  "label": "Example Paper",
  "node_type": "paper",
  "topic_importance": "core",
  "importance_score": 85,
  "visual": {
    "node_style": "filled",
    "importance_ring": "core",
    "node_size": "large",
    "node_size_value": "36"
  }
}
```

## Minimal edge fields

```json
{
  "source": "wiki/papers/paper_a.md",
  "target": "wiki/papers/paper_b.md",
  "topic": "thermal-metamaterials",
  "relation_type": "uses_method",
  "direction": "source_to_target",
  "status": "candidate",
  "confidence": 0.62,
  "needs_human_review": true,
  "evidence": [
    {
      "file": "processing/text/raw__papers__paper_a.txt",
      "line": 120,
      "quote": "..."
    }
  ],
  "visual": {
    "edge_style": "dashed",
    "arrow": true,
    "review_marker": true
  }
}
```

## Visual mapping

```text
confirmed topic relation    -> solid arrow
candidate topic relation    -> dashed arrow
ambiguous topic relation    -> dashed arrow with ambiguity marker
needs_human relation        -> dashed arrow with review marker
rejected relation           -> hidden by default or shown as gray rejected edge
core topic paper            -> strong node ring + large node
important topic paper       -> medium node
background paper            -> small / normal node
peripheral paper            -> tiny / low-emphasis node
unknown importance          -> default node size
missing / unresolved node   -> hollow node
```

## Layer rule

A third-party graph viewer should allow users to toggle:

```text
global bibliographic layer
topic relation layer
topic importance layer
needs-human-review layer
```

Do not merge topic-local causal/method/idea relations into global bibliographic index relations.


## Node-size rule for topic graphs

Topic graph viewers should use node size to show topic-local literature importance. This is separate from the global bibliographic index relation layer.

```text
topic_importance: core        -> node_size: large
topic_importance: important   -> node_size: medium
topic_importance: background  -> node_size: small
topic_importance: peripheral  -> node_size: tiny
topic_importance: unknown     -> node_size: default
```

A paper may be globally important but topic-background, or globally obscure but topic-core. Third-party tools must not collapse global and topic-local importance into a single unlabelled score.
