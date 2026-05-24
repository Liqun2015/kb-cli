# kb shell

Current version: `v0.7.42`

`kb shell` starts a deterministic interactive command shell:

```bash
kb shell
```

With an explicit knowledge base path:

```bash
kb --wiki ./quantum shell
```

The prompt is:

```text
kb>
```

## Core rule

`kb>` is not an LLM chat interface.

It is a repeated command interface for humans, Manager LLM sessions, and local maintenance workflows. Known knowledge-base commands entered in shell mode should keep the same meaning as their batch-mode equivalents. Unknown input is rejected safely.

```text
kb --wiki ./quantum query thermal cloak
```

maps to:

```text
kb> use ./quantum
kb> query thermal cloak
```

## Session commands

These commands are handled by the shell itself:

```text
use <path>   Set the current knowledge base path
pwd          Show the current knowledge base path
clear        Clear the terminal screen
help         Show shell help
exit         Leave shell mode
```

## Strict whitelist command delegation

Only known `kb` batch commands are delegated to the same `kb` binary with the current `--wiki` value.

Unknown input is a safe no-op. The shell must not guess intent, interpret natural language, or pass arbitrary text into an LLM-like fallback.

In implementation terms, the final `else` branch of shell parsing must be a safe return:

```text
if session command:
    handle it
else if first token is a known kb command:
    delegate to batch mode
else:
    return safely without interpretation
```

Examples of known delegated commands:

```text
kb> status --unprocessed
kb> query thermal cloak
kb> grep DOI --path processing/text
kb> extract-text --dry-run
kb> refs --path processing/text
kb> refs-index --dry-run
kb> refs-graph --json --dry-run
kb> keywords thermal cloak --dry-run
kb> links --unresolved
kb> health --dry-run
kb> tasks --dry-run
```

## Manager LLM usage

`kb shell` is useful for a Manager LLM because it can repeatedly scout the knowledge base with deterministic commands before assigning bounded work to Worker LLMs or human reviewers.

A typical Manager LLM shell session:

```text
kb> use ./quantum
kb> health --dry-run
kb> refs-index --dry-run
kb> refs-graph --json --dry-run
kb> keywords thermal cloak --dry-run
kb> tasks --dry-run
```

The Manager LLM should not guess file lists, relation candidates, or unresolved issues. It should use shell commands to collect evidence, then create or review `LLM/tasks/` handoff files.

## Unknown input behavior

The following should not trigger any task, model call, file rewrite, or natural-language interpretation:

```text
kb> 帮我总结这些文献
kb> please organize the wiki
kb> find the important papers and rewrite them
```

Expected behavior:

```text
Unknown command. No action taken. Type `help` for available commands.
```

## Non-goals

`kb shell` does not provide:

```text
LLM chat
implicit natural-language interpretation
RAG answers
semantic retrieval
autonomous wiki edits
background agent execution
network services
```

Future LLM behavior must be introduced through explicit commands or explicit task handoff mechanisms, not by changing the meaning of ordinary `kb>` input.
