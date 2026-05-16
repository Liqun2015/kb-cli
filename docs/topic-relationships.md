# Topic Relationship Overlay Roadmap

Current version: `v0.7.28`

This document defines the first practical V2 step for LLM Wiki: keep global bibliographic index relations global, and store topic-specific interpretive relationships under `topics/<topic>/`.

The goal is deliberately conservative:

```text
For a specific research topic, maintain a reviewable literature relationship overlay.
Do not attempt to build a universal causal knowledge graph.
Do not treat topic-level interpretations as global facts.
```

## Why this layer exists

The global reference index answers questions that are true across all topics:

```text
Paper A cites Paper B.
Paper A's reference list contains Paper B.
A reference entry may match a local paper by DOI, title, author, journal, year, volume, or pages.
```

These global bibliographic relations live under:

```text
processing/refs/
```

They should remain shared by all topics.

However, higher-level relations are usually topic-dependent:

```text
Paper A is a core source for this topic.
Paper A improves Paper B for this topic.
Paper A contradicts Paper B under this topic's assumptions.
Paper A motivates Paper B in this research direction.
Paper A and Paper B share a method, mechanism, or experimental logic that matters only for this topic.
```

Those relations should live under:

```text
topics/<topic>/
```

## Directory contract

A topic directory should be named with a stable, lowercase, hyphen-separated slug:

```text
topics/thermal-metamaterials/
topics/microfluidic-dialysis/
topics/protein-crystallization/
```

Recommended topic layout:

```text
topics/<topic>/
  README.md
  scope.md
  literature.md
  importance.md
  relations/
    causal_relations.md
    method_relations.md
    evidence_relations.md
    idea_relations.md
  graph/
    topic_graph.json
    topic_graph.mmd
  tasks/
    pending.md
  memory/
    confirmed.md
```

Only the top-level `topics/` directory is created by `kb init` at this stage. Concrete topic subdirectories should be created deliberately by a Manager LLM or human maintainer when a topic becomes active.

## File responsibilities

### `README.md`

Explains the topic in human-readable form.

### `scope.md`

Defines what belongs to this topic and what is out of scope. This file prevents Worker LLMs from expanding a bounded relationship task into a broad literature review.

### `literature.md`

Lists the local papers included in this topic overlay. It should refer back to global files under `raw/`, `processing/text/`, `processing/refs/`, and `wiki/` where possible.

### `importance.md`

Records topic-local importance. A paper may be globally important but peripheral to the topic, or globally obscure but central to the topic.

Recommended fields:

```text
paper
importance_level: core | important | background | peripheral | unknown
importance_reason
evidence
needs_human_review
```

### `relations/`

Stores topic-local interpretive relations.

Recommended conservative relation types:

```text
cites        Global bibliographic relation; usually imported from processing/refs/
same_topic   Shared topic or keyword-level relation
uses_method  One paper uses or adapts a method from another
improves     One paper improves, extends, or makes a method more practical
contradicts  One paper contradicts, challenges, or gives a counterexample
causal       A topic-specific causal or explanatory relation candidate
unknown      A relation is suspected, but its type is not yet confirmed
```

Recommended status values:

```text
confirmed
candidate
ambiguous
needs_human
rejected
```

Important: topic-level causal, method, evidence, and idea relations should usually begin as `candidate` unless a human has confirmed them.

### `graph/`

Stores third-party graph exports for the topic. These should reuse the visual conventions in `docs/third-party-skills/`:

```text
confirmed relation        -> solid directed edge
candidate / ambiguous     -> dashed directed edge
missing / unresolved node -> hollow node
needs human review        -> visible review marker
```

### `tasks/`

Stores topic-specific handoff tasks for Worker LLMs or human reviewers. These should include a goal, requirements, evidence, file list, expected output, and uncertainty rules.

### `memory/`

Stores accepted topic-level decisions, especially confirmed relations, rejected candidates, and unresolved issues.

## Manager LLM navigation rule

When a question is about a specific research topic, the Manager LLM should use this order:

```text
1. Identify or create the topic slug.
2. Inspect topics/<topic>/scope.md if it exists.
3. Inspect topics/<topic>/literature.md and importance.md.
4. Inspect topics/<topic>/relations/ for existing topic-specific relationship records.
5. Use global commands such as kb query, kb grep, kb refs-index, kb refs-graph, kb keywords, and kb health for evidence.
6. Create bounded tasks under topics/<topic>/tasks/ or LLM/tasks/ if Worker LLM or human review is needed.
7. Record accepted decisions under topics/<topic>/memory/ and/or LLM/memory/.
```

Do not treat `processing/refs/` as the final location for topic-specific causal or scientific idea relations. `processing/refs/` is the global bibliographic layer.


## Creating a topic workspace

Use `kb topic init <topic>` to create the recommended topic directory skeleton without making any semantic claims.

```bash
kb topic init thermal-metamaterials
kb topic init thermal-metamaterials --title "Thermal Metamaterials"
```

This command creates files for scope, included literature, topic-local importance, topic-local relations, review tables, graph exports, tasks, and memory. It does not infer causal relations, rank papers, call an LLM, or update wiki pages.

## Relationship layers

```text
Global layer:
  processing/refs/
  Bibliographic index relations that apply across all topics.

Topic overlay layer:
  topics/<topic>/
  Topic-local importance, method, evidence, causal, and scientific idea relations.

Wiki expression layer:
  wiki/
  Human-readable pages that summarize accepted knowledge and link to evidence.
```

## Non-goals for the first topic roadmap

Do not build a universal causal graph.

Do not automatically infer topic importance from a single score.

Do not let a Worker LLM confirm bibliographic identity or high-impact causal claims without review.

Do not move global citation/index relations out of `processing/refs/`.

Do not make topic overlays replace `wiki/`; they provide relationship working records that may later be summarized into wiki pages.

## Future command direction

Potential future commands should be incremental:

```text
kb topic init <topic>        Create the recommended topic directory skeleton. Implemented in v0.6.5.
kb topic list                List topic overlays. Implemented in v0.6.6.
kb topic status <topic>      Check topic workspace structure. Implemented in v0.6.6.
kb topic graph <topic>       Export a topic-specific graph.
kb topic tasks <topic>       Generate bounded Worker LLM / human review tasks for a topic.
```

These commands should remain deterministic. They should prepare evidence and tasks for Manager LLM and Worker LLM workflows, not silently make semantic conclusions.

## Schema documents added in v0.6.2

The topic overlay roadmap is implemented through a small set of conservative schema documents:

```text
docs/topic-relation-schema.md
    Defines topic-local directed relation records, relation types, status values, evidence requirements, and causal-relation caution rules.

docs/literature-importance-schema.md
    Defines global vs topic-local importance, importance levels, reason tags, review expectations, and optional score guidance.

docs/topic-v2-roadmap.md
    Records the incremental V2 route so the project does not jump directly into an over-ambitious universal causal graph.

docs/third-party-skills/topic-graph-schema.md
    Defines how third-party graph tools and skills should read and render topic overlays.
```

Manager LLMs should use these documents as navigation and schema constraints before assigning Worker LLMs to write `topics/<topic>/relations/*.md` or `topics/<topic>/importance.md`.

## Conservative implementation rule

At this stage, topic relations are records and candidates, not automatic truth.

```text
Rust commands discover evidence.
Manager LLM organizes bounded tasks.
Worker LLM proposes topic relations with evidence.
Humans confirm high-impact causal, contradiction, bibliographic identity, and core-literature decisions.
```

Do not treat a topic relation schema as permission to auto-generate a causal graph.


## Topic-local importance ranking

Use `kb topic rank <topic>` to generate deterministic importance candidates for one topic workspace. The output must stay under `topics/<topic>/importance/`. Then use `kb topic review <topic>` to build `topics/<topic>/review/review_queue.md` and `topics/<topic>/review/review_summary.md`. Do not put topic-local importance decisions under global `processing/` directories.
