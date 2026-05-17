# Cross-Platform Quick Start

This guide gives the shared workflow. For platform-specific details, see:

- `docs/windows-quickstart.md`
- `docs/unix-quickstart.md`
- `docs/platform-notes.md`

Current version: `v0.7.37`

## 1. Install `kb`

From the `kb-cli` source directory:

```bash
cargo install --path . --force
```

Check that the command is available:

```bash
kb --help
```

## 2. Set up an existing literature folder

Use safe copy mode first:

```bash
kb --source /path/to/literature-folder setup --recursive
```

Windows example:

```powershell
kb --source "D:\github\LLM-wiki\quantum" setup --recursive
```

macOS/Linux example:

```bash
kb --source "$HOME/github/LLM-wiki/quantum" setup --recursive
```

This runs:

```text
init -> ingest -> extract-metadata -> build-wiki
```

It also refreshes:

```text
knowledgebase/processing/manifest.json
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
kb --source /path/to/literature-folder setup --recursive --dry-run
```

`--preview` is an equivalent alias where dry-run behavior is supported:

```bash
kb --source /path/to/literature-folder setup --recursive --preview
```

## 4. Move instead of copy

Only use this when you intentionally want to reorganize the folder:

```bash
kb --source /path/to/literature-folder setup --recursive --move
```

## 5. Include subfolders

```bash
kb --source /path/to/literature-folder setup --recursive
```

Recursive mode skips managed/generated folders including `knowledgebase`, `raw`, `wiki`, `processing`, `references`, `topics`, `interfaces`, `.git`, `.obsidian`, `target`, and `node_modules`.

## 6. Manual workflow

```bash
kb --lib /path/to/literature-folder/knowledgebase init
kb --source /path/to/literature-folder --lib /path/to/literature-folder/knowledgebase ingest --copy --recursive
kb --lib /path/to/literature-folder/knowledgebase extract-metadata
kb --lib /path/to/literature-folder/knowledgebase build-wiki
kb --lib /path/to/literature-folder/knowledgebase status
kb --lib /path/to/literature-folder/knowledgebase build <topic> --dry-run
```

## 7. Inspect manifest status

```bash
kb --lib /path/to/literature-folder/knowledgebase status
kb --lib /path/to/literature-folder/knowledgebase status --json
kb --lib /path/to/literature-folder/knowledgebase status --unprocessed
```

The manifest is written to:

```text
/path/to/literature-folder/knowledgebase/processing/manifest.json
```

## 8. Build topic workbench

After `status` has created `processing/manifest.json`, preview the deterministic topic build pipeline:

```bash
kb --lib /path/to/literature-folder/knowledgebase build <topic> --dry-run
```

Write review artifacts:

```bash
kb --lib /path/to/literature-folder/knowledgebase build <topic>
```

This writes deterministic text/reference/topic task and handoff artifacts. It does not call an LLM or edit final scholarly claims.

## 9. Close the review loop

After a human or AI has edited Markdown pages under `wiki/`, run:

```bash
kb --lib /path/to/literature-folder sync-wiki --dry-run
kb --lib /path/to/literature-folder sync-wiki
kb --lib /path/to/literature-folder lint-static
```

For linting without writing a report:

```bash
kb --lib /path/to/literature-folder lint-static --no-report
```

Then run a local keyword query over the wiki:

```bash
kb --lib /path/to/literature-folder query thermal cloak
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

