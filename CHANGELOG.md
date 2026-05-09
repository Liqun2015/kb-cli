## v0.6.0.4 - Shell Windows path parsing test fix

- Fixed `split_shell_words` so quoted Windows paths preserve backslashes, for example `use "D:\github\llm-wiki\quantum"`.
- The strict shell remains a whitelist command shell: unknown input is still a safe no-op and is not interpreted as natural language.
- No LLM call, no shell command execution, no command behavior expansion, and no Wiki auto-editing was added.

## v0.6.0.1 - Strict shell safety rule

## v0.6.0.3 - links compile fix

- Fixed a `kb links` compile error caused by an outdated local variable name in the link scan report summary.
- `pages_scanned` now uses the actual scanned Markdown page collection.
- No command behavior changes, no LLM calls, no new features.


## v0.6.0.2 - Cargo lockfile compatibility fix

- Removed the stale generated `Cargo.lock` from the distributed source package.
- This avoids a Windows dependency resolution failure where the lockfile pinned `windows-strings` to `0.5.2`, while the available crates.io index resolves `windows-strings` to `0.5.1`.
- `cargo check` / `cargo build` will regenerate a fresh local `Cargo.lock` on the developer machine.
- After a successful local build, review and commit the generated `Cargo.lock` if this project is being managed as an application repository.

- Tightened `kb shell` into a strict whitelist command shell.
- Unknown input now returns safely with a no-action message instead of being delegated as arbitrary batch input.
- Added shell whitelist tests for known structured commands and rejected free-form text.
- Updated `docs/shell.md`, `docs/cli-shell-principle.md`, `docs/commands.md`, `docs/agent-api.md`, and `docs/claude.md` with the final-else safe-return rule.
- Reaffirmed that `kb shell` is not an LLM chat interface and must not guess Manager LLM intent.

## v0.6.0 - Deterministic shell skeleton

- Added `kb shell` as the deterministic interactive command shell.
- Kept `kb repl` only as a compatibility alias; documentation should prefer `kb shell`.
- Shell session commands: `use`, `pwd`, `clear`, `help`, and `exit`.
- Ordinary shell input is delegated to the same batch-mode `kb` executable with the current `--kb-path`, preserving one-to-one command semantics.
- Added `docs/shell.md` and updated CLI/shell/LLM boundary documentation.
- Reinforced that `kb>` is not an LLM chat interface and must not implicitly interpret free-form natural language.

## v0.5.13 - Health / relation-health skeleton

- Added `kb health` as a deterministic LLM Wiki relationship-health dashboard.
- The command summarizes raw papers, extracted text files, wiki pages, refs reports, refs-index reports, refs-graph exports, keyword reports, task files, memory files, lint reports, and third-party skill docs.
- Reports are written under `outputs/reports/health_*.md` unless `--dry-run` / `--preview` is used.
- Added `docs/health.md`.
- `kb health` emits deferred task hints for missing text extraction, uncertain bibliographic index review, keyword/topic review, and initial wiki drafting.
- The command does not call an LLM, run OCR, confirm bibliographic identity, repair files, create wiki pages, or decide scientific idea relations.

## v0.5.12 - Keywords skeleton

- Added `kb keywords` as a deterministic keyword/topic co-occurrence scanner over extracted text.
- Added `docs/keywords.md`.
- `kb keywords` scans `processing/text/` by default and can use explicit terms, a terms file, or deterministic auto-extracted candidate terms.
- Reports are written under `processing/keywords/keywords_*.md` unless `--dry-run` / `--preview` is used.
- `kb init` now creates `processing/keywords/`.
- `kb tasks` now detects keyword/topic candidate reports and prepares a Keyword Topic Relation Worker handoff.
- The command emits keyword/topic relations as candidates only; shared terms are evidence, not confirmed scientific idea relations.
- No LLM call, vector search, semantic idea inference, topic merge, or autonomous Wiki rewrite was added.

## v0.5.10.4 - README Manager/Worker maintenance loop clarification

- Updated the README section "How does the LLM maintain Markdown pages?" to show the intended hierarchy: Manager LLM -> deterministic kb-cli commands -> Worker LLM / human reviewer -> Git review -> wiki and LLM/memory records.
- Clarified that Manager LLM sessions should scout with `kb query`, `kb grep`, `kb extract-text`, `kb refs`, `kb refs-index`, `kb links`, `kb lint-static`, and `kb tasks` before delegating semantic work.
- Clarified that Worker LLMs receive bounded tasks with goal, requirements, file list, evidence lines, expected output, and review rules.
- Reaffirmed that humans remain the final guarantee for uncertain bibliographic index relations.
- No command behavior, LLM call, OCR, vector search, autonomous worker execution, or Wiki auto-editing was added.

## v0.5.10.3 - Documentation convergence and command overview

- Added `docs/commands.md` as the central command map for humans, Manager LLM sessions, Worker LLM tasks, and future third-party skills.
- Added `docs/task-lifecycle.md` to define the `LLM/tasks/` to `LLM/memory/` handoff and review lifecycle.
- Documented command categories, inputs, outputs, LLM boundaries, and deferred-work expectations.
- Clarified task statuses such as `pending`, `assigned`, `in_progress`, `needs_human`, `blocked`, `completed`, and `rejected`.
- Kept the build-helper improvement: `scripts/build_release.bat` and `scripts/build_release.sh` now run `cargo fmt` before `cargo fmt --check`.
- Updated script documentation, README navigation, and release checklist guidance.

## v0.5.10.2 - Third-party skills guidance

- Renamed `docs/third-party-tools/` to `docs/third-party-skills/` to make the directory usable as a reusable skill specification area, not merely a tool-documentation folder.
- Added `claude-code-generation-skill.md`, a direct promptable skill spec for Claude Code or similar coding agents to generate graph viewers, importers, exporters, or review dashboards.
- Kept the graph certainty protocol: confirmed relations render as solid edges, uncertain relations as dashed edges, and unresolved references as hollow nodes.
- Preserved the evidence and human-review requirements for bibliographic index relations.
- No new graph exporter is implemented in this release. Future `refs-graph` work should follow these skills.

# Changelog

## v0.5.10 - Bibliographic index relation candidates

- Added `kb refs-index` as a deterministic bibliographic index relation candidate builder.
- The command scans extracted text under `processing/text/` by default and matches reference entries against local papers in `raw/papers/`.
- Relation statuses include `confirmed`, `candidate`, `ambiguous`, and `missing`.
- Reports are written under `processing/refs/refs_index_*.md` unless `--dry-run` / `--preview` is used.
- Candidate, ambiguous, and missing cases include a deferred human-review task because humans remain the final guarantee for uncertain bibliographic identity decisions.
- No LLM call, OCR, online DOI lookup, autonomous citation graph construction, or Wiki rewriting was added.

## v0.5.10 - Literature relationship core and Manager/Worker terminology

- Standardized LLM hierarchy terminology to `Manager LLM` and `Worker LLM`.
- Added `docs/literature-relationships.md` to make the central project principle explicit: LLM Wiki maintains relationships among references, not merely files.
- Defined three relationship levels: bibliographic index relations, keyword/topic relations, and scientific idea relations.
- Clarified that Rust-native commands extract bibliographic candidates, humans provide the final guarantee for uncertain index relations, and Manager/Worker LLM workflows handle higher-level semantic relationship work.
- Updated README and LLM-related docs to use Manager/Worker terminology consistently.
- No command behavior, LLM call, OCR, vector search, or autonomous relation repair was added.

## v0.5.9 - Links skeleton

- Added `kb links` as a deterministic WikiLink scanner over Markdown pages.
- `kb links` extracts `[[Target]]`, `[[Target|Alias]]`, and `[[Target#Heading|Alias]]` links and classifies them as resolved, unresolved, or ambiguous.
- Added `docs/links.md`.
- Reports include deferred task hints for future Wiki Link Repair Agent work when unresolved or ambiguous links are found.
- The command does not call an LLM, rewrite Wiki pages, create missing pages, rename files, or perform semantic link repair.

## v0.5.8.4 - LLM hierarchy clarification

- Added `docs/llm-hierarchy.md` to define the hierarchy between Manager LLMs and Worker LLMs.
- Clarified that the LLM using structured `kb` commands is the highest-level coordinator: it scouts with deterministic commands, reads task files, delegates bounded work, checks results, and records completion memory.
- Clarified that lower-level LLMs or Worker LLMs should execute specific handoff tasks rather than redefine workflow scope.
- Updated LLM command, agent-skill, command-classification, Claude, and agent API docs to reflect the hierarchy.
- No new command behavior, LLM call, OCR, vector search, or autonomous agent execution was added.


## v0.5.8.3 - LLM structured command guide

- Added `docs/llm-command-guide.md` as a practical operating guide for future LLM maintainers and Claude Code sessions.
- Documented the principle that Rust-native `kb` commands scout first: search, inspect, extract, count, and report before hard semantic work is given to LLM/agent skills.
- Added a command map for LLM operators covering `query`, `grep`, `extract-text`, `refs`, `tasks`, and `memory`.
- Clarified how `LLM/tasks/` and `LLM/memory/` should be used as handoff and audit-memory areas.
- Reinforced that this is a documentation-only polish release: no new command behavior, LLM call, OCR, vector search, or autonomous agent execution was added.

## v0.5.8.2 - Completed task memory

- Added `kb memory` for appending completed task records under `LLM/memory/completed_tasks.md`.
- `kb init` now creates `LLM/memory/`.
- Added `docs/memory.md` to document the completed-task audit-memory workflow.
- Clarified that completed task memories are project-local records for later inspection, not hidden model memory.
- No LLM, OCR, vector search, autonomous task execution, or semantic validation was added.

## v0.5.8.1 - LLM task workspace path

- Moved deterministic handoff reports from the generic `outputs/tasks/` location to the explicit LLM workbench path: `LLM/tasks/`.
- `kb tasks` now writes Markdown reports to `LLM/tasks/` unless `--dry-run`/`--preview` is used.
- `kb init` now creates `LLM/tasks/`.
- Updated `docs/tasks.md`, `docs/command-classification.md`, and `docs/llm-agent-skills.md` to state that deterministic commands are the first scouting pass and that unresolved hard work should be handed upward to future explicit human/LLM/agent skills.
- No OCR, LLM call, vector search, citation graph construction, autonomous Wiki editing, or hidden agent execution was added.

## v0.5.8 - Deferred task handoff skeleton

- Added `kb tasks` to generate deterministic LLM/agent handoff task lists.
- Task groups include target agent, goal, requirements, file list, evidence, source command, and priority.
- Current task groups cover PDF text conversion gaps, reference reconciliation candidates, Wiki source traceability gaps, and broken WikiLinks.
- Added `docs/tasks.md`.
- Reinforced the rule that deterministic commands should do one ability well and defer unresolved semantic work explicitly instead of hiding LLM behavior.
- No OCR, LLM call, vector search, citation graph construction, autonomous Wiki editing, or hidden agent execution was added.

## v0.5.7.1 - Deferred LLM/agent skill boundaries

- Added `docs/llm-agent-skills.md` to document future explicit LLM/agent skill responsibilities.
- Clarified that `kb extract-text` and `kb refs` should do simple deterministic work well, then clearly mark work that requires OCR, semantic repair, citation reconciliation, or LLM synthesis.
- Updated `docs/extract-text.md` with tasks deferred to a future PDF Text Conversion Agent.
- Updated `docs/refs.md` with tasks deferred to future Reference Reconciliation and Citation Graph Building agents.
- Updated command-classification, agent boundary, and Claude guardrail docs so future versions do not hide LLM work inside deterministic commands.
- No OCR, LLM call, vector search, reference graph generation, or autonomous wiki editing was added.

## v0.5.7 - Refs skeleton

- Added `kb refs` as a deterministic reference-hint scanner over extracted text.
- Added `docs/refs.md`.
- `kb refs` scans `processing/text/` by default and supports `--path`, `--limit`, `--citations`, and `--json`.
- The scanner detects reference headings, numbered bibliography entries, DOI values, and optional citation-marker candidates.
- `kb init` now creates `processing/refs/` as a future home for reference reports or graph exports.
- No LLM, OCR, semantic citation reconciliation, vector search, or reference graph generation was added.

## v0.5.6.1 - Command classification principle

- Added `docs/command-classification.md` to classify commands into deterministic core, deterministic search/inspection, text conversion/source processing, task preparation, and explicit future LLM/agent commands.
- Added `kb grep` as a Rust-native deterministic grep-like command for line-level text search.
- Added `kb extract-text` as a deterministic best-effort text extraction command that writes to `processing/text/`.
- Added `docs/grep.md` and `docs/extract-text.md`.
- Clarified that `kb extract-text` is reserved as a future PDF Text Conversion Agent entry point, but OCR, LLM cleanup, layout repair, and agent behavior must be explicit future modes.
- Reinforced that deterministic commands must not silently call LLM APIs or external grep/rg tools.

## v0.5.4 - Clean script helpers

- Removed broad knowledge-base workflow wrapper scripts from `scripts/` to avoid confusion with compiling the Rust executable.
- Added `scripts/build_release.bat` for the Windows Rust verification/build flow.
- Added `scripts/build_release.sh` for the macOS/Linux Rust verification/build flow.
- Added `scripts/README.md` to define what belongs in the scripts directory.
- Kept the review-first Git helper scripts.
- Updated quickstart and platform docs to prefer explicit `kb ...` command sequences for knowledge-base workflows.
- No query behavior, REPL behavior, LLM calls, embeddings, vector search, RAG, or autonomous editing changed.

## v0.5.3 - Remove ambiguous REPL LLM-like commands

- Removed LLM-like command patterns from the experimental `kb repl` parser instead of merely rejecting them at runtime.
- Removed REPL branches and helper functions for ambiguous natural-language/LLM-style commands.
- Updated CLI/shell boundary documentation so placeholder LLM command names are no longer advertised.
- Kept `kb>` as a deterministic command shell only.
- No LLM calls, embeddings, vector search, RAG, autonomous editing, or hidden natural-language command interpretation were added.

## v0.5.2 - CLI / Shell / LLM boundary

- Added `docs/cli-shell-principle.md` to define the relationship between batch mode, interactive shell mode, and future LLM-assisted modes.
- Documented that `kb ...` is deterministic batch mode and `kb>` must remain a deterministic command shell rather than an implicit LLM chat interface.
- Documented that future LLM behavior must be introduced through a deliberately designed explicit interface.
- Disabled LLM-like requests inside the current experimental `kb repl`; they now explain the boundary instead of calling a model.
- No embeddings, vector search, RAG, saved answers, autonomous wiki editing, or hidden LLM calls were added.

## v0.5.1 - README workflow wording clarity

- Clarified the README's Karpathy-style workflow sentence.
- Documented `kb prepare` as generating reviewable task materials rather than directly preparing final `wiki/` pages.
- Clarified that `wiki/` stores maintained Markdown knowledge pages and `rules/` constrains future human/AI maintainers.
- No query behavior, LLM calls, embeddings, vector search, RAG, or autonomous LLM execution was added.

## v0.5.0 - Query skeleton

- Added `kb query` for deterministic local keyword search over `wiki/**/*.md`.
- Query searches titles, relative paths, and Markdown bodies with simple AND semantics across terms.
- Added `--limit`, `--snippets`, `--json`, and `--title-only` options.
- Added `docs/query.md` to document the query boundary and usage examples.
- Updated README, agent boundary notes, and developer guardrails for v0.5.0.
- No LLM calls, embeddings, vector search, RAG, semantic reranking, saved answers, or background indexing were added.

## v0.4.8 - Developer workflow helpers

- Added `scripts/git_safe_push.bat` for Windows users who want a review-first `git status` / `git diff` / `git add` / `git diff --cached` / `git commit` / `git push` flow.
- Added `scripts/git_safe_push.sh` with the same two-review flow for macOS/Linux users.
- Added `docs/git-workflow.md` to document safe commit and push habits for this project.
- Added `docs/release-checklist.md` for small local releases.
- Updated README documentation links and version markers to `v0.4.8`.
- No query, vector search, embeddings, RAG, or autonomous LLM execution was added.

## v0.4.7 - Cross-platform quickstart polish

- Added `docs/unix-quickstart.md` for macOS/Linux users.
- Added `docs/platform-notes.md` to document path syntax, shell differences, wrapper-script policy, and release smoke-test expectations.
- Added `scripts/build_llm_wiki.sh` as a thin Unix shell helper around the Rust CLI bootstrap flow.
- Rewrote `docs/cross-platform-quickstart.md` so Windows and Unix examples are both first-class.
- Updated Windows and core docs to point to the cross-platform documentation set.
- Bumped Cargo package version and schema/version markers to `0.4.7`.
- No query, vector search, embeddings, RAG, or autonomous LLM execution was added.

## v0.4.6.2 - Prepare-only documentation polish

- Updated README and docs so the review-first planning step is documented only as `kb prepare`.
- Removed the legacy command page from `docs/`.
- Hid the reserved legacy spelling from the top-level help output while keeping command compatibility in code.
- Cargo package metadata uses `0.4.6-2` because Cargo requires standard SemVer. The release/package label remains `v0.4.6.2`.

## v0.4.6.1 - Naming alignment for prepare

- Made `kb prepare` the documented user-facing planning command.
- Generated handoff artifacts use `processing/prepare_queue.json`, `processing/proposals/prepare_plan_<timestamp>.md`, and `processing/proposals/prepare_agent_prompt_<timestamp>.md`.
- Updated README and docs to use `prepare` as the primary command name.
- Cargo package metadata uses `0.4.6-1` because Cargo requires standard SemVer. The release/package label remains `v0.4.6.1`.

## v0.4.6 - Tests and documentation stabilization

- Added focused unit/regression tests for `build-wiki` front matter generation and placeholder-link avoidance.
- Added focused unit/regression tests for `lint-static` WikiLink parsing, YAML source-reference parsing, issue detection, `--no-report`, and default report writing.
- Expanded README with a dedicated testing and release-check section.
- Expanded `docs/windows-quickstart.md` with a PowerShell-first workflow, dry-run/preview examples, lint commands, and troubleshooting notes.
- Expanded `docs/lint-static.md` with issue categories, concrete examples, cleanup order, JSON/strict usage, and test coverage notes.
- Bumped schema/version markers to `0.4.6`.
- No query, vector search, embeddings, RAG, or autonomous LLM execution was added.

## v0.4.5 - Build/lint consistency polish

- Kept `--dry-run` and added `--preview` as an equivalent inspection alias for dry-run-capable commands.
- Added `kb lint-static --no-report` as a clearer alias for checking without writing `outputs/reports/lint_static_*.md`.
- Updated `kb build-wiki` so generated paper and note pages include YAML source front matter.
- Generated paper/note pages now include `source_files`, and include `source_ids` when a matching manifest entry is available.
- Removed generated placeholder WikiLinks that caused avoidable broken-link noise, including `[[Concept Placeholder]]`, `[[Topic Placeholder]]`, and `[[LLM_WIKI_SCHEMA]]`.
- Bumped schema/version markers to `0.4.5` and updated README/docs.

## v0.4.4 - Static wiki lint

- Added `kb lint-static` for deterministic Markdown health checks under `wiki/`.
- The static lint pass checks broken `[[WikiLinks]]`, orphan pages, missing source front matter, empty/near-empty pages, and duplicate titles.
- Added `--dry-run`, `--json`, and `--strict` modes for linting.
- Lint reports are written to `outputs/reports/lint_static_<timestamp>.md`.
- Added `docs/lint-static.md` and updated README/agent boundary notes.

## v0.4.3 - Sync wiki source links back to manifest

- Added `kb sync-wiki` to scan Markdown front matter under `wiki/` and update `processing/manifest.json`.
- Wiki pages can now declare `source_ids` and `source_files` in YAML front matter.
- Manifest entries linked to wiki pages are marked as source-linked in the manifest.
- `kb status` now reports source-linked wiki coverage in the manifest summary.
- Updated paper/concept templates and prepare agent prompts to require source-linking front matter.
- Added `docs/sync-wiki.md`.

All notable changes to this project will be documented in this file.

## [0.4.2] - 2026-05-07

### Added
- Added `processing/proposals/prepare_agent_prompt_<timestamp>.md` generation for non-dry-run `kb prepare` runs.
- The generated agent prompt is designed as a copy-paste instruction block for Claude Code, ChatGPT, or another knowledge agent.

### Changed
- Improved `kb prepare` next-step output to point users to both the review proposal and the agent prompt.
- Updated prepare documentation to describe the new prompt handoff artifact.

### Notes
- `kb prepare` still does not call an LLM and still does not directly edit `wiki/`. This release only improves the review-first AI handoff workflow.

## [0.4.1] - 2026-05-07

### Fixed
- Fixed a Rust ownership error in the planning command implementation where `entry.id` was moved before `entry` was borrowed again for proposed wiki page and instruction generation.

### Notes
- This is a prepare-fix release. It does not change `kb prepare` behavior: prepare remains a review-first planning command and still does not call an LLM or directly edit `wiki/`.


## [0.4.0] - 2026-05-06

### Added
- Added `kb prepare` as a conservative prepare-planning skeleton.
- Added `kb prepare --new --dry-run` to preview raw files ready for future LLM/wiki preparation.
- Added `kb prepare --new` to write `processing/prepare_queue.json` and a reviewable proposal file under `processing/proposals/`.
- Added `kb prepare --file <PATH>` to plan work for one manifest-tracked raw file.

### Changed
- Updated README and agent boundary documentation to distinguish prepare planning from autonomous LLM editing.
- Updated manifest schema version marker to `0.4.0`.

### Notes
- This release does not call an LLM and does not directly modify `wiki/`. It creates review artifacts that a human or LLM agent can use under Git diff.

## [0.3.2] - 2026-05-06

### Added
- Added `.gitattributes` to define cross-platform line-ending rules.
- Added binary file rules for common archive, document, image, and executable formats.

### Changed
- Clarified README wording around how future LLM maintainers should maintain Markdown pages under `wiki/`.
- Updated the README to distinguish current scaffold/manifest behavior from planned LLM-driven `prepare`, `query`, and `lint` commands.

### Notes
- This is a documentation and repository-hygiene release. It does not introduce LLM/wiki preparation, query, lint, vector search, RAG, or a database.

## [0.3.1] - 2026-05-05

### Added
- Added `kb status --json` for machine-readable manifest summaries.
- Added `kb status --unprocessed` to list raw files awaiting future prepare work.
- Added next-step hints after successful `kb bootstrap`.

### Changed
- Changed manifest content fingerprints from the earlier FNV-style hash to `sha256:<digest>`; existing FNV entries are migrated without being falsely counted as content changes.
- Retained disappeared raw files in `processing/manifest.json` with status `raw_missing` instead of dropping their history.
- Improved ingest duplicate handling: if a destination name already exists, identical content is skipped; different content is saved with a unique suffix instead of being silently skipped.

### Notes
- This remains a manifest/ingest stability release. It still does not introduce LLM/wiki preparation, query, lint, vector search, RAG, or a database.

## [0.3.0] - 2026-05-05

### Added
- Added `kb status` for scanning `raw/` and refreshing `processing/manifest.json`.
- Added manifest entries with relative path, source kind, extension, file size, content hash, status, first-seen timestamp, last-seen timestamp, and future wiki page links.
- Added `processing/proposals/` as the future review area for AI-generated edit proposals.
- Added automatic manifest refresh after non-dry-run `kb ingest` and as part of `kb bootstrap` workflows.

### Changed
- Updated README and documentation to describe the manifest stage and the new `kb status` command.
- Updated generated knowledge-base README text to include `processing/manifest.json`.

### Notes
- This release still does not introduce LLM/wiki preparation, query, lint, vector search, RAG, a database, or a JSON agent API. It creates the tracking layer needed for future `kb prepare --new` behavior.

## [0.2.0] - 2026-05-05

### Added
- Added the LLM Wiki rules/schema layer generated by `kb init`.
- Added generated `rules/LLM_WIKI_SCHEMA.md` as the AI-maintainer operating contract.
- Added generated templates and policies: `PAPER_PAGE_TEMPLATE.md`, `CONCEPT_PAGE_TEMPLATE.md`, `QUERY_POLICY.md`, and `LINT_POLICY.md`.
- Added expanded wiki folders for people, methods, comparisons, timelines, and saved questions.

### Changed
- Updated initialized `README.md` to describe the three-layer LLM Wiki structure: `raw/`, `wiki/`, and `rules/`.
- Updated project README and documentation boundaries for the v0.2.0 Schema stage.

### Notes
- This release still does not introduce LLM/wiki preparation, query, lint, vector search, RAG, a database, or a JSON agent API. It establishes the contract layer those future commands should follow.

## [0.1.4] - 2026-05-05

### Added
- Added cross-platform `kb ingest` command for organizing source files into `raw/` subfolders.
- Added cross-platform `kb bootstrap` command for running `init + ingest + extract-metadata + build-wiki` in one workflow.
- Added safe default copy behavior for ingest/bootstrap, with explicit `--move` for destructive reorganization.
- Added `--recursive` and `--dry-run` support to ingest/bootstrap.
- Added `raw/archives`, `raw/other`, `processing`, and `references` to the initialized knowledge-base structure.
- Added `docs/cross-platform-quickstart.md`.

### Changed
- Refactored `scripts/build_llm_wiki.bat` into a Windows wrapper around the cross-platform Rust `kb bootstrap` command.
- Updated README, Windows quick-start, agent-boundary notes, and Claude guardrails for the new cross-platform workflow.
- Improved custom path resolution for absolute non-existing target paths.

### Notes
- This release keeps the project file-based and does not introduce RAG, vector search, a database, or a JSON agent API.

## [0.1.3] - 2026-05-05

### Added
- Added `scripts/build_llm_wiki.bat` as a Windows quick-start helper for turning an existing literature folder into an LLM Wiki.
- The helper can initialize a custom knowledge-base path, organize source files into `raw/` subfolders, extract PDF metadata, and build the wiki in one run.
- Added optional helper flags: `--copy`, `--recursive`, `--force-init`, `--no-install`, and `--no-pause`.

### Documentation
- Added README instructions for the Windows one-command quick-start workflow.

### Notes
- This release does not change the core Rust CLI behavior. It adds an onboarding script around existing `kb` commands.

## [0.1.2] - 2026-05-05

### Fixed
- Cleaned `cargo check` warnings without adding new features.
- Handled the `stdout().flush()` result in REPL startup.
- Normalized extracted PDF metadata strings through the existing `clean_string` helper.
- Marked intentionally reserved model-switch/model-config compatibility helpers with `#[allow(dead_code)]`.
- Marked intentionally reserved intent helper methods with `#[allow(dead_code)]`.

### Notes
- This is a warning-cleanup release. It does not introduce RAG, vector search, database storage, or a full JSON agent API.

## [0.1.1] - 2026-05-05

### Fixed
- Unified the user-facing command name as `kb`.
- Added `--force` to `kb init` and `kb extract-metadata`.
- Standardized notes input directory as `raw/notes`.
- Removed domain-specific droplet/microfluidics hard-coding from generic wiki templates.
- Prevented title-less papers from collapsing to `Untitled.md` by falling back to the PDF filename stem.
- Corrected `ModelConfigError` so it no longer reports itself as its own source error.
- Replaced shell-unfriendly `KIMI-k2.5` env var name with `KIMI_K2_5_API_KEY`.
- Changed model-switch requests to include `request_id` and avoid stale output reuse.
- Moved runtime model configuration from `.model_config.json` to `.model_config.example.json`.
- Added runtime model-switch files to `.gitignore`.

### Documentation
- Updated README to describe only implemented v0.1.1 commands.
- Rewrote `docs/agent-api.md` as current-boundary notes instead of a future-feature promise.
- Rewrote `docs/claude.md` to enforce v0.1.1 consistency boundaries.

## [0.1.0]

### Added
- Initial release.
- Knowledge base initialization.
- PDF metadata extraction.
- Wiki page generation (Markdown, Obsidian compatible).
- Interactive REPL mode with intent parsing.
- LLM integration with multi-model support.
- Model configuration management.

### Changed
- Renamed project from `cli_mini_rust` to `kb-cli`.
- Changed binary name from `cli` to `kb`.
