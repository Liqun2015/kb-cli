# claude.md — Development Guardrails for kb-cli

## Current version

`kb-cli v0.6.0.1`

## Core goal

This project is a small Rust CLI for a local Karpathy-style LLM Wiki. It should remain simple, file-based, inspectable, and cross-platform.

The intended three-layer model is:

```text
raw/    original source materials; AI reads but does not rewrite
wiki/   AI-maintained Markdown encyclopedia
rules/  schema/contract layer that constrains AI maintenance behavior
```

## v0.4.0 design decision

This release establishes manifest tracking. `kb status` must scan `raw/` and maintain:

```text
processing/manifest.json
```

The manifest is the bridge between the source-material layer and future AI prepare operations. It should track raw files by relative path, source kind, size, SHA-256 content hash, status, first-seen timestamp, and last-seen timestamp; missing raw files should remain as `raw_missing`.

The rule/schema layer added in v0.2.0 remains required. `kb init` must still generate:

```text
rules/LLM_WIKI_SCHEMA.md
rules/PAPER_PAGE_TEMPLATE.md
rules/CONCEPT_PAGE_TEMPLATE.md
rules/QUERY_POLICY.md
rules/LINT_POLICY.md
```

## Implemented commands

For the canonical command overview, inputs, outputs, LLM boundaries, and deferred-work rules, read:

```text
docs/commands.md
docs/task-lifecycle.md
```

Current implemented commands include:

```text
init
ingest
bootstrap
extract-metadata
extract-text
build-wiki
status
prepare
sync-wiki
lint-static
query
grep
links
refs
refs-index
tasks
memory
shell
list-models
show-model
add-model
switch-model
delete-model
validate-model
```

## CLI / shell / LLM boundary

Preserve the deterministic boundary between batch mode, shell mode, and future LLM modes.

Required principles:

- `kb ...` is batch mode.
- `kb>` is interactive shell mode.
- Core knowledge-base commands must map one-to-one across batch and shell modes.
- `kb>` is not an LLM chat interface.
- Do not interpret free-form natural language inside `kb>` as an LLM request.
- Do not call any LLM API implicitly from `kb>`.
- Future LLM behavior must be introduced only through a deliberately designed explicit interface; do not advertise placeholder LLM command names before that design exists.
- Do not write a separate business-logic implementation for shell mode; shell mode should preserve batch-mode command semantics. The current implementation delegates only known structured shell commands back to the same `kb` binary; unknown input must return safely.

See `docs/cli-shell-principle.md` and `docs/shell.md`.

## Do not invent undocumented features

Do not claim or implement large new systems unless explicitly requested. In particular, do not add:

```text
full RAG
vector database
embedding pipeline
SQLite/Postgres storage
network server
web UI
full JSON agent API
background daemon
cloud sync
```

`prepare` is implemented only as a review-first planning command. It generates `processing/prepare_queue.json` and `processing/proposals/prepare_plan_*.md`; it does not call an LLM or edit `wiki/`. `query` is implemented as deterministic local keyword search only; `lint-static` is implemented as a deterministic local health check.

## Safety rules for init/ingest/bootstrap

- `init` must not move files.
- `init` may create missing generated rule files.
- `init --force` may overwrite generated rule files, but it must not rewrite user source materials.
- `ingest` must not default to destructive behavior.
- `bootstrap` must be safe by default.
- `--copy` is the safe path.
- `--move` is allowed only when explicitly provided.
- `--dry-run` should remain available for inspection.
- Recursive ingest must skip generated/project directories:

```text
raw
wiki
rules
processing
references
outputs
logs
.git
.obsidian
target
node_modules
```

## File classification boundary

Keep classification simple and extension-based:

```text
PDF -> raw/papers
notes/documents -> raw/notes
images -> raw/images
data files -> raw/datasets
archives -> raw/archives
code-like files -> raw/repos
other -> raw/other
```

Do not add content classification, OCR, PDF parsing beyond metadata, or LLM-based sorting in this version line.

## Manifest rules

- `processing/manifest.json` is generated metadata; it describes `raw/` but must not alter raw files.
- The manifest should remain plain JSON and human-inspectable.
- Use stable relative paths with `/` separators.
- Use content hashes to detect changed raw files.
- Do not treat the manifest as a database or introduce SQLite in this version line.

## Documentation rule

README and docs must describe only implemented behavior. Do not list future commands as available unless they are actually implemented.

## Versioning rule

Prepare planning work starts in v0.4.x. Small compatible improvements should increment the patch version:

```text
0.4.0 -> 0.4.1 -> 0.4.2 -> 0.4.3 -> 0.4.4 -> 0.4.5 -> 0.4.6 -> 0.4.7 -> 0.4.8 -> 0.5.0 -> 0.5.1 -> 0.5.2 -> 0.5.3 -> 0.5.7
```

The next major implementation phase after v0.5.10 should refine query ergonomics carefully, then consider saved queries or semantic LLM linting only after the deterministic path is stable.


## Command classification rule

Before adding or changing any command, classify it according to `docs/command-classification.md`:

1. Deterministic core command
2. Deterministic search / inspection command
3. Text conversion / source processing command
4. Task preparation command
5. Explicit LLM / agent command

Do not allow commands to silently cross category boundaries. In particular:

- Deterministic commands must not call LLM APIs.
- Search and inspection commands should be Rust-native whenever practical and must not require external `grep` or `rg`.
- Text conversion commands such as `extract-text` may reserve future agent paths, but OCR/LLM/agent modes must be explicit and opt-in.
- `prepare` generates reviewable task materials; it does not secretly execute LLM work.

## Extract-text future boundary

`kb extract-text` is currently deterministic and best-effort. It may become the future entry point for a PDF Text Conversion Agent, but default behavior must remain simple, non-OCR, and non-LLM.

Do not hide OCR, layout repair, semantic cleanup, summarization, or LLM calls inside `kb extract-text`. Future behavior must be explicit, such as `--ocr`, `--llm-clean`, `--agent`, or `--pipeline full`, after those modes are deliberately designed.


## Deferred LLM/agent skill rule

For commands such as `extract-text`, `refs`, `grep`, and future source-processing commands, keep the default behavior small and deterministic.

When the remaining work requires semantic judgment, layout repair, citation reconciliation, summarization, or wiki drafting, mark it as future explicit LLM/agent skill work instead of implementing hidden LLM behavior.

Use `docs/llm-agent-skills.md` as the place to document those future skill boundaries. In particular:

- `extract-text` may feed a future PDF Text Conversion Agent.
- `refs` may feed a future Reference Reconciliation Agent or Citation Graph Building Agent.
- `prepare` may feed a future Wiki Drafting / Concept Synthesis Agent.

Do not silently convert deterministic commands into agent commands.


## Deferred task handoff rule

When adding or modifying commands, keep each command focused on one deterministic ability. If a task requires OCR, semantic repair, reference reconciliation, citation graph reasoning, concept synthesis, or Wiki drafting, do not hide that work inside the deterministic command.

Instead, make the command report unresolved work in a form that can feed `kb tasks` or a similar handoff report. The task information should include target agent, goal, requirements, files, evidence, source command, and priority.

Do not make `kb tasks` execute LLM work. It is a handoff generator, not an agent runner.


## Completed task memory rule

When implementing or guiding future LLM/agent work, do not rely on chat history as the only record of completed tasks. Use `kb memory` or the `LLM/memory/` convention to record task id, summary, source task report, files touched, and follow-up issues. This memory is project-local audit material, not hidden model state.

## LLM command usage rule

When acting as a future LLM maintainer or when guiding Claude Code, use `docs/llm-command-guide.md` as the practical command playbook.

The LLM should not manually guess file lists, broken links, keyword occurrences, reference hints, missing text conversions, or completed task history when a deterministic command can provide that evidence.

Preferred pattern:

```text
kb command -> evidence/file list/task handoff -> LLM semantic work -> git diff review -> kb check -> kb memory
```

Use deterministic commands as scouts:

- `kb query` for topic-level candidate pages.
- `kb grep` for line-level evidence.
- `kb extract-text` for direct text extraction before summarizing PDFs.
- `kb refs` for reference hints before reconciliation.
- `kb tasks` for deferred work lists.
- `kb memory` for completed-task audit records.

Do not turn these commands into hidden LLM calls. Their purpose is to reduce unnecessary LLM work and make the remaining LLM work more focused.


## LLM role hierarchy

Treat the LLM using `kb-cli` commands as the top-level Manager LLM.

This Manager LLM should use deterministic commands first, then delegate bounded semantic work to lower-level Worker LLMs or future agent skills. Do not collapse these roles into one unclear agent.

When creating or updating task handoff documents, include goal, requirements, exact file list, evidence, non-goals, expected output, and memory-recording requirements.

Worker LLMs must not redefine project scope, invent file lists, or bypass deterministic command evidence.

See `docs/llm-hierarchy.md`.


## `kb links` implementation guardrail

`kb links` must remain a deterministic WikiLink scanner. Do not make it rewrite pages, create missing pages, rename files, call an LLM, or infer semantic intent. It should return unresolved and ambiguous cases as task hints for future Manager/Worker LLM workflows.


## `kb refs-index` development rule

When improving `kb refs-index`, keep it conservative. It may improve deterministic matching and reporting, but it must not silently call LLMs, query online DOI services, rewrite Wiki pages, or declare uncertain title/author matches as final. Human review remains the final guarantee for bibliographic identity.


## Third-party visualization guidance

When updating third-party skill / visualization support, preserve the relationship-certainty protocol in `docs/third-party-skills/`:

- confirmed relations use solid edges;
- candidate or ambiguous relations use dashed edges;
- missing or unresolved references use hollow nodes;
- uncertain relations must preserve evidence and human-review markers.

Do not let a graph export or visualization step silently convert uncertain relations into confirmed ones.


## v0.5.12 graph export rule

Use `kb refs-graph` only as a deterministic export step. Do not ask it to infer scientific idea relations, confirm candidate bibliographic identities, query online DOI services, or rewrite wiki pages.


## v0.5.12 keyword/topic rule

Use `kb keywords` only as a deterministic evidence scanner for keyword/topic co-occurrence. Do not make it infer scientific idea relations, merge concepts, rewrite wiki pages, or call an LLM. Shared keywords are candidates, not conclusions.


## Health command rule

`kb health` is a deterministic dashboard command for Manager LLM sessions. It may summarize missing evidence, reports, and deferred work, but it must not call an LLM, fix files automatically, or certify uncertain bibliographic identities.


## Strict shell safety rule

When editing `kb shell`, keep it as a strict whitelist command shell.

- Do not interpret unknown input.
- Do not add an LLM fallback to the final `else` branch.
- Do not execute arbitrary shell commands from unknown input.
- Do not guess Manager LLM intent from free-form text.
- Unknown input should return safely with a short no-action message.

The shell is valuable because it is safe and boring. Its job is to execute explicit structured commands, not to be clever.


## Topic-specific relationship overlays

Global bibliographic index relations stay under `processing/refs/`. Topic-specific causal, method, evidence, idea, and importance relations belong under `topics/<topic>/`.

Do not ask a Worker LLM to store topic-level interpretation in `processing/refs/`. If a topic-specific relation is uncertain, create or update a bounded task under `topics/<topic>/tasks/` or `LLM/tasks/`, and record accepted decisions under `topics/<topic>/memory/` or `LLM/memory/`.

## v0.6.3 topic schema guardrail

Before adding topic-level relationship features, read:

```text
docs/topic-relation-schema.md
docs/literature-importance-schema.md
docs/topic-v2-roadmap.md
```

Do not implement a universal causal graph. Do not auto-confirm causal, contradiction, improvement, or core-literature relations. Topic relations under `topics/<topic>/` must remain reviewable records with evidence and status fields.


## Static viewer boundary

`kb view` is a static display command. It may render existing Markdown/JSON outputs into `outputs/html/index.html`, but it must not call an LLM, execute shell commands from the browser, or modify source/wiki files. The `kb-view>` box inside the generated HTML is display navigation only.
