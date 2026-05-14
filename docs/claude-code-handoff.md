# Claude Code Handoff Adapter

As of `kb-cli v0.7.13`, Claude Code support is an adapter on top of the generic external-agent handoff protocol.

Start with the generic files:

```bash
kb handoff --topic <topic>
```

Then generate the Claude Code adapter when needed:

```bash
kb handoff --topic <topic> --agent claude-code
# or
kb topic handoff <topic> --agent claude-code
```

Claude-specific files include:

```text
CLAUDE.md
LLM/handoff/adapters/claude-code.md
topics/<topic>/handoff/CLAUDE.md
topics/<topic>/handoff/adapters/claude-code.md
```

Claude Code should read `AGENTS.md` first. `CLAUDE.md` only adds Claude Code-specific entry habits.
