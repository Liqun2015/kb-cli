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
```

## 7. Open in Obsidian

Open the target folder itself, not only `wiki/`:

```text
/path/to/literature-folder
```

The home page will be generated at:

```text
/path/to/literature-folder/wiki/Home.md
```
