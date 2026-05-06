# Windows Quick Start

`v0.4.0` keeps the real quick-start workflow inside the cross-platform Rust CLI. The Windows batch file is only a friendly wrapper around:

```powershell
kb --kb-path <target> bootstrap --copy
```

## Recommended direct command

After installing `kb`:

```powershell
cd D:\github\LLM-wiki\kb-cli
cargo install --path . --force
```

Build a wiki from an existing folder:

```powershell
kb --kb-path "D:\github\LLM-wiki\quantum" bootstrap --copy
```

This will:

```text
1. initialize the folder as a three-layer LLM Wiki
2. generate the rules\ schema layer
3. copy root-level source files into raw\ subfolders
4. extract PDF metadata
5. generate wiki markdown pages
6. refresh processing\manifest.json
```

## Using the batch helper

From the project root:

```powershell
scripts\build_llm_wiki.bat "D:\github\LLM-wiki\quantum"
```

The helper will install `kb` if needed, then run `kb bootstrap`.

## Useful flags

```powershell
scripts\build_llm_wiki.bat "D:\github\LLM-wiki\quantum" --copy
scripts\build_llm_wiki.bat "D:\github\LLM-wiki\quantum" --move
scripts\build_llm_wiki.bat "D:\github\LLM-wiki\quantum" --recursive
scripts\build_llm_wiki.bat "D:\github\LLM-wiki\quantum" --dry-run
scripts\build_llm_wiki.bat "D:\github\LLM-wiki\quantum" --force-metadata
```

## Safety notes

- `--copy` is the default and safest mode.
- `--move` reorganizes files into `raw\` and should be used only when you intend to change the folder layout.
- `--dry-run` previews actions without copying or moving files.
- The recursive mode skips managed folders such as `raw`, `wiki`, `rules`, `processing`, `references`, `outputs`, `logs`, `.git`, `.obsidian`, and `target`.

## Output

Open this folder in Obsidian:

```text
D:\github\LLM-wiki\quantum
```

The generated home page is:

```text
D:\github\LLM-wiki\quantum\wiki\Home.md
```

Manifest file:

```text
D:\github\LLM-wiki\quantum\processing\manifest.json
```

## Rules layer

Before asking an AI agent to maintain the Wiki, review:

```text
D:\github\LLM-wiki\quantum\rules\LLM_WIKI_SCHEMA.md
```


Manifest inspection examples:

```bash
kb --kb-path /path/to/literature-folder status --json
kb --kb-path /path/to/literature-folder status --unprocessed
```
