# Changelog

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
- Manifest entries linked to wiki pages are marked as `compiled`.
- `kb status` now reports compiled files and `status --json` includes `compiled_files`.
- Updated paper/concept templates and compile agent prompts to require source-linking front matter.
- Added `docs/sync-wiki.md`.

All notable changes to this project will be documented in this file.

## [0.4.2] - 2026-05-07

### Added
- Added `processing/proposals/compile_agent_prompt_<timestamp>.md` generation for non-dry-run `kb compile` runs.
- The generated agent prompt is designed as a copy-paste instruction block for Claude Code, ChatGPT, or another knowledge agent.

### Changed
- Improved `kb compile` next-step output to point users to both the review proposal and the agent prompt.
- Updated compile documentation to describe the new prompt handoff artifact.

### Notes
- `kb compile` still does not call an LLM and still does not directly edit `wiki/`. This release only improves the review-first AI handoff workflow.

## [0.4.1] - 2026-05-07

### Fixed
- Fixed a Rust ownership error in `src/commands/compile.rs` where `entry.id` was moved before `entry` was borrowed again for proposed wiki page and instruction generation.

### Notes
- This is a compile-fix release. It does not change `kb compile` behavior: compile remains a review-first planning command and still does not call an LLM or directly edit `wiki/`.


## [0.4.0] - 2026-05-06

### Added
- Added `kb compile` as a conservative compile-planning skeleton.
- Added `kb compile --new --dry-run` to preview raw files ready for future LLM compilation.
- Added `kb compile --new` to write `processing/compile_queue.json` and a reviewable proposal file under `processing/proposals/`.
- Added `kb compile --file <PATH>` to plan work for one manifest-tracked raw file.

### Changed
- Updated README and agent boundary documentation to distinguish compile planning from autonomous LLM editing.
- Updated manifest schema version marker to `0.4.0`.

### Notes
- This release does not call an LLM and does not directly modify `wiki/`. It creates review artifacts that a human or LLM agent can use under Git diff.

## [0.3.2] - 2026-05-06

### Added
- Added `.gitattributes` to define cross-platform line-ending rules.
- Added binary file rules for common archive, document, image, and executable formats.

### Changed
- Clarified README wording around how future LLM maintainers should maintain Markdown pages under `wiki/`.
- Updated the README to distinguish current scaffold/manifest behavior from planned LLM-driven `compile`, `query`, and `lint` commands.

### Notes
- This is a documentation and repository-hygiene release. It does not introduce LLM compilation, query, lint, vector search, RAG, or a database.

## [0.3.1] - 2026-05-05

### Added
- Added `kb status --json` for machine-readable manifest summaries.
- Added `kb status --unprocessed` to list raw files awaiting future compile work.
- Added next-step hints after successful `kb bootstrap`.

### Changed
- Changed manifest content fingerprints from the earlier FNV-style hash to `sha256:<digest>`; existing FNV entries are migrated without being falsely counted as content changes.
- Retained disappeared raw files in `processing/manifest.json` with status `raw_missing` instead of dropping their history.
- Improved ingest duplicate handling: if a destination name already exists, identical content is skipped; different content is saved with a unique suffix instead of being silently skipped.

### Notes
- This remains a manifest/ingest stability release. It still does not introduce LLM compilation, query, lint, vector search, RAG, or a database.

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
- This release still does not introduce LLM compilation, query, lint, vector search, RAG, a database, or a JSON agent API. It creates the tracking layer needed for future `kb compile --new` behavior.

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
- This release still does not introduce LLM compilation, query, lint, vector search, RAG, a database, or a JSON agent API. It establishes the contract layer those future commands should follow.

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
