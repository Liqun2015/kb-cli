# kb-cli

`kb-cli` is a small Rust command-line tool for building, analyzing, and maintaining a local Markdown-based LLM Wiki.

It is designed for researchers, developers, and human/AI collaborative workflows that need a local, reviewable, file-based knowledge system instead of a black-box note database.

## Current Version

Current version: `v0.7.6`

### v0.7.6

Command Naming Cleanup.

This version makes `kb setup` the preferred one-command entry point for creating a usable local LLM Wiki from source materials:

```bash
kb setup
kb setup --dry-run
kb setup --move
```

`kb setup` runs:

```text
init -> ingest -> extract-metadata -> build-wiki
```

Safe copy mode remains the default unless `--move` is explicitly provided. The older `kb bootstrap` command remains available as a compatibility alias, but new documentation should prefer `kb setup`.

This version also updates stale version markers and shell display text so `kb shell` reports the package version from Cargo.

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
outputs/html/relationship_viewer.html
outputs/html/relationship_data.json
```

The page visualizes paper-level bibliographic index relations and topic-local academic viewpoint / importance candidates before LLM execution. Confirmed or accepted edges are rendered as solid lines, candidate or LLM-review-needed edges as dashed lines, and missing or unresolved references as dotted lines.

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

The command checks whether a folder has moved from a loose material pile into a reviewable, human/AI-maintainable Markdown LLM Wiki. It performs deterministic layer checks, audits topic review queue usability, writes an audit report under `outputs/reports/`, and generates an explicit LLM review prompt under `LLM/tasks/`.

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
- `kb prepare` generates reviewable task materials
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
KnowledgeBase/
├── raw/              # Original source materials
├── wiki/             # Markdown knowledge pages
├── rules/            # Human / AI maintenance rules
├── topics/           # Topic-specific relationship workspaces
├── LLM/              # Explicit future LLM/agent workbench
│   ├── tasks/        # Manager index, Worker task items, and handoff reports
│   └── memory/       # Completed-task audit memory
├── processing/       # Deterministic intermediate outputs, extracted text, and section slices
├── outputs/          # Generated reports and static viewer output
├── references/       # Templates and reference materials
└── logs/             # Metadata and logs
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

Turn an existing literature folder into a local LLM Wiki:

```bash
kb --kb-path /path/to/your/literature-folder setup
```

Windows example:

```powershell
kb --kb-path "D:\github\LLM-wiki\quantum" setup
```

macOS/Linux example:

```bash
kb --kb-path "$HOME/github/LLM-wiki/quantum" setup
```

The setup command runs:

```text
init -> ingest -> extract-metadata -> build-wiki
```

By default, `setup` uses safe copy mode unless `--move` is explicitly provided. `bootstrap` is kept as a compatibility alias.

## Core Workflow

The expected workflow is:

1. Put source materials into `raw/`
2. Use deterministic commands to inspect, extract, section, index, and lint files
3. Use `kb prepare` or `kb tasks` to generate reviewable task materials
4. Use `LLM/tasks/index.md` as the Manager LLM task dashboard
5. Build or update Markdown knowledge pages under `wiki/`
6. Use topic commands to inspect topic-specific relationship workspaces
7. Use ranking commands to generate topic-local importance candidates
8. Use `kb topic review <topic>` to build a review queue from candidates
9. Let human reviewers or explicit AI agents revise the outputs under Git review
10. Record accepted decisions under `topics/<topic>/reviewed/` and `LLM/memory/`

This workflow is not meant to replace human judgment. It is meant to make AI-assisted knowledge maintenance more structured, inspectable, and reproducible.

## Common Commands

| Command | Purpose |
|---|---|
| `kb init` | Create the local LLM Wiki structure and generated rules layer. |
| `kb ingest --copy` | Organize source files into `raw/` subfolders. |
| `kb setup` | Run `init + ingest + extract-metadata + build-wiki`. |
| `kb status` | Refresh and inspect `processing/manifest.json`. |
| `kb prepare --new` | Generate reviewable task materials without calling an LLM. |
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
| `kb topic review <topic>` | Build a deterministic review queue from topic-local candidates. |
| `kb view` | Generate and open the static HTML review dashboard. |
| `kb view --relations` | Generate the relationship graph review page and `relationship_data.json`. |
| `kb view --relations --topic <topic>` | Generate the relationship graph review page focused on a topic. |
| `kb view --relations --data-only` | Generate only `outputs/html/relationship_data.json`. |
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
outputs/reports/wiki_audit_<timestamp>.md
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
