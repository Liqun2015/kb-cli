# Third-party Skill Developer Contract

Current version: `v0.7.7`

This document states what third-party skills and tools may assume about LLM Wiki relationship data.

## What tools may assume

Third-party skills and tools may assume that LLM Wiki relationship data distinguishes at least these states:

```text
confirmed
candidate
ambiguous
missing
needs_human
```

Tools may also assume that graph style should reflect certainty:

```text
confirmed -> solid edge
candidate / ambiguous -> dashed edge
missing / unresolved -> hollow node
```

## What tools must not assume

Third-party skills and tools must not assume:

- every edge is a confirmed relationship;
- every dashed edge can be safely upgraded automatically;
- a missing node means the paper does not exist in the world;
- a high token-overlap score is a final bibliographic identity match;
- LLM-generated relations are final without evidence and review;
- keyword co-occurrence equals scientific idea equivalence.

## Evidence-first requirement

Every uncertain relation should keep evidence.

Good evidence includes:

- source file path;
- extracted text file path;
- line number;
- raw reference entry;
- DOI string;
- title / author / year tokens;
- candidate local file path;
- command that generated the candidate.

## Human review requirement

For bibliographic index relations, human review is the final guarantee when deterministic identity is uncertain.

Manager LLM may organize the review process, and Worker LLM may normalize or compare entries, but third-party skills and tools should preserve the distinction between machine candidates and human-confirmed relations.

## Future export commands

Future commands may include:

```bash
kb refs-graph --json
kb refs-graph --dot
kb refs-graph --mermaid
```

These commands should follow the visual and JSON conventions in this directory.
