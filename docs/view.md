# kb view

Current version: `v0.6.4.3`

`kb view` generates a static local HTML viewer for the current LLM Wiki.

It is a display layer, not an execution layer.

```bash
kb view
kb view --no-open
kb view --dry-run
kb view --output-dir outputs/html
```

Default output:

```text
outputs/html/index.html
```

`kb view` opens the generated file with the system default browser by default. Use `kb view --no-open` when you only want to refresh the HTML file without opening a browser. Neither mode starts a server or grants the page permission to execute local commands.

## What it displays

The first version scans and renders the latest available artifacts from:

```text
wiki/
processing/refs/refs_index_*.md
processing/refs/refs_graph_*.json / *.mmd / *.dot
processing/keywords/keywords_*.md
outputs/reports/health_*.md
LLM/tasks/llm_tasks_*.md
LLM/memory/completed_tasks.md
topics/<topic>/
```

The page has:

```text
left sidebar      = Wiki/result navigator
right main area   = tabbed content display
kb-view> box      = display-only page navigation
```

## Display-only command box

The sidebar input box is intentionally not an LLM chat window and not a bridge to the local shell.

Supported display commands include:

```text
help
open overview
open refs-index
open refs-graph
open keywords
open health
open tasks
open memory
open topics
topic <name>
find <keyword>
clear
```

Unknown input returns safely in the page UI.

The command box must not:

```text
execute local kb commands
execute shell commands
call LLM APIs
modify files
generate tasks
write memory
edit wiki pages
```

## Why static first?

`kb view` is intentionally conservative. A static HTML file is easy to review, easy to archive, and safe to open.

A future `kb browser --serve` may provide a local backend bridge to deterministic `kb` commands, but that is not part of `kb view`.

## Relationship to Markdown

Markdown and JSON remain the source of truth. HTML is only a generated viewing layer.

Do not manually edit `outputs/html/index.html` as project state. Re-run `kb view` instead.


## Sidebar command panel layout

Starting in `v0.6.4.2`, the left sidebar keeps the `kb-view>` display command panel anchored at the bottom. The command log and input area are intentionally taller than ordinary navigation controls so the panel feels like a small review console rather than a chat box. It remains display-only and cannot execute local `kb` commands.
