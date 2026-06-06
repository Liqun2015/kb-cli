# `kb review`

`kb review` is reserved for the future literature-review writing workflow.

Current behavior is intentionally conservative: the command only prints a placeholder message and writes no files.

The intended future role is to start a controlled review-writing workflow after the LLM Wiki has enough reviewed topic narrative, paper profiles, and human-accepted relationships.

Current command:

```bash
kb --wiki <workspace> review --dry-run
```

For now, use:

```bash
kb --wiki <workspace> view
kb --wiki <workspace> check
kb --wiki <workspace> check --relations
```
