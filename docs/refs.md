# `kb refs`

`kb refs` scans extracted text for deterministic reference hints. It is a small, Rust-native inspection command, not a citation-understanding agent.

Default input:

```text
processing/text/
```

Typical flow:

```bash
kb extract-text
kb refs
kb refs --json
kb refs --citations
```

## What it detects

The first skeleton detects simple textual patterns:

```text
reference_heading      References / Bibliography / Works Cited / 参考文献
reference_entry        [1] ... or 1. ... style bibliography entries
doi                    DOI values such as 10.xxxx/xxxxx
numeric_citation       [1], [1, 2], [1-3] when --citations is enabled
author_year_citation   (Smith 2020) style markers when --citations is enabled
```

## Examples

Scan the default extracted-text directory:

```bash
kb refs
```

Scan a specific directory:

```bash
kb refs --path processing/text/papers
```

Scan one text file:

```bash
kb refs --path processing/text/raw_papers_example.txt
```

Include noisier in-text citation markers:

```bash
kb refs --citations
```

Output JSON:

```bash
kb refs --json
```

Limit returned hints:

```bash
kb refs --limit 50
```

Use `--limit 0` to print all returned hints.


## Explicitly deferred to future LLM/agent skills

The following tasks are intentionally **not** part of default `kb refs` behavior. They require semantic judgment and should be handled only by future explicit LLM/agent modes:

```text
reference reconciliation across formatting variants
DOI correction or online metadata lookup
matching in-text citations to bibliography entries
disambiguating duplicate or ambiguous references
normalizing author/title/venue/year fields
building a reliable citation graph
explaining how one paper depends on another
```

`kb refs` should output evidence and hints. A future Reference Reconciliation Agent or Citation Graph Building Agent may consume those hints, but it must produce reviewable proposals and uncertainty labels. See `docs/llm-agent-skills.md`.

## Boundary

`kb refs` is a deterministic text-pattern scanner. It does not:

- parse PDF files directly;
- call OCR;
- call an LLM;
- infer whether two papers are actually the same work;
- build a reliable citation graph;
- repair broken reference formatting;
- summarize cited papers.

For now, use it as a rough inspection tool after `kb extract-text`.

Future versions may add explicit commands such as:

```text
kb refs graph
kb refs doi
kb refs export
```

Those future commands should remain deterministic by default. Any LLM-assisted citation reconciliation must be introduced only as an explicit agent/LLM mode.


## Next step: `kb refs-index`

After `kb refs` finds reference hints, run:

```bash
kb refs-index
```

`kb refs-index` builds bibliographic index relation candidates against local papers. It does not replace human review.
