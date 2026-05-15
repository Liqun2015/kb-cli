# Generic Agent Handoff

`kb-cli v0.7.19` treats handoff as a generic external-agent protocol. Claude Code, OpenCode, OpenClaw, and Codex are adapters, not the core protocol.

## Human and Manager entry points

```text
obsidian_main.md              # human-first Obsidian homepage
AGENTS.md                     # generic agent root entry
CLAUDE.md                     # Claude Code adapter root entry, when generated
LLM/handoff/current.md        # shared Manager routing file
topics/<topic>/index.md       # topic homepage
topics/<topic>/tasks/index.md # topic task queue
```

Managers should not infer tasks by scanning the repository tree. They should follow:

```text
root entry file
→ LLM/handoff/current.md
→ topics/<topic>/index.md
→ topics/<topic>/handoff/AGENTS.md
→ topics/<topic>/tasks/index.md
→ one task item under topics/<topic>/tasks/items/
```

## Generic command

```bash
kb handoff
kb handoff --topic <topic>
kb topic handoff <topic>
```

Generic files include:

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
topics/<topic>/handoff/handoff.json
```

## Agent adapters

```bash
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

Adapters add tool-specific entry notes without changing the evidence and review rules.

## Boundary

- `kb-cli` remains deterministic.
- External agents process one bounded task at a time.
- High-value academic judgments require human review.
- `raw/` is never modified by agents.


## Interface Directory Boundary

`interfaces/` is the single directory for non-knowledge, regenerable artifacts. Agents may inspect `interfaces/html/`, `interfaces/reports/`, and `interfaces/logs/`, but must write accepted decisions back to durable Markdown/JSON/TOML files outside `interfaces/`.
