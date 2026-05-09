# `kb extract-text`

`kb extract-text` is a simple deterministic text extraction command.

It converts supported source files into plain text under:

```text
processing/text/
```

The command is intentionally small and explicit. Its purpose is to make later deterministic commands such as `kb grep`, `kb refs`, and `kb query` cheaper and more accurate before any LLM is involved.

## Examples

```bash
kb extract-text --dry-run
kb extract-text
kb extract-text --force
kb extract-text --json
kb extract-text --limit 5
kb extract-text --path raw/papers/example.pdf
kb extract-text --output-dir processing/text/papers
```

After extraction:

```bash
kb grep "References" --path processing/text
kb grep "DOI" --path processing/text
kb grep "10\\.\\d+" --regex --path processing/text
```

## Current scope

Current default behavior is deterministic and best-effort.

Supported input types:

```text
.pdf
.txt
.md
```

For ordinary text-layer PDFs, `kb extract-text` may extract useful plain text. For scanned PDFs, image-based PDFs, complex layouts, or broken encodings, extraction may fail or produce poor text.

When that happens, the command should report the limitation clearly. It should not secretly run OCR, call an LLM, repair text, summarize content, or guess missing structure.


## Explicitly deferred to future LLM/agent skills

The following tasks are intentionally **not** part of default `kb extract-text` behavior. They are future skill work for an explicit PDF Text Conversion Agent:

```text
scanned-page OCR
two-column reading-order repair
garbled text cleanup
section outline recovery
figure/table/caption reconstruction
formula-neighborhood explanation cleanup
semantic summarization
wiki drafting
```

Default `extract-text` should stop at the boundary, report what happened, and leave these cases for a deliberately enabled future mode. See `docs/llm-agent-skills.md`.

## Future role: PDF Text Conversion Agent

`kb extract-text` is reserved as the future entry point for a PDF Text Conversion Agent.

That future agent may eventually coordinate:

```text
1. deterministic text-layer extraction
2. explicit OCR for scanned/image PDFs
3. explicit LLM-assisted cleanup for layout repair, garbled text, or two-column ordering
4. structured conversion of references, figures, tables, and section outlines
```

But the default command must remain simple and non-LLM.

Future advanced behavior must be explicit, for example:

```text
kb extract-text --ocr          # future explicit OCR mode
kb extract-text --llm-clean    # future explicit LLM cleanup mode
kb extract-text --agent        # future explicit PDF conversion agent mode
kb extract-text --pipeline full
```

These modes are not implemented in the current release.

## Principle

`extract-text` should reduce LLM burden, not hide LLM usage.

```text
ordinary text extraction -> deterministic command
scanned pages -> future explicit OCR
layout repair / semantic cleanup -> future explicit LLM or agent stage
summaries / wiki drafting -> future explicit LLM workflow
```
