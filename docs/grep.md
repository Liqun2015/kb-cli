# `kb grep`

`kb grep` is a Rust-native, deterministic, grep-like text search command.

It is intended for line-level maintenance work: finding keywords, WikiLinks, source markers, DOI strings, references headings, TODOs, and other structural traces.

It does not call external `grep` or `rg`. It does not call an LLM.

## Examples

```bash
kb grep thermal
kb grep "thermal cloak" --limit 20
kb grep "source_files:" --path wiki
kb grep "References" --path processing/text
kb grep "DOI" --path processing/text
kb grep "10\\.\\d+" --regex --path processing/text
kb grep "\\[\\[.*\\]\\]" --regex --path wiki
kb grep thermal --json
```

## Default search path

By default, `kb grep` searches:

```text
wiki/
```

Use `--path` to search another directory relative to the knowledge base:

```bash
kb grep "References" --path processing/text
```

## `query` vs `grep`

```text
kb query = page-level keyword discovery with lightweight scoring
kb grep  = line-level exact search for maintenance and inspection
```

Use `query` to find relevant pages. Use `grep` to find exact strings, line numbers, and structure markers.

## Current boundaries

`kb grep` does not:

```text
call grep/rg
call LLMs
use embeddings
perform RAG
parse PDFs directly
build a saved index
```
