# Static Wiki Lint

`kb lint-static` performs local, deterministic health checks over Markdown files under `wiki/`.

It does **not** call an LLM, does **not** modify `raw/`, and does **not** rewrite `wiki/`. By default it writes a report under:

```text
outputs/reports/lint_static_<timestamp>.md
```

## Commands

```bash
# Write a Markdown lint report
kb --kb-path /path/to/kb lint-static

# Preview without writing a report
kb --kb-path /path/to/kb lint-static --dry-run

# Machine-readable summary
kb --kb-path /path/to/kb lint-static --json

# CI/script mode: fail when issues are found
kb --kb-path /path/to/kb lint-static --strict
```

## Checks

The current static lint pass checks:

1. **Broken WikiLinks**  
   `[[Page]]` points to a Markdown page that cannot be resolved.

2. **Orphan pages**  
   A non-index wiki page has no incoming links from other wiki pages.

3. **Missing source front matter**  
   Source-derived pages under folders such as `wiki/papers/`, `wiki/notes/`, `wiki/concepts/`, `wiki/topics/`, and `wiki/methods/` should include YAML front matter with `source_ids` or `source_files`.

4. **Empty or near-empty pages**  
   A non-index page has almost no Markdown body content.

5. **Duplicate titles**  
   Multiple pages use the same first-level heading or inferred title.

## Intended workflow

```text
kb compile --new
        ↓
AI/human writes source-linked wiki Markdown
        ↓
kb sync-wiki
        ↓
kb lint-static
        ↓
fix broken links / missing source front matter
        ↓
git diff → git commit
```

## Boundary

`lint-static` is intentionally shallow. It cannot detect semantic contradictions, stale conclusions, or weak conceptual organization. Those checks belong to future LLM-assisted linting.
