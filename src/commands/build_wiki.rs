use std::fs;
use std::path::Path;

use anyhow::Result;

#[derive(Debug, serde::Deserialize)]
struct PaperMetadata {
    filename: String,
    title: Option<String>,
    author: Option<String>,
    subject: Option<String>,
    pages: usize,
    doi: Option<String>,
}

pub fn execute(custom_kb: Option<&Path>) -> Result<()> {
    let kb_path = crate::commands::init::get_kb_path(custom_kb);

    println!("Building Wiki from: {}", kb_path.display());

    // Create wiki directories if they don't exist.
    let wiki_dirs = vec![
        "wiki/papers",
        "wiki/notes",
        "wiki/concepts",
        "wiki/topics",
        "wiki/indexes",
    ];
    for dir in wiki_dirs {
        let path = kb_path.join(dir);
        if !path.exists() {
            fs::create_dir_all(&path)?;
        }
    }

    // 1. Build paper pages from metadata.
    let metadata_path = kb_path.join("logs/papers_metadata.json");
    if metadata_path.exists() {
        let metadata_content = fs::read_to_string(&metadata_path)?;
        let papers: Vec<PaperMetadata> = serde_json::from_str(&metadata_content)?;

        println!("\nGenerating paper pages...");
        for paper in &papers {
            generate_paper_page(&kb_path, paper)?;
        }
        println!("Generated {} paper pages", papers.len());

        generate_papers_index(&kb_path, &papers)?;
    } else {
        println!("No metadata found. Run 'kb extract-metadata' first.");
    }

    // 2. Build note pages from raw/notes.
    build_note_pages(&kb_path)?;

    // 3. Generate indexes.
    generate_concepts_index(&kb_path)?;
    generate_notes_index(&kb_path)?;
    generate_topics_index(&kb_path)?;
    generate_main_index(&kb_path)?;

    println!("\nWiki build complete!");
    Ok(())
}

fn generate_paper_page(kb_path: &Path, paper: &PaperMetadata) -> Result<()> {
    let title = paper
        .title
        .as_ref()
        .map(|t| t.trim())
        .filter(|t| !t.is_empty())
        .unwrap_or_else(|| filename_stem(&paper.filename));
    let page_stem = paper_page_stem(paper);

    let authors = paper.author.as_ref().map(|a| a.as_str()).unwrap_or("Unknown");
    let subject = paper.subject.as_ref().map(|s| s.as_str()).unwrap_or("N/A");
    let pages = paper.pages;
    let doi = paper
        .doi
        .as_ref()
        .map(|d| format!("DOI: [{}](https://doi.org/{})", d, d))
        .unwrap_or_else(|| "N/A".to_string());

    let content = format!(
        r#"# {title}

## Metadata

| Field | Value |
|-------|-------|
| File | [{filename}](../../raw/papers/{filename}) |
| Authors | {authors} |
| Subject | {subject} |
| Pages | {pages} |
| DOI | {doi} |

## Abstract

*To be filled from the paper or user notes.*

## Key Points

- *To be filled*

## Related Concepts

- [[Concept Placeholder]]

## Reading Notes

*Add your reading notes here.*
"#,
        title = title,
        filename = paper.filename.as_str(),
        authors = authors,
        subject = subject,
        pages = pages,
        doi = doi,
    );

    let output_path = kb_path.join(format!("wiki/papers/{}.md", page_stem));
    fs::write(&output_path, content)?;
    Ok(())
}

fn build_note_pages(kb_path: &Path) -> Result<()> {
    let notes_dir = kb_path.join("raw/notes");

    if !notes_dir.exists() {
        println!("\nraw/notes directory not found, skipping notes...");
        return Ok(());
    }

    println!("\nGenerating note pages...");

    let mut note_count = 0;
    for entry in walkdir::WalkDir::new(&notes_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        let filename = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string();

        let content = format!(
            r#"# {filename}

## File Info

| Field | Value |
|-------|-------|
| Type | {file_type} |
| Path | {rel_path} |
| Size | {size} |

## Description

*To be filled.*

## Notes

*Add your notes here.*
"#,
            filename = filename,
            rel_path = path
                .strip_prefix(&notes_dir)
                .unwrap_or(path)
                .to_str()
                .unwrap_or(""),
            file_type = path
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string(),
            size = format_size(fs::metadata(&path).ok().map(|m| m.len()).unwrap_or(0)),
        );

        let output_path = kb_path.join(format!("wiki/notes/{}.md", sanitize_filename(&filename)));
        fs::write(&output_path, content)?;
        note_count += 1;
    }

    println!("Generated {} note pages", note_count);
    Ok(())
}

fn generate_papers_index(kb_path: &Path, papers: &[PaperMetadata]) -> Result<()> {
    let mut content = String::from("# Papers Index\n\n");
    content.push_str(&format!("Total papers: {}\n\n", papers.len()));
    content.push_str("## All Papers\n\n");

    for paper in papers {
        let title = paper
            .title
            .as_ref()
            .map(|t| t.trim())
            .filter(|t| !t.is_empty())
            .unwrap_or_else(|| filename_stem(&paper.filename));
        let page_stem = paper_page_stem(paper);
        content.push_str(&format!("- [[{}|{}]] - {}\n", page_stem, title, paper.filename.as_str()));
    }

    let output_path = kb_path.join("wiki/indexes/papers_index.md");
    fs::write(&output_path, content)?;
    println!("Generated papers index");
    Ok(())
}

fn generate_concepts_index(kb_path: &Path) -> Result<()> {
    let content = r#"# Concepts Index

This page is a neutral starting point for concepts compiled from local source materials.

## Core Concepts

- [[Concept Placeholder]]

## Candidate Concepts

Add candidate concept pages here as papers, notes, and other materials are processed.

## Maintenance Notes

- Keep concept names short and stable.
- Prefer one concept per page.
- Link each concept back to supporting papers or notes.

---

*This index is generated by `kb build-wiki` and can be edited manually.*
"#;

    let output_path = kb_path.join("wiki/indexes/concepts_index.md");
    fs::write(&output_path, content)?;
    Ok(())
}

fn generate_notes_index(kb_path: &Path) -> Result<()> {
    let notes_dir = kb_path.join("wiki/notes");
    let mut note_names = Vec::new();

    if let Ok(entries) = fs::read_dir(&notes_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("md") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    note_names.push(stem.to_string());
                }
            }
        }
    }

    note_names.sort();

    let mut content = String::from("# Notes Index\n\n");
    content.push_str("Collection of note pages generated from `raw/notes/`.\n\n");
    content.push_str(&format!("Total notes: {}\n\n", note_names.len()));
    content.push_str("## All Notes\n\n");

    if note_names.is_empty() {
        content.push_str("No note pages generated yet. Add files to `raw/notes/` and run `kb build-wiki`.\n");
    } else {
        for name in note_names {
            content.push_str(&format!("- [[{}]]\n", name));
        }
    }

    let output_path = kb_path.join("wiki/indexes/notes_index.md");
    fs::write(&output_path, content)?;
    Ok(())
}

fn generate_topics_index(kb_path: &Path) -> Result<()> {
    let content = r#"# Topics Index

Topics are higher-level collections that group papers, notes, and concepts.

## Candidate Topics

- [[Topic Placeholder]]

## Suggested Workflow

1. Create a topic page only after several source materials point to the same theme.
2. Link the topic page to related papers, notes, and concepts.
3. Keep open questions and next actions on the topic page.

---

*Topics will be expanded based on the user's materials and research needs.*
"#;

    let output_path = kb_path.join("wiki/indexes/topics_index.md");
    fs::write(&output_path, content)?;
    Ok(())
}

fn generate_main_index(kb_path: &Path) -> Result<()> {
    let papers_count = fs::read_dir(kb_path.join("wiki/papers"))
        .map(|entries| entries.filter_map(|x| x.ok()).count())
        .unwrap_or(0);
    let notes_count = fs::read_dir(kb_path.join("wiki/notes"))
        .map(|entries| entries.filter_map(|x| x.ok()).count())
        .unwrap_or(0);
    let concepts_count = fs::read_dir(kb_path.join("wiki/concepts"))
        .map(|entries| entries.filter_map(|x| x.ok()).count())
        .unwrap_or(0);
    let topics_count = fs::read_dir(kb_path.join("wiki/topics"))
        .map(|entries| entries.filter_map(|x| x.ok()).count())
        .unwrap_or(0);
    let total_md = papers_count + notes_count + concepts_count + topics_count + 4; // +4 for indexes

    let content = format!(
        r#"# Knowledge Base

Welcome to your local Markdown knowledge base.

## Overview

This knowledge base follows the Karpathy-style local knowledge workflow:
- Original materials are stored in `raw/`.
- Wiki pages are generated under `wiki/` as Markdown files.
- Obsidian can be used for reading, editing, backlinks, and graph views.

## Running the CLI

### Direct commands

```bash
kb init                    # Initialize knowledge base
kb init --force            # Re-create bootstrap structure
kb extract-metadata        # Extract PDF metadata
kb extract-metadata --force # Overwrite existing metadata
kb build-wiki              # Build wiki pages
kb repl                    # Enter interactive mode
```

### REPL mode

```bash
kb repl

kb> help                   # Show available commands
kb> ask <question>          # Write a model-switch request
kb> list papers             # List paper pages
kb> list notes              # List note pages
kb> search <query>          # Search paper/note page names
kb> exit                    # Exit
```

## Indexes

- [[Papers Index]] - {papers} papers with metadata
- [[Notes Index]] - {notes} notes and reference materials
- [[Concepts Index]] - Core concepts and definitions
- [[Topics Index]] - Thematic collections
- [[Main Index]] - This page

## Statistics

| Category | Count |
|-----------|--------|
| Papers | {papers} |
| Notes | {notes} |
| Concepts | {concepts} |
| Topics | {topics} |
| **Total MD files** | **{total}** |

## Directory Structure

```text
KnowledgeBase/
├── raw/
│   ├── papers/            # Original PDF papers
│   ├── notes/             # Notes and reference materials
│   ├── images/            # Images and figures
│   ├── datasets/          # Data sets
│   └── repos/             # Code repositories
├── wiki/
│   ├── papers/            # Paper wiki pages
│   ├── notes/             # Note wiki pages
│   ├── concepts/          # Concept pages
│   ├── topics/            # Topic pages
│   └── indexes/           # Navigation pages
├── outputs/               # Generated answers and reports
└── logs/
    └── papers_metadata.json
```

### Model-Switch Integration

The REPL writes LLM requests to `.model_switch_input.json`. External model-switch tooling may write responses to `.model_switch_output.json` with the same `request_id`.

---

*Last updated: {timestamp}*
"#,
        papers = papers_count,
        notes = notes_count,
        concepts = concepts_count,
        topics = topics_count,
        total = total_md,
        timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M UTC").to_string()
    );

    let output_path = kb_path.join("wiki/indexes/main_index.md");
    fs::write(&output_path, content)?;
    println!("Generated main index");
    Ok(())
}

fn paper_page_stem(paper: &PaperMetadata) -> String {
    let title = paper
        .title
        .as_ref()
        .map(|t| t.trim())
        .filter(|t| !t.is_empty())
        .unwrap_or_else(|| filename_stem(&paper.filename));
    let title_stem = sanitize_filename(title);
    if title_stem.is_empty() || title_stem == "Untitled" {
        sanitize_filename(filename_stem(&paper.filename))
    } else {
        title_stem
    }
}

fn filename_stem(filename: &str) -> &str {
    Path::new(filename)
        .file_stem()
        .and_then(|s| s.to_str())
        .filter(|s| !s.trim().is_empty())
        .unwrap_or(filename)
}

fn sanitize_filename(name: &str) -> String {
    let mut result = String::new();
    let mut previous_was_separator = false;

    for c in name.chars() {
        if c.is_alphanumeric() || c == '-' || c == '_' {
            result.push(c);
            previous_was_separator = false;
        } else if c.is_whitespace() || c == '.' {
            if !previous_was_separator && !result.is_empty() {
                result.push('_');
                previous_was_separator = true;
            }
        } else if !previous_was_separator && !result.is_empty() {
            result.push('_');
            previous_was_separator = true;
        }
    }

    let mut result = result.trim_matches('_').to_string();
    if result.len() > 100 {
        result.truncate(100);
        result = result.trim_matches('_').to_string();
    }

    if result.is_empty() {
        "untitled".to_string()
    } else {
        result
    }
}

fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}
