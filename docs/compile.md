# Compile Planning

`kb compile` is the first step toward LLM-maintained Markdown wiki pages.

In `v0.4.6`, it is intentionally conservative:

- it reads `processing/manifest.json`;
- it selects raw files that appear ready for future wiki compilation;
- it writes `processing/compile_queue.json`;
- it writes a reviewable Markdown proposal under `processing/proposals/`;
- it writes a copy-paste LLM agent prompt under `processing/proposals/`;
- it does **not** call an LLM;
- it does **not** edit files under `wiki/`.

## Typical usage

```bash
kb --kb-path /path/to/kb status
kb --kb-path /path/to/kb compile --new --dry-run
kb --kb-path /path/to/kb compile --new --preview
kb --kb-path /path/to/kb compile --new
```

For one source file:

```bash
kb --kb-path /path/to/kb compile --file raw/papers/example.pdf
```

Limit selected files:

```bash
kb --kb-path /path/to/kb compile --new --limit 5
```

## Generated files

```text
processing/compile_queue.json
processing/proposals/compile_plan_<timestamp>.md
processing/proposals/compile_agent_prompt_<timestamp>.md
```

The proposal and agent-prompt files are meant to be handed to a future AI maintainer, together with the files under `rules/`.

## Review-first design

The intended workflow is:

```text
raw file appears
        ↓
kb status records it in processing/manifest.json
        ↓
kb compile generates a queue, proposal, and agent prompt
        ↓
human reviews the proposal
        ↓
LLM agent updates wiki/ under Git review
        ↓
human reviews git diff and accepts or rejects changes
```

This keeps `raw/` immutable and keeps AI-generated Markdown changes auditable.


## Closing the compile loop

After the generated agent prompt has been used to create or update Markdown pages under `wiki/`, run:

```bash
kb --kb-path /path/to/kb sync-wiki
```

The sync step reads `source_ids` and `source_files` from wiki page front matter and writes the corresponding `wiki_pages` links back into `processing/manifest.json`.
