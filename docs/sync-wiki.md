# sync-wiki

`kb sync-wiki` closes the loop between AI-maintained Markdown pages and the raw-file manifest.

## Purpose

After an AI agent writes pages under `wiki/`, those pages should declare their source files in YAML front matter. `sync-wiki` scans those declarations and updates `processing/manifest.json`.

## Required front matter

```yaml
---
page_type: paper
source_ids:
  - <manifest-id-from-prepare-plan>
source_files:
  - raw/papers/example.pdf
---
```

`source_ids` may contain manifest IDs. `source_files` should contain raw-relative paths. The command matches either field.

## Usage

```bash
kb --kb-path /path/to/kb sync-wiki --dry-run
kb --kb-path /path/to/kb sync-wiki --preview
kb --kb-path /path/to/kb sync-wiki
kb --kb-path /path/to/kb sync-wiki --json
```

## Safety

The command never edits `raw/` or `wiki/`. It only updates `processing/manifest.json` unless `--dry-run` or `--preview` is used.
