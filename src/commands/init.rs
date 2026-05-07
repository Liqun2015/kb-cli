use std::fs;
use std::path::{Path, PathBuf};
use anyhow::Result;

/// Get the knowledge base path based on command line argument
pub fn get_kb_path(custom_kb: Option<&Path>) -> PathBuf {
    if let Some(path) = custom_kb {
        if path.is_absolute() || path.exists() {
            return path.to_path_buf();
        }
        return std::env::current_dir().unwrap().join(path);
    }

    // Default: look for KnowledgeBase in current directory
    let root = std::env::current_dir().unwrap();
    let current_kb = root.join("KnowledgeBase");

    // Try current directory first
    if current_kb.exists() {
        return current_kb;
    }

    // If not found in current, check parent directory
    if let Some(parent) = root.parent() {
        let parent_kb = parent.join("KnowledgeBase");
        if parent_kb.exists() {
            return parent_kb;
        }
    }

    // Default to current directory
    current_kb
}

pub fn execute(custom_kb: Option<&Path>, force: bool) -> Result<()> {
    let kb_path = get_kb_path(custom_kb);

    println!("Initializing or ensuring KnowledgeBase at: {}", kb_path.display());

    if kb_path.exists() {
        println!("KnowledgeBase path already exists. Ensuring directory structure...");
        println!("Current directories:");
        list_existing_dirs(&kb_path);
    }

    // Define directory structure.
    // raw/ is the immutable source-material layer.
    // wiki/ is the AI-maintained Markdown encyclopedia layer.
    // rules/ is the schema/contract layer that constrains future AI maintainers.
    let dirs = vec![
        "raw/papers",
        "raw/notes",
        "raw/images",
        "raw/datasets",
        "raw/archives",
        "raw/repos",
        "raw/other",
        "wiki/papers",
        "wiki/notes",
        "wiki/concepts",
        "wiki/topics",
        "wiki/people",
        "wiki/methods",
        "wiki/comparisons",
        "wiki/timelines",
        "wiki/questions",
        "wiki/indexes",
        "rules",
        "outputs/answers",
        "outputs/reports",
        "outputs/slides",
        "outputs/figures",
        "processing",
        "processing/proposals",
        "references",
        "logs",
    ];

    let mut created = 0;
    let mut skipped = 0;

    for dir in dirs {
        let path = kb_path.join(dir);
        if path.exists() && !force {
            println!("[SKIPPED] {} (already exists)", dir);
            skipped += 1;
        } else {
            fs::create_dir_all(&path)?;
            println!("[CREATED] {}", dir);
            created += 1;
        }
    }

    // Create Obsidian config
    let obsidian_dir = kb_path.join(".obsidian");
    if !obsidian_dir.exists() {
        fs::create_dir_all(&obsidian_dir)?;
        create_obsidian_config(&obsidian_dir)?;
        println!("[CREATED] Obsidian configuration");
    }

    // Create README
    let readme_path = kb_path.join("README.md");
    if !readme_path.exists() {
        create_readme(&kb_path, &readme_path)?;
        println!("[CREATED] README.md");
    }

    create_rule_files(&kb_path, force)?;

    println!("\nDone: {} directories created, {} skipped", created, skipped);
    Ok(())
}

fn list_existing_dirs(kb_path: &Path) {
    let dirs_to_check = vec![
        "raw",
        "wiki",
        "rules",
        "outputs",
        "processing",
        "processing/proposals",
        "references",
        "logs",
        ".obsidian",
    ];
    for dir in dirs_to_check {
        let path = kb_path.join(dir);
        if path.exists() {
            let file_count = std::fs::read_dir(&path)
                .map(|entries| entries.count())
                .unwrap_or(0);
            println!("  - {} ({} items)", dir, file_count);
        }
    }
}

fn create_obsidian_config(obsidian_dir: &Path) -> Result<()> {
    // app.json
    let app_json = r#"{
  "legacyEditor": false,
  "livePreview": true,
  "promptDelete": false,
  "showLineNumber": true,
  "spellcheck": true,
  "spellcheckLanguages": ["en-US", "zh-CN"]
}"#;
    fs::write(obsidian_dir.join("app.json"), app_json)?;

    // appearance.json
    let appearance_json = r#"{
  "accentColor": "",
  "baseFontSize": 16,
  "theme": "default"
}"#;
    fs::write(obsidian_dir.join("appearance.json"), appearance_json)?;

    // core-plugins.json
    let core_plugins_json = r#"{
  "file-explorer": true,
  "global-search": true,
  "switcher": true,
  "graph": true,
  "backlink": true,
  "outgoing-link": true,
  "tag-pane": true,
  "page-preview": true,
  "daily-notes": false,
  "templates": false,
  "word-count": true
}"#;
    fs::write(obsidian_dir.join("core-plugins.json"), core_plugins_json)?;

    Ok(())
}

fn create_readme(kb_path: &Path, readme_path: &Path) -> Result<()> {
    let content = format!(r#"# Knowledge Base

This is your local LLM Wiki managed by `kb-cli`.

## Three-Layer Structure

1. `raw/` is the source-material layer. Original files stay here. AI tools should read this layer, not rewrite it.
2. `wiki/` is the AI-maintained Markdown encyclopedia layer. It contains summaries, concepts, topic pages, comparisons, timelines, and question notes.
3. `rules/` is the schema/contract layer. It tells future AI maintainers how this Wiki should be organized and updated.

## Structure

```
{}/
├── raw/              # Original materials; source-of-truth layer
│   ├── papers/       # PDF papers
│   ├── notes/        # Notes and documents
│   ├── images/       # Images and figures
│   ├── datasets/     # Data sets
│   ├── archives/     # Compressed source archives
│   ├── repos/        # Code repositories
│   └── other/        # Other source materials
├── wiki/             # AI-maintained Markdown encyclopedia
│   ├── papers/       # Paper summaries
│   ├── notes/        # Note pages
│   ├── concepts/     # Concept definitions
│   ├── topics/       # Thematic collections
│   ├── people/       # People and institution profiles
│   ├── methods/      # Method/protocol pages
│   ├── comparisons/  # Comparative analysis pages
│   ├── timelines/    # Chronological development pages
│   ├── questions/    # Saved question-answer notes
│   └── indexes/      # Navigation indexes
├── rules/            # LLM Wiki schema and AI-maintainer contract
├── outputs/          # Generated outputs
├── processing/       # Intermediate processing files and manifest
│   ├── manifest.json # Generated raw-file registry
│   └── proposals/    # Future AI edit proposals
├── references/       # Templates and reference materials
├── logs/             # Logs and metadata
└── .obsidian/        # Obsidian configuration
```

## Rules Layer

Start here before asking an AI agent to maintain the Wiki:

```text
rules/LLM_WIKI_SCHEMA.md
rules/PAPER_PAGE_TEMPLATE.md
rules/CONCEPT_PAGE_TEMPLATE.md
rules/QUERY_POLICY.md
rules/LINT_POLICY.md
```

## Commands

```bash
# One-command cross-platform bootstrap
kb --kb-path /path/to/this/folder bootstrap --copy

# Manual workflow
kb --kb-path /path/to/this/folder init
kb --kb-path /path/to/this/folder ingest --copy
kb --kb-path /path/to/this/folder extract-metadata
kb --kb-path /path/to/this/folder build-wiki
kb --kb-path /path/to/this/folder status
```

## Safety Notes

- `raw/` contains original source materials. Do not let AI rewrite raw files.
- `wiki/` is the editable AI-maintained layer.
- `rules/` is the contract layer. Review it before allowing AI to modify the Wiki.
- `ingest --copy` copies source files into `raw/` and is safest.
- `ingest --move` moves source files into `raw/` and should be used intentionally.
- `ingest --dry-run` previews actions without changing files.
- `ingest --recursive` scans subfolders while skipping managed directories such as `raw`, `wiki`, `.git`, `.obsidian`, and `target`.
- `status` refreshes `processing/manifest.json`, which tracks raw files by path, kind, size, and content hash.

## Obsidian

Open this folder in Obsidian to:
- Browse wiki pages with [[WikiLinks]]
- Visualize the knowledge graph
- Search across all notes

---

*Generated by kb-cli*
"#, kb_path.display());

    fs::write(readme_path, content)?;
    Ok(())
}

fn create_rule_files(kb_path: &Path, force: bool) -> Result<()> {
    let rules_dir = kb_path.join("rules");
    fs::create_dir_all(&rules_dir)?;

    let rule_files = vec![
        ("LLM_WIKI_SCHEMA.md", llm_wiki_schema()),
        ("PAPER_PAGE_TEMPLATE.md", paper_page_template()),
        ("CONCEPT_PAGE_TEMPLATE.md", concept_page_template()),
        ("QUERY_POLICY.md", query_policy()),
        ("LINT_POLICY.md", lint_policy()),
    ];

    for (filename, content) in rule_files {
        let path = rules_dir.join(filename);
        let existed = path.exists();
        if existed && !force {
            println!("[SKIPPED] rules/{} (already exists)", filename);
        } else {
            fs::write(&path, content)?;
            if existed {
                println!("[UPDATED] rules/{}", filename);
            } else {
                println!("[CREATED] rules/{}", filename);
            }
        }
    }

    Ok(())
}

fn llm_wiki_schema() -> &'static str {
    r#"# LLM Wiki Schema

This file is the operating contract for AI agents maintaining this knowledge base.

## Purpose

The goal is to maintain a local Markdown encyclopedia compiled from source materials. The Wiki should become more useful over time through ingestion, questioning, and periodic linting.

## Three Layers

### 1. Source Layer: `raw/`

`raw/` contains original materials: papers, reports, notes, images, data, archives, and code-like sources.

Rules:
- Treat `raw/` as source-of-truth.
- Read from `raw/`; do not rewrite, rename, or delete source files unless the human explicitly asks.
- Every important Wiki claim should be traceable to one or more source files when possible.

### 2. Wiki Layer: `wiki/`

`wiki/` is the AI-maintained encyclopedia. AI agents may create and update Markdown pages here.

Recommended subfolders:
- `wiki/papers/` — one page per paper or source document.
- `wiki/notes/` — pages compiled from non-paper notes and documents.
- `wiki/concepts/` — reusable concept pages.
- `wiki/topics/` — thematic overview pages.
- `wiki/people/` — people, labs, institutions, or author profiles when useful.
- `wiki/methods/` — methods, algorithms, protocols, and experimental workflows.
- `wiki/comparisons/` — side-by-side analyses.
- `wiki/timelines/` — chronological development pages.
- `wiki/questions/` — valuable Q&A notes worth preserving.
- `wiki/indexes/` — navigation and backlinks.

### 3. Rules Layer: `rules/`

`rules/` contains the AI-maintainer contract. Agents must read this layer before making large updates.

## Daily Operations

### Ingest

When new material is added:
1. Keep the original in `raw/`.
2. Create or update the corresponding page in `wiki/papers/` or `wiki/notes/`.
3. Extract important concepts, methods, people, datasets, and claims.
4. Create or update concept/topic/method pages.
5. Add backlinks between related pages.
6. Update index pages.
7. Record uncertainty explicitly instead of inventing missing information.

### Query

When answering questions:
1. Prefer `wiki/` as the first context source.
2. Fall back to `raw/` only when the Wiki is incomplete or the user asks for source verification.
3. If the answer contains a durable insight, save it to `wiki/questions/` or merge it into relevant concept/topic pages.

### Lint

Periodic health checks should identify:
- Broken WikiLinks.
- Orphan pages.
- Duplicate or overlapping concepts.
- Important terms that appear often but lack concept pages.
- Pages without source references.
- Contradictory or stale claims.
- Empty placeholder pages.

## Source-Linking Front Matter

Every wiki page derived from raw material must start with YAML front matter so `kb sync-wiki` can link wiki pages back to `processing/manifest.json`.

Example:

```yaml
---
page_type: paper
source_ids:
  - <manifest-id-from-compile-plan>
source_files:
  - raw/papers/example.pdf
status: compiled
---
```

Rules:
- `source_ids` should contain manifest IDs from `processing/manifest.json` or `processing/compile_queue.json` when available.
- `source_files` should contain raw paths such as `raw/papers/example.pdf`.
- Concept/topic/method pages may list multiple sources.
- Run `kb sync-wiki` after accepted wiki edits so the manifest records which raw files have corresponding wiki pages.

## Writing Rules

- Use Markdown.
- Use Obsidian-style links: `[[Concept Name]]`.
- Prefer clear section headings.
- Keep source-derived facts separate from interpretation.
- Mark uncertain claims with `Uncertain:` or `Needs verification:`.
- Do not fabricate bibliographic details, results, numbers, dates, or citations.
- Do not overwrite human-written notes unless explicitly instructed.

## Human Review Points

Ask for human review or produce a plan before:
- Renaming many pages.
- Deleting pages.
- Merging concepts.
- Rewriting high-level topic pages.
- Applying broad structural changes across the Wiki.

## Version Note

This schema is generated by `kb-cli v0.4.4`. It is intentionally simple and should evolve with the project.
"#
}

fn paper_page_template() -> &'static str {
    r#"# Paper Page Template

Use this template for pages under `wiki/papers/`.

```markdown
---
page_type: paper
source_ids:
  - <manifest-id-from-compile-plan>
source_files:
  - raw/papers/example.pdf
status: compiled
---

# Title of Paper

## Source

| Field | Value |
|-------|-------|
| Raw file | `raw/papers/example.pdf` |
| Authors | Unknown |
| Year | Unknown |
| Venue | Unknown |
| DOI | Unknown |
| Status | unread / skimmed / compiled / needs review |

## One-Sentence Summary

Write one sentence explaining what this source contributes to the Wiki.

## Abstract / Overview

Summarize the source in plain language. Do not invent missing details.

## Key Contributions

- Contribution 1
- Contribution 2
- Contribution 3

## Methods / Mechanisms

- [[Method Name]]

## Important Concepts

- [[Concept Name]]

## Results and Evidence

Separate reported results from interpretation.

## Limitations

- Limitation or uncertainty.

## Connections

- Related topic: [[Topic Name]]
- Related concept: [[Concept Name]]
- Related source: [[Another Paper Page]]

## Questions Raised

- Question worth following up.

## Notes

Human or AI reading notes go here.
```
"#
}

fn concept_page_template() -> &'static str {
    r#"# Concept Page Template

Use this template for pages under `wiki/concepts/`.

```markdown
---
page_type: concept
source_ids:
  - <manifest-id-from-compile-plan>
source_files:
  - raw/papers/example.pdf
status: evolving
---

# Concept Name

## Definition

Give a concise definition.

## Why It Matters

Explain why this concept matters in this Wiki.

## Core Mechanism

Explain the mechanism, logic, or mathematical relationship when applicable.

## Related Sources

- [[Paper Page]]

## Related Concepts

- [[Related Concept]]

## Examples

- Example 1
- Example 2

## Open Questions

- Question 1

## Notes

Additional notes and caveats.
```
"#
}

fn query_policy() -> &'static str {
    r#"# Query Policy

This policy describes how AI agents should answer questions using this LLM Wiki.

## Source Priority

1. Search `wiki/` first.
2. Use `rules/` to understand formatting and update expectations.
3. Consult `raw/` only when the Wiki is incomplete, contradictory, or the user asks for source verification.

## Answer Behavior

- Answer from the compiled Wiki when possible.
- State uncertainty clearly.
- Distinguish source-derived facts from analysis or inference.
- Prefer linking to relevant Wiki pages using `[[WikiLinks]]`.

## Knowledge Compounding

If a question produces a durable insight, preserve it by either:
- Creating a new page under `wiki/questions/`, or
- Updating the relevant concept/topic/comparison page.

## Do Not

- Do not invent sources.
- Do not silently overwrite human-written notes.
- Do not treat the current answer as final if the Wiki lacks evidence.
- Do not modify `raw/` during question answering.
"#
}

fn lint_policy() -> &'static str {
    r#"# Lint Policy

This policy describes periodic Wiki health checks.

## Static Checks

A static lint pass can be run with `kb lint-static`. It should check for:
- Broken links: `[[Page]]` points to a missing page.
- Orphans: a page has no incoming links.
- Pages missing `source_ids` / `source_files` front matter.
- Empty placeholders.
- Duplicate titles or near-duplicate concepts.
- Raw files without corresponding Wiki pages.
- Wiki paper pages whose raw files are missing.

## Semantic Checks

When an LLM is available, a lint pass may also check for:
- Contradictory claims across pages.
- Overlapping concept pages that should be merged.
- Important recurring terms without concept pages.
- Stale conclusions that need review.

## Safe Workflow

1. Generate a lint report first.
2. Propose fixes before applying broad changes.
3. Use Git diff for human review.
4. Avoid deleting pages automatically.

## Suggested Output

Save lint reports under:

```text
outputs/reports/lint_static_YYYYMMDD_HHMMSS.md
```
"#
}
