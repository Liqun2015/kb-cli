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
    #[serde(default)]
    journal: Option<String>,
    #[serde(default)]
    abstract_text: Option<String>,
    #[serde(default)]
    keywords: Vec<String>,
    #[serde(default)]
    missing_key_info: Vec<String>,
    #[serde(default)]
    needs_llm_key_info: bool,
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
    let metadata_path = paper_metadata_path(&kb_path);
    let mut papers: Vec<PaperMetadata> = Vec::new();
    if metadata_path.exists() {
        let metadata_content = fs::read_to_string(&metadata_path)?;
        papers = serde_json::from_str(&metadata_content)?;

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

    // 3. Build topic pages from topics/<topic>/ workspaces.
    generate_topic_pages(&kb_path, &papers)?;

    // 4. Generate indexes.
    generate_concepts_index(&kb_path)?;
    generate_notes_index(&kb_path)?;
    generate_topics_index(&kb_path)?;
    generate_main_index(&kb_path)?;

    println!("\nWiki build complete!");
    Ok(())
}

fn paper_metadata_path(kb_path: &Path) -> std::path::PathBuf {
    let current = kb_path.join("interfaces/logs/papers_metadata.json");
    if current.exists() {
        return current;
    }
    let legacy = kb_path.join("logs/papers_metadata.json");
    if legacy.exists() {
        return legacy;
    }
    current
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

    let journal = paper
        .journal
        .as_ref()
        .map(|v| v.trim())
        .filter(|v| !v.is_empty())
        .unwrap_or("*Needs LLM extraction*");
    let abstract_text = paper
        .abstract_text
        .as_ref()
        .map(|v| v.trim())
        .filter(|v| !v.is_empty())
        .unwrap_or("*Needs LLM extraction from the paper.*");
    let keywords = if paper.keywords.is_empty() {
        "*Needs LLM extraction*".to_string()
    } else {
        paper
            .keywords
            .iter()
            .map(|kw| format!("`{}`", kw))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let section_dir = section_dir_for_paper(kb_path, paper.filename.as_str());
    let introduction_path = section_dir.join("introduction.txt");
    let references_path = section_dir.join("references.txt");
    let introduction_link = if introduction_path.exists() {
        format!(
            "[extracted introduction](../../{})",
            relative_path_string(kb_path, &introduction_path)
        )
    } else {
        "*Needs LLM/OCR extraction or `kb extract-sections` repair*".to_string()
    };
    let references_link = if references_path.exists() {
        format!(
            "[extracted references](../../{})",
            relative_path_string(kb_path, &references_path)
        )
    } else {
        "*Needs LLM/OCR extraction or `kb extract-sections` repair*".to_string()
    };

    let mut missing_fields = paper.missing_key_info.clone();
    if journal.contains("Needs LLM") && !missing_fields.iter().any(|v| v == "journal") {
        missing_fields.push("journal".to_string());
    }
    if abstract_text.contains("Needs LLM") && !missing_fields.iter().any(|v| v == "abstract") {
        missing_fields.push("abstract".to_string());
    }
    if paper.keywords.is_empty() && !missing_fields.iter().any(|v| v == "keywords") {
        missing_fields.push("keywords".to_string());
    }
    if !introduction_path.exists() && !missing_fields.iter().any(|v| v == "introduction") {
        missing_fields.push("introduction".to_string());
    }
    let missing_display = if missing_fields.is_empty() && !paper.needs_llm_key_info {
        "none detected".to_string()
    } else {
        missing_fields
            .iter()
            .map(|v| format!("`{}`", v))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let paper_profile_target = format!(
        "processing/paper_profiles/{}.md",
        sanitize_filename(filename_stem(paper.filename.as_str()))
    );

    let content = format!(
        r#"{front_matter}# {title}

## Metadata

| Field | Value |
|-------|-------|
| File | [{filename}](../../raw/papers/{filename}) |
| Authors | {authors} |
| Journal | {journal} |
| Subject | {subject} |
| Pages | {pages} |
| DOI | {doi} |
| Keywords | {keywords} |

## Abstract

{abstract_text}

## Extracted Sections

| Section | Status |
|---|---|
| Introduction | {introduction_link} |
| References | {references_link} |

## LLM Extraction Status

Missing or uncertain scholarly fields: {missing_display}.

Worker LLM target profile file:

```text
{paper_profile_target}
```

The Worker LLM should extract paper key information from the PDF/text, preserve uncertainty, and then update this page with evidence-backed fields.

## Key Points

- *To be filled after the paper has been reviewed.*

## Related Topics and Concepts

- Link this paper to topic pages such as `[[Topics Index]]` after the topic narrative is built.

## Reading Notes

*Add your reading notes here.*
"#,
        front_matter = front_matter,
        title = title,
        filename = paper.filename.as_str(),
        authors = authors,
        journal = journal,
        subject = subject,
        pages = pages,
        doi = doi,
        keywords = keywords,
        abstract_text = abstract_text,
        introduction_link = introduction_link,
        references_link = references_link,
        missing_display = missing_display,
        paper_profile_target = paper_profile_target,
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

#[derive(Debug, Clone)]
struct TopicLiteratureRow {
    paper: String,
    role: String,
    status: String,
}

fn generate_topic_pages(kb_path: &Path, papers: &[PaperMetadata]) -> Result<()> {
    let topics_root = kb_path.join("topics");
    if !topics_root.exists() {
        println!(
            "
topics directory not found, skipping topic pages..."
        );
        return Ok(());
    }

    println!(
        "
Generating topic pages..."
    );
    let wiki_topics_dir = kb_path.join("wiki/topics");
    fs::create_dir_all(&wiki_topics_dir)?;
    let paper_link_map = build_paper_link_map(papers);
    let mut topic_count = 0usize;

    for entry in fs::read_dir(&topics_root)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let topic_root = entry.path();
        let slug = entry.file_name().to_string_lossy().to_string();
        let output_path = wiki_topics_dir.join(format!("{}.md", sanitize_filename(&slug)));
        if output_path.exists() {
            continue;
        }

        let title = read_topic_title(&topic_root).unwrap_or_else(|| slug.replace('-', " "));
        let literature_rows = read_topic_literature_rows(&topic_root.join("literature.md"))?;
        let source_path = format!("topics/{}/", slug);
        let front_matter = source_front_matter("topic", &source_path, None);
        let mut content = String::new();
        content.push_str(&front_matter);
        content.push_str(&format!(
            "# {}

",
            title
        ));
        content.push_str(
            "## Story Narrative

",
        );
        content.push_str("*To be drafted by a Manager/Worker LLM from the linked literature. The narrative should explain the topic background, key problem, method lineage, major papers, and unresolved questions, with explicit Wiki links to supporting paper pages.*

");
        content.push_str(
            "## Linked Literature

",
        );
        if literature_rows.is_empty() {
            content.push_str("No topic-local literature rows were found yet. Add papers to `topics/<topic>/literature.md` and rerun `kb build-wiki`.

");
        } else {
            content.push_str(
                "| paper | role | status | wiki link |
",
            );
            content.push_str(
                "|---|---|---|---|
",
            );
            for row in &literature_rows {
                let link = resolve_paper_wikilink(&row.paper, &paper_link_map);
                content.push_str(&format!(
                    "| {} | {} | {} | {} |
",
                    escape_table_cell(&row.paper),
                    escape_table_cell(&row.role),
                    escape_table_cell(&row.status),
                    escape_table_cell(&link)
                ));
            }
            content.push_str(
                "
",
            );
        }
        content.push_str(
            "## Suggested LLM Work

",
        );
        content.push_str(
            "1. Extract paper key information for each linked paper if missing.
",
        );
        content.push_str(
            "2. Build a concise topic story with citations and Wiki links.
",
        );
        content.push_str(
            "3. Keep uncertain relationships as candidates until reviewed by a human.

",
        );
        content.push_str(
            "## Relationship Views

",
        );
        content.push_str(&format!(
            "- Topic workbench: `topics/{}/`
",
            slug
        ));
        content.push_str(&format!(
            "- Topic graph: `topics/{}/graph/topic_graph.md`
",
            slug
        ));
        content.push_str(&format!(
            "- Topic tasks: `topics/{}/tasks/`
",
            slug
        ));

        fs::write(&output_path, content)?;
        topic_count += 1;
    }

    println!("Generated {} new topic pages", topic_count);
    Ok(())
}

fn build_paper_link_map(papers: &[PaperMetadata]) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    for paper in papers {
        let page_stem = paper_page_stem(paper);
        let title = paper
            .title
            .as_ref()
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| filename_stem(&paper.filename));
        let link = format!("[[{}|{}]]", page_stem, title);
        for key in [
            paper.filename.as_str(),
            filename_stem(&paper.filename),
            title,
            page_stem.as_str(),
        ] {
            map.insert(normalize_lookup_key(key), link.clone());
        }
    }
    map
}

fn resolve_paper_wikilink(raw: &str, paper_link_map: &BTreeMap<String, String>) -> String {
    let cleaned = raw.trim().trim_matches('`');
    if cleaned.is_empty() || cleaned == "TODO" || cleaned == "_none_" {
        return "_unresolved_".to_string();
    }
    let key = normalize_lookup_key(cleaned);
    if let Some(link) = paper_link_map.get(&key) {
        return link.clone();
    }
    let stem = filename_stem(cleaned);
    if let Some(link) = paper_link_map.get(&normalize_lookup_key(stem)) {
        return link.clone();
    }
    format!("[[{}]]", cleaned)
}

fn normalize_lookup_key(value: &str) -> String {
    value
        .trim()
        .trim_end_matches(".md")
        .trim_end_matches(".pdf")
        .replace('\\', "/")
        .to_ascii_lowercase()
}

fn read_topic_title(topic_root: &Path) -> Option<String> {
    for name in ["README.md", "index.md", "scope.md"] {
        let path = topic_root.join(name);
        let Ok(content) = fs::read_to_string(path) else {
            continue;
        };
        for line in content.lines() {
            let trimmed = line.trim();
            if let Some(title) = trimmed.strip_prefix("# ") {
                let title = title.trim();
                if !title.is_empty() {
                    return Some(title.to_string());
                }
            }
        }
    }
    None
}

fn read_topic_literature_rows(path: &Path) -> Result<Vec<TopicLiteratureRow>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(path)?;
    let mut rows = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with('|') || trimmed.contains("|---") {
            continue;
        }
        let cells = trimmed
            .trim_matches('|')
            .split('|')
            .map(|cell| cell.trim().trim_matches('`'))
            .collect::<Vec<_>>();
        if cells.is_empty() {
            continue;
        }
        let paper = cells[0].trim();
        if paper.is_empty()
            || paper.eq_ignore_ascii_case("paper")
            || paper.eq_ignore_ascii_case("TODO")
        {
            continue;
        }
        rows.push(TopicLiteratureRow {
            paper: paper.to_string(),
            role: cells.get(1).copied().unwrap_or("candidate").to_string(),
            status: cells.get(2).copied().unwrap_or("candidate").to_string(),
        });
    }
    Ok(rows)
}

fn section_dir_for_paper(kb_path: &Path, filename: &str) -> std::path::PathBuf {
    let text_filename = format!(
        "raw__papers__{}",
        sanitize_section_component(filename_stem(filename))
    );
    kb_path.join("processing/sections").join(text_filename)
}

fn sanitize_section_component(value: &str) -> String {
    let cleaned = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch == '.' {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>();
    let trimmed = cleaned.trim_matches('_');
    if trimmed.is_empty() {
        "source".to_string()
    } else {
        trimmed.to_string()
    }
}

fn escape_table_cell(value: &str) -> String {
    value.replace('|', "\\|").replace('\n', " ")
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
    let topics_dir = kb_path.join("wiki/topics");
    let mut topic_names = Vec::new();

    if let Ok(entries) = fs::read_dir(&topics_dir) {
        for entry in entries.filter_map(|entry| entry.ok()) {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("md") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    topic_names.push(stem.to_string());
                }
            }
        }
    }
    topic_names.sort();

    let mut content = String::from("# Topics Index\n\n");
    content.push_str("Topics are higher-level story pages that link source papers, methods, concepts, and open questions.\n\n");
    content.push_str(&format!("Total topics: {}\n\n", topic_names.len()));
    content.push_str("## Topic Pages\n\n");
    if topic_names.is_empty() {
        content.push_str("No concrete topic pages have been generated yet. Run `kb topic init <topic>`, add rows to `topics/<topic>/literature.md`, then run `kb build-wiki`.\n\n");
    } else {
        for name in topic_names {
            content.push_str(&format!("- [[{}]]\n", name));
        }
        content.push_str("\n");
    }
    content.push_str("## LLM Narrative Contract\n\n");
    content.push_str("For each topic, the Manager/Worker LLM should build a short evidence-backed story: background, core problem, key papers, method lineage, open questions, and links back to paper pages.\n\n");
    content.push_str("## Suggested Workflow\n\n");
    content.push_str("1. Extract key information from every paper.\n");
    content.push_str("2. Review topic-local literature importance and relation candidates.\n");
    content.push_str("3. Draft the topic story in the matching `wiki/topics/<topic>.md` page.\n");
    content.push_str("4. Keep uncertain links explicit until a human reviewer confirms them.\n\n");
    content.push_str("---\n\n*This index is generated by `kb build-wiki` and can be edited manually after generation.*\n");

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
kb --wiki /path/to/literature-folder/knowledgebase init
kb --source /path/to/literature-folder --wiki /path/to/literature-folder/knowledgebase ingest --copy --recursive
kb --wiki /path/to/literature-folder/knowledgebase extract-metadata
kb --wiki /path/to/literature-folder/knowledgebase build-wiki
kb --wiki /path/to/literature-folder/knowledgebase status
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
├── interfaces/              # Disposable generated artifacts, reports, HTML, and logs
│   └── logs/
│       └── papers_metadata.json
├── processing/           # Intermediate processing files
│   ├── manifest.json     # Raw-file registry generated by kb status
│   └── proposals/        # Future AI edit proposals
└── references/           # Reference materials and templates
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
            journal: None,
            abstract_text: None,
            keywords: Vec::new(),
            missing_key_info: Vec::new(),
            needs_llm_key_info: false,
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
