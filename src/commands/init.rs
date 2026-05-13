use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

pub const DEFAULT_WORKSPACE_DIR: &str = "knowledgebase";
pub const LEGACY_WORKSPACE_DIR: &str = "KnowledgeBase";
pub const ROOT_MARKER_FILE: &str = "llm-wiki.toml";
pub const SCHEMA_VERSION: &str = "0.1";

/// Resolve a user-provided path. Relative paths are interpreted from the current directory.
pub fn resolve_user_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir().unwrap().join(path)
    }
}

/// Get the knowledge base path based on command line argument.
///
/// v0.7.8 prefers the lowercase `knowledgebase/` workspace directory,
/// while still detecting the older `KnowledgeBase/` layout for compatibility.
pub fn get_kb_path(custom_kb: Option<&Path>) -> PathBuf {
    if let Some(path) = custom_kb {
        return resolve_user_path(path);
    }

    let root = std::env::current_dir().unwrap();

    for candidate in [
        root.join(DEFAULT_WORKSPACE_DIR),
        root.join(LEGACY_WORKSPACE_DIR),
    ] {
        if candidate.exists() {
            return candidate;
        }
    }

    if let Some(parent) = root.parent() {
        for candidate in [
            parent.join(DEFAULT_WORKSPACE_DIR),
            parent.join(LEGACY_WORKSPACE_DIR),
        ] {
            if candidate.exists() {
                return candidate;
            }
        }
    }

    root.join(DEFAULT_WORKSPACE_DIR)
}

pub fn execute(custom_kb: Option<&Path>, force: bool) -> Result<()> {
    execute_with_source(custom_kb, None, force)
}

pub fn execute_with_source(
    custom_kb: Option<&Path>,
    source_path: Option<&Path>,
    force: bool,
) -> Result<()> {
    let kb_path = get_kb_path(custom_kb);

    println!(
        "Initializing or ensuring knowledge base at: {}",
        kb_path.display()
    );

    if kb_path.exists() {
        println!("Knowledge base path already exists. Ensuring directory structure...");
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
        "outputs/html",
        "LLM/tasks",
        "LLM/tasks/items",
        "topics",
        "LLM/memory",
        "topics",
        "outputs/slides",
        "outputs/figures",
        "processing",
        "processing/proposals",
        "processing/text",
        "processing/sections",
        "processing/refs",
        "processing/keywords",
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

    create_root_marker(&kb_path, source_path, force)?;

    create_rule_files(&kb_path, force)?;

    println!(
        "\nDone: {} directories created, {} skipped",
        created, skipped
    );
    Ok(())
}

fn list_existing_dirs(kb_path: &Path) {
    let dirs_to_check = vec![
        "raw",
        "wiki",
        "rules",
        "outputs",
        "LLM",
        "LLM/tasks",
        "LLM/tasks/items",
        "topics",
        "processing",
        "processing/proposals",
        "processing/text",
        "processing/sections",
        "processing/refs",
        "processing/keywords",
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

fn create_root_marker(kb_path: &Path, source_path: Option<&Path>, force: bool) -> Result<()> {
    let marker_path = kb_path.join(ROOT_MARKER_FILE);
    let existed = marker_path.exists();
    if existed && !force {
        println!("[SKIPPED] {} (already exists)", ROOT_MARKER_FILE);
        return Ok(());
    }

    let content = root_marker_content(kb_path, source_path);
    fs::write(&marker_path, content)?;

    if existed {
        println!("[UPDATED] {}", ROOT_MARKER_FILE);
    } else {
        println!("[CREATED] {}", ROOT_MARKER_FILE);
    }
    Ok(())
}

fn root_marker_content(kb_path: &Path, source_path: Option<&Path>) -> String {
    let workspace_name = kb_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(DEFAULT_WORKSPACE_DIR);

    let (source_mode, source_dir) = match source_path {
        Some(source) => {
            let resolved_source = resolve_user_path(source);
            let mode = if resolved_source == kb_path {
                "internal"
            } else {
                "external"
            };
            (mode, relative_or_absolute(kb_path, &resolved_source))
        }
        None => ("unspecified", String::from(".")),
    };

    format!(
        r#"# LLM Wiki root marker generated by kb-cli.
# This file identifies this directory as a managed local LLM Wiki workspace.

[project]
type = "llm-wiki"
name = "{workspace_name}"
schema_version = "{schema_version}"
created_by = "kb-cli"
created_with = "kb-cli v{version}"

[layout]
raw_dir = "raw"
wiki_dir = "wiki"
rules_dir = "rules"
processing_dir = "processing"
outputs_dir = "outputs"
llm_tasks_dir = "LLM/tasks"
llm_memory_dir = "LLM/memory"
topics_dir = "topics"
references_dir = "references"
logs_dir = "logs"

[source]
mode = "{source_mode}"
source_dir = "{source_dir}"

[policy]
llm_edit_scope = "inside_knowledgebase_only"
allow_edit_raw = false
allow_edit_wiki = true
allow_edit_rules = false
allow_edit_processing = true
allow_edit_outputs = true
"#,
        workspace_name = toml_escape(workspace_name),
        schema_version = SCHEMA_VERSION,
        version = env!("CARGO_PKG_VERSION"),
        source_mode = source_mode,
        source_dir = toml_escape(&source_dir),
    )
}

fn relative_or_absolute(from_dir: &Path, to_path: &Path) -> String {
    if to_path == from_dir {
        return String::from(".");
    }

    let from_components: Vec<_> = from_dir.components().collect();
    let to_components: Vec<_> = to_path.components().collect();

    let mut common = 0;
    while common < from_components.len()
        && common < to_components.len()
        && from_components[common] == to_components[common]
    {
        common += 1;
    }

    if common == 0 {
        return to_path.display().to_string();
    }

    let mut relative = PathBuf::new();
    for _ in common..from_components.len() {
        relative.push("..");
    }
    for component in &to_components[common..] {
        relative.push(Path::new(component.as_os_str()));
    }

    if relative.as_os_str().is_empty() {
        String::from(".")
    } else {
        relative.display().to_string()
    }
}

fn toml_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
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
    let content = format!(
        r#"# Knowledge Base

This is your local LLM Wiki managed by `kb-cli`.

## Three-Layer Structure

1. `raw/` is the source-material layer. Original files stay here. AI tools should read this layer, not rewrite it.
2. `wiki/` is the AI-maintained Markdown encyclopedia layer. It contains summaries, concepts, topic pages, comparisons, timelines, and question notes.
3. `rules/` is the schema/contract layer. It tells future AI maintainers how this Wiki should be organized and updated.

## Structure

```
{}/
├── llm-wiki.toml     # Root marker identifying this folder as an LLM Wiki workspace
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
├── LLM/              # LLM workbench; explicit handoff materials for future agents
│   └── tasks/        # Deferred task lists generated by kb tasks
├── outputs/          # Generated outputs
├── processing/       # Intermediate processing files and manifest
│   ├── manifest.json # Generated raw-file registry
│   ├── proposals/    # Future AI edit proposals
│   └── text/         # Extracted plain text for grep/query/refs workflows
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
# One-command cross-platform setup
kb --source /path/to/literature-folder setup --recursive

# Manual workflow
kb --kb-path /path/to/literature-folder/knowledgebase init
kb --source /path/to/literature-folder --kb-path /path/to/literature-folder/knowledgebase ingest --copy --recursive
kb --kb-path /path/to/literature-folder/knowledgebase extract-metadata
kb --kb-path /path/to/literature-folder/knowledgebase build-wiki
kb --kb-path /path/to/literature-folder/knowledgebase status
```

## Root Marker

`llm-wiki.toml` identifies this directory as a managed local LLM Wiki workspace. It records the layout, the source-material boundary, and the default LLM edit policy. Future agents should read it before modifying files.

## Safety Notes

- `raw/` contains original source materials. Do not let AI rewrite raw files.
- `wiki/` is the editable AI-maintained layer.
- `rules/` is the contract layer. Review it before allowing AI to modify the Wiki.
- `LLM/tasks/` contains explicit task handoff lists for future human/LLM/agent work. It is not an automatic execution directory.
- `LLM/memory/` records completed task outcomes for later inspection. It is an audit memory, not hidden model state.
- `topics/` stores topic-specific relationship overlays such as causal, method, evidence, idea, and topic-local importance notes.
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
"#,
        kb_path.display()
    );

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
  - <manifest-id-from-prepare-plan>
source_files:
  - raw/papers/example.pdf
status: compiled
---
```

Rules:
- `source_ids` should contain manifest IDs from `processing/manifest.json` or `processing/prepare_queue.json` when available.
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

This schema is generated by `kb-cli v0.7.8`. It is intentionally simple and should evolve with the project.
"#
}

fn paper_page_template() -> &'static str {
    r#"# Paper Page Template

Use this template for pages under `wiki/papers/`.

```markdown
---
page_type: paper
source_ids:
  - <manifest-id-from-prepare-plan>
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
  - <manifest-id-from-prepare-plan>
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
