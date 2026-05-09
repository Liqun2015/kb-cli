# Static Wiki Lint

`kb lint-static` performs local, deterministic health checks over Markdown files under `wiki/`.

It does **not** call an LLM, does **not** modify `raw/`, and does **not** rewrite `wiki/`. By default it writes a reviewable report under:

```text
outputs/reports/lint_static_<timestamp>.md
```

Current version: `v0.6.4.1`

## Commands

```bash
# Write a Markdown lint report
kb --kb-path /path/to/kb lint-static

# Check without writing a report
kb --kb-path /path/to/kb lint-static --no-report

# Equivalent inspection aliases
kb --kb-path /path/to/kb lint-static --dry-run
kb --kb-path /path/to/kb lint-static --preview

# Machine-readable summary
kb --kb-path /path/to/kb lint-static --json

# CI/script mode: fail when issues are found
kb --kb-path /path/to/kb lint-static --strict
```

`--dry-run` remains supported. `--preview` is a general inspection alias. For `lint-static`, `--no-report` is the clearest wording because the only skipped write is the Markdown report file.

## What it checks

| Check | Meaning | Typical fix |
|---|---|---|
| Broken WikiLinks | A `[[Page]]` target cannot be resolved to a Markdown page in `wiki/`. | Create the missing page, rename the link, or remove the link. |
| Orphan pages | A non-index wiki page has no incoming links from other wiki pages. | Link useful pages from `Home.md`, index pages, topic pages, or concept pages. |
| Missing source front matter | Source-derived pages lack `source_ids` or `source_files`. | Add YAML front matter that points back to `processing/manifest.json` IDs or `raw/` paths. |
| Empty or near-empty pages | A non-index page has almost no Markdown body content. | Add useful content or remove the placeholder page. |
| Duplicate titles | Multiple pages use the same first-level heading or inferred title. | Rename, merge, or clarify the pages after human review. |

## WikiLink resolution

The linter recognizes common Obsidian-style forms:

```markdown
[[Page Name]]
[[Page Name|Displayed Alias]]
[[Page Name#Section Heading]]
[[folder/Page Name]]
```

It resolves links against page title, file stem, relative wiki path, and relative path without `.md`.

Examples that should resolve when `wiki/papers/Example.md` exists:

```markdown
[[Example]]
[[papers/Example]]
[[wiki/papers/Example.md]]
```

## Source front matter

Source-derived pages should include YAML front matter near the top of the file:

```yaml
---
page_type: paper
source_files:
  - "raw/papers/example.pdf"
source_ids:
  - "raw_example"
generated_by: kb build-wiki
---
```

`lint-static` accepts source references from these keys:

```text
source_ids
source_files
sources
raw_files
```

`kb build-wiki` now emits this front matter for generated paper and note pages. Manually created concept, topic, method, comparison, timeline, and question pages should also carry source references when they are derived from source materials.

## Testing note

The current regression tests cover the most fragile lint behavior:

```text
WikiLink target cleaning
YAML source-reference parsing
clean generated pages passing lint
broken links being reported
missing source front matter being reported
lint-static --no-report not writing reports
lint-static default mode writing one Markdown report
```

Run them with:

```bash
cargo test
```

Recommended release check:

```bash
cargo fmt --check
cargo test
cargo check
cargo build --release
```

## v0.4.5 consistency note

`kb build-wiki` generates source front matter for paper and note pages. A freshly generated source-derived page should not be reported as missing source information merely because it was created by the CLI.

Generated placeholder links that previously caused guaranteed broken-link noise, such as `[[Concept Placeholder]]`, `[[Topic Placeholder]]`, and `[[LLM_WIKI_SCHEMA]]`, are no longer emitted by `build-wiki`.

## Intended workflow

```text
kb prepare --new
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

## Recommended cleanup order

1. Fix broken links first. They usually indicate renamed, missing, or over-eager pages.
2. Add `source_ids` / `source_files` to source-derived pages.
3. Review empty pages and decide whether to fill or delete them.
4. Link useful orphan pages from index/topic/concept pages.
5. Merge or rename duplicate titles only after a human checks whether they really describe the same concept.

This order keeps the work reviewable and avoids chasing cosmetic orphan-page warnings while the source-traceability layer is still incomplete.

## JSON and strict mode

For scripts:

```bash
kb --kb-path /path/to/kb lint-static --json
```

For CI-like checks:

```bash
kb --kb-path /path/to/kb lint-static --strict
```

`--strict` returns a non-zero exit code when any issue group is found. It is best used after the Wiki has passed a normal report review at least once.

## Boundary

`lint-static` is intentionally shallow. It cannot detect semantic contradictions, stale conclusions, weak conceptual organization, hallucinated claims, or poor research judgment. Those checks belong to future LLM-assisted linting and human review.

Do not treat a clean static lint report as proof that the Wiki is scientifically correct. Treat it as proof that the local Markdown graph is structurally healthier and easier to review.
