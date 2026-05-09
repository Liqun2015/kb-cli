# Topic Relation Schema

Current version: `v0.6.2`

This document defines a conservative schema for topic-specific literature relations under `topics/<topic>/`.

The goal is not to build a universal causal graph. The goal is to give Manager LLMs, Worker LLMs, humans, and third-party skills a stable way to record and review topic-local relationships among references.

## Scope

Topic relations belong under:

```text
topics/<topic>/relations/
```

They are different from global bibliographic index relations under:

```text
processing/refs/
```

Global index relations answer:

```text
Paper A cites Paper B.
A reference entry may match a local paper.
```

Topic relations answer:

```text
Within this topic, Paper A motivates, improves, supports, contradicts, or uses Paper B.
```

A topic relation must not be treated as a global fact unless a human reviewer explicitly promotes it.

## Minimal relation record

A topic relation record should contain at least:

```yaml
relation_id: thermal-metamaterials-rel-0001
topic: thermal-metamaterials
source: wiki/papers/paper_a.md
target: wiki/papers/paper_b.md
relation_type: uses_method
relation_subtype: adapts_transformation_method
direction: source_to_target
status: candidate
confidence: 0.62
needs_human_review: true
evidence:
  - file: processing/text/raw__papers__paper_a.txt
    line: 120
    quote: "..."
  - file: wiki/papers/paper_b.md
    line: 34
    quote: "..."
created_by: Worker LLM
reviewed_by: null
review_decision: pending
notes: "Candidate relation generated for topic-level review."
```

The same structure may be represented in Markdown table form when easier for human review.

## Required fields

| Field | Meaning |
|---|---|
| `relation_id` | Stable local identifier for this topic relation. |
| `topic` | Topic slug matching `topics/<topic>/`. |
| `source` | The source node. For directed edges, this is the arrow origin. |
| `target` | The target node. For directed edges, this is the arrow destination. |
| `relation_type` | One of the controlled relation types below. |
| `direction` | Usually `source_to_target`; use `bidirectional` only for symmetric topic relations. |
| `status` | Review state: `candidate`, `confirmed`, `rejected`, `ambiguous`, or `needs_human`. |
| `confidence` | 0.0 to 1.0 machine/LLM confidence. It is not a substitute for human review. |
| `needs_human_review` | Boolean. True for causal, improvement, contradiction, and high-impact claims unless already reviewed. |
| `evidence` | File/line/quote evidence supporting the candidate relation. |

## Controlled relation types

Start with a small set:

```text
cites
same_topic
uses_method
improves
supports
contradicts
causal_or_motivates
extends
background_for
unknown
```

Do not create a new relation type casually. Prefer `unknown` plus notes when unsure.

## Status values

```text
confirmed
    A human reviewer has accepted the relation for this topic.

candidate
    The relation is plausible but not yet reviewed.

ambiguous
    Evidence suggests multiple possible targets or meanings.

needs_human
    The relation may be important, causal, or identity-sensitive and needs human review.

rejected
    The candidate was reviewed and rejected.
```

## Direction rule

Topic relations are directed by default.

Examples:

```text
Paper A uses method from Paper B      => A -> B, relation_type = uses_method
Paper A improves Paper B              => A -> B, relation_type = improves
Paper A contradicts Paper B           => A -> B, relation_type = contradicts
Paper A motivates Paper B             => A -> B, relation_type = causal_or_motivates
Paper A and Paper B share a topic      => bidirectional or two directed same_topic records
```

When direction is not clear, use:

```text
direction: unresolved
status: needs_human
```

## Evidence rule

Every topic relation must preserve evidence. Evidence may come from:

```text
processing/text/
wiki/
processing/refs/
processing/keywords/
LLM/tasks/
LLM/memory/
```

A Worker LLM may propose a relation, but it must not omit evidence lines.

## Causal relation caution

Causal or motivating relations are high-risk.

Use:

```text
relation_type: causal_or_motivates
status: candidate or needs_human
needs_human_review: true
```

until a human reviewer confirms the relation.

Do not let a model confidence score turn a causal candidate into a confirmed causal relation.

## Graph visual mapping

Third-party skills should render topic relations as directed edges:

```text
confirmed      -> solid arrow
candidate      -> dashed arrow
ambiguous      -> dashed arrow with ambiguity marker
needs_human    -> dashed arrow with review marker
rejected       -> hidden by default or shown as gray struck-through edge
missing target -> hollow node
```

## Non-goals

Do not use this schema to replace global bibliographic index relations.

Do not infer a universal causal graph across all topics.

Do not auto-confirm causal, contradiction, or improvement relations.

Do not let Worker LLMs redefine the topic scope.
