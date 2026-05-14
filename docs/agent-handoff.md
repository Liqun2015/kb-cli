# Generic Agent Handoff

`kb-cli v0.7.13` treats handoff as a generic external-agent protocol. Claude Code, OpenCode, and OpenClaw are adapters, not the core protocol.

## Generic command

```bash
kb handoff
kb handoff --topic <topic>
kb topic handoff <topic>
```

Generic files include:

```text
AGENTS.md
LLM/handoff/index.md
LLM/handoff/protocol.md
LLM/handoff/manager.md
LLM/handoff/worker.md
LLM/handoff/task_schema.md
LLM/handoff/safety.md
LLM/handoff/handoff.json
topics/<topic>/handoff/AGENTS.md
topics/<topic>/handoff/handoff.json
```

## Agent adapters

```bash
kb handoff --agent claude-code
kb handoff --agent opencode
kb handoff --agent openclaw
kb handoff --all-agents
kb topic handoff <topic> --agent claude-code
```

Adapters add tool-specific entry notes without changing the evidence and review rules.

## Boundary

- `kb-cli` remains deterministic.
- External agents process one bounded task at a time.
- High-value academic judgments require human review.
- `raw/` is never modified by agents.
