# Changelog

All notable changes to `kb-cli` are recorded here.

This project uses small, review-first releases. Many releases are documentation, workflow, or boundary-stabilization releases rather than feature-heavy releases.

## v0.7.3

LLM Task Index Workflow.

Main changes:

- `kb tasks` now writes `LLM/tasks/index.md` as the Manager LLM task dashboard.
- `kb tasks` now writes bounded Worker task files under `LLM/tasks/items/<task_id>.md`.
- The existing timestamped `LLM/tasks/llm_tasks_<timestamp>.md` handoff report remains as an audit snapshot.
- Added `--index-only` for refreshing the Manager/Worker task index without writing a new timestamped snapshot.
- Added `--no-index` for preserving the old timestamped-report-only behavior.
- Added documentation for Manager LLM task intake and Worker LLM task assignment.

No hidden LLM call is added. The new task index is a routing document, not an automatic execution mechanism.

## v0.7.2

Transformation Audit and Review Queue Usability.

Main changes:

- Added `kb audit-wiki`, a deterministic proof-audit command for checking whether a folder has become a reviewable Markdown LLM Wiki.
- The audit checks source materials, rules, manifest, processing outputs, wiki pages, source traceability, reference/index outputs, topic workspaces, explicit LLM handoff records, memory records, and review outputs.
- The audit writes `outputs/reports/wiki_audit_<timestamp>.md`.
- Unless disabled with `--no-llm-prompt`, the audit also writes `LLM/tasks/wiki_audit_llm_review_<timestamp>.md` so a Manager LLM can independently evaluate the proof chain.
- Added review queue usability auditing for `topics/*/review/review_queue.md`.
- Improved `kb topic review <topic>` output so `review_queue.md` includes editable decision fields: `final_decision`, `confidence`, `uncertainty`, `review_notes`, and `accepted_record_path`.
- Added `docs/audit-wiki.md`.
- No hidden LLM call, automatic scientific judgment, wiki rewriting, or automatic topic-candidate acceptance is added.

## v0.7.1

Topic Review Command Skeleton.

Main changes:

- Implemented the deterministic `kb topic review <topic>` command skeleton.
- Added `docs/topic-review-command.md` to describe inputs, outputs, options, safety boundaries, and acceptance criteria for the command.
- Updated the topic review workflow so `kb topic rank <topic>` can be followed by `kb topic review <topic>`.
- Defined default output files under `topics/<topic>/review/`:
  - `review_queue.md`
  - `review_summary.md`
- Added implementation notes for a conservative Rust command skeleton under `docs/implementation-notes/topic-review-command-rust.md`.
- Added small topic-review examples under `examples/topic-review/`.
- `kb topic review <topic>` does not accept, reject, or reinterpret candidates automatically. It only builds a queue and summary from existing candidate files.
- No hidden LLM call, vector search, semantic importance judgment, autonomous wiki editing, or automatic topic graph construction is added in this version.

## v0.7.0

Topic Review Workflow Foundation.

Main changes:

- Defined the review workflow that connects `kb topic rank <topic>` outputs to accepted topic-local decisions.
- Added `docs/topic-review-workflow.md` to describe the path from importance candidates to review queue, accepted records, rejected records, deferred work, and completion memory.
- Added `docs/topic-review-schema.md` to define review-safe fields, status vocabulary, evidence requirements, uncertainty notes, and forbidden overclaims.
- Updated `README.md` to make topic review a first-class workflow after topic-local importance ranking.
- Preserved the distinction between global bibliographic index relations under `processing/refs/` and topic-specific interpretive relations under `topics/<topic>/`.
- No automatic topic-level scientific interpretation, hidden LLM execution, vector search, or autonomous wiki rewriting is added in this version.

## v0.6.9

README cleanup and changelog split.

Main changes:

- Turned `README.md` into a concise project entry page.
- Moved long historical release notes into `CHANGELOG.md`.
- Moved the detailed Manager LLM / Worker LLM maintenance workflow into `docs/llm-maintenance.md`.
- Kept the project positioning from `v0.6.8` while reducing README length and duplication.
- No core CLI behavior is changed in this version.

## v0.6.8

README project-positioning update.

Main changes:

- Clarified the purpose of `kb-cli` as a small Rust CLI for building, analyzing, and maintaining a local Markdown-based LLM Wiki.
- Documented the relationship between the Karpathy-style local Markdown knowledge-base workflow and the later topic-local importance ranking workflow.
- Emphasized local source materials, reviewable generated knowledge, and traceable AI-assisted maintenance.
- No core CLI behavior was changed.

## v0.6.7

`kb topic rank <topic>` now generates deterministic topic-local literature importance candidates under:

```text
topics/<topic>/importance/
```

These candidates are intended for later human review rather than automatic semantic judgment.

## v0.6.6.1

`kb view` now uses the knowledge-base directory name as the HTML viewer title and updates the viewer subtitle to better reflect the research-oriented purpose of the LLM Wiki.

## v0.6.6

`kb topic list` and `kb topic status <topic>` now inspect topic-specific relationship workspaces without making semantic claims.

## v0.6.5

Previous topic-related workflow improvements.

## v0.6.3

`v0.6.3` upgrades `kb refs-graph` output into a conservative V2 graph export.

Main changes:

- The global bibliographic graph now exposes directed edges.
- The graph includes graph kind, relation layer, relation type, relation label, confidence, evidence, human-review flags, visual style hints, and placeholder node-weight fields.
- The graph remains the global bibliographic index layer.
- It does not infer topic-specific causal, method, evidence, or idea relations; those belong under `topics/<topic>/`.

## v0.6.2

`v0.6.2` adds the first topic-level schema foundation.

Main changes:

- Defines how a specific topic directory should record directed relation candidates, topic-local importance, evidence, review status, and third-party topic graph fields.
- Keeps global bibliographic index relations under `processing/refs/`.
- Keeps topic-local causal, method, evidence, idea, and importance records under `topics/<topic>/`.

New schema documents:

```text
docs/topic-v2-roadmap.md
docs/topic-relation-schema.md
docs/literature-importance-schema.md
docs/third-party-skills/topic-graph-schema.md
```

## v0.6.1

`v0.6.1` defines the topic relationship overlay roadmap.

Main changes:

- Added `docs/topic-relationships.md`.
- `kb init` now creates a top-level `topics/` directory.
- Recursive ingest treats `topics/` as a managed directory and skips it.
- README distinguishes global bibliographic relations from topic-specific interpretive relations.
- The V2 target is intentionally conservative: topic-centered literature relationship overlays, not a universal causal knowledge graph.
- No LLM call, causal inference engine, vector search, automatic topic graph builder, or autonomous Wiki editing was added.

## v0.6.0.5

`v0.6.0.5` updates safe Git helper scripts so diff review does not open Git's pager.

Main changes:

- Git helper scripts now use `git --no-pager status`.
- Git helper scripts now use `git --no-pager diff`.
- Git helper scripts now use `git --no-pager diff --cached`.

## v0.6.0.1

`v0.6.0.1` tightens `kb shell` into a strict whitelist command shell for Manager LLM or human maintenance sessions.

Main changes:

- Unknown input is rejected safely instead of being interpreted.
- Added `kb keywords`.
- Added `docs/keywords.md`.
- `kb init` now creates `processing/keywords/`.
- `kb keywords` scans `processing/text/` by default and writes `processing/keywords/keywords_*.md` unless `--dry-run` / `--preview` is used.
- Explicit terms, terms files, auto-extracted candidate terms, JSON output, and candidate relation rows are supported.
- `kb tasks` now detects keyword/topic candidate reports and creates a bounded handoff for a Keyword Topic Relation Worker.
- No LLM call, vector search, semantic idea inference, wiki rewrite, or topic merge was added.

## v0.5.10.4

`v0.5.10.4` updates the README's Markdown-maintenance section so it reflects the current LLM hierarchy:

```text
Manager LLM -> deterministic kb-cli commands -> Worker LLM / human reviewer -> Git review -> wiki + LLM/memory
```

Main changes:

- Clarified that the Manager LLM should first scout with structured commands such as `kb query`, `kb grep`, `kb extract-text`, `kb refs`, `kb refs-index`, `kb refs-graph`, `kb links`, `kb lint-static`, and `kb tasks`.
- Clarified that Worker LLMs should receive bounded tasks with explicit goals, requirements, file lists, evidence lines, expected outputs, and review rules.
- Documentation-only release.
- No LLM calls, OCR, vector search, autonomous Worker execution, or automatic Wiki editing was added.

## v0.5.10.2

`v0.5.10.2` uses `docs/third-party-skills/` as a dedicated Markdown skill-specification directory for third-party graph visualization, integration work, and Claude Code generation.

Main changes:

- Defined the literature relationship visual protocol:

```text
confirmed relation        -> solid edge
candidate / ambiguous     -> dashed edge
missing / unresolved node -> hollow node
needs human review        -> explicit evidence and review marker
```

- Documentation-only at `v0.5.10.2`.
- In `v0.5.11`, `kb refs-graph` begins to implement this protocol with JSON, Mermaid, and Graphviz DOT export.

## v0.5.10

`v0.5.10` adds `kb refs-index`, a deterministic skeleton for building bibliographic index relation candidates between extracted reference entries and local papers.

Main changes:

- Added `kb refs-index`.
- Added `docs/refs-index.md`.
- `kb refs-index` scans `processing/text/` by default and writes Markdown reports under `processing/refs/` unless `--dry-run` / `--preview` is used.
- Relation statuses include `confirmed`, `candidate`, `ambiguous`, and `missing`.
- Candidate, ambiguous, and missing cases return a deferred human-review task.
- No LLM call, OCR, online DOI lookup, autonomous citation graph building, or Wiki rewriting was introduced.

Architecture-rule clarification in the same release family:

- Standardized the LLM hierarchy terminology as **Manager LLM / Worker LLM**.
- Made the core purpose explicit: LLM Wiki maintains relationships among references.

## v0.5.9

`v0.5.9` adds `kb links`, a deterministic WikiLink scanner.

Main changes:

- Added `kb links`.
- Added `docs/links.md`.
- Extracts WikiLinks from Markdown pages.
- Resolves targets when there is exactly one local match.
- Marks unresolved or ambiguous links for future Wiki Link Repair Agent work.
- Supports `--resolved`, `--unresolved`, `--ambiguous`, `--limit`, `--path`, and `--json`.
- Returns deferred task hints instead of guessing semantic repairs.
- No runtime LLM integration, OCR, vector search, autonomous agent execution, or automatic Wiki rewriting was introduced.

## v0.5.8.3

`v0.5.8.3` is a documentation and operating-rule polish release.

Main changes:

- Added `docs/llm-command-guide.md`.
- Documented how LLM operators should use `kb query`, `kb grep`, `kb extract-text`, `kb refs`, `kb tasks`, and `kb memory` before doing semantic work.
- Clarified that `LLM/tasks/` is the handoff area and `LLM/memory/` is the completed-task audit area.
- Reinforced the one-command-one-ability rule.
- No new command behavior, OCR, LLM call, vector search, citation graph construction, autonomous Wiki editing, or hidden agent execution was added.

## v0.5.8.2

`v0.5.8.2` adds `kb memory`, a deterministic audit-memory command for recording completed task outcomes under `LLM/memory/completed_tasks.md`.

Related `v0.5.8.1` change:

- `kb tasks` writes Markdown handoff reports under `LLM/tasks/`.
- `kb init` now creates `LLM/tasks/` and `LLM/memory/`.
- Documentation treats `LLM/` as the explicit future agent workbench.
- Task groups include target agent, goal, requirements, file list, evidence, source command, and priority.
- Task groups cover PDF text conversion gaps, reference reconciliation candidates, Wiki source traceability gaps, and broken WikiLinks.
- No OCR, LLM call, vector search, citation graph construction, autonomous Wiki editing, or hidden agent execution was added.

## v0.5.7

`v0.5.7` adds the first deterministic reference-hint scanner.

Main changes:

- Added `kb refs`.
- Added `docs/refs.md`.
- `kb refs` scans `processing/text/` by default.
- Detects reference headings, bibliography-entry patterns, DOI values, and optional citation markers.
- Supports `--path`, `--limit`, `--citations`, and `--json`.
- `kb init` now creates `processing/refs/` as the future home for reference reports or graph exports.
- No LLM, OCR, semantic citation reconciliation, vector search, or reference graph generation was added.

## v0.5.6.1

`v0.5.6.1` is a project-rule stabilization release.

Main changes:

- Added `docs/command-classification.md`.
- Added `kb grep` as a Rust-native line-level search command.
- Added `kb extract-text` as a deterministic best-effort text extraction command.
- Added `docs/grep.md` and `docs/extract-text.md`.
- Documented `extract-text` as the future entry point for a PDF Text Conversion Agent.
- Kept OCR/LLM/agent modes explicit and not implemented by default.
- Reinforced that deterministic commands should reduce LLM burden rather than hide LLM usage.

## v0.5.4

`v0.5.4` cleans up `scripts/` so only small, deterministic, low-confusion developer helpers remain.

Main changes:

- Removed old broad knowledge-base workflow wrapper scripts from `scripts/`.
- Added `scripts/build_release.bat` for Windows release builds.
- Added `scripts/build_release.sh` for macOS/Linux release builds.
- Added `scripts/README.md` to define what belongs in `scripts/`.
- Kept `scripts/git_safe_push.bat` and `scripts/git_safe_push.sh` as review-first Git helpers.
- Updated README and platform docs to prefer explicit `kb ...` command sequences for knowledge-base workflows.
- No LLM calls, embeddings, vector search, RAG, autonomous editing, or hidden natural-language command interpretation were added.

## v0.5.2

`v0.5.2` establishes the CLI / shell / LLM boundary before expanding interactive features.

Main changes:

- Added `docs/cli-shell-principle.md`.
- Documented that `kb ...` is batch mode and `kb>` is deterministic interactive shell mode.
- Documented that future LLM behavior must use a deliberately designed explicit interface.
- Disabled LLM-like requests inside `kb shell`; they explain the boundary instead of calling a model.
- Kept query deterministic and local.

## v0.5.1

`v0.5.1` is a documentation clarity release.

Main changes:

- Clarified the README description of the Karpathy-style workflow.
- Made the role of `kb prepare` explicit: it generates reviewable task materials, not final wiki pages.
- Clarified that `wiki/` contains maintained Markdown knowledge pages and `rules/` constrains future human/AI maintainers.
- No query behavior or LLM functionality changed.

## v0.5.0

`v0.5.0` adds the first query skeleton.

Main changes:

- Added `kb query` for local keyword search over `wiki/**/*.md`.
- Added AND-style matching across query terms.
- Added lightweight scoring from title, path, body, and snippet matches.
- Added `--limit`, `--snippets`, `--json`, and `--title-only`.
- Added `docs/query.md`.
- No LLM calls or semantic retrieval were added.

Useful query commands:

```bash
kb query thermal cloak
kb query thermal cloak --limit 5
kb query thermal cloak --snippets 3
kb query thermal cloak --title-only
kb query thermal cloak --json
```

## v0.4.8

`v0.4.8` is a developer workflow stabilization release before `v0.5.0`.

Main changes:

- Added `scripts/git_safe_push.bat` for Windows.
- Added `scripts/git_safe_push.sh` for macOS/Linux.
- Added `docs/git-workflow.md`.
- Added `docs/release-checklist.md`.
- Updated README and docs index links to include the Git workflow and release checklist.
- Kept the Rust core workflow unchanged.
- Did not add query, vector search, embeddings, RAG, or autonomous LLM execution.

## v0.4.7

`v0.4.7` is a cross-platform documentation stabilization release before `v0.5.0`.

Main changes:

- Added `docs/unix-quickstart.md` for macOS/Linux users.
- Added `docs/platform-notes.md`.
- Updated `docs/cross-platform-quickstart.md` so Windows and Unix paths are both first-class examples.
- Updated `docs/windows-quickstart.md` to align with the cross-platform documentation set.
- Kept the Rust core workflow unchanged.

## v0.4.6.2

`v0.4.6.2` was a documentation polish release before `v0.5.0`.

Main changes:

- `kb prepare` is now the primary command for generating AI/human handoff artifacts.
- Generated artifacts use prepare-oriented names:

```text
processing/prepare_queue.json
processing/proposals/prepare_plan_<timestamp>.md
processing/proposals/prepare_agent_prompt_<timestamp>.md
```

- Cargo metadata used `0.4.6-2` for SemVer compatibility, while the release package was labeled `v0.4.6.2`.
- Did not add query, vector search, embeddings, RAG, or autonomous LLM execution.

## v0.4.6

`v0.4.6` is a testing and documentation stabilization release before `v0.5.0`.

Main changes:

- Added focused Rust regression tests for `build-wiki` source front matter generation.
- Added focused Rust regression tests for `lint-static` WikiLink parsing, source front matter parsing, issue detection, and report-writing behavior.
- Documented local test commands to run before tagging a release.
- Expanded the README with a clearer testing/release checklist.
- Expanded `docs/windows-quickstart.md`.
- Expanded `docs/lint-static.md`.
- Updated version/schema markers to `0.4.6`.

Recommended release check:

```bash
cargo fmt --check
cargo test
cargo check
cargo build --release
```

## v0.4.5

`v0.4.5` is a small consistency release before `v0.5.0`.

Main changes:

- `kb build-wiki` writes YAML source front matter on generated `wiki/papers/*.md` and `wiki/notes/*.md` pages.
- Generated source-derived pages include `source_files` and include `source_ids` when a matching manifest entry is available.
- Generated placeholder WikiLinks such as `[[Concept Placeholder]]`, `[[Topic Placeholder]]`, and `[[LLM_WIKI_SCHEMA]]` were removed.
- `--dry-run` remains supported.
- `--preview` is available as an equivalent inspection alias for dry-run-capable commands.
- `kb lint-static --no-report` is available as a clearer alias for checking without writing `outputs/reports/lint_static_*.md`.

## v0.4.4

`v0.4.4` adds the second step in the traceable AI-maintained Wiki loop: static linting.

Main changes:

- Added `kb lint-static`.
- Checks broken `[[WikiLinks]]`, orphan pages, source-derived pages missing `source_ids` / `source_files`, empty pages, and duplicate titles.
- Does not call an LLM and does not rewrite `wiki/`.
- Writes a reviewable report under `outputs/reports/`.

## v0.4.3

`v0.4.3` closes the first prepare loop with `kb sync-wiki`.

Main changes:

- `kb sync-wiki` scans YAML front matter under `wiki/`.
- Reads `source_ids` and `source_files`.
- Records corresponding `wiki_pages` back into `processing/manifest.json`.

Related `v0.4.2` change:

- `kb prepare` writes a copy-paste agent prompt:

```text
processing/proposals/prepare_agent_prompt_<timestamp>.md
```

- `kb prepare` still does not call an LLM and does not directly edit `wiki/`.

## v0.4.1

`v0.4.1` is a small prepare-fix release.

Main changes:

- Fixed a Rust ownership issue in `kb prepare` where a manifest entry ID was moved before the entry was later borrowed to generate proposed wiki pages and instructions.
- No behavior change from `v0.4.0`.

## v0.4.0

`v0.4.0` is the first prepare skeleton release.

Main changes:

- Added a review-first command for planning how raw files should be prepared for Markdown wiki pages.
- Does not call an LLM.
- Does not edit `wiki/` directly.

Useful prepare commands:

```bash
kb --kb-path /path/to/kb prepare --new --dry-run
kb --kb-path /path/to/kb prepare --new
kb --kb-path /path/to/kb prepare --file raw/papers/example.pdf
```

Generated files:

```text
processing/prepare_queue.json
processing/proposals/prepare_plan_<timestamp>.md
```

## v0.3.2

Small cross-platform hygiene release.

Main changes:

- Kept README clarification about how future LLM maintainers should update Markdown pages.
- Added `.gitattributes` to make line-ending behavior explicit across Windows, Linux, and macOS.

Recommended one-time normalization:

```bash
git add .gitattributes
git add --renormalize .
git status
```

## v0.3.1

`kb status` now has safer manifest tracking and better inspection output.

Main changes:

- Scans `raw/`.
- Writes `processing/manifest.json`.
- Records each raw file's relative path, kind, extension, size, SHA-256 content hash, first-seen time, last-seen time, status, and future wiki page links.
- Missing raw files are retained as `raw_missing` instead of being silently dropped.
- Prepares the project for `kb prepare --new`.

Useful status modes:

```bash
kb --kb-path /path/to/kb status
kb --kb-path /path/to/kb status --json
kb --kb-path /path/to/kb status --unprocessed
kb --kb-path /path/to/kb status --dry-run
```

## v0.2.0

`kb init` creates the third LLM Wiki layer:

```text
rules/
├── LLM_WIKI_SCHEMA.md
├── PAPER_PAGE_TEMPLATE.md
├── CONCEPT_PAGE_TEMPLATE.md
├── QUERY_POLICY.md
└── LINT_POLICY.md
```

This layer is the operating contract for future AI agents. It explains how to treat `raw/`, how to maintain `wiki/`, how to answer questions, and how to lint the Wiki.
