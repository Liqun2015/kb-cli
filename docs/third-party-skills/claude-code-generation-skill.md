# Claude Code Generation Skill for Third-party Graph Tools

Current version: `v0.7.40`

This skill is intended for developers who want Claude Code or another coding agent to generate a third-party visualization or inspection tool for LLM Wiki literature relationships.

## Role

You are implementing a third-party skill or tool that reads LLM Wiki relationship data and displays it without changing its meaning.

The tool may be a web UI, Graphviz exporter, Mermaid exporter, Cytoscape exporter, Obsidian-like graph view, review dashboard, or JSON validator.

## Core principle

Do not treat every relation as confirmed.

LLM Wiki relationship data carries certainty state. Your tool must preserve and display that state.

```text
confirmed relation        -> solid edge
candidate / ambiguous     -> dashed edge
missing / unresolved node -> hollow node
needs human review        -> explicit review marker and evidence
```

## Required inputs

The tool should expect relationship records with fields compatible with `relation-graph-json-schema.md`, including:

```text
source
target
relation_type
status
confidence
evidence
needs_human_review
visual
```

## Required behavior

1. Render confirmed relations as solid edges.
2. Render candidate or ambiguous relations as dashed edges.
3. Render missing or unresolved targets as hollow nodes.
4. Preserve evidence text and source file paths.
5. Make `needs_human_review` visible to the user.
6. Never upgrade a candidate relation to confirmed automatically.
7. Never hide missing or ambiguous nodes just to make the graph cleaner.
8. Never use LLM inference to silently change relation status.

## Optional behavior

The tool may add filters:

```text
show only confirmed
show uncertain relations
show only needs_human_review
show missing references
show evidence panel
export selected subgraph
```

The tool may export:

```text
JSON
DOT / Graphviz
Mermaid
Cytoscape JSON
HTML
SVG
```

## Human review boundary

For bibliographic index relations, human review is the final guarantee when identity is uncertain.

A Manager LLM may organize the review workflow, and a Worker LLM may normalize reference strings, but the tool must keep candidate / ambiguous / missing status visible until a human-confirmed record exists.

## Do not do

Do not:

- assume dashed edges are confirmed;
- remove hollow nodes by default;
- overwrite evidence;
- merge nodes without a human-confirmed mapping;
- hide low-confidence relationships;
- turn keyword co-occurrence into scientific idea equivalence;
- call an LLM unless the user explicitly asks for an LLM-enabled version;
- change source relationship files without a review-first workflow.

## Acceptance criteria

A generated third-party skill or tool is acceptable only if:

```text
confirmed -> solid edge
candidate / ambiguous -> dashed edge
missing / unresolved -> hollow node
needs_human_review -> visible warning / marker
evidence -> preserved and inspectable
```

The tool should be boring, explicit, and review-friendly before it is visually impressive.
