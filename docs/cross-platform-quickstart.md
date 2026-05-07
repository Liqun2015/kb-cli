# Cross-Platform Quick Start

This guide works on Windows, macOS, Linux, and Unix-like systems because the core workflow is now implemented inside the Rust CLI.

## 1. Install `kb`

From the `kb-cli` source directory:

```bash
cargo install --path . --force
```

Check that the command is available:

```bash
kb --help
```

## 2. Bootstrap an existing literature folder

Use safe copy mode first:

```bash
kb --kb-path /path/to/literature-folder bootstrap --copy
```

Windows example:

```powershell
kb --kb-path "D:\github\LLM-wiki\quantum" bootstrap --copy
```

macOS/Linux example:

```bash
kb --kb-path "$HOME/github/LLM-wiki/quantum" bootstrap --copy
```

This runs:

```text
init -> ingest -> extract-metadata -> build-wiki
```

It also refreshes:

```text
processing/manifest.json
```

`init` also creates the generated rules layer:

```text
rules/LLM_WIKI_SCHEMA.md
rules/PAPER_PAGE_TEMPLATE.md
rules/CONCEPT_PAGE_TEMPLATE.md
rules/QUERY_POLICY.md
rules/LINT_POLICY.md
```

## 3. Preview before changing files

```bash
kb --kb-path /path/to/literature-folder bootstrap --copy --dry-run
```

## 4. Move instead of copy

Only use this when you intentionally want to reorganize the folder:

```bash
kb --kb-path /path/to/literature-folder bootstrap --move
```

## 5. Include subfolders

```bash
kb --kb-path /path/to/literature-folder bootstrap --copy --recursive
```

Recursive mode skips managed/generated folders including `raw`, `wiki`, `processing`, `references`, `outputs`, `logs`, `.git`, `.obsidian`, and `target`.

## 6. Manual workflow

```bash
kb --kb-path /path/to/literature-folder init
kb --kb-path /path/to/literature-folder ingest --copy
kb --kb-path /path/to/literature-folder extract-metadata
kb --kb-path /path/to/literature-folder build-wiki
kb --kb-path /path/to/literature-folder status
kb --kb-path /path/to/literature-folder prepare --new --dry-run
```

## 7. Inspect manifest status

```bash
kb --kb-path /path/to/literature-folder status
```

The manifest is written to:

```text
/path/to/literature-folder/processing/manifest.json
```

## 8. Open in Obsidian

Open the target folder itself, not only `wiki/`:

```text
/path/to/literature-folder
```

The home page will be generated at:

```text
/path/to/literature-folder/wiki/Home.md
```

## 9. Review the rules layer

Before asking an AI agent to maintain the Wiki, open:

```text
/path/to/literature-folder/rules/LLM_WIKI_SCHEMA.md
```

This file is the operating contract for future AI updates to `wiki/`.


Manifest inspection examples:

```bash
kb --kb-path /path/to/literature-folder status --json
kb --kb-path /path/to/literature-folder status --unprocessed
```

## 10. Plan prepare work

After `status` has created `processing/manifest.json`, preview future AI wiki-maintenance work:

```bash
kb --kb-path /path/to/literature-folder prepare --new --dry-run
```

Write review artifacts:

```bash
kb --kb-path /path/to/literature-folder prepare --new
```

This writes `processing/prepare_queue.json` and a proposal under `processing/proposals/`. It does not call an LLM or edit `wiki/`.
