# Relation Graph Visual Protocol

Current version: `v0.7.29`

This document defines the recommended visual protocol for third-party graph skills and tools that display LLM Wiki literature relationships.

## Core rule

Visual style must reflect relationship certainty.

```text
solid edge  = confirmed relationship
dashed edge = candidate / uncertain relationship
hollow node = unresolved, missing, or pending reference
node size   = literature importance
```


## Node-size rule

`v0.6.3.1` reserves node size for literature importance.

```text
core literature       -> large node
important literature  -> medium node
background literature -> small node
peripheral literature -> tiny / low-emphasis node
unknown importance    -> default node size
```

This rule should be interpreted conservatively. A graph viewer may render node size from `visual.node_size` or `visual.node_size_value`, but it must not infer importance when the graph export says `importance_level: unknown`.

`kb refs-graph` currently emits default / unknown placeholders only. Future `kb rank` or topic-local importance records may provide real importance levels and scores.

Recommended mapping:

| Importance level | `visual.node_size` | `visual.node_size_value` | Meaning |
|---|---:|---:|---|
| `core` | `large` | `36` | Core or classic literature for the graph context |
| `important` | `medium` | `28` | Important but not necessarily foundational literature |
| `background` | `small` | `20` | Useful background literature |
| `peripheral` | `tiny` | `14` | Peripheral or low-emphasis literature |
| `unknown` | `default` | `20` | Importance has not been ranked yet |

## Relation statuses

| Status | Meaning | Recommended edge | Recommended node style | Review requirement |
|---|---|---|---|---|
| `confirmed` | A high-confidence bibliographic relation, usually DOI exact match | solid | filled | auditable, but not normally blocking |
| `candidate` | A plausible match based on title, author, year, journal, filename, or token overlap | dashed | filled | human review required |
| `ambiguous` | More than one plausible target exists | dashed to each candidate | filled candidates | human review required |
| `missing` | A reference entry exists, but no local target has been found | dashed to unresolved node | hollow | human review / acquisition decision required |
| `needs_human` | The tool knows the case cannot be guaranteed automatically | dashed or warning edge | hollow or marked | human review required |

## Confirmed relations

A confirmed relation may be rendered as:

```text
Paper A ━━━━━ Paper B
```

Use this only when the relationship has strong deterministic evidence, such as exact DOI match.

## Candidate relations

A candidate relation should be rendered as:

```text
Paper A - - - Paper B
```

This means that a relation may exist, but the graph must not communicate final certainty.

## Ambiguous relations

Ambiguous relations should preserve all plausible candidates:

```text
Paper A - - - Candidate B1
Paper A - - - Candidate B2
Paper A - - - Candidate B3
```

The visualization may group these candidates, but it must not choose one without review.

## Missing / unresolved references

A missing reference should create an unresolved hollow node:

```text
Paper A - - - ○ Unresolved Reference 001
```

The hollow node represents a reference entry that exists in the source paper but has no confirmed local paper target.

## Human final guarantee

Bibliographic index identity is not guaranteed by visual style alone.

Different journals use different citation formats, and identical references may appear with different author abbreviations, journal abbreviations, page ranges, or title variants. Third-party skills and tools must preserve the `status`, `evidence`, and `needs_human_review` fields so a human reviewer can inspect the match.

## Directed edge rule

`v0.6.3` makes the direction rule explicit.

```text
confirmed global bibliographic relation -> solid directed arrow
candidate / ambiguous relation          -> dashed directed arrow
missing / unresolved target             -> dashed directed arrow to hollow node
```

A directed global edge means:

```text
source paper or extracted source text -> referenced paper candidate or unresolved reference entry
```

It does not mean topic-specific causality. Topic-local causal, method, evidence, and idea arrows belong to `topics/<topic>/graph/` overlays.

Third-party visualizers may render arrows, arrowheads, labels, or edge badges, but they must preserve `direction`, `relation_type`, `status`, `evidence`, `needs_human_review`, and `human_final_guarantee_required`.
