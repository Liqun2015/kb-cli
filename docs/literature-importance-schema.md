# Literature Importance Schema

Current version: `v0.7.30`

This document defines a lightweight, reviewable way to record literature importance in LLM Wiki.

The goal is not to compute a universal ranking of all papers. The practical goal is to support topic-centered work: different papers may have different importance under different research topics.

## Two levels of importance

LLM Wiki distinguishes:

```text
global_importance
    General importance independent of a specific topic.

topic_importance
    Importance within a specific topic overlay under topics/<topic>/.
```

Global bibliographic importance may be influenced by citation count, journal quality, classic status, review status, or publication date.

Topic importance may be influenced by whether the paper is central to the research question, provides a method, supplies evidence, offers a counterexample, or only serves as background.

## Recommended topic file

Topic-local importance should be recorded in:

```text
topics/<topic>/importance.md
```

## Minimal record

```yaml
paper: wiki/papers/example_paper.md
topic: thermal-metamaterials
importance_level: core
importance_score: 85
importance_reasons:
  - method_source
  - directly_related_to_topic
  - cited_by_multiple_topic_papers
evidence:
  - file: processing/refs/refs_index_20260509_120000.md
    line: 42
    quote: "..."
assigned_by: Manager LLM
reviewed_by: human
review_status: confirmed
notes: "Core method source for this topic."
```

## Importance levels

Use this small controlled vocabulary first:

```text
core
important
background
peripheral
unknown
```

### `core`

The paper is central to the topic. Removing it would damage the topic map.

Examples:

```text
classic theory source
method origin paper
key review for this specific topic
paper directly defining the topic's main problem
```

### `important`

The paper provides strong support, method details, or important contrast, but is not the central source.

### `background`

The paper helps explain the context, field history, or adjacent method.

### `peripheral`

The paper is only weakly related.

### `unknown`

The importance has not been assessed.

## Importance reasons

Use short reason tags where possible:

```text
classic_highly_cited
recent_top_journal
directly_related_to_topic
method_source
evidence_source
counterexample
review_article
benchmark_or_dataset
used_by_many_local_papers
human_marked_core
weak_background
unknown
```

## Score guidance

`importance_score` is optional. When used, keep it simple:

```text
90-100  core
70-89   important
40-69   background
1-39    peripheral
0       unknown or rejected
```

Scores are aids for sorting, not final truth.

## Visual mapping

Graph tools should render literature importance as node size:

```text
core        -> large node
important   -> medium node
background  -> small node
peripheral  -> tiny node
unknown     -> default node
```

This is only a visual convention. It does not remove the need for evidence, review status, and topic context.

## Human review rule

A Manager LLM or Worker LLM may propose importance, but high-impact labels such as `core` should be reviewed by a human when they will affect research direction.

## Decay and freshness

Do not implement automatic decay aggressively at this stage.

A conservative future rule may distinguish:

```text
classic theory: slow decay
recent top-journal paper: high initial recency weight
temporary note or bug: fast decay
```

For now, record freshness as evidence instead of auto-changing importance.

Recommended optional fields:

```yaml
published_year: 2024
source_quality: peer_reviewed_journal
freshness_note: recent_top_journal
classic_note: null
```

## Non-goals

Do not create a universal score that overrides topic judgment.

Do not treat citation count as the only importance measure.

Do not let model confidence replace human review for core literature decisions.


## Deterministic candidate generation

`kb topic rank <topic>` may propose `core`, `important`, `background`, or `peripheral` labels using simple topic-local signals. These are candidates only. Confirmed labels belong in `topics/<topic>/importance/confirmed_importance.md`.
