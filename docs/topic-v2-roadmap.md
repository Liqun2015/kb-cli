# Topic V2 Roadmap

Current version: `v0.7.38`

This roadmap keeps the V2 direction practical. Earlier discussions introduced directed relations, node attributes, causal links, importance, and hybrid retrieval. Those ideas are useful, but the near-term goal must stay conservative.

## Near-term goal

```text
For a specific research topic, maintain a reviewable, evidence-backed literature relationship overlay.
```

Do not attempt to build a universal causal knowledge graph.

Do not make every relation global.

Do not let LLMs auto-confirm important causal or bibliographic claims.

## What remains global

Global bibliographic index relations remain under:

```text
processing/refs/
```

These include:

```text
A cites B
reference entry matches local paper
DOI/title/author/year candidate match
```

This layer applies across all topics and should remain topic-independent.

## What becomes topic-local

Topic-specific interpretation lives under:

```text
topics/<topic>/
```

This includes:

```text
topic-local importance
causal or motivating relation candidates
method inheritance
support / contradiction / evidence relations
scientific idea links
```

## Incremental route

### v0.6.1

Topic relationship overlay roadmap.

### v0.6.2

Topic relation schema and literature importance schema.

### v0.6.3

Upgrade graph export fields for directed/topic relation compatibility.

### v0.6.4

Static HTML viewer for reviewing generated Markdown/JSON results.

### v0.6.5

`kb topic init <topic>` creates the topic workspace skeleton for scope, literature, importance, relations, review, graph, tasks, and memory.

### v0.6.6

`kb topic list` and `kb topic status <topic>` inspect topic workspaces before Manager LLM assigns topic-local Worker LLM tasks.

### later

Only after the schemas are stable, consider hybrid retrieval and LLM-assisted topic relation generation.

## Manager LLM rule

The Manager LLM should use deterministic commands first:

```text
kb health
kb query
kb grep
kb refs-index
kb refs-graph
kb keywords
kb tasks
kb topic list
kb topic status <topic>
```

Then it should inspect the relevant topic directory:

```text
topics/<topic>/scope.md
topics/<topic>/literature.md
topics/<topic>/importance/
topics/<topic>/relations/
```

Only then should it assign bounded tasks to Worker LLMs.

## Worker LLM rule

Worker LLMs may propose topic relations, but they must include:

```text
goal
file list
evidence lines
relation_type
status
needs_human_review
uncertainty notes
```

They must not silently promote a candidate relation to a confirmed relation.

## Human guarantee

Humans remain the final guarantee for:

```text
bibliographic identity
core literature decisions
important causal claims
high-impact contradiction claims
```


## v0.6.7 topic-local importance candidates

`kb topic rank <topic>` is the first conservative implementation step after topic workspace scaffolding. It writes candidate importance reports under `topics/<topic>/importance/` because literature importance depends on the research topic. It does not decide final importance and does not call an LLM.
