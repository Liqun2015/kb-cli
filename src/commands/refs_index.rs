use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use clap::Args;
use regex::Regex;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

#[derive(Debug, Clone, Args)]
pub struct RefsIndexArgs {
    #[arg(
        long,
        value_name = "PATH",
        help = "Text file or directory to scan. Defaults to processing/text/."
    )]
    pub path: Option<PathBuf>,

    #[arg(
        long,
        default_value_t = 100,
        help = "Maximum number of relation rows to print. Use 0 for no limit."
    )]
    pub limit: usize,

    #[arg(
        long,
        help = "Preview relation indexing without writing processing/refs/refs_index_*.md"
    )]
    pub dry_run: bool,

    #[arg(long, help = "Alias for --dry-run")]
    pub preview: bool,

    #[arg(long, help = "Print a machine-readable JSON relation report")]
    pub json: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PaperMetadata {
    filename: String,
    title: Option<String>,
    author: Option<String>,
    subject: Option<String>,
    creator: Option<String>,
    pages: usize,
    file_size: u64,
    doi: Option<String>,
}

#[derive(Debug, Clone)]
struct LocalPaper {
    path: String,
    filename: String,
    stem: String,
    title: Option<String>,
    author: Option<String>,
    doi: Option<String>,
    title_tokens: BTreeSet<String>,
    filename_tokens: BTreeSet<String>,
}

#[derive(Debug, Clone)]
struct ReferenceEntry {
    source_text_file: String,
    line: usize,
    raw: String,
    doi: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct RelationEvidence {
    source_text_file: String,
    source_line: usize,
    reference_text: String,
    matched_by: String,
    score: f64,
    notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct IndexRelation {
    status: String,
    source_text_file: String,
    source_line: usize,
    target_paper: Option<String>,
    target_title: Option<String>,
    target_doi: Option<String>,
    candidates: Vec<String>,
    evidence: RelationEvidence,
    review_required: bool,
}

#[derive(Debug, Clone, Serialize)]
struct DeferredReviewTask {
    target_reviewer: String,
    goal: String,
    requirements: Vec<String>,
    files: Vec<String>,
    evidence: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct RefsIndexReport {
    schema_version: String,
    generated_by: String,
    generated_at: String,
    search_path: String,
    local_papers: usize,
    reference_entries: usize,
    relation_count: usize,
    returned_count: usize,
    limit: usize,
    dry_run: bool,
    report_path: Option<String>,
    counts: BTreeMap<String, usize>,
    relations: Vec<IndexRelation>,
    deferred_tasks: Vec<DeferredReviewTask>,
}

pub fn execute(custom_kb: Option<&Path>, args: &RefsIndexArgs) -> Result<()> {
    let kb_path = crate::commands::init::get_kb_path(custom_kb);
    if !kb_path.exists() {
        return Err(anyhow!(
            "knowledge base path does not exist: {}. Run `kb init` first.",
            kb_path.display()
        ));
    }

    let mut report = run_refs_index(&kb_path, args)?;
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

fn run_refs_index(kb_path: &Path, args: &RefsIndexArgs) -> Result<RefsIndexReport> {
    let search_root = match &args.path {
        Some(path) if path.is_absolute() => path.clone(),
        Some(path) => kb_path.join(path),
        None => kb_path.join("processing/text"),
    };

    if !search_root.exists() {
        return Err(anyhow!(
            "search path does not exist: {}. Run `kb extract-text` first or pass --path.",
            search_root.display()
        ));
    }

    let local_papers = load_local_papers(kb_path)?;
    let references = collect_reference_entries(kb_path, &search_root)?;

    let mut all_relations = Vec::new();
    let mut counts = BTreeMap::new();

    for reference in &references {
        let relation = match_reference(reference, &local_papers);
        *counts.entry(relation.status.clone()).or_insert(0) += 1;
        all_relations.push(relation);
    }

    all_relations.sort_by(|a, b| {
        status_rank(&a.status)
            .cmp(&status_rank(&b.status))
            .then_with(|| a.source_text_file.cmp(&b.source_text_file))
            .then_with(|| a.source_line.cmp(&b.source_line))
    });

    let relation_count = all_relations.len();
    let returned_relations = if args.limit > 0 {
        all_relations
            .into_iter()
            .take(args.limit)
            .collect::<Vec<_>>()
    } else {
        all_relations
    };

    let deferred_tasks = build_deferred_review_tasks(&returned_relations);

    Ok(RefsIndexReport {
        schema_version: "0.5.10".to_string(),
        generated_by: "kb-cli refs-index".to_string(),
        generated_at: chrono::Utc::now().to_rfc3339(),
        search_path: relative_path_string(kb_path, &search_root),
        local_papers: local_papers.len(),
        reference_entries: references.len(),
        relation_count,
        returned_count: returned_relations.len(),
        limit: args.limit,
        dry_run: args.dry_run || args.preview,
        report_path: None,
        counts,
        relations: returned_relations,
        deferred_tasks,
    })
}

fn load_local_papers(kb_path: &Path) -> Result<Vec<LocalPaper>> {
    let mut metadata_by_filename = BTreeMap::new();
    let metadata_path = kb_path.join("logs/papers_metadata.json");
    if metadata_path.exists() {
        if let Ok(content) = fs::read_to_string(&metadata_path) {
            if let Ok(items) = serde_json::from_str::<Vec<PaperMetadata>>(&content) {
                for item in items {
                    metadata_by_filename.insert(item.filename.clone(), item);
                }
            }
        }
    }

    let raw_papers = kb_path.join("raw/papers");
    if !raw_papers.exists() {
        return Ok(Vec::new());
    }

    let mut papers = Vec::new();
    for path in collect_files_with_extensions(&raw_papers, &["pdf", "txt", "md"])? {
        let filename = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(&filename)
            .to_string();
        let metadata = metadata_by_filename.get(&filename);
        let title = metadata.and_then(|m| m.title.clone());
        let author = metadata.and_then(|m| m.author.clone());
        let doi = metadata
            .and_then(|m| m.doi.clone())
            .or_else(|| extract_doi(&filename));
        let title_tokens = title.as_deref().map(tokenize_for_match).unwrap_or_default();
        let filename_tokens = tokenize_for_match(&stem);

        papers.push(LocalPaper {
            path: relative_path_string(kb_path, &path),
            filename,
            stem,
            title,
            author,
            doi: doi.map(normalize_doi),
            title_tokens,
            filename_tokens,
        });
    }

    papers.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(papers)
}

fn collect_reference_entries(kb_path: &Path, search_root: &Path) -> Result<Vec<ReferenceEntry>> {
    let numbered_entry = Regex::new(r"^\s*(\[\s*\d+\s*\]|\d+\s*[\.)])\s+\S+")?;
    let doi_re = doi_regex()?;
    let mut entries = Vec::new();

    for path in collect_files_with_extensions(search_root, &["txt", "md"])? {
        let Ok(content) = fs::read_to_string(&path) else {
            continue;
        };
        for (line_idx, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            let is_numbered_entry = numbered_entry.is_match(trimmed);
            let doi = extract_doi(trimmed).map(normalize_doi);
            if is_numbered_entry || doi.is_some() {
                entries.push(ReferenceEntry {
                    source_text_file: relative_path_string(kb_path, &path),
                    line: line_idx + 1,
                    raw: trimmed.to_string(),
                    doi,
                });
            } else if doi_re.is_match(trimmed) {
                entries.push(ReferenceEntry {
                    source_text_file: relative_path_string(kb_path, &path),
                    line: line_idx + 1,
                    raw: trimmed.to_string(),
                    doi: extract_doi(trimmed).map(normalize_doi),
                });
            }
        }
    }

    entries.sort_by(|a, b| {
        a.source_text_file
            .cmp(&b.source_text_file)
            .then_with(|| a.line.cmp(&b.line))
    });
    Ok(entries)
}

fn match_reference(reference: &ReferenceEntry, local_papers: &[LocalPaper]) -> IndexRelation {
    let mut notes = Vec::new();

    if let Some(reference_doi) = &reference.doi {
        let doi_matches = local_papers
            .iter()
            .filter(|paper| paper.doi.as_deref() == Some(reference_doi.as_str()))
            .collect::<Vec<_>>();
        if doi_matches.len() == 1 {
            let paper = doi_matches[0];
            return relation_for_paper(
                "confirmed",
                reference,
                paper,
                "doi_exact",
                1.0,
                false,
                notes,
            );
        }
        if doi_matches.len() > 1 {
            notes.push(
                "The DOI matches multiple local papers. Human review is required.".to_string(),
            );
            return relation_for_candidates(
                "ambiguous",
                reference,
                doi_matches.iter().map(|p| p.path.clone()).collect(),
                "doi_exact_multiple",
                1.0,
                true,
                notes,
            );
        }
        notes.push(
            "DOI was found in the reference, but no local paper has the same DOI.".to_string(),
        );
    }

    let reference_tokens = tokenize_for_match(&reference.raw);
    let mut scored = local_papers
        .iter()
        .filter_map(|paper| {
            score_paper_match(&reference.raw, &reference_tokens, paper).map(|score| (paper, score))
        })
        .collect::<Vec<_>>();

    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    if scored.is_empty() {
        let mut evidence_notes = notes;
        evidence_notes.push("No deterministic local-paper match was found. This may be a missing source or a formatting issue.".to_string());
        return IndexRelation {
            status: "missing".to_string(),
            source_text_file: reference.source_text_file.clone(),
            source_line: reference.line,
            target_paper: None,
            target_title: None,
            target_doi: None,
            candidates: Vec::new(),
            evidence: RelationEvidence {
                source_text_file: reference.source_text_file.clone(),
                source_line: reference.line,
                reference_text: reference.raw.clone(),
                matched_by: if reference.doi.is_some() {
                    "doi_missing"
                } else {
                    "no_match"
                }
                .to_string(),
                score: 0.0,
                notes: evidence_notes,
            },
            review_required: true,
        };
    }

    let best_score = scored[0].1;
    let best_count = scored
        .iter()
        .take_while(|(_, score)| (*score - best_score).abs() < 0.0001)
        .count();

    if best_count > 1 {
        notes.push("Multiple local papers have the same top deterministic score.".to_string());
        return relation_for_candidates(
            "ambiguous",
            reference,
            scored
                .iter()
                .take(best_count)
                .map(|(paper, _)| paper.path.clone())
                .collect(),
            "title_or_filename_tokens",
            best_score,
            true,
            notes,
        );
    }

    let paper = scored[0].0;
    notes.push(
        "Candidate match was inferred from title/filename tokens and needs human confirmation."
            .to_string(),
    );
    relation_for_paper(
        "candidate",
        reference,
        paper,
        "title_or_filename_tokens",
        best_score,
        true,
        notes,
    )
}

fn relation_for_paper(
    status: &str,
    reference: &ReferenceEntry,
    paper: &LocalPaper,
    matched_by: &str,
    score: f64,
    review_required: bool,
    notes: Vec<String>,
) -> IndexRelation {
    IndexRelation {
        status: status.to_string(),
        source_text_file: reference.source_text_file.clone(),
        source_line: reference.line,
        target_paper: Some(paper.path.clone()),
        target_title: paper.title.clone(),
        target_doi: paper.doi.clone(),
        candidates: vec![paper.path.clone()],
        evidence: RelationEvidence {
            source_text_file: reference.source_text_file.clone(),
            source_line: reference.line,
            reference_text: reference.raw.clone(),
            matched_by: matched_by.to_string(),
            score,
            notes,
        },
        review_required,
    }
}

fn relation_for_candidates(
    status: &str,
    reference: &ReferenceEntry,
    candidates: Vec<String>,
    matched_by: &str,
    score: f64,
    review_required: bool,
    notes: Vec<String>,
) -> IndexRelation {
    IndexRelation {
        status: status.to_string(),
        source_text_file: reference.source_text_file.clone(),
        source_line: reference.line,
        target_paper: None,
        target_title: None,
        target_doi: reference.doi.clone(),
        candidates,
        evidence: RelationEvidence {
            source_text_file: reference.source_text_file.clone(),
            source_line: reference.line,
            reference_text: reference.raw.clone(),
            matched_by: matched_by.to_string(),
            score,
            notes,
        },
        review_required,
    }
}

fn score_paper_match(
    reference_text: &str,
    reference_tokens: &BTreeSet<String>,
    paper: &LocalPaper,
) -> Option<f64> {
    let reference_norm = normalize_text(reference_text);

    if let Some(title) = &paper.title {
        let title_norm = normalize_text(title);
        if title_norm.len() > 20 && reference_norm.contains(&title_norm) {
            return Some(0.95);
        }
    }

    let title_score = token_overlap_score(reference_tokens, &paper.title_tokens);
    let filename_score = token_overlap_score(reference_tokens, &paper.filename_tokens) * 0.8;
    let score = title_score.max(filename_score);

    if score >= 0.65 && reference_tokens.len() >= 3 {
        Some(round_score(score))
    } else {
        None
    }
}

fn token_overlap_score(
    reference_tokens: &BTreeSet<String>,
    candidate_tokens: &BTreeSet<String>,
) -> f64 {
    if candidate_tokens.is_empty() {
        return 0.0;
    }
    let hits = candidate_tokens
        .iter()
        .filter(|token| reference_tokens.contains(*token))
        .count();
    hits as f64 / candidate_tokens.len() as f64
}

fn build_deferred_review_tasks(relations: &[IndexRelation]) -> Vec<DeferredReviewTask> {
    let review_relations = relations
        .iter()
        .filter(|relation| relation.review_required)
        .collect::<Vec<_>>();
    if review_relations.is_empty() {
        return Vec::new();
    }

    let mut files = BTreeSet::new();
    let mut evidence = Vec::new();
    for relation in review_relations.iter().take(50) {
        files.insert(relation.source_text_file.clone());
        if let Some(target) = &relation.target_paper {
            files.insert(target.clone());
        }
        for candidate in &relation.candidates {
            files.insert(candidate.clone());
        }
        evidence.push(format!(
            "{}:{} [{} score={:.2}] {}",
            relation.source_text_file,
            relation.source_line,
            relation.status,
            relation.evidence.score,
            relation
                .evidence
                .reference_text
                .chars()
                .take(180)
                .collect::<String>()
        ));
    }

    vec![DeferredReviewTask {
        target_reviewer: "Human Reference Index Reviewer".to_string(),
        goal: "Review candidate, ambiguous, and missing bibliographic index relations. Human review is the final guarantee because reference formats vary across journals.".to_string(),
        requirements: vec![
            "Confirm DOI matches when available.".to_string(),
            "For title/author/year matches, compare the original reference string with the candidate local paper metadata.".to_string(),
            "Mark confirmed, rejected, duplicate, or missing-local-source decisions explicitly.".to_string(),
            "Do not let an LLM make final bibliographic identity decisions without human review.".to_string(),
        ],
        files: files.into_iter().collect(),
        evidence,
    }]
}

fn write_markdown_report(kb_path: &Path, report: &RefsIndexReport) -> Result<PathBuf> {
    let refs_dir = kb_path.join("processing/refs");
    fs::create_dir_all(&refs_dir)?;
    let stamp = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();
    let path = refs_dir.join(format!("refs_index_{stamp}.md"));
    fs::write(&path, render_markdown_report(report))?;
    Ok(path)
}

fn render_markdown_report(report: &RefsIndexReport) -> String {
    let mut out = String::new();
    out.push_str("# Bibliographic Index Relation Candidates\n\n");
    out.push_str(&format!("- Generated by: `{}`\n", report.generated_by));
    out.push_str(&format!("- Generated at: `{}`\n", report.generated_at));
    out.push_str(&format!("- Search path: `{}`\n", report.search_path));
    out.push_str(&format!("- Local papers: `{}`\n", report.local_papers));
    out.push_str(&format!(
        "- Reference entries: `{}`\n",
        report.reference_entries
    ));
    out.push_str(&format!("- Relations: `{}`\n\n", report.relation_count));

    out.push_str("## Counts\n\n");
    if report.counts.is_empty() {
        out.push_str("No relation candidates were found.\n\n");
    } else {
        for (status, count) in &report.counts {
            out.push_str(&format!("- `{}`: `{}`\n", status, count));
        }
        out.push('\n');
    }

    out.push_str("## Relation candidates\n\n");
    if report.relations.is_empty() {
        out.push_str("No relation candidates returned.\n\n");
    } else {
        out.push_str("| Status | Source | Target | Score | Review | Evidence |\n");
        out.push_str("|---|---|---|---:|---|---|\n");
        for relation in &report.relations {
            let source = format!("{}:{}", relation.source_text_file, relation.source_line);
            let target = relation
                .target_paper
                .clone()
                .unwrap_or_else(|| relation.candidates.join("; "));
            out.push_str(&format!(
                "| `{}` | `{}` | `{}` | {:.2} | `{}` | {} |\n",
                relation.status,
                source,
                target,
                relation.evidence.score,
                relation.review_required,
                escape_table_cell(
                    &relation
                        .evidence
                        .reference_text
                        .chars()
                        .take(160)
                        .collect::<String>()
                )
            ));
        }
        out.push('\n');
    }

    if !report.deferred_tasks.is_empty() {
        out.push_str("## Deferred human / LLM task handoff\n\n");
        for task in &report.deferred_tasks {
            out.push_str(&format!("### {}\n\n", task.target_reviewer));
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
    }

    out
}

fn print_report(report: &RefsIndexReport) {
    println!("kb refs-index");
    println!("Search path: {}", report.search_path);
    println!("Local papers: {}", report.local_papers);
    println!("Reference entries: {}", report.reference_entries);
    println!("Relations: {}", report.relation_count);
    println!("Dry run: {}", report.dry_run);
    if let Some(path) = &report.report_path {
        println!("Report: {path}");
    }
    println!();

    if !report.counts.is_empty() {
        println!("Counts by status:");
        for (status, count) in &report.counts {
            println!("  {status}: {count}");
        }
        println!();
    }

    if report.relations.is_empty() {
        println!("No bibliographic index relation candidates found.");
        println!(
            "Run `kb extract-text` and `kb refs` first, or pass --path to scan existing text."
        );
        return;
    }

    for relation in &report.relations {
        println!(
            "{}:{} [{} score={:.2} review={}]",
            relation.source_text_file,
            relation.source_line,
            relation.status,
            relation.evidence.score,
            relation.review_required
        );
        if let Some(target) = &relation.target_paper {
            println!("    target: {target}");
        } else if !relation.candidates.is_empty() {
            println!("    candidates: {}", relation.candidates.join("; "));
        }
        println!("    {}", relation.evidence.reference_text);
    }

    if report.limit > 0 && report.relation_count > report.returned_count {
        println!();
        println!(
            "Showing {} of {} relations. Use --limit 0 to show all.",
            report.returned_count, report.relation_count
        );
    }

    if !report.deferred_tasks.is_empty() {
        println!();
        println!("Deferred review tasks:");
        for task in &report.deferred_tasks {
            println!("  - {}: {}", task.target_reviewer, task.goal);
        }
    }
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

fn extract_doi(text: &str) -> Option<String> {
    doi_regex().ok().and_then(|re| {
        re.find(text)
            .map(|hit| hit.as_str().trim_end_matches('.').to_string())
    })
}

fn doi_regex() -> Result<Regex> {
    Ok(Regex::new(r"(?i)\b10\.\d{4,9}/[-._;()/:A-Z0-9]+\b")?)
}

fn normalize_doi(value: String) -> String {
    value.trim().trim_end_matches('.').to_ascii_lowercase()
}

fn tokenize_for_match(text: &str) -> BTreeSet<String> {
    let stopwords = stopwords();
    let mut current = String::new();
    let mut tokens = BTreeSet::new();

    for ch in text.chars() {
        if ch.is_alphanumeric() {
            current.extend(ch.to_lowercase());
        } else if !current.is_empty() {
            push_token(&mut tokens, &stopwords, &current);
            current.clear();
        }
    }
    if !current.is_empty() {
        push_token(&mut tokens, &stopwords, &current);
    }
    tokens
}

fn push_token(tokens: &mut BTreeSet<String>, stopwords: &HashSet<&'static str>, token: &str) {
    if token.len() < 4 {
        return;
    }
    if stopwords.contains(token) {
        return;
    }
    tokens.insert(token.to_string());
}

fn stopwords() -> HashSet<&'static str> {
    [
        "with",
        "from",
        "into",
        "that",
        "this",
        "using",
        "used",
        "based",
        "analysis",
        "study",
        "paper",
        "journal",
        "research",
        "method",
        "methods",
        "results",
        "model",
        "models",
        "effect",
        "effects",
        "system",
        "systems",
        "thermal",
        "energy",
        "materials",
        "science",
        "engineering",
        "elsevier",
        "springer",
    ]
    .into_iter()
    .collect()
}

fn normalize_text(text: &str) -> String {
    text.chars()
        .map(|ch| {
            if ch.is_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn round_score(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

fn status_rank(status: &str) -> usize {
    match status {
        "missing" => 0,
        "ambiguous" => 1,
        "candidate" => 2,
        "confirmed" => 3,
        _ => 4,
    }
}

fn escape_table_cell(value: &str) -> String {
    value.replace('|', "\\|").replace('\n', " ")
}

fn relative_path_string(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}
