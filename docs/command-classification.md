# Command Classification Principle

`kb-cli` should grow by adding small, reviewable commands with clear categories. Do not add commands opportunistically without first deciding what kind of command they are.

The goal is to keep the system maintainable, reviewable, rollback-friendly, and LLM-efficient. Deterministic local tooling should do the mechanical work first. LLMs should be saved for the parts that genuinely require semantic judgment, synthesis, repair, or writing.

## The five command categories

Every `kb-cli` command must belong to one of these categories.

### 1. Deterministic core commands

These commands maintain the knowledge-base structure and metadata.

Examples:

```text
kb init
kb ingest
kb status
kb sync-wiki
kb lint-static
```

Rules:

- Must not call an LLM.
- Must not call OCR.
- Must not interpret free-form natural language.
- Should be safe, predictable, and inspectable.
- Should support `--dry-run` or `--preview` when they modify files or reports.

### 2. Deterministic search / inspection commands

These commands find, match, count, and inspect local files.

Examples:

```text
kb query
kb grep
kb links
future: kb backlinks
future: kb orphans
future: kb sources
future: kb stats
future: kb health
```

Rules:

- Prefer Rust-native implementations.
- Do not require external `grep`, `rg`, `find`, shell scripts, or platform-specific tools.
- Do not call an LLM.
- Should report exactly what was scanned and what matched.
- Should support machine-readable output where useful, typically `--json`.

### 3. Text conversion / source processing commands

These commands convert source materials into text or structured intermediate files that later commands can inspect.

Examples:

```text
kb extract-text
kb extract-sections
kb extract-metadata
kb refs
future: kb extract-images
future: kb extract-tables
```

Rules:

- Default mode must be deterministic and best-effort.
- Failures should be reported clearly instead of hidden behind automatic repair.
- OCR, LLM cleanup, layout repair, and semantic reconstruction must be explicit future modes, not default behavior.
- These commands may become agent entry points later, but only through explicit flags or subcommands.

For example, `kb extract-text` is a simple text extraction command, and `kb extract-sections` is a deterministic section slicing command for `Introduction` and `References`. OCR, multimodal reading, and LLM cleanup must remain explicit Worker tasks, not implicit command behavior. `kb refs` is also in this category: it scans extracted text for reference hints, but it does not build a semantic citation graph or call an LLM.

### 4. Task preparation commands

These commands turn deterministic findings into reviewable work items for a human or future LLM/agent.

Example:

```text
kb topic tasks thermal
kb handoff --all-agents
```

Future examples:

```text
kb tasks --missing-sources
kb tasks --broken-links
kb tasks --orphans
kb topic tasks thermal --needs-llm
```

Rules:

- Should generate reviewable task materials.
- Should not secretly execute the task.
- Should not call an LLM by default.
- Should make it clear what evidence triggered each task.

### 5. Explicit LLM / agent commands

These commands are reserved for future work.

Possible future forms:

```text
kb ask
kb chat
kb agent run
kb extract-text --agent
kb extract-text --llm-clean
```

Rules:

- Any command that calls an LLM must make that fact explicit in its name, flag, documentation, or prompt.
- LLM behavior must not be hidden inside deterministic commands.
- LLM output must be reviewable before it modifies long-term wiki content.
- Prefer generating proposals over direct writes.

## Boundary rules

Commands must not silently cross category boundaries.

Especially:

- Deterministic core commands must not call LLM APIs.
- Search commands must not become semantic answer generators.
- Shell mode must not become natural-language chat.
- Extraction commands may reserve future agent paths, but OCR/LLM must be explicitly enabled.
- `tasks`, `topic tasks`, and `handoff` should generate task materials, not secretly execute them.


## Deferred skill markers

When a deterministic command reaches the edge of what it can do reliably, it should mark the remaining work clearly instead of silently becoming an agent.

Examples:

```text
extract-text can say: text layer not found; scanned/OCR workflow may be needed.
refs can say: DOI/reference hints found; semantic reconciliation is not performed.
grep can say: pattern matches found; interpretation is left to the user or a future explicit agent.
```

These deferred items are future LLM/agent skill inputs. They should be documented in `docs/llm-agent-skills.md` before an agent mode is implemented.

## Development checklist for new commands

Before adding or changing a command, answer these questions in the implementation note or PR description:

1. Which command category does it belong to?
2. Does it read files, write files, or both?
3. Does it need `--dry-run` / `--preview`?
4. Does it need `--json`?
5. Does it call external tools?
6. Does it call an LLM or OCR?
7. If it can become an agent later, what is the explicit future activation path?
8. What should happen when the command fails?

## Core principle

Use deterministic Rust-native tooling first. Use LLMs only where semantic judgment is necessary.

```text
file traversal / grep / regex / front matter / links / manifest / reports -> Rust
OCR -> future explicit OCR stage
layout repair / synthesis / summaries / concept merging / wiki drafting -> future explicit LLM or agent stage
```


## Task handoff contract

Every new command should have one narrow deterministic ability. If the command encounters work outside that ability, it should return or contribute to a deferred task list instead of silently crossing boundaries.

A deferred task should include:

- target agent or human role
- goal
- requirements
- files
- evidence
- source command
- priority

`kb tasks` is the shared deterministic handoff command for consolidating this information into `LLM/tasks/index.md`, bounded Worker task files under `LLM/tasks/items/`, and timestamped handoff snapshots under `LLM/tasks/`. `kb memory` is the deterministic completion-memory command for recording finished work under `LLM/memory/`. The `LLM/` directory is intentionally a future agent workbench: deterministic commands scout the terrain first, unresolved hard tasks are handed upward to a human manager or explicit LLM/agent skill, and completed outcomes are recorded as auditable project memory.

Examples:

- `extract-text` may identify PDFs that need OCR or LLM-assisted layout repair.
- `extract-sections` may identify missing `Introduction` / `References` sections that need OCR or LLM-assisted section recovery.
- `refs` may identify reference hints that need semantic reconciliation.
- `lint-static` may identify broken links or missing source front matter that need Wiki maintenance.

The handoff list is not an execution plan for the CLI. It is an auditable input for an upper-level manager, human reviewer, or future explicit LLM/agent skill.

## LLM operator documentation requirement

As commands become more numerous, clear documentation is part of the command contract. Future LLM maintainers should not be expected to infer command purpose from source code alone.

When a command is added or substantially changed, update the relevant command document and, if it affects LLM workflows, update `docs/llm-command-guide.md`.

The command guide should explain:

- what evidence the command can produce;
- what files it reads and writes;
- what work it intentionally does not do;
- what deferred task or LLM/agent skill should handle the remaining work;
- how the command fits into the `LLM/tasks/` and `LLM/memory/` workflow.


## LLM hierarchy

Command classification is separate from LLM role hierarchy.

The top-level Manager LLM is allowed to use deterministic `kb` commands as tools. Its job is to scout the project, gather evidence, generate or inspect `LLM/tasks/`, delegate bounded work, run checks, and record completion memory.

Lower-level Worker LLMs should execute specific deferred tasks. They should not redefine command categories, expand scope without approval, or decide the overall workflow.

See `docs/llm-hierarchy.md` for the full hierarchy.


## v0.5.10: `kb refs-index` classification

`kb refs-index` is a deterministic search / inspection command for the first relationship layer of LLM Wiki: bibliographic index relations. It may produce high-confidence DOI-exact matches, but candidate, ambiguous, and missing relations must remain reviewable. Human review is the final guarantee for uncertain bibliographic identity decisions.


## Third-party skill guidance

Third-party graph and visualization guidance belongs under `docs/third-party-skills/`. These documents are not runtime commands; they define how external tools and skills should interpret relationship certainty and review status.

## Command map and lifecycle references

The canonical command overview is maintained in:

```text
docs/commands.md
```

Task status and handoff rules are maintained in:

```text
docs/task-lifecycle.md
```

When adding a new command, update `docs/commands.md` with its ability, input area, output area, LLM boundary, and deferred-work rule. If the command creates or consumes handoff tasks, also update `docs/task-lifecycle.md` when status semantics change.


## v0.5.12: `kb refs-graph` classification

`kb refs-graph` is a deterministic text/reference processing command. It exports graph data from existing `refs-index` reports.

- It may read `processing/refs/refs_index_*.md`.
- It may write `processing/refs/refs_graph_*.json`, `.mmd`, or `.dot`.
- It must preserve `status`, `evidence`, `visual`, and `needs_human_review` fields.
- It must not call LLM APIs, perform online DOI lookup, confirm candidate matches, or rewrite wiki pages.
- Uncertain graph edges remain deferred human-review tasks.


## v0.5.12: `kb keywords` classification

`kb keywords` is a deterministic text/reference processing command. It detects keyword/topic co-occurrence candidates among extracted text files and writes reviewable evidence to `processing/keywords/`.

It must not call an LLM, infer scientific idea relations, merge topics, rewrite wiki pages, or treat shared keywords as confirmed relationships.

When keyword/topic candidates are found, unresolved semantic interpretation should be handed to a Manager LLM and bounded Worker task.


## Health / reporting commands

`kb health` is a deterministic Manager LLM dashboard command. It summarizes relationship-network completeness, review backlog, and missing reports. It must not call an LLM, repair files, or claim scientific relationships. When hard work remains, it should emit bounded deferred task hints and point the user to `kb tasks`.


## v0.5.13: `kb health` classification

`kb health` belongs to the health / reporting category. It is deterministic and Manager-LLM-facing. It summarizes relationship-network completeness, missing reports, review backlog, and deferred task hints. It must not call an LLM, repair files, confirm bibliographic identity, or decide scientific idea relations.

## v0.6.0.1: `kb shell` classification

`kb shell` belongs to the interactive shell category.

- It is deterministic.
- It is not an LLM chat interface.
- It handles only session commands directly: `use`, `pwd`, `clear`, `help`, and `exit`.
- Known knowledge-base commands should map to batch-mode commands with the current `--lib`.
- Unknown input must return safely without interpretation, LLM calls, shell escapes, or arbitrary execution.
- It is suitable for Manager LLM sessions that need to repeatedly run structured commands before assigning bounded work to Worker LLMs.


## Topic overlay commands

Future `kb topic ...` commands should be classified as deterministic scaffolding commands unless they explicitly opt into LLM/agent behavior.

Their job is to create, inspect, and export `topics/<topic>/` relationship overlays. They must not silently infer causal, method, evidence, or scientific idea relations. When interpretation is needed, they should generate bounded tasks for the Manager LLM to assign to Worker LLMs or human reviewers.

The global bibliographic relation layer remains under `processing/refs/`; topic-specific interpretive records belong under `topics/<topic>/`.


## Static viewer boundary

`kb view` is a static display command. It may render existing Markdown/JSON outputs into `interfaces/html/index.html`, but it must not call an LLM, execute shell commands from the browser, or modify source/wiki files. The **About launching LLM** panel only provides copyable external-agent commands/prompts; the `kb-view>` box is display navigation only.
