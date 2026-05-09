# Cross-Platform Quick Start

This guide gives the shared workflow. For platform-specific details, see:

- `docs/windows-quickstart.md`
- `docs/unix-quickstart.md`
- `docs/platform-notes.md`

Current version: `v0.6.3.1`

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

`--preview` is an equivalent alias where dry-run behavior is supported:

```bash
kb --kb-path /path/to/literature-folder bootstrap --copy --preview
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

Recursive mode skips managed/generated folders including `raw`, `wiki`, `processing`, `references`, `topics`, `outputs`, `logs`, `.git`, `.obsidian`, `target`, and `node_modules`.

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
kb --kb-path /path/to/literature-folder status --json
kb --kb-path /path/to/literature-folder status --unprocessed
```

The manifest is written to:

```text
/path/to/literature-folder/processing/manifest.json
```

## 8. Plan prepare work

After `status` has created `processing/manifest.json`, preview future AI/human wiki-maintenance work:

```bash
kb --kb-path /path/to/literature-folder prepare --new --dry-run
```

Write review artifacts:

```bash
kb --kb-path /path/to/literature-folder prepare --new
```

This writes `processing/prepare_queue.json` and proposal files under `processing/proposals/`. It does not call an LLM or edit `wiki/`.

## 9. Close the review loop

After a human or AI has edited Markdown pages under `wiki/`, run:

```bash
kb --kb-path /path/to/literature-folder sync-wiki --dry-run
kb --kb-path /path/to/literature-folder sync-wiki
kb --kb-path /path/to/literature-folder lint-static
```

For linting without writing a report:

```bash
kb --kb-path /path/to/literature-folder lint-static --no-report
```

Then run a local keyword query over the prepared wiki:

```bash
kb --kb-path /path/to/literature-folder query thermal cloak
```

## 10. Open in Obsidian

Open the target folder itself, not only `wiki/`:

```text
/path/to/literature-folder
```

The home page will be generated at:

```text
/path/to/literature-folder/wiki/Home.md
```

Before asking an AI agent to maintain the Wiki, review:

```text
/path/to/literature-folder/rules/LLM_WIKI_SCHEMA.md
```

## Review and push changes

For project changes, use the review-first Git workflow:

```text
docs/git-workflow.md
docs/release-checklist.md
```

Windows:

```bat
scripts\git_safe_push.bat "your commit message"
```

macOS/Linux:

```bash
chmod +x scripts/git_safe_push.sh
scripts/git_safe_push.sh "your commit message"
```

