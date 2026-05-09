# kb topic

`kb topic` manages topic-specific literature relationship workspaces.

The command is deterministic scaffolding. It does not call an LLM, infer causal relations, rank literature, or edit wiki pages.

## Initialize a topic workspace

```bash
kb topic init thermal-metamaterials
kb topic init thermal-metamaterials --title "Thermal Metamaterials"
kb topic init thermal-metamaterials --dry-run
kb topic init thermal-metamaterials --force
```

The topic name is normalized into a filesystem-friendly slug. For example:

```text
Thermal Metamaterials -> thermal-metamaterials
```

## Generated structure

```text
topics/<topic>/
  README.md
  scope.md
  literature.md
  importance/
    importance_candidates.md
    importance_review.md
    confirmed_importance.md
  relations/
    causal_relations.md
    method_relations.md
    evidence_relations.md
    idea_relations.md
  review/
    relation_review.md
    importance_review.md
    unresolved.md
  graph/
    README.md
  tasks/
    README.md
  memory/
    confirmed_relations.md
    completed_tasks.md
```

## Layer boundary

Global bibliographic index relations remain under:

```text
processing/refs/
```

Topic-specific importance, causal, method, evidence, idea, review, task, and memory records belong under:

```text
topics/<topic>/
```

## Manager / Worker workflow

1. Manager LLM or human creates a topic workspace with `kb topic init`.
2. Human or Manager LLM edits `scope.md` and `literature.md`.
3. Deterministic commands such as `kb query`, `kb grep`, `kb refs-index`, `kb keywords`, and `kb health` provide evidence.
4. Worker LLM receives bounded tasks under `topics/<topic>/tasks/` or `LLM/tasks/`.
5. High-impact claims are reviewed under `topics/<topic>/review/`.
6. Accepted decisions are recorded under `topics/<topic>/memory/`.

## Non-goals

`kb topic init` does not:

- infer causal relations;
- decide which literature is core;
- call an LLM;
- build a topic graph;
- rewrite wiki pages;
- move global citation/index records out of `processing/refs/`.
