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

    // Define directory structure
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
        "wiki/indexes",
        "outputs/answers",
        "outputs/reports",
        "outputs/slides",
        "outputs/figures",
        "processing",
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

    println!("\nDone: {} directories created, {} skipped", created, skipped);
    Ok(())
}

fn list_existing_dirs(kb_path: &Path) {
    let dirs_to_check = vec!["raw", "wiki", "outputs", "processing", "references", "logs", ".obsidian"];
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

This is your local knowledge base managed by `kb-cli`.

## Structure

```
{}/
├── raw/              # Original materials
│   ├── papers/       # PDF papers
│   ├── notes/        # Notes and documents
│   ├── images/       # Images and figures
│   ├── datasets/     # Data sets
│   ├── archives/     # Compressed source archives
│   ├── repos/        # Code repositories
│   └── other/        # Other source materials
├── wiki/             # Markdown Wiki pages
│   ├── papers/       # Paper summaries
│   ├── notes/        # Note pages
│   ├── concepts/     # Concept definitions
│   ├── topics/       # Thematic collections
│   └── indexes/      # Navigation indexes
├── outputs/          # Generated outputs
├── processing/       # Intermediate processing files
├── references/       # Templates and reference materials
├── logs/             # Logs and metadata
└── .obsidian/        # Obsidian configuration
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
```

## Safety Notes

- `ingest --copy` copies source files into `raw/` and is safest.
- `ingest --move` moves source files into `raw/` and should be used intentionally.
- `ingest --dry-run` previews actions without changing files.
- `ingest --recursive` scans subfolders while skipping managed directories such as `raw`, `wiki`, `.git`, `.obsidian`, and `target`.

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
