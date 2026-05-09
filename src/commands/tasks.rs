use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use clap::Args;
use regex::Regex;
use serde::Serialize;
use walkdir::WalkDir;

#[derive(Debug, Clone, Args)]
pub struct TasksArgs {
    #[arg(
        long,
        help = "Preview deferred task scan without writing LLM/tasks/*.md"
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

    let mut report = run_task_scan(&kb_path, args)?;
    let dry_run = args.dry_run || args.preview;

    if !dry_run {
        let report_path = write_markdown_report(&kb_path, &report)?;
        report.report_path = Some(relative_path_string(&kb_path, &report_path));
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
    if let Some(task) = scan_reference_reconciliation_tasks(kb_path)? {
        tasks.push(task);
    }
    if let Some(task) = scan_bibliographic_index_review_tasks(kb_path)? {
        tasks.push(task);
    }
    if let Some(task) = scan_keyword_topic_relation_tasks(kb_path)? {
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
        schema_version: "0.5.10".to_string(),
        generated_by: "kb-cli tasks".to_string(),
        generated_at: chrono::Utc::now().to_rfc3339(),
        dry_run: args.dry_run || args.preview,
        task_count,
        returned_count: returned_tasks.len(),
        limit: args.limit,
        report_path: None,
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
