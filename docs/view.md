# `kb view`

`kb view` generates the reader-facing **research issue browser**.

It is for users who want to understand the knowledge base through scientific questions and the way papers inherit, extend, verify, improve, or compete with one another. It is not for maintainers checking workspace internals.

## What the page should show

`interfaces/html/browse.html` should focus on:

- research issues and topic branches;
- the central question for each topic when available;
- representative papers under each issue;
- paper-to-paper inheritance and evolution links;
- relation labels such as method inheritance, extension, improvement, support, contradiction, and background;
- recommended reading paths derived from topic literature and relation records;
- paper cards and paper-linked notes when useful for reading.

It should not explain the detailed LLM Wiki workflow, directory structure, JSON artifacts, task state, agent handoff, or Manager/Worker usage. Those belong to `kb check`.

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

## Boundary with `kb check`

```text
kb view   browse research issues and literature inheritance/evolution relations
kb check  inspect system/workflow state, workspace structure, JSON, relation candidates, task status, and agent handoff
kb review reserved for the future literature-review writing workflow
```
