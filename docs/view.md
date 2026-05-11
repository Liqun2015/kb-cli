# kb view

Current version: `v0.7.5`

`kb view` generates static local HTML viewers for the current LLM Wiki.

It is a display layer, not an execution layer.

## Regular dashboard

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

## Relationship graph review mode

Starting in `v0.7.5`, relationship graph review is part of the existing `kb view` command:

```bash
kb view --relations
kb view --relations --no-open
kb view --relations --topic thermal-metamaterials
kb view --relations --data-only
```

This intentionally does **not** add a separate `kb view-relations` command.

Default outputs:

```text
outputs/html/relationship_viewer.html
outputs/html/relationship_data.json
```

`relationship_viewer.html` is a static review page for inspecting paper-level bibliographic index relations and topic-local academic viewpoint / importance candidates before LLM execution.

`relationship_data.json` is the machine-readable data source for the page and for future third-party graph tools.

### `--topic <topic>`

```bash
kb view --relations --topic thermal-metamaterials
```

The generated page keeps global relationship data but marks the requested topic as the default focus when that topic workspace exists under:

```text
topics/thermal-metamaterials/
```

If the topic does not exist, the command still generates global relationship data and records a warning in the JSON/page.

### `--data-only`

```bash
kb view --relations --data-only
```

Generates only:

```text
outputs/html/relationship_data.json
```

It does not generate HTML and does not open a browser.

## Relationship visual protocol

The relationship viewer uses conservative visual semantics:

```text
confirmed / accepted       solid edge
candidate / needs review   dashed edge
ambiguous                  dashed edge with ambiguous status
missing / unresolved       dotted edge
```

The page must not make candidate relations look confirmed.

The page must not invent semantic academic claims. It only visualizes existing deterministic relation data, topic-local candidates, and review tasks.

## What the regular dashboard displays

The regular `outputs/html/index.html` scans and renders the latest available artifacts from:

```text
wiki/
processing/refs/refs_index_*.md
processing/refs/refs_graph_*.json / *.mmd / *.dot
processing/keywords/keywords_*.md
outputs/reports/health_*.md
LLM/tasks/index.md and LLM/tasks/llm_tasks_*.md
LLM/memory/completed_tasks.md
topics/<topic>/
```

The page has:

```text
left sidebar      = Wiki/result navigator
right main area   = tabbed content display
kb-view> box      = display-only page navigation
```

The regular dashboard sidebar includes a link to:

```text
relationship_viewer.html
```

The relationship graph page includes a return link to:

```text
index.html
```

## Display-only command box

The sidebar input box in the regular dashboard is intentionally not an LLM chat window and not a bridge to the local shell.

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

## Relationship to Markdown and JSON

Markdown and JSON remain the source of truth. HTML is only a generated viewing layer.

Do not manually edit generated HTML as project state. Re-run `kb view` or `kb view --relations` instead.

## Windows browser opening note

On Windows, `kb view` opens generated HTML through `rundll32.exe url.dll,FileProtocolHandler` instead of `cmd /C start`. This is more robust on systems where `start` reports access denied for local HTML files. Use `--no-open` to generate files without opening a browser.
