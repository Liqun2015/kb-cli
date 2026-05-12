use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::Result;

use crate::commands::manifest::Manifest;

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
        "wiki/people",
        "wiki/methods",
        "wiki/comparisons",
        "wiki/timelines",
        "wiki/questions",
        "wiki/indexes",
    ];
    for dir in wiki_dirs {
        let path = kb_path.join(dir);
        if !path.exists() {
            fs::create_dir_all(&path)?;
        }
    }

    let manifest_lookup = load_manifest_lookup(&kb_path).unwrap_or_default();

    // 1. Build paper pages from metadata.
    let metadata_path = kb_path.join("logs/papers_metadata.json");
    if metadata_path.exists() {
        let metadata_content = fs::read_to_string(&metadata_path)?;
        let papers: Vec<PaperMetadata> = serde_json::from_str(&metadata_content)?;

        println!("\nGenerating paper pages...");
        for paper in &papers {
            generate_paper_page(&kb_path, paper, &manifest_lookup)?;
        }
        println!("Generated {} paper pages", papers.len());

        generate_papers_index(&kb_path, &papers)?;
    } else {
        println!("No metadata found. Run 'kb extract-metadata' first.");
    }

    // 2. Build note pages from raw/notes.
    build_note_pages(&kb_path, &manifest_lookup)?;

    // 3. Generate indexes.
    generate_concepts_index(&kb_path)?;
    generate_notes_index(&kb_path)?;
    generate_topics_index(&kb_path)?;
    generate_main_index(&kb_path)?;

    println!("\nWiki build complete!");
    Ok(())
}

fn generate_paper_page(
    kb_path: &Path,
    paper: &PaperMetadata,
    manifest_lookup: &BTreeMap<String, String>,
) -> Result<()> {
    let title = paper
        .title
        .as_ref()
        .map(|t| t.trim())
        .filter(|t| !t.is_empty())
        .unwrap_or_else(|| filename_stem(&paper.filename));
    let page_stem = paper_page_stem(paper);

    let authors = paper
        .author
        .as_ref()
        .map(|a| a.as_str())
        .unwrap_or("Unknown");
    let subject = paper.subject.as_ref().map(|s| s.as_str()).unwrap_or("N/A");
    let pages = paper.pages;
    let doi = paper
        .doi
        .as_ref()
        .map(|d| format!("DOI: [{}](https://doi.org/{})", d, d))
        .unwrap_or_else(|| "N/A".to_string());

    let source_path = format!("raw/papers/{}", paper.filename.as_str());
    let front_matter = source_front_matter(
        "paper",
        &source_path,
        manifest_lookup.get(&source_path).map(|s| s.as_str()),
    );

    let content = format!(
        r#"{front_matter}# {title}

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

Add links to concrete concept pages after the paper has been reviewed.

## Reading Notes

*Add your reading notes here.*
"#,
        front_matter = front_matter,
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

fn build_note_pages(kb_path: &Path, manifest_lookup: &BTreeMap<String, String>) -> Result<()> {
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

        let source_path = relative_path_string(kb_path, path);
        let front_matter = source_front_matter(
            "note",
            &source_path,
            manifest_lookup.get(&source_path).map(|s| s.as_str()),
        );
        let note_display_path = path
            .strip_prefix(&notes_dir)
            .unwrap_or(path)
            .to_str()
            .unwrap_or("")
            .replace('\\', "/");

        let content = format!(
            r#"{front_matter}# {filename}

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
            front_matter = front_matter,
            filename = filename,
            rel_path = note_display_path,
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
        content.push_str(&format!(
            "- [[{}|{}]] - {}\n",
            page_stem,
            title,
            paper.filename.as_str()
        ));
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

No concrete concept pages have been linked yet.

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
        content.push_str(
            "No note pages generated yet. Add files to `raw/notes/` and run `kb build-wiki`.\n",
        );
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

No concrete topic pages have been linked yet.

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
    let papers_count = count_md_files(&kb_path.join("wiki/papers"));
    let notes_count = count_md_files(&kb_path.join("wiki/notes"));
    let concepts_count = count_md_files(&kb_path.join("wiki/concepts"));
    let topics_count = count_md_files(&kb_path.join("wiki/topics"));
    let people_count = count_md_files(&kb_path.join("wiki/people"));
    let methods_count = count_md_files(&kb_path.join("wiki/methods"));
    let comparisons_count = count_md_files(&kb_path.join("wiki/comparisons"));
    let timelines_count = count_md_files(&kb_path.join("wiki/timelines"));
    let questions_count = count_md_files(&kb_path.join("wiki/questions"));
    let total_md = papers_count
        + notes_count
        + concepts_count
        + topics_count
        + people_count
        + methods_count
        + comparisons_count
        + timelines_count
        + questions_count
        + 4; // generated index pages

    let content = format!(
        r#"# Knowledge Base

Welcome to your local Markdown LLM Wiki.

## Three-Layer Model

- `raw/` stores original source materials. Treat this as read-only source-of-truth.
- `wiki/` stores the AI-maintained Markdown encyclopedia.
- `rules/` stores the operating contract for future AI maintainers.

Before asking an AI agent to edit the Wiki, review the schema file at:

```text
rules/LLM_WIKI_SCHEMA.md
```

## Running the CLI

### Cross-platform setup

```bash
kb --source /path/to/literature-folder setup --recursive
```

### Manual workflow

```bash
kb --kb-path /path/to/literature-folder/knowledgebase init
kb --source /path/to/literature-folder --kb-path /path/to/literature-folder/knowledgebase ingest --copy --recursive
kb --kb-path /path/to/literature-folder/knowledgebase extract-metadata
kb --kb-path /path/to/literature-folder/knowledgebase build-wiki
kb --kb-path /path/to/literature-folder/knowledgebase status
```

## Indexes

- [[Papers Index]] - {papers} papers with metadata
- [[Notes Index]] - {notes} notes and reference materials
- [[Concepts Index]] - Core concepts and definitions
- [[Topics Index]] - Thematic collections
- [[Main Index]] - This page

## Wiki Areas

| Area | Count | Purpose |
|------|------:|---------|
| Papers | {papers} | Source-specific paper summaries |
| Notes | {notes} | Compiled notes and documents |
| Concepts | {concepts} | Reusable concept pages |
| Topics | {topics} | Thematic collections |
| People | {people} | People, labs, institutions |
| Methods | {methods} | Methods, protocols, algorithms |
| Comparisons | {comparisons} | Side-by-side analyses |
| Timelines | {timelines} | Chronological development pages |
| Questions | {questions} | Durable Q&A notes |
| **Total MD files** | **{total}** | Includes generated index pages |

## Directory Structure

```text
knowledgebase/
├── raw/                  # Original source materials
│   ├── papers/
│   ├── notes/
│   ├── images/
│   ├── datasets/
│   ├── archives/
│   ├── repos/
│   └── other/
├── wiki/                 # AI-maintained Markdown encyclopedia
│   ├── papers/
│   ├── notes/
│   ├── concepts/
│   ├── topics/
│   ├── people/
│   ├── methods/
│   ├── comparisons/
│   ├── timelines/
│   ├── questions/
│   └── indexes/
├── rules/                # LLM Wiki schema and AI-maintainer contract
├── outputs/              # Generated answers and reports
├── processing/           # Intermediate processing files
│   ├── manifest.json     # Raw-file registry generated by kb status
│   └── proposals/        # Future AI edit proposals
├── references/           # Reference materials and templates
└── logs/
    └── papers_metadata.json
```

## CLI / Shell / LLM Boundary

`kb>` is treated as a deterministic command shell, not an LLM chat interface. Future LLM behavior must use a deliberately designed explicit interface after the architecture is intentionally decided.

---

*Last updated: {timestamp}*
"#,
        papers = papers_count,
        notes = notes_count,
        concepts = concepts_count,
        topics = topics_count,
        people = people_count,
        methods = methods_count,
        comparisons = comparisons_count,
        timelines = timelines_count,
        questions = questions_count,
        total = total_md,
        timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M UTC").to_string()
    );

    let main_index_path = kb_path.join("wiki/indexes/main_index.md");
    fs::write(&main_index_path, &content)?;

    let home_path = kb_path.join("wiki/Home.md");
    fs::write(&home_path, content)?;

    println!("Generated main index and Home.md");
    Ok(())
}

fn load_manifest_lookup(kb_path: &Path) -> Result<BTreeMap<String, String>> {
    let manifest_path = kb_path.join("processing/manifest.json");
    if !manifest_path.exists() {
        return Ok(BTreeMap::new());
    }

    let content = fs::read_to_string(&manifest_path)?;
    let manifest: Manifest = serde_json::from_str(&content)?;
    Ok(manifest
        .entries
        .into_iter()
        .map(|entry| (entry.path, entry.id))
        .collect())
}

fn source_front_matter(page_type: &str, source_path: &str, source_id: Option<&str>) -> String {
    let mut out = String::new();
    out.push_str("---\n");
    out.push_str(&format!("page_type: {}\n", page_type));
    out.push_str("source_files:\n");
    out.push_str(&format!("  - {}\n", yaml_quote(source_path)));

    if let Some(source_id) = source_id {
        out.push_str("source_ids:\n");
        out.push_str(&format!("  - {}\n", yaml_quote(source_id)));
    }

    out.push_str("status: compiled\n");
    out.push_str("generated_by: kb build-wiki\n");
    out.push_str("---\n\n");
    out
}

fn yaml_quote(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

fn relative_path_string(kb_path: &Path, path: &Path) -> String {
    path.strip_prefix(kb_path)
        .unwrap_or(path)
        .components()
        .map(|component| component.as_os_str().to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join("/")
}

fn count_md_files(dir: &Path) -> usize {
    fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(|x| x.ok())
                .filter(|entry| entry.path().extension().and_then(|s| s.to_str()) == Some("md"))
                .count()
        })
        .unwrap_or(0)
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

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use std::collections::BTreeMap;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_test_kb(name: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "kb_cli_build_wiki_{}_{}_{}",
            name,
            std::process::id(),
            nanos
        ))
    }

    #[test]
    fn source_front_matter_includes_source_file_and_id() {
        let front_matter =
            source_front_matter("paper", "raw/papers/example paper.pdf", Some("raw_123456"));

        assert!(front_matter.starts_with("---\n"));
        assert!(front_matter.contains("page_type: paper"));
        assert!(front_matter.contains("source_files:"));
        assert!(front_matter.contains("\"raw/papers/example paper.pdf\""));
        assert!(front_matter.contains("source_ids:"));
        assert!(front_matter.contains("\"raw_123456\""));
        assert!(front_matter.contains("status: compiled"));
        assert!(front_matter.contains("generated_by: kb build-wiki"));
    }

    #[test]
    fn generated_paper_page_has_lint_compatible_front_matter() -> Result<()> {
        let kb_path = unique_test_kb("paper_front_matter");
        fs::create_dir_all(kb_path.join("wiki/papers"))?;

        let paper = PaperMetadata {
            filename: "example.pdf".to_string(),
            title: Some("Example Paper".to_string()),
            author: Some("A. Author".to_string()),
            subject: None,
            pages: 7,
            doi: None,
        };

        let mut manifest_lookup = BTreeMap::new();
        manifest_lookup.insert(
            "raw/papers/example.pdf".to_string(),
            "raw_example".to_string(),
        );

        generate_paper_page(&kb_path, &paper, &manifest_lookup)?;
        let content = fs::read_to_string(kb_path.join("wiki/papers/Example_Paper.md"))?;

        assert!(content.starts_with("---\n"));
        assert!(content.contains("page_type: paper"));
        assert!(content.contains("source_files:"));
        assert!(content.contains("raw/papers/example.pdf"));
        assert!(content.contains("source_ids:"));
        assert!(content.contains("raw_example"));
        assert!(!content.contains("[[Concept Placeholder]]"));
        assert!(!content.contains("[[Topic Placeholder]]"));
        assert!(!content.contains("[[LLM_WIKI_SCHEMA"));

        let _ = fs::remove_dir_all(&kb_path);
        Ok(())
    }

    #[test]
    fn sanitize_filename_is_stable_for_common_titles() {
        assert_eq!(
            sanitize_filename("A Diffusion-Based Framework.pdf"),
            "A_Diffusion-Based_Framework_pdf"
        );
        assert_eq!(
            sanitize_filename("  paper: title / with * marks  "),
            "paper_title_with_marks"
        );
        assert_eq!(sanitize_filename("!!!"), "untitled");
    }
}
