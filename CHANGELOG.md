# Changelog

All notable changes to this project will be documented in this file.

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
