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

    // Create wiki directories if they don't exist
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

    // 1. Build paper pages from metadata
    let metadata_path = kb_path.join("logs/papers_metadata.json");
    if metadata_path.exists() {
        let metadata_content = fs::read_to_string(&metadata_path)?;
        let papers: Vec<PaperMetadata> = serde_json::from_str(&metadata_content)?;

        println!("\nGenerating paper pages...");
        for paper in &papers {
            generate_paper_page(&kb_path, paper)?;
        }
        println!("Generated {} paper pages", papers.len());

        // Generate papers index
        generate_papers_index(&kb_path, &papers)?;
    } else {
        println!("No metadata found. Run 'extract-metadata' first.");
    }

    // 2. Build note pages from Notes directory
    build_note_pages(&kb_path)?;

    // 3. Generate indexes
    generate_concepts_index(&kb_path)?;
    generate_notes_index(&kb_path)?;
    generate_topics_index(&kb_path)?;
    generate_main_index(&kb_path)?;

    println!("\nWiki build complete!");
    Ok(())
}

fn generate_paper_page(kb_path: &Path, paper: &PaperMetadata) -> Result<()> {
    let title = paper.title.as_ref().map(|t| t.as_str()).unwrap_or("Untitled");
    let safe_title = sanitize_filename(title);

    let authors = paper.author.as_ref().map(|a| a.as_str()).unwrap_or("Unknown");
    let pages = paper.pages;
    let doi = paper.doi.as_ref().map(|d| format!("DOI: [{}](https://doi.org/{})", d, d))
        .unwrap_or_else(|| "".to_string());

    let content = format!(r#"# {title}

## Metadata

| Field | Value |
|-------|-------|
| File | [{filename}](../../raw/papers/{filename}) |
| Authors | {authors} |
| Pages | {pages} |
| DOI | {doi} |

## Abstract

*To be filled*

## Key Points

- Point 1
- Point 2
- Point 3

## Related

[[Droplet Generation]]
[[Microfluidics]]

## Notes

*Add your reading notes here*
"#,
        title = title,
        filename = paper.filename,
        authors = authors,
        pages = pages,
        doi = if !doi.is_empty() { &doi } else { "N/A" }
    );

    let output_path = kb_path.join(format!("wiki/papers/{}.md", safe_title));
    fs::write(&output_path, content)?;
    Ok(())
}

fn build_note_pages(kb_path: &Path) -> Result<()> {
    let notes_dir = kb_path.join("raw/Notes");

    if !notes_dir.exists() {
        println!("\nNotes directory not found, skipping...");
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
        let filename = path.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string();

        let content = format!(r#"# {filename}

## File Info

| Field | Value |
|-------|-------|
| Type | {file_type} |
| Path | {rel_path} |
| Size | {size} |

## Description

*To be filled*

## Notes

*Add your notes here*
"#,
        filename = filename,
        rel_path = path.strip_prefix(&notes_dir)
            .unwrap_or(path)
            .to_str()
            .unwrap_or(""),
        file_type = path.extension()
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
        let title = paper.title.as_ref().map(|t| t.as_str()).unwrap_or("Untitled");
        let safe_title = sanitize_filename(title);
        content.push_str(&format!("- [[{}]] - {}\n", safe_title, paper.filename));
    }

    let output_path = kb_path.join("wiki/indexes/papers_index.md");
    fs::write(&output_path, content)?;
    println!("Generated papers index");
    Ok(())
}

fn generate_concepts_index(kb_path: &Path) -> Result<()> {
    let content = r#"# Concepts Index

Overview of key concepts in droplet microfluidics.

## Core Concepts

### Droplet Generation
- [[Droplet Generation]]
- [[T-Junction]]
- [[Flow Focusing]]

### Fundamentals
- [[Microfluidics]]
- [[Emulsion]]
- [[Surface Tension]]

## Fluid Dynamics
- Capillary Number
- Reynolds Number
- Viscosity Ratio
- Shear Flow

## Device Types
- [[Lab on a Chip]]
- Digital Microfluidics
- Continuous Flow Microfluidics

## Applications
- [[Single-Cell Analysis]]
- [[Digital PCR]]
- High-Throughput Screening
- Nanoparticle Synthesis
- Drug Delivery
- Microdroplet applications

## Materials
- PDMS
- Glass
- Thermoplastics
- Hydrogels

---

*This index will be updated as new concepts are added*
"#;

    let output_path = kb_path.join("wiki/indexes/concepts_index.md");
    fs::write(&output_path, content)?;
    Ok(())
}

fn generate_notes_index(kb_path: &Path) -> Result<()> {
    // Get note count
    let notes_dir = kb_path.join("wiki/notes");
    let note_count = fs::read_dir(&notes_dir)
        .map(|entries| entries.filter_map(|e| e.ok()).count())
        .unwrap_or(0);

    let content = format!(r#"# Notes Index

Collection of notes and reference materials in knowledge base.

## Notes Summary

Total notes: {}

## Notes by Type

### Presentations
- 20102010年Miller-液滴生成闭环控制始祖_pptm.md
- 20172017年赵远锦综述_pptx.md
- Droplet 技术应用的现状和趋势_pptx.md
- 单细胞包裹改_ppt.md

### Documents
- Controllable geometry-mediated droplet fission using "off-the-shelf" capillary microfluidics device_doc.md
- Drag-induced Breakup Mechanism for Droplet Generation in Dripping within Flow Focusing Microfluidics.md
- Microdroplets： a sea of applications_doc.md
- T-Junction_Ren_docx.md
- 液滴形成及操控方式概述_txt.md

### Code
- patch_real_droplet_ellipse_c.md
- ilove_cpp.md
- ustc_cpp.md

### Images
- Image007.tif.md
- Image008.tif.md
- Image013.tif.md
- Image014.tif.md

### Archives
- 2020国自然申请_zip.md

## Key Topics Covered

1. **Droplet Generation**
   - T-junction mechanisms
   - Flow focusing configurations
   - Closed-loop control (Miller 2010)
   - Regime analysis

2. **Applications**
   - Single-cell encapsulation
   - Digital PCR
   - Microdroplet applications

3. **Review Materials**
   - 2017 comprehensive review by Zhao Yuanjin

---

*Notes are organized from raw materials in KnowledgeBase/raw/Notes/*
"#, note_count);

    let output_path = kb_path.join("wiki/indexes/notes_index.md");
    fs::write(&output_path, content)?;
    Ok(())
}

fn generate_topics_index(kb_path: &Path) -> Result<()> {
    let content = r#"# Topics Index

Organized topics for deeper exploration of droplet microfluidics.

## Research Topics

### Droplet Generation
- [[Droplet Microfluidics Overview]]
- Passive vs Active Methods
- Regime Analysis
- Scaling Laws

### Device Design
- Geometry Optimization
- Surface Treatments
- Multi-phase Systems
- 3D vs 2D Designs

### Applications
- [[Single-Cell Analysis]]
- [[Digital PCR]]
- High-Throughput Screening
- Nanoparticle Synthesis
- Drug Delivery
- Microdroplet applications

## Thematic Collections

### Historical Development
- Early droplet generation methods
- Evolution of device designs
- Key breakthrough papers

### Current Trends
- AI and Machine Learning Integration
- In-situ Analysis Techniques
- Multi-material Systems

### Emerging Areas
- Acoustic Manipulation
- Magnetic Control
- Optofluidics

## Cross-Cutting Topics

- Surface Chemistry
- Interfacial Phenomena
- Transport Processes
- Scale-up Strategies

---

*Topics will be expanded based on research needs and literature analysis.*
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

    let content = format!(r#"# Knowledge Base

Welcome to droplets microfluidics knowledge base.

## Overview

This knowledge base follows the Karpathy approach to local knowledge management:
- Original materials are stored in `raw/`
- Wiki pages are in `wiki/` as Markdown files
- Obsidian can be used for visualization and linking

## Running the CLI

### Direct commands (no LLM):
- `cli init` - Initialize knowledge base
- `cli extract-metadata` - Extract PDF metadata
- `cli build-wiki` - Build wiki pages
- `cli list papers` - List all papers
- `cli list notes` - List all notes
- `cli search <query>` - Search for papers/notes

### REPL mode (requires LLM integration):
- `cli repl` - Enter interactive mode
  - `kb> help` - Show available commands
  - `kb> ask <question>` - Ask a question (requires model-switch)
  - `kb> exit` - Exit REPL

## Indexes

- [[Papers Index]] - {} papers with metadata
- [[Notes Index]] - {} notes and reference materials
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

```
KnowledgeBase/
├── raw/
│   ├── papers/          # Original PDF papers
│   ├── Notes/           # Notes and reference materials
│   ├── images/             # Images and figures
│   ├── datasets/           # Data sets
│   └── repos/             # Code repositories
└── wiki/
    ├── papers/          # Paper wiki pages
    ├── notes/           # Note wiki pages
    ├── concepts/        # Concept pages
    ├── topics/         # Topic pages
    └── indexes/        # Navigation pages
└── logs/
    └── papers_metadata.json  # Extracted paper metadata
```

## Usage

### Bash Mode Commands

```bash
./cli.exe init                    # Initialize knowledge base
./cli.exe extract-metadata        # Extract PDF metadata
./cli.exe build-wiki             # Build wiki pages
./cli.exe list papers            # List papers
./cli.exe list notes             # List notes
./cli.exe search <query>        # Search for papers/notes
```

### REPL Mode Commands

```bash
./cli.exe repl                    # Enter interactive mode

kb> help                    # Show available commands
kb> ask "什么是 T-junction?"   # Ask a question (requires model-switch)
kb> list papers | head -10    # List papers (first 10)
kb> search "flow focusing"     # Search for papers
kb> exit                    # Exit
```

### Model-Switch Integration

The CLI integrates with model-switch for LLM capabilities. When model-switch is running, it will write responses to `.model_switch_output.json`.

**Default Model Configuration:**
- Model URL: http://scc.ustc.edu.cn/portal/api/ask
- API Key: DEEPSEEK_API_USTC
- Unit: USTC

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

fn sanitize_filename(name: &str) -> String {
    let mut result = String::new();
    for c in name.chars() {
        if c.is_alphanumeric() || c == ' ' || c == '-' || c == '_' {
            result.push(c);
        } else if c.is_whitespace() {
            result.push('_');
        } else {
            result.push('_');
        }
    }
    // Limit length
    if result.len() > 100 {
        result.truncate(100);
    }
    result.trim().to_string()
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
