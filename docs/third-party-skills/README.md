# Third-party Skills

Current version: `v0.6.3.1`

This directory records stable Markdown skill specifications for third-party skills and tools, graph views, web frontends, Obsidian-like plugins, Claude Code projects, Manager LLM workflows, and Worker LLMs that interact with LLM Wiki relationship data.

LLM Wiki is a relationship-maintenance system. Its central object is not a single paper file, but the evolving, reviewable network among references.

## Why this directory is called `third-party-skills`

The documents here are not just documentation for one visualization tool. They are reusable skill contracts.

A developer can hand this directory to Claude Code or another coding agent and ask it to generate a graph viewer, JSON importer, Mermaid exporter, Graphviz exporter, or review UI. The generated tool should follow the relationship-state, evidence, and human-review rules defined here.

## Documents

- `relation-graph-visual-protocol.md` defines the visual mapping from relationship status to graph style.
- `relation-graph-json-schema.md` defines recommended JSON fields for graph exporters and visualization skills and tools.
- `relation-graph-examples.md` gives concrete examples for confirmed, candidate, ambiguous, and missing relations.
- `developer-contract.md` states what third-party skills and tools may assume and what they must not assume.
- `claude-code-generation-skill.md` provides a direct promptable skill spec for Claude Code or similar coding agents.

## Design summary

```text
confirmed relation        -> solid edge
candidate / ambiguous     -> dashed edge
missing / unresolved node -> hollow node
needs human review        -> explicit review marker and evidence
literature importance     -> node size
```

Third-party skills should never silently convert uncertain relations into confirmed relations. Human review remains the final guarantee for bibliographic index identity.


## Topic-specific graph overlays

Third-party graph viewers should distinguish global bibliographic graphs from topic-specific overlays.

```text
processing/refs/refs_graph_*.json       global bibliographic index graph
topics/<topic>/graph/topic_graph.json   topic-specific relationship overlay
```

Global graphs should primarily show citation/index candidates. Topic graphs may show causal, method, evidence, idea, and topic-local importance relations, but they must preserve relation status, evidence, and human-review markers.

## Topic graph skill schema

`v0.6.3` adds:

```text
docs/third-party-skills/topic-graph-schema.md
```

Third-party visualizers should treat global bibliographic graphs and topic-specific overlays as separate layers. Topic-local causal, method, evidence, idea, and importance fields must not be silently merged into the global reference index.

## V2 directed graph export

`v0.6.3` upgrades the global `refs-graph` JSON shape so third-party skills can read directed bibliographic edges and placeholder node-weight fields.

This does not introduce a universal causal graph. It only makes the existing global bibliographic index graph more explicit:

```text
source paper / extracted source text -> referenced paper candidate / unresolved reference
```

Topic-specific causal and method relations should still be read from `topics/<topic>/` overlays.


## Node size and literature importance

Graph skills should reserve node size for literature importance. Edge style represents certainty, arrow direction represents relation direction, hollow nodes represent unresolved references, and node size represents importance.

`kb refs-graph` currently emits `unknown` / `default` importance placeholders only. Future ranking or topic-local importance data may set `core`, `important`, `background`, or `peripheral` values.
