# Topic Review Schema

This document defines lightweight fields for topic-local review records. The schema is intentionally simple so records can be written as Markdown, YAML front matter, JSON, or TOML in future versions.

The schema applies to review records under:

```text
topics/<topic>/review/
topics/<topic>/reviewed/
```

## Minimal Record

A topic review record should contain these fields:

```yaml
review_id: tr-0001
topic: thermal-metamaterials
candidate_id: importance-0001
source_item: raw/papers/example.pdf
source_kind: paper
proposed_decision: core
review_status: candidate
evidence:
  - processing/text/example.txt
  - processing/keywords/keywords_latest.md
uncertainty: evidence incomplete
reviewer: human-or-manager-llm
created_at: 2026-05-10
updated_at: 2026-05-10
```

## Fields

| Field | Required | Meaning |
|---|---:|---|
| `review_id` | yes | Stable ID for the review record. |
| `topic` | yes | Topic workspace name. |
| `candidate_id` | yes | ID of the generated or proposed candidate. |
| `source_item` | yes | Source paper, wiki page, relation record, or extracted text file being reviewed. |
| `source_kind` | yes | `paper`, `note`, `concept`, `method`, `relation`, `keyword`, `graph_node`, or `unknown`. |
| `proposed_decision` | yes | Proposed topic-local judgment. |
| `review_status` | yes | Current review status. |
| `evidence` | yes | Evidence paths, snippets, line references, or report paths. |
| `uncertainty` | recommended | What remains uncertain. |
| `reviewer` | recommended | Human, Manager LLM, Worker LLM, or tool identity. |
| `created_at` | recommended | Creation date. |
| `updated_at` | recommended | Last update date. |
| `notes` | optional | Free-form reviewer notes. |

## Review Status

Allowed `review_status` values:

```text
candidate
accepted
rejected
deferred
needs_human_review
superseded
```

### candidate

The item was generated or proposed but has not been reviewed.

### accepted

The item has been reviewed and accepted as topic-local knowledge.

### rejected

The item has been reviewed and rejected.

### deferred

The item cannot be decided yet because evidence is incomplete, ambiguous, or unavailable.

### needs_human_review

The item is too interpretive or too important for automatic acceptance.

### superseded

The item has been replaced by a newer review record.

## Proposed Decision Values

For topic-local importance:

```text
core
supporting
background
peripheral
unknown
```

For topic-local directed relations:

```text
motivates
extends
improves
contrasts_with
uses_method_from
provides_evidence_for
provides_counterexample_to
shares_mechanism_with
opens_research_gap_for
unknown_relation
```

These relation labels are interpretive. They require evidence and review.

## Evidence Format

Evidence can be represented as paths or structured entries.

Simple Markdown/YAML form:

```yaml
evidence:
  - processing/text/example.txt
  - processing/refs/refs_index_latest.md
  - wiki/papers/example.md
```

Structured form:

```yaml
evidence:
  - path: processing/text/example.txt
    lines: 120-145
    note: method description supports the topic relation
  - path: processing/keywords/keywords_latest.md
    note: keyword co-occurrence candidate only, not final proof
```

## Accepted Record Example

```yaml
review_id: tr-0042
topic: thermal-metamaterials
candidate_id: importance-thermal-0017
source_item: wiki/papers/transformation-thermodynamics-review.md
source_kind: paper
proposed_decision: core
review_status: accepted
evidence:
  - path: wiki/papers/transformation-thermodynamics-review.md
    note: explains coordinate-transformation basis repeatedly used by this topic
  - path: processing/keywords/keywords_20260510.md
    note: keyword evidence only; semantic status reviewed separately
uncertainty: exact priority among core papers remains topic-dependent
reviewer: human
created_at: 2026-05-10
updated_at: 2026-05-10
notes: Accepted as core background for transformation-based thermal metamaterials.
```

## Rejected Record Example

```yaml
review_id: tr-0043
topic: thermal-metamaterials
candidate_id: importance-thermal-0021
source_item: raw/papers/unrelated-materials-paper.pdf
source_kind: paper
proposed_decision: supporting
review_status: rejected
evidence:
  - path: processing/text/unrelated-materials-paper.txt
    note: shares generic keywords but does not address heat control or metamaterial design
uncertainty: none
reviewer: manager-llm-reviewed-by-human
created_at: 2026-05-10
updated_at: 2026-05-10
notes: Rejected because keyword overlap was misleading.
```

## Deferred Record Example

```yaml
review_id: tr-0044
topic: thermal-metamaterials
candidate_id: relation-thermal-0009
source_item: topics/thermal-metamaterials/relations/relation-0009.md
source_kind: relation
proposed_decision: uses_method_from
review_status: deferred
evidence:
  - path: processing/text/paper-a.txt
  - path: processing/text/paper-b.txt
uncertainty: extracted text is incomplete; method relationship may require reading figures or equations
reviewer: worker-llm
created_at: 2026-05-10
updated_at: 2026-05-10
notes: Defer until PDF layout or equations are reviewed.
```

## v0.7.1 Queue-building Note

`v0.7.1` defines `kb topic review <topic>` as a deterministic queue builder. The command populates `topics/<topic>/review/review_queue.md` from candidate files, and leaves every generated row in `candidate` status.

It must not write accepted/rejected/deferred decisions. Those belong under `topics/<topic>/reviewed/` after explicit review.

## Review Queue File

A future queue file may live at:

```text
topics/<topic>/review/review_queue.md
```

Suggested columns after the v0.7.2 usability update:

```markdown
| review_id | candidate_id | source_item | proposed_decision | final_decision | status | reviewer | confidence | evidence | uncertainty | review_notes | accepted_record_path |
|---|---|---|---|---|---|---|---|---|---|---|---|
| tr-0001 | importance-0001 | wiki/papers/example.md | core | pending | candidate | human | medium | processing/text/example.txt | TODO | TODO | topics/<topic>/reviewed/ |
```

The first four columns preserve provenance and proposed candidate meaning. The later columns make the queue usable as a review workbench: reviewers can record a final decision, confidence, uncertainty, notes, and the path where an accepted record should be written.

Use `kb audit-wiki` to check whether existing queues include the required review-workbench columns.

## Review Log

A topic review log may live at:

```text
topics/<topic>/reviewed/review-log.md
```

Each entry should state:

```text
date
review_id
action
old_status
new_status
reviewer
summary
evidence
```

## Compatibility Rule

This schema is deliberately lightweight. Future Rust commands should be able to parse a strict subset, but humans should still be able to read and edit records manually.

Do not make the first implementation depend on a complex database. Keep the record format local, inspectable, and Git-diff friendly.
