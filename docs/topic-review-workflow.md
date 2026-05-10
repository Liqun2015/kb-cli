# Topic Review Workflow

This document describes how topic-local importance and relationship candidates should move from generated candidate files to reviewed topic-local knowledge.

Starting in `v0.7.1`, the first deterministic command skeleton is defined as:

```bash
kb topic review <topic>
```

The command is intentionally conservative. It builds a review queue and summary from existing topic-local candidate files. It must not automatically accept, reject, or reinterpret scientific claims.

## Purpose

`kb topic rank <topic>` may generate topic-local importance candidates, but generated candidates are not reviewed knowledge. The topic review workflow creates a bounded, evidence-backed path from candidate files to human-reviewed or explicitly delegated decisions.

The goal is to make topic-local judgments:

- visible
- evidence-backed
- Git-diff friendly
- reviewable by humans or explicit LLM roles
- clearly separated from global bibliographic relations

## Directory Layout

A topic workspace should follow this shape:

```text
topics/<topic>/
├── relations/           # topic-specific relation candidates
├── importance/          # topic-local importance candidates
├── review/              # review queue and working review records
├── reviewed/            # accepted / rejected / deferred review results
├── evidence/            # optional copied evidence snippets or evidence indexes
└── graph/               # optional topic-specific graph overlays
```

`importance/` and `relations/` may contain generated candidates. `review/` contains work in progress. `reviewed/` contains decisions that passed human or explicit Manager LLM review.

## Workflow

```text
1. Initialize or inspect the topic workspace
        ↓
2. Generate topic-local candidates
        ↓
3. Build a deterministic review queue
        ↓
4. Review one bounded item at a time
        ↓
5. Store accepted / rejected / deferred decisions
        ↓
6. Record completed work in LLM/memory/
```

### 1. Initialize or inspect the topic

```bash
kb topic init <topic>
kb topic status <topic>
```

The topic workspace should exist before ranking or review work begins.

### 2. Generate candidates

```bash
kb topic rank <topic>
```

The output under `topics/<topic>/importance/` should be treated as review input. It should not be presented as final importance.

### 3. Build a review queue

`v0.7.1` defines the first deterministic command skeleton:

```bash
kb topic review <topic>
```

The command reads candidate files from:

```text
topics/<topic>/importance/
```

and write:

```text
topics/<topic>/review/review_queue.md
topics/<topic>/review/review_summary.md
```

The command may also create the review directory when missing:

```text
topics/<topic>/review/
```

The queue should include one row per candidate file or candidate item that can be reviewed later.

Suggested queue columns:

```markdown
| review_id | candidate_id | source_item | proposed_decision | status | reviewer | evidence |
|---|---|---|---|---|---|---|
| tr-0001 | importance-0001 | wiki/papers/example.md | core | candidate | unassigned | processing/text/example.txt |
```

The summary should state:

- topic name
- candidate directory
- number of candidate files found
- number of review items generated
- output paths
- warnings, such as missing importance directory or empty evidence fields
- next recommended human / Manager LLM action

### 4. Review one item at a time

A review item should be narrow. Good examples:

```text
Decide whether Paper A is core literature for thermal metamaterials.
Decide whether Paper B provides a method predecessor for Paper C under this topic.
Decide whether this importance candidate should be rejected because evidence is too weak.
```

Bad examples:

```text
Rewrite the whole topic.
Build the complete causal graph.
Decide all important papers automatically.
```

### 5. Store decisions

Accepted, rejected, and deferred decisions should be stored under:

```text
topics/<topic>/reviewed/
```

Recommended files:

```text
accepted.md
rejected.md
deferred.md
review-log.md
```

The format does not need to be complex at first. It only needs to preserve evidence, status, uncertainty, and reviewer notes.

### 6. Record completion memory

Accepted work should also be recorded under:

```text
LLM/memory/completed_tasks.md
```

This keeps topic review decisions discoverable without relying on chat history.

## Decision Status Vocabulary

Use the following review statuses:

| Status | Meaning |
|---|---|
| `candidate` | Generated or proposed, not yet reviewed. |
| `accepted` | Reviewed and accepted as topic-local knowledge. |
| `rejected` | Reviewed and rejected. |
| `deferred` | Review postponed because evidence is insufficient or ambiguous. |
| `needs_human_review` | Requires human judgment before acceptance. |
| `superseded` | Replaced by a newer decision. |

## Topic-local Importance Vocabulary

Topic importance should use conservative labels:

| Label | Meaning |
|---|---|
| `core` | Central to the topic and repeatedly needed for topic explanation. |
| `supporting` | Useful evidence, method, or context for the topic. |
| `background` | Helpful but not topic-defining. |
| `peripheral` | Only weakly connected to the topic. |
| `unknown` | Not enough evidence to judge. |

Do not treat these labels as universal literature importance. They are topic-local.

## Evidence Requirements

Every accepted topic-local decision should include at least one of the following:

- source file path
- extracted text path
- evidence lines or quoted snippets
- reference index report path
- keyword report path
- graph export path
- reviewer explanation

If evidence is missing, the item should remain `candidate`, `deferred`, or `needs_human_review`.

## Manager LLM Role

A Manager LLM may:

- inspect topic workspaces
- run `kb topic review <topic>` to build a review queue
- collect candidate records
- group candidates into bounded review tasks
- assign review tasks to Worker LLMs or human reviewers
- check whether evidence is present
- write review summaries

A Manager LLM must not silently accept all generated candidates.

## Worker LLM Role

A Worker LLM may review one bounded candidate at a time.

It should receive:

```text
goal
candidate record
file list
evidence
allowed output path
review status options
uncertainty rules
forbidden actions
```

It must preserve uncertainty and must not invent evidence.

## Forbidden Actions

Do not:

- edit `raw/`
- treat topic-local importance as global importance
- mix global citation edges with topic-specific causal claims
- accept candidates without evidence
- overwrite reviewed decisions without marking the old decision as `superseded`
- use hidden LLM calls inside deterministic commands
- claim full scientific correctness from keyword co-occurrence alone

## Minimal v0.7.1 Outcome

The minimal outcome for `v0.7.1` is a deterministic queue-building command contract.

The command supports this basic path:

```bash
kb topic rank <topic>
kb topic review <topic>
```

After running, users should see:

```text
topics/<topic>/review/review_queue.md
topics/<topic>/review/review_summary.md
```

A future implementation may add:

```bash
kb topic review <topic> --status
kb topic review <topic> --accept <candidate-id>
kb topic review <topic> --reject <candidate-id>
kb topic review <topic> --defer <candidate-id>
```

For now, the important step is to connect ranking output to a review queue without adding automatic scientific judgment.
