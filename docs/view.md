# `kb view`

`kb view` generates the reader-facing LLM Wiki knowledge portal.

It is for users who want to browse what the knowledge base says, not for developers or Manager LLMs checking the worksite internals.

## Purpose

Use `kb view` to open the knowledge layer from a reader's perspective:

- research directions;
- topic branches;
- core knowledge points;
- representative papers;
- paper cards and Wiki pages;
- WikiLinks;
- images referenced by the Wiki;
- recommended reading paths as they are written into the Wiki.

It should not display low-level system details such as JSON, task state, agent handoff files, or workflow status. Those belong to `kb check`.

## Command

```bash
kb --wiki <workspace> view
```

Preview only:

```bash
kb --wiki <workspace> view --dry-run
```

Generate without opening the browser:

```bash
kb --wiki <workspace> view --no-open
```

## Output

```text
interfaces/html/browse.html
```

The generated HTML is an interface artifact. The source of truth remains the Markdown files under `wiki/`.

## Boundary with `kb check`

```text
kb check   inspect the system worksite, task scene, refs graph, relationship candidates, JSON, and handoff state
kb view    browse the user-facing knowledge portal
kb review  reserved for the future literature-review writing workflow
```
