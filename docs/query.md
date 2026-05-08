# Query Skeleton

`kb query` searches local Markdown pages under `wiki/` using deterministic keyword matching.

This query skeleton was introduced in v0.5.0 and remains intentionally small and inspectable in v0.5.4.

## What it does

- Scans `wiki/**/*.md`.
- Removes YAML front matter before body search.
- Extracts the first `# Heading` as the page title.
- Splits the user query into whitespace-separated keywords.
- Requires all query terms to appear in the page title, path, or body.
- Ranks matching pages with a simple score based on title, path, body, and snippet matches.
- Prints matching paths, titles, scores, matched terms, and line snippets.
- Can emit JSON for scripts.

## What it does not do

`kb query` does **not** use:

```text
LLM calls
embeddings
vector search
RAG
semantic reranking
saved answer generation
network access
background indexing
```

It is a local, on-demand Markdown keyword search only.

## Basic usage

```bash
kb query thermal cloak
kb query "thermal cloak"
kb --kb-path /path/to/KnowledgeBase query thermal cloak
```

Multiple terms use AND semantics. For example:

```bash
kb query thermal cloak
```

returns pages that contain both `thermal` and `cloak` somewhere in the title, path, or Markdown body.

## Limit results

```bash
kb query thermal cloak --limit 5
```

Use `--limit 0` to return all matching pages.

## Control snippets

```bash
kb query thermal cloak --snippets 3
kb query thermal cloak --snippets 0
```

Snippets are matching Markdown body lines. They are intended for quick review, not as durable citations.

## Title/path-only search

```bash
kb query thermal cloak --title-only
```

This searches only page titles and relative paths. It ignores Markdown body text.

## JSON output

```bash
kb query thermal cloak --json
```

The JSON report includes:

```text
schema_version
generated_by
generated_at
query
terms
title_only
pages_scanned
match_count
returned_count
results[]
```

Each result includes:

```text
path
title
score
matched_terms
snippets[]
```

## Suggested workflow

After building or editing the wiki:

```bash
kb sync-wiki
kb lint-static
kb query thermal cloak
```

Use `kb query` after `kb lint-static` so you are searching a structurally healthier wiki.

## Design boundary

The current command is deliberately simple. Future versions may add saved queries, better tokenization, ranking improvements, and optional semantic retrieval. Those should remain explicit features, not hidden behavior.
