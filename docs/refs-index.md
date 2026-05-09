# kb refs-index

`kb refs-index` builds **bibliographic index relation candidates** among local references.

It is part of the LLM Wiki literature-relationship core: the goal is not merely to store papers, but to maintain reviewable relationships among papers.

## Purpose

`kb refs-index` tries to connect reference entries found in extracted text to local papers under `raw/papers/`.

It uses deterministic evidence only:

- DOI exact matches when available
- local paper metadata from `logs/papers_metadata.json`
- local paper filenames
- title / filename token overlap
- reference entry lines from `processing/text/`

## Basic usage

```bash
kb refs-index
```

Default input:

```text
processing/text/
```

Default report output:

```text
processing/refs/refs_index_YYYYMMDD_HHMMSS.md
```

Useful examples:

```bash
kb extract-text
kb refs
kb refs-index
kb refs-index --dry-run
kb refs-index --json
kb refs-index --limit 50
kb refs-index --path processing/text/papers
```

## Relation statuses

`kb refs-index` classifies relation candidates as:

| Status | Meaning | Final guarantee |
|---|---|---|
| `confirmed` | DOI exactly matches one local paper | Still auditable, but high-confidence |
| `candidate` | title / filename tokens suggest one local paper | human review required |
| `ambiguous` | multiple local papers match equally well | human review required |
| `missing` | no local paper match was found | human review required |

## Human final guarantee

Bibliographic index relations are the most direct relationship layer in LLM Wiki, but they are also easy to get wrong.

Different journals use different reference styles. The same paper may be written with different author abbreviations, title casing, journal abbreviations, page ranges, or DOI formatting.

Therefore:

```text
Rust-native commands find candidates.
Humans provide the final guarantee.
Manager LLM may organize the review process.
Worker LLM may assist with normalization.
But final bibliographic identity confirmation belongs to a human reviewer.
```

## Deferred task handoff

When `candidate`, `ambiguous`, or `missing` relations are found, the command returns a deferred review task. The task includes:

- target reviewer
- goal
- requirements
- file list
- evidence lines

This task is meant for the Manager LLM to route to the right Worker LLM or human reviewer.

## Boundaries

`kb refs-index` does **not**:

- call an LLM
- query Crossref / Google Scholar / Semantic Scholar
- download missing papers
- merge duplicate references automatically
- build a final citation graph
- declare human-review matches as final
- rewrite wiki pages

It only builds an evidence-backed candidate table.

## Recommended workflow

```bash
kb extract-text
kb grep "References" --path processing/text
kb refs --path processing/text
kb refs-index
kb tasks
```

Then the Manager LLM can inspect the generated reports and decide which review tasks should go to a Worker LLM and which must go directly to a human.

## Third-party visualization

`kb refs-index` status values should be graph-tool friendly:

```text
confirmed -> solid edge
candidate -> dashed edge
ambiguous -> dashed edges to multiple candidates
missing   -> dashed edge to a hollow unresolved node
```

See `docs/third-party-skills/` for the recommended visual protocol, JSON field guidance, and Claude Code generation skill.
