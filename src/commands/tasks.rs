use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use clap::Args;
use regex::Regex;
use serde::Serialize;
use serde_json::Value;
use walkdir::WalkDir;

#[derive(Debug, Clone, Args)]
pub struct TasksArgs {
    #[arg(
        long,
        help = "Preview deferred task scan without writing LLM/tasks/ files"
    )]
    pub dry_run: bool,

    #[arg(long, help = "Alias for --dry-run")]
    pub preview: bool,

    #[arg(
        long,
        default_value_t = 0,
        help = "Maximum number of task groups to return. Use 0 for no limit."
    )]
    pub limit: usize,

    #[arg(long, help = "Print a machine-readable JSON task report")]
    pub json: bool,

    #[arg(
        long,
        help = "Write only LLM/tasks/index.md and task item files; skip the timestamped handoff snapshot"
    )]
    pub index_only: bool,

    #[arg(
        long,
        help = "Do not write or update LLM/tasks/index.md or LLM/tasks/items/*.md"
    )]
    pub no_index: bool,
}

#[derive(Debug, Clone, Serialize)]
struct DeferredTask {
    id: String,
    category: String,
    target_agent: String,
    goal: String,
    requirements: Vec<String>,
    files: Vec<String>,
    evidence: Vec<String>,
    source_command: String,
    priority: String,
}

#[derive(Debug, Clone, Serialize)]
struct TasksReport {
    schema_version: String,
    generated_by: String,
    generated_at: String,
    dry_run: bool,
    task_count: usize,
    returned_count: usize,
    limit: usize,
    report_path: Option<String>,
    index_path: Option<String>,
    task_item_dir: Option<String>,
    task_item_count: usize,
    tasks: Vec<DeferredTask>,
}

pub fn execute(custom_kb: Option<&Path>, args: &TasksArgs) -> Result<()> {
    let kb_path = crate::commands::init::get_kb_path(custom_kb);
    if !kb_path.exists() {
        return Err(anyhow!(
            "knowledge base path does not exist: {}. Run `kb init` first.",
            kb_path.display()
        ));
    }

    if args.index_only && args.no_index {
        return Err(anyhow!(
            "--index-only and --no-index cannot be used together"
        ));
    }

    let mut report = run_task_scan(&kb_path, args)?;
    let dry_run = args.dry_run || args.preview;

    if !dry_run {
        let mut report_path = None;
        if !args.index_only {
            let path = write_markdown_report(&kb_path, &report)?;
            report.report_path = Some(relative_path_string(&kb_path, &path));
            report_path = Some(path);
        }

        if !args.no_index {
            let item_dir = write_task_item_files(&kb_path, &report)?;
            report.task_item_dir = Some(relative_path_string(&kb_path, &item_dir));
            report.task_item_count = report.tasks.len();
            let index_path = write_task_index(&kb_path, &report, report_path.as_deref())?;
            report.index_path = Some(relative_path_string(&kb_path, &index_path));
        }
    }

    if args.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print_report(&report);
    }

    Ok(())
}

fn run_task_scan(kb_path: &Path, args: &TasksArgs) -> Result<TasksReport> {
    let mut tasks = Vec::new();

    if let Some(task) = scan_pdf_text_conversion_tasks(kb_path)? {
        tasks.push(task);
    }
    if let Some(task) = scan_paper_key_info_extraction_tasks(kb_path)? {
        tasks.push(task);
    }
    if let Some(task) = scan_paper_section_extraction_tasks(kb_path)? {
        tasks.push(task);
    }
    if let Some(task) = scan_reference_reconciliation_tasks(kb_path)? {
        tasks.push(task);
    }
    if let Some(task) = scan_bibliographic_index_review_tasks(kb_path)? {
        tasks.push(task);
    }
    if let Some(task) = scan_keyword_topic_relation_tasks(kb_path)? {
        tasks.push(task);
    }
    if let Some(task) = scan_topic_story_narrative_tasks(kb_path)? {
        tasks.push(task);
    }
    if let Some(task) = scan_wiki_source_traceability_tasks(kb_path)? {
        tasks.push(task);
    }
    if let Some(task) = scan_wiki_link_repair_tasks(kb_path)? {
        tasks.push(task);
    }

    let task_count = tasks.len();
    let returned_tasks = if args.limit > 0 {
        tasks.into_iter().take(args.limit).collect::<Vec<_>>()
    } else {
        tasks
    };

    Ok(TasksReport {
        schema_version: "0.7.37".to_string(),
        generated_by: "kb-cli tasks".to_string(),
        generated_at: chrono::Utc::now().to_rfc3339(),
        dry_run: args.dry_run || args.preview,
        task_count,
        returned_count: returned_tasks.len(),
        limit: args.limit,
        report_path: None,
        index_path: None,
        task_item_dir: None,
        task_item_count: 0,
        tasks: returned_tasks,
    })
}

fn scan_pdf_text_conversion_tasks(kb_path: &Path) -> Result<Option<DeferredTask>> {
    let raw_papers = kb_path.join("raw/papers");
    if !raw_papers.exists() {
        return Ok(None);
    }

    let text_dir = kb_path.join("processing/text");
    let mut files = Vec::new();
    let mut evidence = Vec::new();

    for pdf in collect_files_with_extensions(&raw_papers, &["pdf"])? {
        let output_path = text_output_path_for(kb_path, &text_dir, &pdf);
        let missing = !output_path.exists();
        let empty = output_path
            .metadata()
            .map(|metadata| metadata.len() == 0)
            .unwrap_or(false);
        if missing || empty {
            files.push(relative_path_string(kb_path, &pdf));
            evidence.push(format!(
                "{} -> {} ({})",
                relative_path_string(kb_path, &pdf),
                relative_path_string(kb_path, &output_path),
                if missing {
                    "missing extracted text"
                } else {
                    "empty extracted text"
                }
            ));
        }
    }

    if files.is_empty() {
        return Ok(None);
    }

    Ok(Some(DeferredTask {
        id: "pdf-text-conversion-001".to_string(),
        category: "text_conversion".to_string(),
        target_agent: "PDF Text Conversion Agent".to_string(),
        goal: "Convert source PDFs that do not yet have usable extracted text into clean text files under processing/text/.".to_string(),
        requirements: vec![
            "Do not rewrite the original raw/ files.".to_string(),
            "Prefer deterministic extraction first; use OCR only when the PDF is scanned or image-based.".to_string(),
            "Use LLM cleanup only when text order, layout, tables, or formulas require semantic repair.".to_string(),
            "Write converted text to processing/text/ and record failures explicitly.".to_string(),
        ],
        files,
        evidence,
        source_command: "kb extract-text".to_string(),
        priority: "high".to_string(),
    }))
}

fn scan_paper_key_info_extraction_tasks(kb_path: &Path) -> Result<Option<DeferredTask>> {
    let raw_papers = kb_path.join("raw/papers");
    if !raw_papers.exists() {
        return Ok(None);
    }

    let metadata_by_filename = load_paper_metadata_values(kb_path)?;
    let text_dir = kb_path.join("processing/text");
    let sections_dir = kb_path.join("processing/sections");
    let profile_dir = kb_path.join("processing/paper_profiles");
    let wiki_papers_dir = kb_path.join("wiki/papers");
    let mut files = Vec::new();
    let mut evidence = Vec::new();

    for pdf in collect_files_with_extensions(&raw_papers, &["pdf"])? {
        let filename = pdf
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("")
            .to_string();
        let stem = pdf
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("paper");
        let text_path = text_output_path_for(kb_path, &text_dir, &pdf);
        let section_dir = section_output_dir_for(kb_path, &sections_dir, &text_path);
        let intro_path = section_dir.join("introduction.txt");
        let profile_path = profile_dir.join(format!("{}.md", sanitize_path_component(stem)));
        let wiki_page = wiki_papers_dir.join(format!("{}.md", sanitize_path_component(stem)));

        let meta = metadata_by_filename.get(&filename);
        let mut missing = Vec::new();
        if metadata_string(meta, "title").is_none() {
            missing.push("title");
        }
        if metadata_string(meta, "author").is_none() {
            missing.push("authors");
        }
        if metadata_string(meta, "journal").is_none() {
            missing.push("journal");
        }
        if metadata_string(meta, "abstract_text").is_none() {
            missing.push("abstract");
        }
        if metadata_keywords_empty(meta) {
            missing.push("keywords");
        }
        let intro_missing_or_short = !intro_path.exists()
            || intro_path
                .metadata()
                .map(|metadata| metadata.len() < 300)
                .unwrap_or(true);
        if intro_missing_or_short {
            missing.push("introduction");
        }

        if missing.is_empty() && profile_path.exists() {
            continue;
        }

        files.push(relative_path_string(kb_path, &pdf));
        files.push(relative_path_string(kb_path, &text_path));
        files.push(relative_path_string(kb_path, &intro_path));
        files.push(relative_path_string(kb_path, &profile_path));
        files.push(relative_path_string(kb_path, &wiki_page));
        evidence.push(format!(
            "{} missing/uncertain key info: {}",
            relative_path_string(kb_path, &pdf),
            missing.join(", ")
        ));
    }

    files.sort();
    files.dedup();

    if evidence.is_empty() {
        return Ok(None);
    }

    Ok(Some(DeferredTask {
        id: "paper-key-info-extraction-001".to_string(),
        category: "paper_key_information".to_string(),
        target_agent: "Paper Key-Info Extraction Worker LLM".to_string(),
        goal: "Extract each paper's title, authors, journal, abstract, keywords, and introduction, then write reviewable paper-profile notes and update the corresponding Wiki paper pages.".to_string(),
        requirements: vec![
            "Use the PDF, extracted text, and extracted sections as evidence; do not invent missing bibliographic fields.".to_string(),
            "For each paper, write a structured profile under processing/paper_profiles/<paper>.md with title, authors, journal, abstract, keywords, introduction summary, and evidence snippets.".to_string(),
            "Update wiki/papers/<paper>.md only with evidence-backed fields and preserve uncertainty explicitly.".to_string(),
            "Do not treat PDF document-info metadata as equivalent to article abstract or journal unless the paper text confirms it.".to_string(),
            "Keep links from paper pages to topic pages and from topic pages back to paper pages in WikiLink form where possible.".to_string(),
        ],
        files,
        evidence,
        source_command: "kb extract-metadata + kb extract-text + kb extract-sections + kb build-wiki".to_string(),
        priority: "high".to_string(),
    }))
}

fn scan_paper_section_extraction_tasks(kb_path: &Path) -> Result<Option<DeferredTask>> {
    let text_dir = kb_path.join("processing/text");
    if !text_dir.exists() {
        return Ok(None);
    }

    let sections_dir = kb_path.join("processing/sections");
    let mut files = Vec::new();
    let mut evidence = Vec::new();

    for text_file in collect_files_with_extensions(&text_dir, &["txt", "md"])? {
        let article_dir = section_output_dir_for(kb_path, &sections_dir, &text_file);
        let intro_path = article_dir.join("introduction.txt");
        let refs_path = article_dir.join("references.txt");
        let intro_missing_or_empty = !intro_path.exists()
            || intro_path
                .metadata()
                .map(|metadata| metadata.len() < 120)
                .unwrap_or(true);
        let refs_missing_or_empty = !refs_path.exists()
            || refs_path
                .metadata()
                .map(|metadata| metadata.len() < 120)
                .unwrap_or(true);

        if intro_missing_or_empty || refs_missing_or_empty {
            let rel_text = relative_path_string(kb_path, &text_file);
            files.push(rel_text.clone());
            evidence.push(format!(
                "{} -> {} / {} (introduction: {}, references: {})",
                rel_text,
                relative_path_string(kb_path, &intro_path),
                relative_path_string(kb_path, &refs_path),
                if intro_missing_or_empty {
                    "missing_or_short"
                } else {
                    "present"
                },
                if refs_missing_or_empty {
                    "missing_or_short"
                } else {
                    "present"
                },
            ));
        }
    }

    if files.is_empty() {
        return Ok(None);
    }

    Ok(Some(DeferredTask {
        id: "paper-section-extraction-001".to_string(),
        category: "paper_section_extraction".to_string(),
        target_agent: "Paper Section OCR/LLM Worker".to_string(),
        goal: "Recover Introduction and References sections for papers whose deterministic section slicing is missing or incomplete.".to_string(),
        requirements: vec![
            "Run `kb extract-sections` first and use its output as the deterministic baseline.".to_string(),
            "For scanned/image-based PDFs, use explicit OCR or multimodal LLM reading; do not pretend deterministic extraction succeeded.".to_string(),
            "Write recovered sections to processing/sections/<source-key>/introduction.txt and references.txt.".to_string(),
            "Preserve original wording as much as possible; do not summarize these section text files.".to_string(),
            "Record uncertainty and extraction notes in section_manifest.md.".to_string(),
            "Do not modify raw/ source files.".to_string(),
        ],
        files,
        evidence,
        source_command: "kb extract-sections".to_string(),
        priority: "high".to_string(),
    }))
}

fn scan_reference_reconciliation_tasks(kb_path: &Path) -> Result<Option<DeferredTask>> {
    let text_dir = kb_path.join("processing/text");
    if !text_dir.exists() {
        return Ok(None);
    }

    let heading_re =
        Regex::new(r"(?i)^\s*(references|bibliography|works cited|参考文献|参考资料|文献)\s*$")?;
    let doi_re = Regex::new(r"(?i)\b10\.\d{4,9}/[-._;()/:A-Z0-9]+\b")?;
    let numbered_entry_re = Regex::new(r"^\s*(\[\s*\d+\s*\]|\d+\s*[\.)])\s+\S+")?;

    let mut file_evidence: BTreeMap<String, usize> = BTreeMap::new();
    let mut evidence = Vec::new();

    for text_file in collect_files_with_extensions(&text_dir, &["txt", "md"])? {
        let Ok(content) = fs::read_to_string(&text_file) else {
            continue;
        };
        let mut local_hits = 0usize;
        for (line_idx, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if heading_re.is_match(trimmed)
                || doi_re.is_match(trimmed)
                || numbered_entry_re.is_match(trimmed)
            {
                local_hits += 1;
                if evidence.len() < 20 {
                    evidence.push(format!(
                        "{}:{}: {}",
                        relative_path_string(kb_path, &text_file),
                        line_idx + 1,
                        trimmed.chars().take(160).collect::<String>()
                    ));
                }
            }
        }
        if local_hits > 0 {
            file_evidence.insert(relative_path_string(kb_path, &text_file), local_hits);
        }
    }

    if file_evidence.is_empty() {
        return Ok(None);
    }

    let files = file_evidence
        .iter()
        .map(|(path, count)| format!("{path} ({count} reference hints)"))
        .collect::<Vec<_>>();

    Ok(Some(DeferredTask {
        id: "reference-reconciliation-001".to_string(),
        category: "reference_processing".to_string(),
        target_agent: "Reference Reconciliation Agent".to_string(),
        goal: "Turn deterministic reference hints into a cleaner reference inventory, while avoiding unsupported claims about citation identity.".to_string(),
        requirements: vec![
            "Use kb refs output as evidence, not as a final citation graph.".to_string(),
            "Normalize DOI values where possible and keep original lines for auditability.".to_string(),
            "Do not merge references unless title/DOI/author evidence is sufficient.".to_string(),
            "Return a table with source file, reference string, DOI if present, confidence, and unresolved issues.".to_string(),
        ],
        files,
        evidence,
        source_command: "kb refs".to_string(),
        priority: "medium".to_string(),
    }))
}

fn scan_bibliographic_index_review_tasks(kb_path: &Path) -> Result<Option<DeferredTask>> {
    let refs_dir = kb_path.join("processing/refs");
    if !refs_dir.exists() {
        return Ok(None);
    }

    let mut files = Vec::new();
    let mut evidence = Vec::new();

    for report in collect_files_with_extensions(&refs_dir, &["md"])? {
        let Some(name) = report.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        if !name.starts_with("refs_index_") {
            continue;
        }
        let Ok(content) = fs::read_to_string(&report) else {
            continue;
        };
        let mut local_hits = 0usize;
        for line in content.lines() {
            if line.contains("| `candidate` |")
                || line.contains("| `ambiguous` |")
                || line.contains("| `missing` |")
            {
                local_hits += 1;
                if evidence.len() < 20 {
                    evidence.push(format!(
                        "{}: {}",
                        relative_path_string(kb_path, &report),
                        line.chars().take(180).collect::<String>()
                    ));
                }
            }
        }
        if local_hits > 0 {
            files.push(format!(
                "{} ({} uncertain index relations)",
                relative_path_string(kb_path, &report),
                local_hits
            ));
        }
    }

    if files.is_empty() {
        return Ok(None);
    }

    Ok(Some(DeferredTask {
        id: "bibliographic-index-review-001".to_string(),
        category: "literature_relationships".to_string(),
        target_agent: "Human Reference Index Reviewer".to_string(),
        goal:
            "Review uncertain bibliographic index relation candidates generated by kb refs-index."
                .to_string(),
        requirements: vec![
            "Confirm, reject, or mark missing each candidate / ambiguous / missing relation."
                .to_string(),
            "Use DOI, title, author, journal, volume, pages, and date evidence where available."
                .to_string(),
            "Do not let an LLM serve as the final guarantee for bibliographic identity."
                .to_string(),
            "Record completed review decisions with kb memory.".to_string(),
        ],
        files,
        evidence,
        source_command: "kb refs-index".to_string(),
        priority: "high".to_string(),
    }))
}

fn scan_keyword_topic_relation_tasks(kb_path: &Path) -> Result<Option<DeferredTask>> {
    let keywords_dir = kb_path.join("processing/keywords");
    if !keywords_dir.exists() {
        return Ok(None);
    }

    let mut files = Vec::new();
    let mut evidence = Vec::new();

    for report in collect_files_with_extensions(&keywords_dir, &["md"])? {
        let Some(name) = report.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        if !name.starts_with("keywords_") {
            continue;
        }
        let Ok(content) = fs::read_to_string(&report) else {
            continue;
        };
        let mut local_hits = 0usize;
        for line in content.lines() {
            if line.contains("| `candidate` |")
                || line.contains("keyword/topic relation")
                || line.contains("shared terms")
            {
                local_hits += 1;
                if evidence.len() < 20 {
                    evidence.push(format!(
                        "{}: {}",
                        relative_path_string(kb_path, &report),
                        line.chars().take(180).collect::<String>()
                    ));
                }
            }
        }
        if local_hits > 0 {
            files.push(format!(
                "{} ({} keyword/topic relation hints)",
                relative_path_string(kb_path, &report),
                local_hits
            ));
        }
    }

    if files.is_empty() {
        return Ok(None);
    }

    Ok(Some(DeferredTask {
        id: "keyword-topic-review-001".to_string(),
        category: "literature_relationships".to_string(),
        target_agent: "Keyword Topic Relation Worker".to_string(),
        goal: "Review keyword/topic co-occurrence candidates and decide whether they represent meaningful literature relationships.".to_string(),
        requirements: vec![
            "Do not treat shared keywords alone as confirmed scientific idea relations.".to_string(),
            "Use source files, evidence lines, and domain context before promoting a relation.".to_string(),
            "Keep uncertain relations as candidates and record unresolved issues.".to_string(),
            "Record completed decisions with kb memory.".to_string(),
        ],
        files,
        evidence,
        source_command: "kb keywords".to_string(),
        priority: "medium".to_string(),
    }))
}

fn scan_topic_story_narrative_tasks(kb_path: &Path) -> Result<Option<DeferredTask>> {
    let topics_dir = kb_path.join("topics");
    if !topics_dir.exists() {
        return Ok(None);
    }

    let mut files = Vec::new();
    let mut evidence = Vec::new();

    for entry in fs::read_dir(&topics_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let slug = entry.file_name().to_string_lossy().to_string();
        let topic_root = entry.path();
        let literature = topic_root.join("literature.md");
        let scope = topic_root.join("scope.md");
        let topic_wiki_page = kb_path.join("wiki/topics").join(format!("{}.md", slug));
        let graph = topic_root.join("graph/topic_graph.md");
        let task_dir = topic_root.join("tasks");

        let has_literature = literature
            .metadata()
            .map(|metadata| metadata.len() > 80)
            .unwrap_or(false);
        let narrative_missing = !topic_wiki_page.exists()
            || fs::read_to_string(&topic_wiki_page)
                .map(|content| {
                    content.contains("To be drafted by a Manager/Worker LLM")
                        || !content.contains("## Story Narrative")
                })
                .unwrap_or(true);

        if has_literature && narrative_missing {
            files.push(relative_path_string(kb_path, &literature));
            files.push(relative_path_string(kb_path, &scope));
            files.push(relative_path_string(kb_path, &graph));
            files.push(relative_path_string(kb_path, &topic_wiki_page));
            files.push(relative_path_string(kb_path, &task_dir));
            evidence.push(format!(
                "topic `{}` has topic-local literature but no completed Wiki story narrative",
                slug
            ));
        }
    }

    files.sort();
    files.dedup();

    if evidence.is_empty() {
        return Ok(None);
    }

    Ok(Some(DeferredTask {
        id: "topic-story-narrative-001".to_string(),
        category: "topic_story_narrative".to_string(),
        target_agent: "Topic Story Manager/Worker LLM".to_string(),
        goal: "Build concise topic Wiki narratives from topic-local literature, importance hints, and relation candidates, while preserving links back to supporting papers.".to_string(),
        requirements: vec![
            "Use topics/<topic>/literature.md as the starting bibliography and link every cited paper back to its wiki/papers page where possible.".to_string(),
            "Explain the topic as a short story: background, key problem, method lineage, core papers, competing ideas, and open questions.".to_string(),
            "Use topic relation files and graph outputs as hints, not final truth; mark uncertain edges as candidates.".to_string(),
            "Keep the final narrative in wiki/topics/<topic>.md under `## Story Narrative`.".to_string(),
            "Do not fabricate citations; unresolved papers should remain as unresolved WikiLinks or notes for human review.".to_string(),
        ],
        files,
        evidence,
        source_command: "kb topic build <topic> + kb build-wiki + kb view --wiki".to_string(),
        priority: "high".to_string(),
    }))
}

fn scan_wiki_source_traceability_tasks(kb_path: &Path) -> Result<Option<DeferredTask>> {
    let wiki_dir = kb_path.join("wiki");
    if !wiki_dir.exists() {
        return Ok(None);
    }

    let mut files = Vec::new();
    let mut evidence = Vec::new();

    for page in collect_files_with_extensions(&wiki_dir, &["md"])? {
        let rel = relative_path_string(kb_path, &page);
        let source_derived = rel.starts_with("wiki/papers/") || rel.starts_with("wiki/notes/");
        if !source_derived {
            continue;
        }
        let Ok(content) = fs::read_to_string(&page) else {
            continue;
        };
        if !content.contains("source_files:") && !content.contains("source_ids:") {
            files.push(rel.clone());
            evidence.push(format!(
                "{rel}: missing source_files/source_ids front matter"
            ));
        }
    }

    if files.is_empty() {
        return Ok(None);
    }

    Ok(Some(DeferredTask {
        id: "wiki-source-traceability-001".to_string(),
        category: "wiki_traceability".to_string(),
        target_agent: "Wiki Source Traceability Agent".to_string(),
        goal: "Add or repair source traceability for source-derived Wiki pages without inventing sources.".to_string(),
        requirements: vec![
            "Only add source_files/source_ids when there is clear evidence in raw/ or processing/manifest.json.".to_string(),
            "Do not invent citations or source IDs.".to_string(),
            "Prefer small front matter edits over rewriting page content.".to_string(),
            "After edits, run kb sync-wiki and kb lint-static.".to_string(),
        ],
        files,
        evidence,
        source_command: "kb lint-static".to_string(),
        priority: "high".to_string(),
    }))
}

fn scan_wiki_link_repair_tasks(kb_path: &Path) -> Result<Option<DeferredTask>> {
    let wiki_dir = kb_path.join("wiki");
    if !wiki_dir.exists() {
        return Ok(None);
    }

    let pages = collect_files_with_extensions(&wiki_dir, &["md"])?;
    let mut page_keys = BTreeSet::new();
    for page in &pages {
        let rel = relative_path_string(kb_path, page);
        page_keys.insert(normalize_page_key(&rel));
        if let Some(stem) = page.file_stem().and_then(|s| s.to_str()) {
            page_keys.insert(normalize_page_key(stem));
        }
    }

    let wikilink_re = Regex::new(r"\[\[([^\]\n]+)\]\]")?;
    let mut files = BTreeSet::new();
    let mut evidence = Vec::new();

    for page in &pages {
        let Ok(content) = fs::read_to_string(page) else {
            continue;
        };
        for (line_idx, line) in content.lines().enumerate() {
            for captures in wikilink_re.captures_iter(line) {
                let Some(target_match) = captures.get(1) else {
                    continue;
                };
                let target = clean_wikilink_target(target_match.as_str());
                if target.is_empty() {
                    continue;
                }
                if !page_keys.contains(&normalize_page_key(&target)) {
                    let rel = relative_path_string(kb_path, page);
                    files.insert(rel.clone());
                    if evidence.len() < 40 {
                        evidence.push(format!(
                            "{}:{}: broken link [[{}]]",
                            rel,
                            line_idx + 1,
                            target
                        ));
                    }
                }
            }
        }
    }

    if files.is_empty() {
        return Ok(None);
    }

    Ok(Some(DeferredTask {
        id: "wiki-link-repair-001".to_string(),
        category: "wiki_structure".to_string(),
        target_agent: "Wiki Link Repair Agent".to_string(),
        goal: "Repair broken WikiLinks by either creating missing concept pages or correcting incorrect link targets.".to_string(),
        requirements: vec![
            "Do not create empty placeholder pages unless there is a clear reason and source evidence.".to_string(),
            "Prefer correcting spelling/path errors before creating new pages.".to_string(),
            "When creating a concept page, include source traceability or mark it as a human/LLM draft needing review.".to_string(),
            "Run kb lint-static again after edits.".to_string(),
        ],
        files: files.into_iter().collect(),
        evidence,
        source_command: "kb lint-static".to_string(),
        priority: "medium".to_string(),
    }))
}

fn load_paper_metadata_values(kb_path: &Path) -> Result<BTreeMap<String, Value>> {
    let path = paper_metadata_path(kb_path);
    if !path.exists() {
        return Ok(BTreeMap::new());
    }
    let content = fs::read_to_string(path)?;
    let value: Value = serde_json::from_str(&content)?;
    let mut map = BTreeMap::new();
    if let Some(items) = value.as_array() {
        for item in items {
            let Some(filename) = item
                .get("filename")
                .and_then(|value| value.as_str())
                .map(|value| value.to_string())
            else {
                continue;
            };
            map.insert(filename, item.clone());
        }
    }
    Ok(map)
}

fn paper_metadata_path(kb_path: &Path) -> PathBuf {
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

fn metadata_string(meta: Option<&Value>, key: &str) -> Option<String> {
    meta.and_then(|value| value.get(key))
        .and_then(|value| value.as_str())
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string())
}

fn metadata_keywords_empty(meta: Option<&Value>) -> bool {
    let Some(value) = meta.and_then(|value| value.get("keywords")) else {
        return true;
    };
    if let Some(items) = value.as_array() {
        return items
            .iter()
            .filter_map(|item| item.as_str())
            .all(|item| item.trim().is_empty());
    }
    value
        .as_str()
        .map(|value| value.trim().is_empty())
        .unwrap_or(true)
}

fn collect_files_with_extensions(root: &Path, exts: &[&str]) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    if root.is_file() {
        if has_extension(root, exts) {
            files.push(root.to_path_buf());
        }
        return Ok(files);
    }

    for entry in WalkDir::new(root)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
    {
        let path = entry.path();
        if has_extension(path, exts) {
            files.push(path.to_path_buf());
        }
    }
    files.sort();
    Ok(files)
}

fn has_extension(path: &Path, exts: &[&str]) -> bool {
    let Some(ext) = path.extension().and_then(|s| s.to_str()) else {
        return false;
    };
    let ext = ext.to_ascii_lowercase();
    exts.iter().any(|candidate| *candidate == ext)
}

fn section_output_dir_for(kb_path: &Path, output_dir: &Path, source: &Path) -> PathBuf {
    let text_root = kb_path.join("processing/text");
    let rel = source
        .strip_prefix(&text_root)
        .or_else(|_| source.strip_prefix(kb_path))
        .unwrap_or(source);
    let mut stem = rel.to_string_lossy().replace('\\', "__").replace('/', "__");
    if let Some(idx) = stem.rfind('.') {
        stem.truncate(idx);
    }
    output_dir.join(sanitize_path_component(&stem))
}
fn sanitize_path_component(value: &str) -> String {
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

fn text_output_path_for(kb_path: &Path, output_dir: &Path, source: &Path) -> PathBuf {
    let rel = source.strip_prefix(kb_path).unwrap_or(source);
    let mut filename = rel.to_string_lossy().replace('\\', "__").replace('/', "__");
    if let Some(idx) = filename.rfind('.') {
        filename.truncate(idx);
    }
    output_dir.join(format!("{filename}.txt"))
}

fn clean_wikilink_target(raw: &str) -> String {
    raw.split('|')
        .next()
        .unwrap_or(raw)
        .split('#')
        .next()
        .unwrap_or(raw)
        .trim()
        .trim_end_matches(".md")
        .to_string()
}

fn normalize_page_key(value: &str) -> String {
    value
        .trim()
        .trim_end_matches(".md")
        .replace('\\', "/")
        .to_ascii_lowercase()
}

fn relative_path_string(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn write_markdown_report(kb_path: &Path, report: &TasksReport) -> Result<PathBuf> {
    let tasks_dir = kb_path.join("LLM/tasks");
    fs::create_dir_all(&tasks_dir)?;
    let stamp = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();
    let path = tasks_dir.join(format!("llm_tasks_{stamp}.md"));
    fs::write(&path, render_markdown_report(report))?;
    Ok(path)
}

fn write_task_item_files(kb_path: &Path, report: &TasksReport) -> Result<PathBuf> {
    let item_dir = kb_path.join("LLM/tasks/items");
    fs::create_dir_all(&item_dir)?;

    for task in &report.tasks {
        let filename = format!("{}.md", sanitize_task_id(&task.id));
        let path = item_dir.join(filename);
        fs::write(path, render_task_item(task))?;
    }

    Ok(item_dir)
}

fn render_task_item(task: &DeferredTask) -> String {
    let mut out = String::new();
    out.push_str(&format!("# Worker Task: {}\n\n", task.id));
    out.push_str("This is a bounded Worker LLM / human task generated by `kb tasks`.\n\n");
    out.push_str("## Task Metadata\n\n");
    out.push_str(&format!("- Task ID: `{}`\n", task.id));
    out.push_str("- Status: `pending`\n");
    out.push_str(&format!("- Priority: `{}`\n", task.priority));
    out.push_str(&format!("- Category: `{}`\n", task.category));
    out.push_str(&format!("- Target agent: `{}`\n", task.target_agent));
    out.push_str(&format!("- Source command: `{}`\n\n", task.source_command));

    out.push_str("## Goal\n\n");
    out.push_str(&task.goal);
    out.push_str("\n\n");

    out.push_str("## Requirements\n\n");
    for requirement in &task.requirements {
        out.push_str(&format!("- {}\n", requirement));
    }
    out.push_str("\n");

    out.push_str("## Input Files\n\n");
    if task.files.is_empty() {
        out.push_str("No specific files were detected. Re-run the source command or inspect the knowledge base manually.\n");
    } else {
        for file in &task.files {
            out.push_str(&format!("- `{}`\n", file));
        }
    }
    out.push_str("\n");

    out.push_str("## Evidence\n\n");
    if task.evidence.is_empty() {
        out.push_str(
            "No evidence lines were collected. Treat this task as a planning hint, not as proof.\n",
        );
    } else {
        for item in &task.evidence {
            out.push_str(&format!("- {}\n", item));
        }
    }
    out.push_str("\n");

    out.push_str("## Worker Output Rules\n\n");
    out.push_str("- Work only on the files listed above unless the Manager LLM or human maintainer expands the scope.\n");
    out.push_str("- Preserve uncertainty; do not turn candidate evidence into final knowledge without review.\n");
    out.push_str("- List every changed or created file.\n");
    out.push_str("- Return unresolved issues and follow-up recommendations.\n");
    out.push_str("- Do not modify `raw/` source materials.\n");
    out.push_str("- Do not commit changes directly; leave the result for Git diff review.\n\n");

    out.push_str("## Completion\n\n");
    out.push_str("After the work is reviewed and accepted, record it with:\n\n");
    out.push_str("```bash\n");
    out.push_str(&format!(
        "kb memory --task-id {} --summary \"<what changed>\"\n",
        task.id
    ));
    out.push_str("```\n");

    out
}

fn write_task_index(
    kb_path: &Path,
    report: &TasksReport,
    report_snapshot: Option<&Path>,
) -> Result<PathBuf> {
    let tasks_dir = kb_path.join("LLM/tasks");
    fs::create_dir_all(&tasks_dir)?;
    let path = tasks_dir.join("index.md");
    let content = render_task_index(kb_path, report, report_snapshot)?;
    fs::write(&path, content)?;
    Ok(path)
}

fn render_task_index(
    kb_path: &Path,
    report: &TasksReport,
    report_snapshot: Option<&Path>,
) -> Result<String> {
    let topic_queues = collect_topic_review_queues(kb_path)?;
    let task_reports = collect_existing_task_reports(kb_path)?;

    let mut out = String::new();
    out.push_str("# LLM Task Index\n\n");
    out.push_str("This is the Manager LLM task dashboard for the local LLM Wiki.\n\n");
    out.push_str(&format!("- Generated by: `{}`\n", report.generated_by));
    out.push_str(&format!("- Generated at: `{}`\n", report.generated_at));
    out.push_str(&format!("- Schema version: `{}`\n", report.schema_version));
    out.push_str("- Intended reader: `Manager LLM / human maintainer`\n");
    if let Some(path) = report_snapshot {
        out.push_str(&format!(
            "- Current handoff snapshot: `{}`\n",
            relative_path_string(kb_path, path)
        ));
    }
    out.push_str("- Worker task directory: `LLM/tasks/items/`\n");
    out.push_str("- Completed memory: `LLM/memory/completed_tasks.md`\n\n");

    out.push_str("## Manager LLM Operating Rules\n\n");
    out.push_str("1. Read this index first.\n");
    out.push_str("2. Select one pending task or review queue at a time.\n");
    out.push_str("3. Assign bounded work to a Worker LLM or human reviewer.\n");
    out.push_str("4. Do not let Worker LLMs browse the whole knowledge base unless the task explicitly says so.\n");
    out.push_str("5. Require evidence, uncertainty, changed-file lists, and follow-up notes.\n");
    out.push_str(
        "6. After human/Git review accepts the result, record completion with `kb memory`.\n\n",
    );

    out.push_str("## Pending Worker Task Items\n\n");
    if report.tasks.is_empty() {
        out.push_str("No pending deferred Worker task groups were detected by the current `kb tasks` scan.\n\n");
    } else {
        out.push_str(
            "| priority | task_id | target_agent | category | source_command | task_file |\n",
        );
        out.push_str("|---|---|---|---|---|---|\n");
        for task in &report.tasks {
            let task_file = format!("LLM/tasks/items/{}.md", sanitize_task_id(&task.id));
            out.push_str(&format!(
                "| `{}` | `{}` | {} | `{}` | `{}` | `{}` |\n",
                table_cell(&task.priority),
                table_cell(&task.id),
                table_cell(&task.target_agent),
                table_cell(&task.category),
                table_cell(&task.source_command),
                table_cell(&task_file)
            ));
        }
        out.push_str("\n");
    }

    out.push_str("## Topic Review Queues\n\n");
    if topic_queues.is_empty() {
        out.push_str(
            "No topic review queues were found. Generate one with `kb topic review <topic>`.\n\n",
        );
    } else {
        out.push_str("| topic | review_queue | review_summary |\n");
        out.push_str("|---|---|---|\n");
        for queue in &topic_queues {
            out.push_str(&format!(
                "| `{}` | `{}` | `{}` |\n",
                table_cell(&queue.topic),
                table_cell(&queue.queue_path),
                table_cell(queue.summary_path.as_deref().unwrap_or("missing"))
            ));
        }
        out.push_str("\n");
    }

    out.push_str("## Existing Task and Audit Reports\n\n");
    if task_reports.is_empty() {
        out.push_str("No existing task or audit review reports were found under `LLM/tasks/`.\n\n");
    } else {
        out.push_str("| file | note |\n");
        out.push_str("|---|---|\n");
        for path in &task_reports {
            out.push_str(&format!(
                "| `{}` | Historical handoff snapshot or audit-review task |\n",
                table_cell(path)
            ));
        }
        out.push_str("\n");
    }

    out.push_str("## Recommended Manager LLM Loop\n\n");
    out.push_str("```text\n");
    out.push_str("LLM/tasks/index.md\n");
    out.push_str("        ↓\n");
    out.push_str("choose one pending task or topic review queue\n");
    out.push_str("        ↓\n");
    out.push_str("assign a bounded Worker LLM / human task\n");
    out.push_str("        ↓\n");
    out.push_str("review changed files with Git diff\n");
    out.push_str("        ↓\n");
    out.push_str("record accepted work with kb memory\n");
    out.push_str("```\n\n");

    out.push_str("## Boundary\n\n");
    out.push_str("This index does not authorize hidden LLM execution. It is a routing document. The Manager LLM may use it to assign work, but every Worker output should remain reviewable by a human.\n");

    Ok(out)
}

#[derive(Debug, Clone)]
struct TopicReviewQueueRef {
    topic: String,
    queue_path: String,
    summary_path: Option<String>,
}

fn collect_topic_review_queues(kb_path: &Path) -> Result<Vec<TopicReviewQueueRef>> {
    let topics_dir = kb_path.join("topics");
    let mut queues = Vec::new();
    if !topics_dir.exists() {
        return Ok(queues);
    }

    for entry in fs::read_dir(topics_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let topic = entry.file_name().to_string_lossy().to_string();
        let topic_path = entry.path();
        let queue = topic_path.join("review/review_queue.md");
        if !queue.exists() {
            continue;
        }
        let summary = topic_path.join("review/review_summary.md");
        queues.push(TopicReviewQueueRef {
            topic,
            queue_path: relative_path_string(kb_path, &queue),
            summary_path: summary
                .exists()
                .then(|| relative_path_string(kb_path, &summary)),
        });
    }
    queues.sort_by(|a, b| a.topic.cmp(&b.topic));
    Ok(queues)
}

fn collect_existing_task_reports(kb_path: &Path) -> Result<Vec<String>> {
    let tasks_dir = kb_path.join("LLM/tasks");
    let mut reports = Vec::new();
    if !tasks_dir.exists() {
        return Ok(reports);
    }

    for entry in fs::read_dir(tasks_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        if name == "index.md" || !name.ends_with(".md") {
            continue;
        }
        reports.push(relative_path_string(kb_path, &path));
    }
    reports.sort();
    Ok(reports)
}

fn sanitize_task_id(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

fn table_cell(value: &str) -> String {
    value.replace('|', "\\|").replace('\n', " ")
}

fn render_markdown_report(report: &TasksReport) -> String {
    let mut out = String::new();
    out.push_str("# LLM / Agent Task Handoff List\n\n");
    out.push_str(&format!("- Generated by: `{}`\n", report.generated_by));
    out.push_str(&format!("- Generated at: `{}`\n", report.generated_at));
    out.push_str(&format!("- Task groups: `{}`\n\n", report.task_count));

    if report.tasks.is_empty() {
        out.push_str("No deferred LLM/agent task groups were detected.\n");
        return out;
    }

    for task in &report.tasks {
        out.push_str(&format!("## {}\n\n", task.id));
        out.push_str(&format!("- Category: `{}`\n", task.category));
        out.push_str(&format!("- Target agent: `{}`\n", task.target_agent));
        out.push_str(&format!("- Source command: `{}`\n", task.source_command));
        out.push_str(&format!("- Priority: `{}`\n\n", task.priority));
        out.push_str(&format!("**Goal:** {}\n\n", task.goal));
        out.push_str("**Requirements:**\n\n");
        for requirement in &task.requirements {
            out.push_str(&format!("- {}\n", requirement));
        }
        out.push_str("\n**Files:**\n\n");
        for file in &task.files {
            out.push_str(&format!("- `{}`\n", file));
        }
        out.push_str("\n**Evidence:**\n\n");
        for item in &task.evidence {
            out.push_str(&format!("- {}\n", item));
        }
        out.push('\n');
    }
    out
}

fn print_report(report: &TasksReport) {
    println!("kb tasks");
    println!("Generated at: {}", report.generated_at);
    println!("Dry run: {}", report.dry_run);
    println!("Task groups: {}", report.task_count);
    if let Some(path) = &report.report_path {
        println!("Report: {path}");
    }
    if let Some(path) = &report.index_path {
        println!("Task index: {path}");
    }
    if let Some(path) = &report.task_item_dir {
        println!("Task items: {path} ({} files)", report.task_item_count);
    }
    println!();

    if report.tasks.is_empty() {
        println!("No deferred LLM/agent task groups were detected.");
        return;
    }

    for task in &report.tasks {
        println!("[{}] {} -> {}", task.priority, task.id, task.target_agent);
        println!("  Goal: {}", task.goal);
        println!("  Files: {}", task.files.len());
        println!("  Source command: {}", task.source_command);
        if !task.evidence.is_empty() {
            println!("  Evidence: {}", task.evidence[0]);
        }
        println!();
    }

    if report.limit > 0 && report.task_count > report.returned_count {
        println!(
            "Showing {} of {} task groups. Use --limit 0 to show all.",
            report.returned_count, report.task_count
        );
    }
}
