# kb-cli

`kb-cli` is a small Rust command-line tool for building, analyzing, and maintaining a local Markdown-based LLM Wiki.

It is designed for researchers, developers, and human/AI collaborative workflows that need a local, reviewable, file-based knowledge system instead of a black-box note database.

## Current Version

Current version: `v0.7.24`

### v0.7.24

Library Flag release. The global library selector is now `--lib` across the CLI. This aligns every command with the explicit create syntax:

```bash
kb create --lib <A> --from <B> --about <topic>
kb --lib <A> view
kb --lib <A> build <topic>
```

`--lib` means “the target directory of LLM Wiki library/database.”

### v0.7.23

LLM Launch Panel release. `kb view` now adds a safe **启动 LLM** button to `interfaces/html/index.html`. It opens a static panel with copyable Claude Code / Codex launch commands and bounded Manager/Worker prompts. The HTML still does not execute shell commands or call an LLM; it only helps the operator start an external agent deliberately.

### v0.7.22

Create Command release. `kb create` is now the recommended one-command path for turning an existing literature folder into a named, agent-ready LLM Wiki database.

```bash
kb create --lib <A> --from <B> --about <topic>
```

This is equivalent to:

```bash
kb --lib <A> --source <B> setup --recursive --topic <topic>
kb --lib <A> build <topic>
```

`setup` and `build` are still available when debugging or rerunning only part of the workflow.

### v0.7.20

Interface Directory release. The generated, non-source artifact layer is now named `interfaces/` rather than `outputs/`, because its real role is human-machine interaction and machine-machine handoff/communication. `interfaces/README.md`, `AGENTS.md`, `CLAUDE.md`, and handoff files explicitly tell agents that `interfaces/` is not the durable knowledge source. Generated HTML/reports/logs remain disposable; Markdown/JSON/TOML under `wiki/`, `topics/`, `processing/`, `LLM/`, `rules/`, and `llm-wiki.toml` remain the source of truth.

### v0.7.18

Remove Legacy Prepare release. The old top-level `kb prepare` command and the legacy `kb topic prepare <topic>` alias have been removed from the active CLI. Their responsibilities are now handled by the clearer build pipeline:

```text
raw/source handling        -> kb setup / kb ingest / kb status
text and section evidence  -> kb extract-text / kb extract-sections
reference candidates       -> kb refs / kb refs-index / kb refs-graph
topic importance/relations -> kb topic build <topic>
topic task handoff         -> kb topic tasks <topic>
agent/Obsidian entrypoints -> kb handoff / kb topic handoff <topic>
one-command assembly       -> kb build <topic>
```

The recommended user path is now:

```bash
kb create --lib <A> --from <B> --about <topic>
```

Expanded form:

```bash
kb --lib <A> --source <B> setup --recursive --topic <topic>
kb --lib <A> build <topic>
```

### v0.7.17

Workflow naming realignment release. The second, topic-specific LLM Wiki workflow moved from old `prepare` wording to `build` wording. `kb topic build <topic>` became the preferred wrapper for topic-local importance ranking plus directed relation graph compilation. In v0.7.18 the legacy alias was removed from the active CLI.

The two recommended workflows are now:

```text
Workflow 1: init/setup -> build global evidence and generic citation/reference layers -> inspect with kb view
Workflow 2: kb topic build <topic> -> kb topic review/tasks/handoff -> inspect with kb view --relations --topic <topic>
```

For the complete post-setup assembly, continue to use:

```bash
kb build thermal-metamaterials
```

### v0.7.16

One-command build pipeline release. `kb build <topic>` now bundles the deterministic post-setup preparation sequence into a single command that prepares text evidence, reference candidates, topic graph/review/tasks, and all generic/agent-specific handoff files.

```bash
kb build thermal-metamaterials
kb build thermal-metamaterials --force
kb build thermal-metamaterials --dry-run
```

Equivalent expanded pipeline:

```bash
kb extract-text
kb extract-sections
kb refs
kb refs-index
kb refs-graph
kb topic build thermal-metamaterials
kb topic review thermal-metamaterials
kb topic tasks thermal-metamaterials
kb handoff --all-agents
kb topic handoff thermal-metamaterials --all-agents
```

The single command remains deterministic: it does not call an LLM and does not confirm scholarly relations. The individual commands remain available for debugging or partial rebuilds.

### v0.7.15

Unified Entry Points release. `kb handoff` now creates `obsidian_main.md` as the human-first Obsidian homepage and `LLM/handoff/current.md` as the shared Manager routing file. Topic handoff now also creates `topics/<topic>/index.md` so managers can step from root entry → current route → topic homepage → task queue → one bounded task item.

New / updated entry files:

```text
obsidian_main.md
LLM/handoff/current.md
topics/<topic>/index.md
```

Manager agents should not infer tasks from the file tree. They should follow the routing chain described in `LLM/handoff/current.md`.

### v0.7.14

Codex adapter release. `kb handoff --agent codex` and `kb topic handoff <topic> --agent codex` now generate Codex-specific handoff notes while preserving the generic agent protocol.

### v0.7.13

Generic Agent Handoff release. `kb handoff` now produces a universal external-agent entry layer first, while Claude Code, OpenCode, OpenClaw, and Codex are optional adapters.

New / updated commands:

```bash
kb handoff
kb handoff --topic <topic>
kb handoff --all-topics
kb handoff --agent claude-code
kb handoff --agent opencode
kb handoff --agent openclaw
kb handoff --agent codex
kb handoff --all-agents
kb topic handoff <topic> --agent claude-code
kb topic handoff <topic> --agent opencode
kb topic handoff <topic> --agent openclaw
kb topic handoff <topic> --agent codex
```

Default generic handoff files include:

```text
AGENTS.md
obsidian_main.md
LLM/handoff/current.md
LLM/handoff/index.md
LLM/handoff/protocol.md
LLM/handoff/manager.md
LLM/handoff/worker.md
LLM/handoff/task_schema.md
LLM/handoff/safety.md
LLM/handoff/handoff.json
topics/<topic>/index.md
topics/<topic>/handoff/AGENTS.md
topics/<topic>/handoff/protocol.md
topics/<topic>/handoff/handoff.json
```

Agent-specific adapters are generated only when requested:

```text
CLAUDE.md
LLM/handoff/adapters/claude-code.md
LLM/handoff/adapters/opencode.md
LLM/handoff/adapters/openclaw.md
LLM/handoff/adapters/codex.md
topics/<topic>/handoff/adapters/<agent>.md
```

Build-time topic defaults remain explicit:

- `kb --source <literature-dir> setup` defaults the topic to the source directory name when `--topic` is omitted.
- `kb --lib <database-name> init` defaults the topic to the database directory name when `--topic` is omitted.
- `kb init` without `--lib` exits with a prompt instead of silently creating `knowledgebase/`.

External agents should read `AGENTS.md`, process one bounded task at a time, and leave changes for human diff review.

### v0.7.10

Topic Relation View Fix. `kb view --relations` and `kb view --relations --topic <topic>` now have distinct meanings:

```bash
kb view --relations
```

opens the topic relation overview and lists all topic workspaces under `topics/`.

```bash
kb view --relations --topic thermal-metamaterials
```

opens the concrete directed relation graph for only that topic. The generated `relationship_data.json` is filtered at the data layer, not merely hidden in the browser.

### v0.7.9

Topic build precursor release. Historically, this release introduced the user-level wrapper for the topic workflow: it runs topic-local importance ranking and topic relation graph compilation in one step. It pairs:

```bash
kb topic rank <topic>
kb topic relations <topic>
```

into:

```bash
kb topic build <topic>
kb view --relations --topic <topic>
```

In `v0.7.17`, the preferred command name for this second workflow became `kb topic build <topic>`. In `v0.7.18`, the old alias was removed from the active CLI.

`kb topic relations <topic>` compiles directed relation rows from `topics/<topic>/relations/*.md` and writes graph artifacts under `topics/<topic>/graph/`. It does not call an LLM or decide final scholarly claims.

### v0.7.8

Root marker release. `kb init` and `kb setup` now create `llm-wiki.toml` in the knowledge-base root. This file identifies the folder as an LLM Wiki workspace, records the managed layout, and documents the source-material boundary for future human/LLM agents.

### v0.7.7

Workspace Layout for Existing Literature Folders.

This version makes `kb setup` friendlier for real literature folders. The recommended workflow is now:

```bash
kb --source /path/to/literature-folder setup --recursive
```

It creates a clean child workspace:

```text
/path/to/literature-folder/knowledgebase/
├── llm-wiki.toml
├── raw/
├── wiki/
├── rules/
├── processing/
├── interfaces/
├── LLM/
└── topics/
```

The original literature folder remains the human source-material pile. The generated `knowledgebase/` directory is the LLM Wiki workspace. `setup` then runs:

```text
init -> ingest -> extract-metadata -> build-wiki
```

Safe copy mode remains the default unless `--move` is explicitly provided. `kb bootstrap` remains available as a compatibility alias for `kb setup`. Advanced users can still use `--lib` to initialize or operate on a specific knowledge base directory.

## LLM Wiki Root Marker

Every initialized workspace contains:

```text
llm-wiki.toml
```

This is the root marker for the local LLM Wiki. It records the workspace layout, the source-material boundary, and the default LLM edit policy. For the recommended layout, `kb --source /path/to/literature-folder setup --recursive` writes `source_dir = ".."`, meaning the original literature folder is outside the managed `knowledgebase/` workspace.


### v0.7.5

View Relations Mode.

This version extends the existing `kb view` command with a relationship graph review mode:

```bash
kb view --relations
kb view --relations --no-open
kb view --relations --topic thermal-metamaterials
kb view --relations --data-only
```

The regular dashboard remains:

```bash
kb view
```

`kb view --relations` generates:

```text
interfaces/html/relationship_viewer.html
interfaces/html/relationship_data.json
```

As of v0.7.10, `kb view --relations` is the topic overview, while `kb view --relations --topic <topic>` is the strict single-topic relation graph. Confirmed or accepted edges are rendered as solid lines, candidate or LLM-review-needed edges as dashed lines, and missing or unresolved references as dotted lines.

No new `kb view-relations` command is added. The relationship graph remains part of the unified `kb view` HTML viewing system.

### v0.7.4

Paper Section Extraction for Literature Relations.

This version adds:

```bash
kb extract-sections
```

The command slices `Introduction` and `References` sections from extracted paper text under `processing/text/` and writes them under:

```text
processing/sections/<source-key>/introduction.txt
processing/sections/<source-key>/references.txt
processing/sections/<source-key>/section_manifest.md
```

These two sections are treated as high-value evidence for building literature relationships: `Introduction` usually exposes motivation, gaps, and foundational citations; `References` exposes the explicit citation boundary. If deterministic slicing fails, the command writes an explicit OCR/LLM fallback task for image-based papers or ambiguous section boundaries.

No hidden OCR, hidden LLM call, citation-identity judgment, or scientific relationship inference is added.

### v0.7.3

LLM Task Index Workflow.

This version makes `LLM/tasks/index.md` a first-class Manager LLM dashboard. Running:

```bash
kb tasks
```

now writes:

```text
LLM/tasks/index.md
LLM/tasks/items/<task_id>.md
LLM/tasks/llm_tasks_<timestamp>.md
```

The index is the task list for the Manager LLM. The item files are bounded Worker LLM / human task files. The timestamped report remains an audit snapshot.

### v0.7.2

Transformation Audit and Review Queue Usability.

This version adds:

```bash
kb audit-wiki
```

The command checks whether a folder has moved from a loose material pile into a reviewable, human/AI-maintainable Markdown LLM Wiki. It performs deterministic layer checks, audits topic review queue usability, writes an audit report under `interfaces/reports/`, and generates an explicit LLM review prompt under `LLM/tasks/`.

It also improves the `kb topic review <topic>` queue format so reviewers can record `final_decision`, `confidence`, `uncertainty`, `review_notes`, and `accepted_record_path`.

### v0.7.1

Topic Review Command Skeleton.

This version adds the first deterministic command skeleton for the topic review workflow:

```bash
kb topic review <topic>
```

The command reads topic-local importance candidates under `topics/<topic>/importance/` and generates a review workspace under `topics/<topic>/review/`, including:

```text
topics/<topic>/review/review_queue.md
topics/<topic>/review/review_summary.md
```

The command must not accept, reject, or reinterpret candidates automatically. It only organizes reviewable files so a human reviewer, Manager LLM, or Worker LLM can inspect one bounded item at a time.

### v0.7.0

Topic Review Workflow Foundation.

This version defines the review workflow that connects `kb topic rank <topic>` outputs to human/AI review, accepted topic-local relationship records, and completed memory records.

## Project Idea

`kb-cli` starts from Andrej Karpathy's proposal for operating a local knowledge base in Markdown:

- source materials are kept under `raw/`
- `kb build <topic>` generates reviewable topic tasks, handoff files, and agent/Obsidian entry points
- Markdown knowledge pages are maintained under `wiki/`
- future human/AI maintainers are constrained by the `rules/` layer

The core idea is simple:

> keep source materials local, keep generated knowledge reviewable, and make future AI-assisted maintenance traceable.

Later, `kb-cli` also follows the idea that knowledge nodes can be ranked around a specific topic. This allows the system to generate topic-local importance candidates for literature, concepts, and notes within a research context.

`kb-cli` does not assume that the machine already understands the knowledge base. Instead, it creates structured workspaces where humans and AI agents can inspect, revise, rank, and maintain knowledge step by step.

## Design Principles

1. **Local first**: the knowledge base remains a readable local folder.
2. **Markdown first**: knowledge pages are easy to edit, diff, and version-control.
3. **Review before belief**: generated outputs are candidates for review, not final truth.
4. **Human/AI collaborative maintenance**: intermediate files should be visible and auditable.
5. **No unnecessary semantic overclaiming**: commands should not claim deep understanding unless that behavior is explicitly implemented and reviewable.

## Directory Layout

A typical `kb-cli` knowledge base is organized around reviewable layers:

```text
knowledgebase/
├── raw/              # Original source materials
├── wiki/             # Markdown knowledge pages
├── rules/            # Human / AI maintenance rules
├── topics/           # Topic-specific relationship workspaces
├── LLM/              # Explicit future LLM/agent workbench
│   ├── tasks/        # Manager index, Worker task items, and handoff reports
│   └── memory/       # Completed-task audit memory
├── processing/       # Deterministic intermediate outputs, extracted text, and section slices
├── interfaces/          # Generated reports and static viewer output
├── references/       # Templates and reference materials
```

Topic-specific review work lives under:

```text
topics/<topic>/
├── importance/       # generated topic-local importance candidates
├── review/           # review queue and working review files
├── reviewed/         # accepted / rejected / deferred decisions
└── graph/            # optional topic-specific graph overlays
```

## Installation

```bash
cargo build --release
cargo install --path . --force
```

The binary name is:

```bash
kb
```

## Quick Start

Turn an existing literature folder into a local LLM Wiki without polluting the original folder:

```bash
kb create --lib /path/to/your/llm-wiki --from /path/to/your/literature-folder --about <topic>
```

Windows example:

```powershell
kb create --lib "D:\github\LLM-wiki\quantum-kb" --from "D:\github\LLM-wiki\quantum" --about quantum
```

macOS/Linux example:

```bash
kb create --lib "$HOME/github/LLM-wiki/quantum-kb" --from "$HOME/github/LLM-wiki/quantum" --about quantum
```

Expanded two-step form:

```bash
kb --lib <A> --source <B> setup --recursive --topic <topic>
kb --lib <A> build <topic>
```

This creates:

```text
/path/to/your/llm-wiki/
```

and runs the full pre-LLM preparation path:

```text
setup --recursive -> build <topic>
```

Use preview mode first when working with a large or messy literature directory:

```bash
kb create --lib /path/to/your/llm-wiki --from /path/to/your/literature-folder --about <topic> --dry-run
```

By default, `create` copies files into `<A>/raw/`. Use `--move` only when you explicitly want to move source files.

Advanced/manual layout remains available:

```bash
kb --lib /path/to/specific/knowledgebase init
kb --source /path/to/literature-folder --lib /path/to/specific/knowledgebase ingest --recursive
```

## Core Workflow

The expected workflow is:

1. Keep source materials in the original literature folder
2. Use `kb create --lib <A> --from <B> --about <topic>` to create the target LLM Wiki database, copy materials into `<A>/raw/`, and run the deterministic build pipeline
3. Use `kb --lib <A> --source <B> setup --recursive --topic <topic>` and `kb --lib <A> build <topic>` separately only when debugging or rerunning partial stages
4. Use `LLM/tasks/index.md` as the Manager LLM task dashboard
5. Build or update Markdown knowledge pages under `wiki/`
6. Use topic commands to inspect topic-specific relationship workspaces
7. Use `kb topic build <topic>` to run topic-local importance ranking and relation graph compilation
8. Use `kb view --relations` to inspect all topic workspaces, then `kb view --relations --topic <topic>` to inspect one topic graph
9. Use `kb topic review <topic>` to build a review queue from candidates when needed
10. Use `kb topic tasks <topic>` to generate topic-local Worker handoff files
11. Let human reviewers or explicit AI agents revise the outputs under Git review
12. Record accepted decisions under `topics/<topic>/reviewed/` and `LLM/memory/`

This workflow is not meant to replace human judgment. It is meant to make AI-assisted knowledge maintenance more structured, inspectable, and reproducible.

## Common Commands

| Command | Purpose |
|---|---|
| `kb init` | Create the local LLM Wiki structure and generated rules layer. |
| `kb ingest --copy` | Organize source files into `raw/` subfolders. |
| `kb --source <folder> setup --recursive` | Create `<folder>/knowledgebase`, then run `init + ingest + extract-metadata + build-wiki`. |
| `kb build <topic>` | Run the one-command deterministic pipeline: extract text/sections, build refs, assemble topic review/tasks, and generate all handoff files. |
| `kb status` | Refresh and inspect `processing/manifest.json`. |
| `kb sync-wiki` | Link accepted wiki pages back to the manifest. |
| `kb lint-static` | Check broken WikiLinks, orphan pages, empty pages, and source-front-matter issues. |
| `kb query <terms...>` | Search Markdown pages with local keyword matching. |
| `kb grep <pattern>` | Search text-like files with line-level matching. |
| `kb extract-text` | Extract plain text into `processing/text/`. |
| `kb extract-sections` | Slice Introduction and References into `processing/sections/`. |
| `kb refs` | Scan extracted text for reference headings, entries, DOI values, and citation hints. |
| `kb refs-index` | Build bibliographic index relation candidates. |
| `kb refs-graph` | Export bibliographic index candidates as graph data. |
| `kb keywords` | Detect keyword/topic co-occurrence candidates. |
| `kb health` | Summarize deterministic LLM Wiki health. |
| `kb audit-wiki` | Audit whether the folder has become a reviewable human/AI-maintainable Markdown LLM Wiki. |
| `kb tasks` | Generate deferred task handoff lists, Worker task items, and `LLM/tasks/index.md`. |
| `kb memory` | Append completed-task records under `LLM/memory/`. |
| `kb topic init <topic>` | Create a topic-specific relationship workspace. |
| `kb topic list` | List topic workspaces. |
| `kb topic status <topic>` | Inspect a topic workspace. |
| `kb topic rank <topic>` | Generate deterministic topic-local importance candidates. |
| `kb topic relations <topic>` | Compile topic-local directed relation records into graph artifacts. |
| `kb topic build <topic>` | Run topic ranking and relation graph compilation together. |
| `kb topic review <topic>` | Build a deterministic review queue from topic-local candidates. |
| `kb topic tasks <topic>` | Generate topic-local Manager/Worker LLM handoff tasks. |
| `kb view` | Generate and open the static HTML review dashboard. |
| `kb view --relations` | Generate the topic relation overview page and `relationship_data.json`. |
| `kb view --relations --topic <topic>` | Generate the strict single-topic directed relation graph. |
| `kb view --relations --data-only` | Generate only `interfaces/html/relationship_data.json`. |
| `kb shell` | Start a deterministic interactive command shell. |

For detailed command behavior, see `docs/commands.md`, `docs/extract-sections.md`, `docs/llm-task-index.md`, `docs/audit-wiki.md`, and `docs/topic-review-command.md`.

## LLM Task Index Workflow

Starting in `v0.7.3`, `kb tasks` creates a Manager LLM dashboard and bounded Worker task files:

```bash
kb tasks
kb tasks --index-only
kb tasks --no-index
```

Default output:

```text
LLM/tasks/index.md                 # Manager LLM task dashboard
LLM/tasks/items/<task_id>.md        # bounded Worker LLM / human task files
LLM/tasks/llm_tasks_<timestamp>.md  # timestamped audit snapshot
```

The intended handoff is:

```text
Manager LLM opens LLM/tasks/index.md
        ↓
selects one pending task or topic review queue
        ↓
assigns one bounded item to a Worker LLM / human reviewer
        ↓
Worker output is reviewed with Git diff
        ↓
accepted work is recorded with kb memory
```

See `docs/tasks.md`, `docs/task-lifecycle.md`, and `docs/llm-task-index.md`.

## Transformation Audit

Starting in `v0.7.2`, the project can audit its own central claim:

```bash
kb audit-wiki
```

The command checks the presence and traceability of source materials, processing artifacts, Markdown wiki pages, reference/index outputs, topic workspaces, review queues, explicit LLM handoff tasks, completed memory records, and generated review outputs.

It writes:

```text
interfaces/reports/wiki_audit_<timestamp>.md
LLM/tasks/wiki_audit_llm_review_<timestamp>.md
```

The first file is deterministic evidence. The second file is a bounded prompt for a Manager LLM to independently review whether the proof chain is convincing.

## Topic-local Importance Ranking

The topic-ranking workflow helps researchers inspect which materials may be important within a specific topic.

For example:

```bash
kb topic rank quantum
```

may generate reviewable importance candidates under:

```text
topics/quantum/importance/
```

These files should be reviewed by humans or downstream AI agents before being treated as reliable knowledge. The ranking output is a structured review aid, not an automatic final judgment.

## Topic Review Workflow

Starting in `v0.7.1`, the first deterministic command skeleton is defined for building a topic review queue:

```bash
kb topic review <topic>
```

The intended path is:

```text
kb topic rank <topic>
        ↓
topics/<topic>/importance/ candidates
        ↓
kb topic review <topic>
        ↓
topics/<topic>/review/review_queue.md
topics/<topic>/review/review_summary.md
        ↓
human / Manager LLM / Worker LLM review
        ↓
accepted topic-local decisions under topics/<topic>/reviewed/
        ↓
completion record under LLM/memory/
```

See:

```text
docs/topic-review-workflow.md
docs/topic-review-schema.md
docs/topic-review-command.md
```

## Relationship Layers

LLM Wiki is not merely a local document archive. Its central purpose is to maintain an evolving, reviewable, and evidence-backed relationship network among references.

The relationship network has three levels:

1. **Bibliographic index relations**: citation, DOI, title, author, journal, page, and date relationships.
2. **Keyword / topic relations**: shared terms, methods, models, devices, and research topics.
3. **Scientific idea relations**: method transfer, mechanism similarity, theoretical inheritance, research gaps, and cross-domain inspiration.

Rust-native commands should extract and report candidates. Human review remains the final guarantee when identity, meaning, or scientific interpretation is uncertain.

## LLM Maintenance Boundary

In this project, an "LLM-maintained wiki" does **not** mean that an LLM freely edits original files or silently decides the whole workflow.

The intended hierarchy is:

```text
Manager LLM / human reviewer
        ↓ uses deterministic kb-cli commands
kb query / grep / extract-text / refs / refs-index / refs-graph / keywords / links / lint-static / health / tasks
        ↓ produce evidence, candidate relations, unresolved issues, and bounded task lists
Worker LLM / human reviewer
        ↓ completes one assigned task with explicit requirements
Git diff / human review
        ↓ accepted edits and decisions are recorded
wiki/ + topics/ + LLM/memory/
```

See `docs/llm-maintenance.md`.

## Documentation Map

```text
CHANGELOG.md                         Release history
docs/commands.md                     Command overview
docs/llm-maintenance.md              Manager LLM / Worker LLM maintenance model
docs/topic-review-workflow.md        Topic review workflow
docs/topic-review-schema.md          Topic review record schema
docs/topic-review-command.md         v0.7.1 command contract for kb topic review <topic>
docs/implementation-notes/topic-review-command-rust.md
                                      Suggested Rust implementation notes for v0.7.1
```

## Non-goals

`kb-cli` is not intended to be:

- a black-box RAG database
- a fully automatic paper-reading system
- a replacement for human literature review
- a semantic search engine by default
- a system that claims knowledge correctness without review

## License

MIT License - see [LICENSE](LICENSE).
