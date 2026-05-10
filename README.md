# kb-cli

`kb-cli` is a small Rust command-line tool for building, analyzing, and maintaining a local Markdown-based LLM Wiki.

It is designed for researchers, developers, and human/AI collaborative workflows that need a local, reviewable, file-based knowledge system instead of a black-box note database.

## Current Version

Current version: `v0.7.0`

### v0.7.0

Topic Review Workflow Foundation.

This version defines the review workflow that connects `kb topic rank <topic>` outputs to human/AI review, accepted topic-local relationship records, and completed memory records. It introduces explicit documentation for topic review queues, evidence-backed decisions, accepted/rejected/deferred statuses, and review-safe handoff between Manager LLMs, Worker LLMs, and human reviewers.

This release is still conservative: it defines the workflow and command contract before adding automatic topic-level scientific interpretation.

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
│   ├── tasks/        # Deferred task handoff reports
│   └── memory/       # Completed-task audit memory
├── processing/       # Deterministic intermediate outputs
├── outputs/          # Generated reports and static viewer output
├── references/       # Templates and reference materials
└── logs/             # Metadata and logs
```

The structure is intentionally simple. Most files are plain Markdown, JSON, or reviewable text outputs, so the knowledge base can be inspected, edited, version-controlled, and maintained over time.

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

By default, `bootstrap` uses safe copy mode unless `--move` is explicitly provided.

## Core Workflow

The expected workflow is:

1. Put source materials into `raw/`
2. Use deterministic commands to inspect, extract, index, and lint files
3. Use `kb prepare` or `kb tasks` to generate reviewable task materials
4. Build or update Markdown knowledge pages under `wiki/`
5. Use topic commands to inspect topic-specific relationship workspaces
6. Use ranking commands to generate topic-local importance candidates
7. Let human reviewers or explicit AI agents revise the outputs under Git review
8. Record accepted decisions under `LLM/memory/`

This workflow is not meant to replace human judgment. It is meant to make AI-assisted knowledge maintenance more structured, inspectable, and reproducible.

## Common Commands

| Command | Purpose |
|---|---|
| `kb init` | Create the local LLM Wiki structure and generated rules layer. |
| `kb ingest --copy` | Organize source files into `raw/` subfolders. |
| `kb bootstrap --copy` | Run `init + ingest + extract-metadata + build-wiki`. |
| `kb status` | Refresh and inspect `processing/manifest.json`. |
| `kb prepare --new` | Generate reviewable task materials without calling an LLM. |
| `kb sync-wiki` | Link accepted wiki pages back to the manifest. |
| `kb lint-static` | Check broken WikiLinks, orphan pages, empty pages, and source-front-matter issues. |
| `kb query <terms...>` | Search Markdown pages with local keyword matching. |
| `kb grep <pattern>` | Search text-like files with line-level matching. |
| `kb extract-text` | Extract plain text into `processing/text/`. |
| `kb refs` | Scan extracted text for reference headings, entries, DOI values, and citation hints. |
| `kb refs-index` | Build bibliographic index relation candidates. |
| `kb refs-graph` | Export bibliographic index candidates as graph data. |
| `kb keywords` | Detect keyword/topic co-occurrence candidates. |
| `kb health` | Summarize deterministic LLM Wiki health. |
| `kb tasks` | Generate deferred human/LLM/agent task handoff lists. |
| `kb memory` | Append completed-task records under `LLM/memory/`. |
| `kb topic init <topic>` | Create a topic-specific relationship workspace. |
| `kb topic list` | List topic workspaces. |
| `kb topic status <topic>` | Inspect a topic workspace. |
| `kb topic rank <topic>` | Generate deterministic topic-local importance candidates. |
| `kb view` | Generate and open the static HTML review dashboard. |
| `kb shell` | Start a deterministic interactive command shell. |

For detailed command behavior, see `docs/commands.md`.

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

Starting in `v0.7.0`, topic-local importance and relationship work is treated as a review workflow rather than a one-step ranking result.

The intended path is:

```text
kb topic rank <topic>
        ↓
topics/<topic>/importance/ candidates
        ↓
review queue with evidence, decision status, and uncertainty notes
        ↓
human / Manager LLM / Worker LLM review
        ↓
accepted topic-local decisions under topics/<topic>/reviewed/
        ↓
completion record under LLM/memory/
```

See `docs/topic-review-workflow.md` and `docs/topic-review-schema.md`.

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

See `docs/llm-maintenance.md` for the detailed workflow.

## Static HTML Viewer

`kb view` generates a local, static HTML review dashboard:

```bash
kb view
kb view --no-open
```

Default output:

```text
outputs/html/index.html
```

The viewer is intentionally safe and simple. It displays local review information and does not execute local `kb` commands, call LLMs, or modify files.

## Documentation Map

Start here:

```text
CHANGELOG.md                 Release history
docs/commands.md             Command overview and command category map
docs/llm-maintenance.md      Manager LLM / Worker LLM maintenance workflow
docs/task-lifecycle.md       Task handoff status vocabulary and memory workflow
docs/llm-command-guide.md    Practical guide for LLMs using structured kb commands
docs/llm-hierarchy.md        Manager LLM / Worker LLM role hierarchy
```

Command docs:

```text
docs/query.md                Local keyword query skeleton
docs/links.md                WikiLink scan and deterministic link-resolution hints
docs/grep.md                 Rust-native line-level text search
docs/extract-text.md         Deterministic text extraction and future PDF conversion-agent boundary
docs/refs.md                 Deterministic reference-hint scanning over extracted text
docs/refs-index.md           Bibliographic index relation candidate building
docs/refs-graph.md           Graph export for bibliographic index relations
docs/keywords.md             Keyword/topic co-occurrence candidate scanner
docs/health.md               Deterministic LLM Wiki relationship-health dashboard
docs/tasks.md                Deferred human/LLM/agent task handoff lists
docs/memory.md               Completed-task memory records under LLM/memory/
docs/view.md                 Static HTML viewer generated by kb view
```

Topic and relationship docs:

```text
docs/literature-relationships.md      Reference-relationship core principle
docs/topic.md                         Topic workspace command reference
docs/topic-relationships.md           Topic-specific relationship overlay roadmap
docs/topic-relation-schema.md         Controlled fields for topic-local directed relation records
docs/literature-importance-schema.md  Lightweight global/topic-local literature importance schema
docs/topic-v2-roadmap.md              Conservative topic-overlay roadmap
docs/topic-review-workflow.md         Review workflow for topic-local ranking and relation decisions
docs/topic-review-schema.md           Review record fields and status vocabulary
docs/third-party-skills/              Third-party graph visualization skill guidance
```

Platform and development docs:

```text
docs/windows-quickstart.md       PowerShell and .bat helper
docs/unix-quickstart.md          macOS/Linux bash or zsh
docs/cross-platform-quickstart.md Shared command flow
docs/platform-notes.md           Path and wrapper-script policy
docs/git-workflow.md             Review-first Git commit/push workflow
docs/release-checklist.md        Small-release checklist
docs/command-classification.md   Command categories and LLM/OCR boundary rules
docs/cli-shell-principle.md      Batch mode / shell mode / LLM boundary
docs/shell.md                    Deterministic `kb shell` usage
```

## Current Scope

This version provides a conservative, file-based local LLM Wiki scaffold. It includes cross-platform bootstrap/ingest workflows, the generated Wiki rule/schema layer, manifest tracking, static linting, deterministic keyword search, WikiLink scanning, Rust-native grep, best-effort text extraction, reference-hint scanning, bibliographic index candidate building, graph export, keyword candidate reports, task handoff reports, completed-task memory records, topic workspaces, topic-local importance candidates, topic review workflow documentation, and a static HTML viewer.

It still does **not** provide full RAG, vector search, embeddings, OCR, hidden LLM cleanup, autonomous LLM/wiki preparation, or automatic scientific interpretation.

`kb prepare` currently plans and documents prepare work; it does not call an LLM or edit `wiki/` by itself.

## Non-goals

`kb-cli` is not intended to be:

- a black-box RAG database
- a fully automatic paper-reading system
- a replacement for human literature review
- a semantic search engine by default
- a system that claims knowledge correctness without review
- a hidden LLM chat surface inside `kb shell`

Its purpose is to provide a clear local structure and CLI workflow for maintaining research-oriented Markdown knowledge bases.

## Development Notes

This project is developed conservatively.

When adding new commands, prefer:

- explicit file outputs
- reviewable intermediate results
- deterministic behavior where possible
- simple folder conventions
- clear distinction between generated candidates and reviewed knowledge

Avoid:

- hidden state
- opaque automatic rewriting
- unreviewable semantic claims
- unnecessary changes to existing directory conventions
- implicit LLM behavior triggered from free-form shell text

## Testing and Release Check

Before tagging or handing a build to another agent, run:

```bash
cargo fmt --check
cargo test
cargo check
cargo build --release
```

For documentation-only releases, at minimum review the Markdown diff:

```bash
git status
git diff README.md CHANGELOG.md docs/llm-maintenance.md docs/topic-review-workflow.md docs/topic-review-schema.md
```

## License

MIT License - see [LICENSE](LICENSE).
