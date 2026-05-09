# Relation Graph Visual Protocol

Current version: `v0.5.10.2`

This document defines the recommended visual protocol for third-party graph skills and tools that display LLM Wiki literature relationships.

## Core rule

Visual style must reflect relationship certainty.

```text
solid edge  = confirmed relationship
dashed edge = candidate / uncertain relationship
hollow node = unresolved, missing, or pending reference
```

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
