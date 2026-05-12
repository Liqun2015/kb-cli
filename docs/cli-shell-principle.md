# CLI / Shell Principle

Current version: `v0.7.7`

This document records the boundary between batch mode, interactive shell mode, and future LLM-assisted modes.

## Core principle

`kb` and `kb>` must share the same deterministic command semantics.

```text
kb ...    = batch mode / one-shot command execution
kb> ...   = interactive command shell / repeated deterministic command execution
LLM       = future explicit layer, not hidden shell behavior
```

The interactive shell is a command shell, not a natural-language chat interface.

## One-to-one mapping requirement

Core knowledge-base commands should have a clear mapping between batch mode and shell mode.

For example:

```bash
kb --kb-path ./quantum query thermal cloak
```

maps to shell mode like:

```text
kb> use ./quantum
kb> query thermal cloak
```

Likewise, deterministic commands should keep the same meaning across modes:

| Batch mode | Shell mode |
|---|---|
| `kb init` | `kb> init` |
| `kb ingest` | `kb> ingest` |
| `kb prepare` | `kb> prepare` |
| `kb build-wiki` | `kb> build-wiki` |
| `kb sync-wiki` | `kb> sync-wiki` |
| `kb lint-static` | `kb> lint-static` |
| `kb query thermal cloak` | `kb> query thermal cloak` |

Shell-only commands may exist for session management, such as:

```text
kb> use ./quantum
kb> pwd
kb> history
kb> clear
kb> help
kb> exit
```

These commands should manage the shell session. They should not redefine the knowledge-base workflow.

## Not an LLM chat interface

`kb>` must not silently interpret free-form natural language as an LLM request.

Do not make free-form natural-language requests work implicitly in the deterministic shell.

If LLM-assisted behavior is added later, it must use a deliberately designed explicit interface. Do not reserve or advertise placeholder LLM command names before that design exists.

## Handler sharing requirement

Do not implement two separate business-logic stacks for batch mode and shell mode.

Preferred architecture:

```text
argument parsing / shell parsing
        ↓
shared command dispatch
        ↓
shared command handlers
        ↓
filesystem changes / reports / output
```

This keeps behavior consistent and testable.

## Current shell status

`kb shell` is the deterministic shell entry point.

The shell handles only session commands directly (`use`, `pwd`, `clear`, `help`, `exit`). Known knowledge-base commands are delegated back to the same batch-mode `kb` executable with the current `--kb-path` value. This preserves one-to-one command semantics without creating a separate business-logic stack.

Unknown input is a safe no-op. The final `else` branch in shell parsing must return safely; it must not become a natural-language interpretation step, LLM fallback, model call, agent trigger, shell escape, or arbitrary command executor.

`kb repl` is retained only as a compatibility alias. Documentation should prefer `kb shell`.

## Non-goals for the shell layer

Do not add the following implicitly to `kb>`:

```text
LLM chat
agent swarms
RAG answers
semantic retrieval
web access
autonomous wiki edits
background indexing
network servers
```

The shell should remain a predictable local command interface. Its safety value comes from rejecting unknown input rather than trying to be clever.
