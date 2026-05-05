# CLAUDE.md

## Project

`kb-cli`

## Current Version Target

`v0.1.2` is a warning-cleanup release built on `v0.1.1`. Its purpose is to keep behavior unchanged while making `cargo check` cleaner and preserving the command/document/template consistency already established in `v0.1.1`.

## Project Positioning

This is a local-first Rust CLI for maintaining a Karpathy-style Markdown knowledge base:

- Source materials live in `KnowledgeBase/raw/`.
- Generated or curated Markdown pages live in `KnowledgeBase/wiki/`.
- Obsidian is the preferred reading and editing frontend.
- The Rust project and the `KnowledgeBase/` data directory must remain separate.

Recommended layout:

```text
workspace/
├─ kb-cli/              # Rust tool project
└─ KnowledgeBase/       # User data managed by the tool
```

## Implemented v0.1.2 Commands

Only these commands should be treated as implemented:

```bash
kb init [--force]
kb extract-metadata [--force]
kb build-wiki
kb repl
kb list-models
kb show-model
kb add-model [OPTIONS]
kb switch-model <id>
kb delete-model <id>
kb validate-model [id]
```

## Not Implemented Yet

Do not claim or document these as working features until code exists and has been checked:

```bash
kb add-paper
kb search
kb list-papers
kb list-notes
kb --format json ...
kb --use-llm search ...
```

The REPL contains simple interactive list/search helpers, but they are not top-level bash subcommands.

## Strict Development Rules

1. Keep the binary command name as `kb`.
2. Keep the default knowledge-base directory name as `KnowledgeBase`.
3. Keep note input under `raw/notes` using lowercase `notes`.
4. Do not reintroduce `raw/Notes`.
5. Do not reintroduce `cli`, `cli.exe`, or `cli_mini_rust` as user-facing command names.
6. Do not hard-code domain-specific examples such as droplet microfluidics into generic wiki templates.
7. Do not introduce vector databases, RAG, embeddings, network calls, or large frameworks in v0.1.x consistency patches.
8. Do not store real API keys in the repository.
9. Keep `.model_config.json`, `.model_switch_input.json`, and `.model_switch_output.json` out of Git.
10. Prefer small, reviewable changes over sweeping refactors.

## Model Configuration Rules

- `.model_config.example.json` may be committed.
- `.model_config.json` is local runtime state and must remain ignored.
- Environment variable names should be shell-friendly, for example `KIMI_K2_5_API_KEY`, not `KIMI-k2.5`.

## Model-Switch Bridge Rules

The file bridge must avoid stale outputs.

Current expected request file:

```text
.model_switch_input.json
```

Current expected output file:

```text
.model_switch_output.json
```

Each request must contain `request_id`. A response should include the same `request_id`. If no matching response exists, the CLI should report that the request was written instead of pretending that an answer is available.

## Before Coding

For any new task, first produce a short plan with:

```markdown
## Task Understanding
## Files Affected
## What Will Change
## What Will Not Change
## Verification Plan
```

Do not start large implementation work before the plan is accepted.

## Verification Checklist

After any patch, check at minimum:

```bash
cargo fmt
cargo check
kb --help
kb init --help
kb extract-metadata --help
```

If the current environment cannot run Cargo, say so plainly and perform static checks instead.
