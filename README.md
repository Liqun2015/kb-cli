# kb-cli

> **v0.5.1 note:** README wording has been clarified around the Karpathy-style workflow. `kb query` remains a deterministic local keyword search over `wiki/**/*.md`; it does not use an LLM, embeddings, vector search, or RAG.

> A small Rust CLI for building and maintaining a local Markdown LLM Wiki.
>
> Current version: `v0.5.1`

`kb-cli` follows a Karpathy-style local knowledge workflow: keep source materials in `raw/`, use `kb prepare` to generate reviewable task materials, maintain Markdown knowledge pages under `wiki/`, and constrain future human/AI maintainers through the `rules/` layer.

## How does the LLM maintain Markdown pages?

In this project, an "LLM-maintained wiki" does **not** mean that the LLM edits the original source files. The workflow is intentionally separated into three layers:

1. `raw/` stores original source materials. These files are treated as read-only records.
2. `wiki/` stores AI-maintained Markdown pages. These pages are summaries, concept notes, comparisons, timelines, indexes, and question-driven analyses derived from `raw/`.
3. `rules/` stores the operating contract for future AI maintainers. It defines page templates, linking rules, update policy, query policy, and lint policy.

The intended maintenance loop is:

```text
raw material enters raw/
        ↓
kb records the file in processing/manifest.json
        ↓
an LLM reads the source material and the rules layer
        ↓
the LLM proposes Markdown updates under wiki/
        ↓
the user reviews the changes through normal file diff / Git diff
        ↓
accepted changes become part of the long-term wiki
```

A single source file may update multiple wiki pages. For example, one new paper may create or update:

```text
wiki/papers/<paper-title>.md
wiki/concepts/<key-concept>.md
wiki/methods/<method-name>.md
wiki/topics/<research-topic>.md
wiki/indexes/papers_index.md
```

The LLM is expected to maintain the wiki by:

- creating a source-specific summary page under `wiki/papers/` or `wiki/notes/`;
- extracting important concepts and updating `wiki/concepts/`;
- linking related pages with Obsidian-style links such as `[[Concept Name]]`;
- adding source references back to the corresponding file in `raw/`;
- updating index pages under `wiki/indexes/`;
- preserving uncertainty instead of inventing unsupported conclusions;
- following the templates and policies generated under `rules/`.

The current version of `kb-cli` provides the filesystem structure, ingestion workflow, rules layer, manifest tracking, static linting, and a first prepare-planning command needed for this maintenance loop. `kb prepare` does **not** call an LLM or directly edit `wiki/` yet. Instead, it generates a reviewable prepare queue, proposal file, and copy-paste agent prompt that an LLM agent can follow under Git review.

```bash
kb prepare --new --dry-run
kb prepare --new
kb prepare --file raw/papers/example.pdf
```

Local keyword query is now available:

```bash
kb query thermal cloak
kb query "thermal cloak" --limit 5
kb query thermal cloak --json
```

Durable saved answers and LLM-assisted query synthesis are planned for later stages.

Static wiki linting is now available:

```bash
kb lint-static
```

Until full autonomous prepare is implemented, users can use the generated `processing/proposals/prepare_plan_*.md` and `processing/proposals/prepare_agent_prompt_*.md` files plus the generated `rules/` files as instructions for Claude Code, ChatGPT, or another coding/knowledge agent to update the Markdown wiki manually under Git review.

## Platform Quick Starts

Use the guide that matches your shell:

```text
docs/windows-quickstart.md      PowerShell and .bat helper
docs/unix-quickstart.md         macOS/Linux bash or zsh
docs/cross-platform-quickstart.md  Shared command flow
docs/platform-notes.md          Path and wrapper-script policy
docs/git-workflow.md            Review-first Git commit/push workflow
docs/release-checklist.md       Small-release checklist
docs/query.md                   Local keyword query skeleton
```

## Current Scope

This version provides a conservative, file-based local LLM Wiki scaffold. It includes cross-platform bootstrap/ingest workflows, the generated Wiki rule/schema layer, a manifest registry under `processing/manifest.json`, static linting, and deterministic keyword search over `wiki/`. It still does **not** provide full RAG, vector search, embeddings, or autonomous LLM/wiki preparation. `kb prepare` currently plans and documents the prepare work; it does not perform LLM edits by itself.

## What Changed in v0.5.1

`v0.5.1` is a documentation clarity release. It does not change query behavior or add any LLM functionality.

Main changes:

- Clarified the README description of the Karpathy-style workflow.
- Made the role of `kb prepare` explicit: it generates reviewable task materials, not final wiki pages.
- Clarified that `wiki/` contains maintained Markdown knowledge pages and `rules/` constrains future human/AI maintainers.

## What Changed in v0.5.0

`v0.5.0` adds the first query skeleton. It is intentionally simple: local Markdown keyword search over `wiki/**/*.md`, with no LLM calls and no semantic retrieval.

Main changes:

- Added `kb query` for local keyword search.
- Added AND-style matching across query terms.
- Added lightweight scoring from title, path, body, and snippet matches.
- Added `--limit`, `--snippets`, `--json`, and `--title-only`.
- Added `docs/query.md`.

Useful query commands:

```bash
kb query thermal cloak
kb query thermal cloak --limit 5
kb query thermal cloak --snippets 3
kb query thermal cloak --title-only
kb query thermal cloak --json
```


## What Changed in v0.4.8

`v0.4.8` is a developer workflow stabilization release before `v0.5.0`. It deliberately does not add query, vector search, embeddings, RAG, or autonomous LLM execution. The goal is to make local changes easier to review, commit, push, and release safely.

Main changes:

- Added `scripts/git_safe_push.bat` for Windows.
- Added `scripts/git_safe_push.sh` for macOS/Linux.
- Added `docs/git-workflow.md` to document the review-first Git workflow.
- Added `docs/release-checklist.md` for small local release preparation.
- Updated README and docs index links to include the Git workflow and release checklist.
- Kept the Rust core workflow unchanged.

Recommended Windows helper usage:

```bat
scripts\git_safe_push.bat "v0.4.8 developer workflow helpers"
```

Recommended macOS/Linux helper usage:

```bash
chmod +x scripts/git_safe_push.sh
scripts/git_safe_push.sh "v0.4.8 developer workflow helpers"
```

## What Changed in v0.4.7

`v0.4.7` is a cross-platform documentation stabilization release before `v0.5.0`. It deliberately does not add query, vector search, embeddings, RAG, or autonomous LLM execution. The goal is to make the same local Wiki workflow easy to run on Windows, macOS, and Linux.

Main changes:

- Added `docs/unix-quickstart.md` for macOS/Linux users.
- Added `docs/platform-notes.md` to define how platform differences should be handled.
- Added `scripts/build_llm_wiki.sh` as a thin Unix shell wrapper around the Rust CLI.
- Updated `docs/cross-platform-quickstart.md` so Windows and Unix paths are both first-class examples.
- Updated `docs/windows-quickstart.md` to align with the v0.4.7 cross-platform documentation set.
- Kept the Rust core workflow unchanged.

Quick-start documents:

```text
docs/windows-quickstart.md
docs/unix-quickstart.md
docs/cross-platform-quickstart.md
docs/platform-notes.md
```

Recommended macOS/Linux helper usage:

```bash
chmod +x scripts/build_llm_wiki.sh
scripts/build_llm_wiki.sh "$HOME/github/LLM-wiki/quantum"
```

## What Changed in v0.4.6.2

`v0.4.6.2` was a documentation polish release before `v0.5.0`. It did not add query, vector search, embeddings, RAG, or autonomous LLM execution. Its purpose was to keep the user-facing workflow language focused on `prepare`.

Main changes:

- `kb prepare` is now the primary command for generating AI/human handoff artifacts.
- Generated artifacts now use prepare-oriented names:
  - `processing/prepare_queue.json`
  - `processing/proposals/prepare_plan_<timestamp>.md`
  - `processing/proposals/prepare_agent_prompt_<timestamp>.md`
- Cargo metadata used `0.4.6-2` for SemVer compatibility, while the release package was labeled `v0.4.6.2`.

Recommended command:

```bash
kb prepare --new --dry-run
kb prepare --new
kb prepare --file raw/papers/example.pdf
```



## What Changed in v0.4.6

`v0.4.6` is a testing and documentation stabilization release before `v0.5.0`. It deliberately does not add query, vector search, embeddings, RAG, or autonomous LLM execution. The goal is to make the existing build/sync/lint loop easier to verify, teach, and run on Windows.

Main changes:

- Added focused Rust regression tests for `build-wiki` source front matter generation.
- Added focused Rust regression tests for `lint-static` WikiLink parsing, source front matter parsing, issue detection, and report-writing behavior.
- Documented the exact local test commands to run before tagging a release.
- Expanded the README with a clearer v0.4.6 testing/release checklist.
- Expanded `docs/windows-quickstart.md` with a PowerShell-first workflow, dry-run/preview examples, lint commands, and common troubleshooting notes.
- Expanded `docs/lint-static.md` with issue categories, examples, cleanup order, and CI/script usage.
- Updated version/schema markers to `0.4.6`.

Recommended release check:

```bash
cargo fmt --check
cargo test
cargo check
cargo build --release
```

## What Changed in v0.4.5

`v0.4.5` is a small consistency release before `v0.5.0`. It does not add query, vector search, embeddings, RAG, or autonomous LLM execution. The goal is to make the existing build/sync/lint loop more self-consistent and less noisy.

Main changes:

- `kb build-wiki` now writes YAML source front matter on generated `wiki/papers/*.md` and `wiki/notes/*.md` pages.
- Generated source-derived pages include `source_files`, and include `source_ids` when a matching manifest entry is available.
- Generated placeholder WikiLinks such as `[[Concept Placeholder]]`, `[[Topic Placeholder]]`, and `[[LLM_WIKI_SCHEMA]]` were removed to avoid guaranteed broken-link noise.
- `--dry-run` remains supported.
- `--preview` is available as an equivalent inspection alias for dry-run-capable commands.
- `kb lint-static --no-report` is available as a clearer alias for checking without writing `outputs/reports/lint_static_*.md`.

Useful lint commands:

```bash
kb --kb-path /path/to/kb lint-static
kb --kb-path /path/to/kb lint-static --no-report
kb --kb-path /path/to/kb lint-static --dry-run
kb --kb-path /path/to/kb lint-static --preview
kb --kb-path /path/to/kb lint-static --json
kb --kb-path /path/to/kb lint-static --strict
```

## What Changed in v0.4.4

`v0.4.4` adds the second step in the traceable AI-maintained Wiki loop: static linting. After `kb prepare` creates an agent task and `kb sync-wiki` links accepted Markdown pages back to the manifest, `kb lint-static` checks whether the Wiki is structurally healthy.

Useful lint commands:

```bash
kb --kb-path /path/to/kb lint-static
kb --kb-path /path/to/kb lint-static --dry-run
kb --kb-path /path/to/kb lint-static --json
kb --kb-path /path/to/kb lint-static --strict
```

The current static lint pass checks broken `[[WikiLinks]]`, orphan pages, source-derived pages missing `source_ids` / `source_files`, empty pages, and duplicate titles. It does not call an LLM and does not rewrite `wiki/`; it writes a reviewable report under `outputs/reports/`.

## What Changed in v0.4.3

`v0.4.3` closes the first prepare loop with `kb sync-wiki`. `kb prepare` can generate a queue and agent prompt; after a human or AI writes source-linked Markdown pages under `wiki/`, `kb sync-wiki` scans their YAML front matter and records the corresponding `wiki_pages` back into `processing/manifest.json`.

`v0.4.2` made `kb prepare` more directly usable as an AI handoff step. In addition to the JSON queue and human-readable prepare plan, non-dry-run prepare now writes a copy-paste agent prompt:

```text
processing/proposals/prepare_agent_prompt_<timestamp>.md
```

The prompt tells Claude Code, ChatGPT, or another knowledge agent exactly how to maintain `wiki/` while respecting the key constraints: never modify `raw/`, read the generated `rules/` files first, create small reviewable Markdown edits, preserve uncertainty, and report every changed page for Git review.

`kb prepare` still does **not** call an LLM and still does **not** directly edit `wiki/`.


## What Changed in v0.4.1

`v0.4.1` is a small prepare-fix release. It fixes a Rust ownership issue in `kb prepare` where a manifest entry ID was moved before the entry was later borrowed to generate proposed wiki pages and instructions.

There is no behavior change from `v0.4.0`: `kb prepare` still generates reviewable prepare plans only. It does not call an LLM and does not edit `wiki/` directly.

## What Changed in v0.4.0

This is the first **prepare skeleton** release. It adds a review-first command for planning how raw files should be prepared for Markdown wiki pages, without calling an LLM and without editing `wiki/` directly.

Useful prepare commands:

```bash
# Preview raw files that are ready for future LLM/wiki preparation
kb --kb-path /path/to/kb prepare --new --dry-run

# Write processing/prepare_queue.json and a reviewable proposal under processing/proposals/
kb --kb-path /path/to/kb prepare --new

# Plan compilation for one raw file listed in processing/manifest.json
kb --kb-path /path/to/kb prepare --file raw/papers/example.pdf
```

Generated files:

```text
processing/prepare_queue.json
processing/proposals/prepare_plan_<timestamp>.md
```

This release is deliberately conservative: it creates the planning artifact that a human or LLM agent can review and execute later under Git diff.

## What Changed in v0.3.2

This is a small cross-platform hygiene release. It keeps the README clarification about how future LLM maintainers should update Markdown pages and adds `.gitattributes` to make line-ending behavior explicit across Windows, Linux, and macOS.

Recommended one-time normalization after updating:

```bash
git add .gitattributes
git add --renormalize .
git status
```

## What Changed in v0.3.1

`kb status` now has safer manifest tracking and better inspection output. It scans `raw/` and writes a manifest registry:

```text
processing/
├── manifest.json
└── proposals/
```

The manifest records each raw file's relative path, kind, extension, size, SHA-256 content hash, first-seen time, last-seen time, status, and future wiki page links. Missing raw files are retained as `raw_missing` instead of being silently dropped. This prepares the project for the next stage: `kb prepare --new`.

Useful status modes:

```bash
kb --kb-path /path/to/kb status
kb --kb-path /path/to/kb status --json
kb --kb-path /path/to/kb status --unprocessed
kb --kb-path /path/to/kb status --dry-run
```

## What Changed in v0.2.0

`kb init` creates the third LLM Wiki layer:

```text
rules/
├── LLM_WIKI_SCHEMA.md
├── PAPER_PAGE_TEMPLATE.md
├── CONCEPT_PAGE_TEMPLATE.md
├── QUERY_POLICY.md
└── LINT_POLICY.md
```

This layer is the "operating contract" for future AI agents. It explains how to treat `raw/`, how to maintain `wiki/`, how to answer questions, and how to lint the Wiki.

## Installation

```bash
cargo build --release
cargo install --path . --force
```

The binary name is:

```bash
kb
```

## Testing and Release Check

Before tagging or handing a build to another agent, run the lightweight local checks:

```bash
cargo fmt --check
cargo test
cargo check
cargo build --release
```

The regression tests added in `v0.4.6` focus on the fragile parts of the current workflow:

```text
build-wiki source front matter
placeholder WikiLink avoidance
lint-static source parsing
lint-static broken-link detection
lint-static --no-report / default report behavior
```

These tests are intentionally small. They protect the reviewable local-Wiki loop without introducing a database, an LLM API, or network access.

## Cross-Platform Quick Start

Turn an existing literature folder into an LLM Wiki in one command:

```bash
kb --kb-path /path/to/your/literature-folder bootstrap --copy
```

Windows example:

```powershell
kb --kb-path "D:\github\LLM-wiki\quantum" bootstrap --copy
```

macOS/Linux example:

```bash
kb --kb-path "$HOME/github/LLM-wiki/quantum" bootstrap --copy
```

The bootstrap command runs:

```text
init -> ingest -> extract-metadata -> build-wiki
```

`ingest` and `bootstrap` also refresh `processing/manifest.json` when they change the raw-file set.

By default, `bootstrap` uses safe copy mode unless `--move` is explicitly provided.

## Manual Workflow

```bash
# 1. Create the local LLM Wiki directory structure and rules layer
kb --kb-path /path/to/kb init

# 2. Organize root-level source files into raw/ subfolders
kb --kb-path /path/to/kb ingest --copy

# 3. Extract metadata from PDFs under raw/papers/
kb --kb-path /path/to/kb extract-metadata

# 4. Generate Markdown wiki pages and indexes
kb --kb-path /path/to/kb build-wiki

# 5. Refresh and inspect the raw-file manifest
kb --kb-path /path/to/kb status

# 6. Plan future LLM/wiki preparation work without editing wiki/ directly
kb --kb-path /path/to/kb prepare --new --dry-run
```

Useful ingest modes:

```bash
# Safest: copy files into raw/
kb --kb-path /path/to/kb ingest --copy

# Move files into raw/
kb --kb-path /path/to/kb ingest --move

# Include subfolders while skipping raw/wiki/.git/.obsidian/target/etc.
kb --kb-path /path/to/kb ingest --copy --recursive

# Preview planned actions without copying or moving
kb --kb-path /path/to/kb ingest --dry-run
```

Useful bootstrap modes:

```bash
kb --kb-path /path/to/kb bootstrap --copy
kb --kb-path /path/to/kb bootstrap --move
kb --kb-path /path/to/kb bootstrap --copy --recursive
kb --kb-path /path/to/kb bootstrap --copy --dry-run
kb --kb-path /path/to/kb bootstrap --copy --force-metadata
```

## Prepare Planning

`kb prepare` is a planning command. It reads `processing/manifest.json`, selects raw files that have not yet been linked to wiki pages, and generates a review artifact for future AI-maintained Markdown edits.

```bash
# Preview what would be selected
kb --kb-path /path/to/kb prepare --new --dry-run

# Write queue and proposal files
kb --kb-path /path/to/kb prepare --new

# Plan one source file
kb --kb-path /path/to/kb prepare --file raw/papers/example.pdf

# Limit the number of selected files
kb --kb-path /path/to/kb prepare --new --limit 5
```

`prepare` currently writes planning files only. It does not call an LLM and does not modify `wiki/`.

## Windows Helper Script

Windows users can also use the bundled wrapper script:

```bat
scripts\build_llm_wiki.bat "D:\github\LLM-wiki\quantum"
```

This script delegates the real work to the cross-platform Rust command:

```bat
kb --kb-path <target> bootstrap --copy
```

## Knowledge Base Layout

```text
KnowledgeBase/
├── raw/
│   ├── papers/       # Original PDF papers
│   ├── notes/        # Notes and documents
│   ├── images/       # Images and figures
│   ├── datasets/     # Data sets
│   ├── archives/     # Compressed source archives
│   ├── repos/        # Code/repository-like source files
│   └── other/        # Other source materials
├── wiki/
│   ├── papers/       # Generated paper pages
│   ├── notes/        # Generated note pages
│   ├── concepts/     # Concept pages
│   ├── topics/       # Topic pages
│   ├── people/       # People/institution pages
│   ├── methods/      # Method/protocol pages
│   ├── comparisons/  # Comparison pages
│   ├── timelines/    # Timeline pages
│   ├── questions/    # Saved Q&A pages
│   └── indexes/      # Navigation indexes
├── rules/            # LLM Wiki schema and AI-maintainer contract
├── outputs/          # Generated answers and reports
├── processing/       # Intermediate processing files
│   ├── manifest.json # Raw-file registry generated by `kb status`
│   └── proposals/    # Future AI edit proposals
├── references/       # Templates and reference materials
└── logs/             # Metadata and logs
```

## Implemented Commands

| Command | Status | Purpose |
|---|---:|---|
| `kb init [--force]` | implemented | Create the local LLM Wiki structure and generated rules layer. |
| `kb ingest [--copy\|--move] [--recursive] [--dry-run\|--preview]` | implemented | Organize source files into `raw/` subfolders. |
| `kb bootstrap [--copy\|--move] [--dry-run\|--preview]` | implemented | Run `init + ingest + extract-metadata + build-wiki`. |
| `kb extract-metadata [--force]` | implemented | Extract basic metadata from PDFs under `raw/papers/`. |
| `kb build-wiki` | implemented | Generate Markdown paper/note pages and index pages. |
| `kb status [--dry-run\|--preview] [--json] [--unprocessed]` | implemented | Refresh `processing/manifest.json`, print JSON summary, or list unprocessed raw files. |
| `kb prepare [--new\|--file PATH] [--dry-run\|--preview]` | implemented skeleton | Generate `processing/prepare_queue.json` and reviewable prepare proposals without calling an LLM. |
| `kb sync-wiki [--dry-run\|--preview] [--json]` | implemented skeleton | Read `source_ids` / `source_files` front matter from `wiki/**/*.md` and update manifest `wiki_pages`. |
| `kb lint-static [--dry-run\|--preview\|--no-report] [--json] [--strict]` | implemented skeleton | Check broken WikiLinks, orphan pages, missing source front matter, empty pages, and duplicate titles. |
| `kb query <terms...> [--limit N] [--snippets N] [--json] [--title-only]` | implemented skeleton | Search `wiki/**/*.md` with local keyword matching. |
| `kb repl` | implemented | Start a simple interactive shell. |
| `kb list-models` | implemented | List configured LLM models. |
| `kb show-model` | implemented | Show the current model configuration. |
| `kb add-model ...` | implemented | Add a model configuration. |
| `kb switch-model <id>` | implemented | Switch the active model. |
| `kb delete-model <id>` | implemented | Delete a model configuration. |
| `kb validate-model [id]` | implemented | Check local model configuration fields. |

## Not Yet Implemented

These are planned directions, not current features:

```text
autonomous AI prepare execution  # `kb prepare` currently plans work only
semantic LLM lint                # Deeper checks for contradictions, stale claims, weak organization
AI apply logic using the manifest
vector search / RAG / JSON agent API
```

## File Classification Used by `ingest`

```text
.pdf                         -> raw/papers
.md/.markdown/.txt/.docx     -> raw/notes
.png/.jpg/.jpeg/.svg/etc.    -> raw/images
.csv/.xlsx/.json/.xml/etc.   -> raw/datasets
.zip/.rar/.7z/.tar/.gz/etc.  -> raw/archives
.rs/.py/.js/.ts/.go/etc.     -> raw/repos
other ordinary files         -> raw/other
```

`ingest --recursive` skips generated/project folders such as:

```text
raw, wiki, processing, references, outputs, logs, .git, .obsidian, target, node_modules
```

It also skips common executable/script/config files such as `.bat`, `.cmd`, `.ps1`, `.sh`, `.exe`, `.dll`, `README.md`, `Cargo.toml`, and `.gitignore`.

## Manifest Documentation

See:

```text
docs/manifest.md
```

## Model Configuration

The real runtime config file is local-only:

```text
.model_config.json
```

It is ignored by Git. Start from the example file if needed:

```bash
cp .model_config.example.json .model_config.json
```

Model-switch request/response files are also ignored:

```text
.model_switch_input.json
.model_switch_output.json
```

Each new REPL LLM request writes a `request_id` to `.model_switch_input.json`; matching output should include the same `request_id` to avoid stale responses.

## Notes for Future Agents

`docs/agent-api.md` records the current boundary: this project is not yet a full machine-readable agent API. Do not assume `--format json`, vector search, `add-paper`, autonomous LLM prepare execution, semantic LLM lint, saved query answers, or full RAG exists in `v0.5.1`.

## License

MIT License - see [LICENSE](LICENSE).

### Sync wiki links back to manifest

After an AI agent or human creates Markdown pages under `wiki/`, run:

```bash
kb --kb-path /path/to/kb sync-wiki
```

`sync-wiki` scans YAML front matter in `wiki/**/*.md`, reads `source_ids` and `source_files`, and updates `processing/manifest.json` so raw files become linked to their prepared wiki pages. Preview first with:

```bash
kb --kb-path /path/to/kb sync-wiki --dry-run
# Equivalent preview alias:
kb --kb-path /path/to/kb sync-wiki --preview
```


### Static wiki lint

After `sync-wiki`, run a structural health check:

```bash
kb --kb-path /path/to/kb lint-static
```

Check without writing a report:

```bash
kb --kb-path /path/to/kb lint-static --no-report
# Equivalent aliases:
kb --kb-path /path/to/kb lint-static --dry-run
kb --kb-path /path/to/kb lint-static --preview
```

The report is saved under:

```text
outputs/reports/lint_static_<timestamp>.md
```
