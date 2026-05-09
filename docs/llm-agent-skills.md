# Deferred LLM / Agent Skills

This document records work that deterministic `kb-cli` commands should deliberately leave for future explicit LLM or agent modes.

The purpose is to reduce unnecessary LLM usage: simple file traversal, text extraction, grep-like search, DOI matching, and reference-hint scanning should be handled by Rust-native commands first. LLMs should be used only when semantic judgment, repair, synthesis, or writing is actually needed.

## Core rule

Deterministic commands should do the simple work well and mark the boundary clearly.

They must not silently call OCR, LLM APIs, web lookup, vector search, or autonomous editing.

Future LLM/agent work must be explicit, reviewable, and grounded in deterministic evidence such as:

```text
processing/text/
processing/refs/
outputs/reports/
wiki/**/*.md
processing/manifest.json
```

## Skill 1: PDF Text Conversion Agent

Related deterministic command:

```bash
kb extract-text
```

Current deterministic responsibility:

- copy `.txt` / `.md` text into `processing/text/`;
- attempt best-effort text-layer extraction from ordinary PDFs;
- record failures or poor extraction clearly;
- avoid OCR, LLM cleanup, summarization, or semantic repair by default.

Future LLM/agent responsibilities, only through explicit modes:

- decide whether a PDF is scanned, image-based, two-column, garbled, or layout-broken;
- coordinate explicit OCR when allowed;
- repair reading order for two-column papers;
- clean garbled or duplicated text;
- preserve section boundaries, captions, tables, equations, and references where possible;
- produce a reviewable conversion report explaining what was changed and what remains uncertain.

Possible future explicit interfaces:

```text
kb extract-text --ocr
kb extract-text --llm-clean
kb extract-text --agent
kb extract-text --pipeline full
```

## Skill 2: Reference Reconciliation Agent

Related deterministic command:

```bash
kb refs
```

Current deterministic responsibility:

- find reference headings;
- find simple bibliography-entry patterns;
- find DOI-like strings;
- optionally find numeric and author-year citation markers;
- output hints, not conclusions.

Future LLM/agent responsibilities, only through explicit modes:

- decide whether two differently formatted references are the same work;
- repair broken reference entries split across lines;
- normalize author names, titles, venues, years, and DOI strings;
- distinguish bibliography entries from ordinary numbered lists;
- connect in-text citations to bibliography entries when the evidence is sufficient;
- produce uncertainty labels instead of pretending ambiguous matches are certain.

Possible future explicit interfaces:

```text
kb refs reconcile --agent
kb refs normalize --llm
kb refs export --review
```

## Skill 3: Citation Graph Building Agent

Related deterministic foundation:

```bash
kb refs --json
kb grep "10\.\d+" --regex --path processing/text
```

Current deterministic responsibility:

- collect raw citation and reference hints.

Future LLM/agent responsibilities, only through explicit modes:

- infer source-paper to cited-paper relationships from incomplete evidence;
- group citations across multiple papers;
- identify likely duplicate reference records;
- create a reviewable citation graph proposal;
- never overwrite long-term graph files without review.

Possible future explicit interfaces:

```text
kb refs graph --agent
kb refs graph --review-only
```

## Skill 4: Layout Repair / Structure Recovery Agent

Related deterministic foundation:

```bash
kb extract-text
kb grep
```

Future LLM/agent responsibilities, only through explicit modes:

- recover section structure from noisy extracted text;
- repair two-column reading order;
- separate body text, captions, tables, references, appendices, and footnotes;
- mark all uncertain repairs.

Possible future explicit interfaces:

```text
kb extract-text --llm-clean
kb extract-text --structure --agent
```

## Skill 5: Wiki Drafting / Concept Synthesis Agent

Related deterministic foundation:

```bash
kb prepare
kb query
kb grep
kb refs
```

Future LLM/agent responsibilities, only through explicit modes:

- summarize papers into `wiki/papers/`;
- update concept pages under `wiki/concepts/`;
- propose backlinks and index-page updates;
- merge evidence across sources;
- preserve uncertainty and source references;
- produce diffs or proposals for human review.

This work must remain separate from deterministic search, extraction, and scanning commands.

## Output expectations for future skills

Future LLM/agent skills should prefer reviewable artifacts such as:

```text
processing/proposals/*.md
processing/refs/*.json
processing/refs/*.md
outputs/reports/*.md
```

They should not silently modify `raw/`, and they should not directly rewrite long-term `wiki/` content without a review path.


## Handoff source

Future LLM/agent skills should receive work from explicit task handoff reports, not from hidden CLI behavior. `kb tasks` writes these reports under `LLM/tasks/` and includes the target agent, goal, requirements, file list, and evidence needed for review. The `LLM/` directory is the planned handoff/workbench layer for future explicit skills; deterministic commands should only prepare these materials, not execute the agent work implicitly.


## Completed-task memory

When a human, LLM, or future sub-agent finishes a task from `LLM/tasks/`, the result should be recorded with `kb memory`. Completed memories are written to `LLM/memory/completed_tasks.md` and should include the task id, source task report, files touched or produced, a concise summary, and unresolved follow-up items.

This keeps the agent workflow inspectable without relying on chat history or hidden model state.


## Manager vs worker hierarchy

The LLM that uses `kb-cli` commands is the Manager LLM. It is the highest-level coordinator in the LLM layer.

Worker LLMs are lower-level executors. They should receive bounded tasks from `LLM/tasks/` or from the Manager LLM, with explicit goals, requirements, file lists, evidence, non-goals, and expected outputs.

A worker task should not rediscover the whole project, redefine scope, or decide the global workflow. The Manager LLM should use deterministic commands first, then delegate only the hard semantic part.

See `docs/llm-hierarchy.md`.


## Wiki Link Repair Agent

Triggered by `kb links` or `kb tasks` when unresolved or ambiguous WikiLinks are found. The agent should decide whether each link should target an existing page, create a new page proposal, be renamed, preserve its alias, or be converted to plain text. It must report source file, line number, original link, chosen action, confidence, and unresolved issues.


## Human-backed bibliographic index review

`kb refs-index` produces candidate bibliographic index relations. When it emits candidate, ambiguous, or missing rows, the next step is not automatic citation-graph construction. The Manager LLM should prepare bounded review packets for humans or Worker LLMs.

Worker LLMs may help normalize titles, compare author strings, and summarize evidence. Humans remain the final guarantee for uncertain bibliographic identity decisions.


## Graph Visualization Support Skill Boundary

Future graph exporters, visualization assistants, and third-party skills should follow `docs/third-party-skills/`. They may format relationship data for Graphviz, Mermaid, Cytoscape, web frontends, or Obsidian-like graph views, but they must preserve `status`, `evidence`, and `needs_human_review`.
