# `kb keywords`

Current version: `v0.7.8`

`kb keywords` detects deterministic keyword / topic co-occurrence candidates among extracted text files.

It belongs to the second level of the LLM Wiki literature relationship model:

```text
Level 1: bibliographic index relations  -> refs / refs-index / refs-graph
Level 2: keyword / topic relations       -> keywords
Level 3: scientific idea relations       -> future explicit Manager/Worker LLM workflows
```

## Purpose

`kb keywords` does one thing: it finds shared keyword/topic traces across local text files.

It does **not** decide that two papers have the same scientific idea. It only reports evidence that they share terms or topic markers.

The hard semantic question is deferred:

```text
Do these shared keywords represent a meaningful research relationship?
```

That question belongs to a Manager LLM / Worker LLM workflow, and important conclusions should remain reviewable by humans.

## Inputs

Default input:

```text
processing/text/
```

This means the usual flow is:

```bash
kb extract-text
kb keywords
```

You can also scan a specific path:

```bash
kb keywords thermal cloak --path processing/text
kb keywords diffusion conductivity --path processing/text/papers
```

## Outputs

Default report path:

```text
processing/keywords/keywords_YYYYMMDD_HHMMSS.md
```

The report contains:

- selected terms;
- files containing each term;
- line-level evidence snippets;
- candidate keyword/topic relations between files;
- a deferred Worker task for semantic review.

## Usage

Auto-extract simple candidate terms:

```bash
kb keywords
```

Scan explicit terms:

```bash
kb keywords thermal cloak metamaterial
kb keywords "thermal cloak" "transformation thermodynamics"
```

Use a newline-separated terms file:

```bash
kb keywords --terms-file references/thermal_terms.txt
```

Preview without writing a report:

```bash
kb keywords --dry-run
kb keywords --preview
```

JSON output:

```bash
kb keywords thermal cloak --json
```

Limit printed relation rows:

```bash
kb keywords --limit 50
```

## Deterministic behavior

`kb keywords` is Rust-native and deterministic. It does not call:

- LLM APIs;
- OCR;
- vector databases;
- online scholarly services;
- external `grep` / `rg` commands.

## Relation status

All keyword/topic relations are currently emitted as:

```text
candidate
```

Reason: keyword co-occurrence is evidence, not proof.

A future Worker may promote a candidate into a wiki relationship explanation, but only after reading the relevant evidence and preserving uncertainty.

## Deferred task

When keyword/topic candidates are found, the report includes a task for:

```text
Keyword Topic Relation Worker
```

The task asks the Worker to:

- review shared keyword evidence;
- decide whether the relation is scientifically meaningful;
- avoid treating shared terms as confirmed scientific idea relations;
- preserve file lists and evidence lines;
- record accepted decisions with `kb memory`.

## Boundaries

`kb keywords` does not:

- summarize papers;
- infer scientific idea relations;
- rewrite wiki pages;
- merge topics;
- create concept pages;
- confirm that two papers are methodologically related.

Those tasks belong to future explicit Manager LLM / Worker LLM workflows.
