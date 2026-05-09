use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use clap::Args;
use serde::Serialize;
use walkdir::WalkDir;

#[derive(Debug, Clone, Args)]
pub struct KeywordsArgs {
    #[arg(value_name = "TERM", num_args = 0.., help = "Optional keyword or phrase to scan for. If omitted, kb-cli extracts simple candidate terms deterministically.")]
    pub terms: Vec<String>,

    #[arg(long, value_name = "PATH", help = "Text file or directory to scan. Defaults to processing/text/.")]
    pub path: Option<PathBuf>,

    #[arg(long = "terms-file", value_name = "PATH", help = "Optional newline-separated keyword list. Relative paths are resolved under the knowledge base.")]
    pub terms_file: Option<PathBuf>,

    #[arg(long = "min-count", default_value_t = 2, help = "Minimum count for auto-extracted terms. Explicit terms are reported when present at least once.")]
    pub min_count: usize,

    #[arg(long, default_value_t = 100, help = "Maximum number of keyword relation rows to print. Use 0 for no limit.")]
    pub limit: usize,

    #[arg(long, help = "Preview keyword relation scan without writing processing/keywords/keywords_*.md")]
    pub dry_run: bool,

    #[arg(long, help = "Alias for --dry-run")]
    pub preview: bool,

    #[arg(long, help = "Print a machine-readable JSON keyword relation report")]
    pub json: bool,
}

#[derive(Debug, Clone)]
struct TextDocument {
    rel_path: String,
    content: String,
}

#[derive(Debug, Clone, Serialize)]
struct KeywordHit {
    term: String,
    file: String,
    count: usize,
    evidence: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct KeywordRelation {
    relation_type: String,
    status: String,
    source_file: String,
    target_file: String,
    shared_terms: Vec<String>,
    evidence: Vec<String>,
    review_required: bool,
}

#[derive(Debug, Clone, Serialize)]
struct DeferredKeywordTask {
    target_agent: String,
    goal: String,
    requirements: Vec<String>,
    files: Vec<String>,
    evidence: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct KeywordsReport {
    schema_version: String,
    generated_by: String,
    generated_at: String,
    search_path: String,
    mode: String,
    terms_requested: Vec<String>,
    files_scanned: usize,
    term_count: usize,
    hit_count: usize,
    relation_count: usize,
    returned_count: usize,
    limit: usize,
    dry_run: bool,
    report_path: Option<String>,
    hits: Vec<KeywordHit>,
    relations: Vec<KeywordRelation>,
    deferred_tasks: Vec<DeferredKeywordTask>,
}

pub fn execute(custom_kb: Option<&Path>, args: &KeywordsArgs) -> Result<()> {
    let kb_path = crate::commands::init::get_kb_path(custom_kb);
    if !kb_path.exists() {
        return Err(anyhow!(
            "knowledge base path does not exist: {}. Run `kb init` first.",
            kb_path.display()
        ));
    }

    let mut report = run_keywords(&kb_path, args)?;
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

fn run_keywords(kb_path: &Path, args: &KeywordsArgs) -> Result<KeywordsReport> {
    let search_root = match &args.path {
        Some(path) if path.is_absolute() => path.clone(),
        Some(path) => kb_path.join(path),
        None => kb_path.join("processing/text"),
    };

    if !search_root.exists() {
        return Err(anyhow!(
            "keyword scan path does not exist: {}. Run `kb extract-text` first, or pass --path.",
            search_root.display()
        ));
    }

    let documents = collect_documents(kb_path, &search_root)?;
    let requested_terms = load_requested_terms(kb_path, args)?;
    let explicit_mode = !requested_terms.is_empty();
    let selected_terms = if explicit_mode {
        requested_terms.clone()
    } else {
        auto_extract_terms(&documents, args.min_count)
    };

    let hits = scan_keyword_hits(&documents, &selected_terms, explicit_mode, args.min_count);
    let all_relations = build_keyword_relations(&hits);
    let relation_count = all_relations.len();
    let returned_relations = if args.limit > 0 {
        all_relations.into_iter().take(args.limit).collect::<Vec<_>>()
    } else {
        all_relations
    };
    let deferred_tasks = build_deferred_tasks(&returned_relations);

    Ok(KeywordsReport {
        schema_version: "0.5.12".to_string(),
        generated_by: "kb-cli keywords".to_string(),
        generated_at: chrono::Utc::now().to_rfc3339(),
        search_path: relative_path_string(kb_path, &search_root),
        mode: if explicit_mode {
            "explicit_terms".to_string()
        } else {
            "auto_terms".to_string()
        },
        terms_requested: selected_terms,
        files_scanned: documents.len(),
        term_count: hits.iter().map(|hit| hit.term.clone()).collect::<BTreeSet<_>>().len(),
        hit_count: hits.len(),
        relation_count,
        returned_count: returned_relations.len(),
        limit: args.limit,
        dry_run: args.dry_run || args.preview,
        report_path: None,
        hits,
        relations: returned_relations,
        deferred_tasks,
    })
}

fn load_requested_terms(kb_path: &Path, args: &KeywordsArgs) -> Result<Vec<String>> {
    let mut terms = Vec::new();
    for term in &args.terms {
        push_normalized_term(&mut terms, term);
    }

    if let Some(path) = &args.terms_file {
        let terms_path = if path.is_absolute() {
            path.clone()
        } else {
            kb_path.join(path)
        };
        let content = fs::read_to_string(&terms_path)
            .map_err(|err| anyhow!("could not read terms file {}: {}", terms_path.display(), err))?;
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            push_normalized_term(&mut terms, trimmed);
        }
    }

    terms.sort();
    terms.dedup();
    Ok(terms)
}

fn push_normalized_term(terms: &mut Vec<String>, input: &str) {
    let term = normalize_phrase(input);
    if !term.is_empty() {
        terms.push(term);
    }
}

fn collect_documents(kb_path: &Path, root: &Path) -> Result<Vec<TextDocument>> {
    let mut documents = Vec::new();
    if root.is_file() {
        if let Ok(content) = fs::read_to_string(root) {
            documents.push(TextDocument {
                rel_path: relative_path_string(kb_path, root),
                content,
            });
        }
        return Ok(documents);
    }

    for entry in WalkDir::new(root)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
    {
        let path = entry.path();
        if !is_text_like(path) {
            continue;
        }
        let content = match fs::read_to_string(path) {
            Ok(content) => content,
            Err(_) => continue,
        };
        documents.push(TextDocument {
            rel_path: relative_path_string(kb_path, path),
            content,
        });
    }
    documents.sort_by(|a, b| a.rel_path.cmp(&b.rel_path));
    Ok(documents)
}

fn auto_extract_terms(documents: &[TextDocument], min_count: usize) -> Vec<String> {
    let stopwords = stopwords();
    let mut counts: HashMap<String, usize> = HashMap::new();
    for doc in documents {
        for token in tokenize_words(&doc.content) {
            if token.len() < 4 || stopwords.contains(token.as_str()) {
                continue;
            }
            *counts.entry(token).or_insert(0) += 1;
        }
    }

    let threshold = min_count.max(1);
    let mut items = counts
        .into_iter()
        .filter(|(_, count)| *count >= threshold)
        .collect::<Vec<_>>();
    items.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    items.into_iter().take(100).map(|(term, _)| term).collect()
}

fn scan_keyword_hits(
    documents: &[TextDocument],
    terms: &[String],
    explicit_mode: bool,
    min_count: usize,
) -> Vec<KeywordHit> {
    let mut hits = Vec::new();
    for doc in documents {
        let normalized_content = normalize_phrase(&doc.content);
        for term in terms {
            let count = count_occurrences(&normalized_content, term);
            if count == 0 {
                continue;
            }
            if !explicit_mode && count < min_count.max(1) {
                continue;
            }
            hits.push(KeywordHit {
                term: term.clone(),
                file: doc.rel_path.clone(),
                count,
                evidence: collect_evidence_lines(&doc.content, term, 2),
            });
        }
    }
    hits.sort_by(|a, b| {
        a.term
            .cmp(&b.term)
            .then_with(|| b.count.cmp(&a.count))
            .then_with(|| a.file.cmp(&b.file))
    });
    hits
}

fn build_keyword_relations(hits: &[KeywordHit]) -> Vec<KeywordRelation> {
    let mut files_by_term: BTreeMap<String, Vec<&KeywordHit>> = BTreeMap::new();
    for hit in hits {
        files_by_term.entry(hit.term.clone()).or_default().push(hit);
    }

    let mut shared_terms_by_pair: BTreeMap<(String, String), BTreeSet<String>> = BTreeMap::new();
    let mut evidence_by_pair: BTreeMap<(String, String), Vec<String>> = BTreeMap::new();

    for (term, term_hits) in files_by_term {
        for i in 0..term_hits.len() {
            for j in (i + 1)..term_hits.len() {
                let a = &term_hits[i].file;
                let b = &term_hits[j].file;
                if a == b {
                    continue;
                }
                let pair = if a < b {
                    (a.clone(), b.clone())
                } else {
                    (b.clone(), a.clone())
                };
                shared_terms_by_pair
                    .entry(pair.clone())
                    .or_default()
                    .insert(term.clone());
                let evidence = evidence_by_pair.entry(pair).or_default();
                if evidence.len() < 6 {
                    evidence.push(format!(
                        "shared term `{}` appears in `{}` and `{}`",
                        term, a, b
                    ));
                }
            }
        }
    }

    let mut relations = shared_terms_by_pair
        .into_iter()
        .map(|((source_file, target_file), terms)| {
            let key = (source_file.clone(), target_file.clone());
            KeywordRelation {
                relation_type: "keyword_topic".to_string(),
                status: "candidate".to_string(),
                source_file,
                target_file,
                shared_terms: terms.into_iter().collect(),
                evidence: evidence_by_pair.remove(&key).unwrap_or_default(),
                review_required: true,
            }
        })
        .collect::<Vec<_>>();

    relations.sort_by(|a, b| {
        b.shared_terms
            .len()
            .cmp(&a.shared_terms.len())
            .then_with(|| a.source_file.cmp(&b.source_file))
            .then_with(|| a.target_file.cmp(&b.target_file))
    });
    relations
}

fn build_deferred_tasks(relations: &[KeywordRelation]) -> Vec<DeferredKeywordTask> {
    if relations.is_empty() {
        return Vec::new();
    }

    let mut files = BTreeSet::new();
    let mut evidence = Vec::new();
    for relation in relations.iter().take(20) {
        files.insert(relation.source_file.clone());
        files.insert(relation.target_file.clone());
        evidence.push(format!(
            "{} <-> {} shared terms: {}",
            relation.source_file,
            relation.target_file,
            relation.shared_terms.join(", ")
        ));
    }

    vec![DeferredKeywordTask {
        target_agent: "Keyword Topic Relation Worker".to_string(),
        goal: "Review keyword/topic co-occurrence candidates and decide which ones are scientifically meaningful.".to_string(),
        requirements: vec![
            "Do not treat shared keywords alone as confirmed scientific idea relations.".to_string(),
            "Use source files and evidence lines to decide whether a relation is meaningful.".to_string(),
            "Promote only meaningful relations into wiki pages or future relation reports.".to_string(),
            "Record completed review decisions with kb memory.".to_string(),
        ],
        files: files.into_iter().collect(),
        evidence,
    }]
}

fn write_markdown_report(kb_path: &Path, report: &KeywordsReport) -> Result<PathBuf> {
    let out_dir = kb_path.join("processing/keywords");
    fs::create_dir_all(&out_dir)?;
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let report_path = out_dir.join(format!("keywords_{timestamp}.md"));
    fs::write(&report_path, render_markdown(report))?;
    Ok(report_path)
}

fn render_markdown(report: &KeywordsReport) -> String {
    let mut out = String::new();
    out.push_str("# Keyword / Topic Relation Candidate Report\n\n");
    out.push_str(&format!("- Generated at: `{}`\n", report.generated_at));
    out.push_str(&format!("- Search path: `{}`\n", report.search_path));
    out.push_str(&format!("- Mode: `{}`\n", report.mode));
    out.push_str(&format!("- Files scanned: `{}`\n", report.files_scanned));
    out.push_str(&format!("- Terms found: `{}`\n", report.term_count));
    out.push_str(&format!("- Hits: `{}`\n", report.hit_count));
    out.push_str(&format!("- Candidate relations: `{}`\n\n", report.relation_count));

    out.push_str("## Terms\n\n");
    if report.terms_requested.is_empty() {
        out.push_str("No terms were selected.\n\n");
    } else {
        for term in &report.terms_requested {
            out.push_str(&format!("- `{}`\n", term));
        }
        out.push('\n');
    }

    out.push_str("## Keyword Hits\n\n");
    if report.hits.is_empty() {
        out.push_str("No keyword hits found.\n\n");
    } else {
        out.push_str("| Term | File | Count | Evidence |\n");
        out.push_str("|---|---|---:|---|\n");
        for hit in &report.hits {
            out.push_str(&format!(
                "| `{}` | `{}` | `{}` | {} |\n",
                escape_table_cell(&hit.term),
                escape_table_cell(&hit.file),
                hit.count,
                escape_table_cell(&hit.evidence.join("<br>"))
            ));
        }
        out.push('\n');
    }

    out.push_str("## Candidate Keyword / Topic Relations\n\n");
    if report.relations.is_empty() {
        out.push_str("No keyword/topic relation candidates found.\n\n");
    } else {
        out.push_str("| Source | Target | Status | Shared terms | Review required |\n");
        out.push_str("|---|---|---|---|---|\n");
        for relation in &report.relations {
            out.push_str(&format!(
                "| `{}` | `{}` | `{}` | {} | `{}` |\n",
                escape_table_cell(&relation.source_file),
                escape_table_cell(&relation.target_file),
                relation.status,
                escape_table_cell(&relation.shared_terms.join(", ")),
                relation.review_required
            ));
        }
        out.push('\n');
    }

    out.push_str("## Deferred Manager/Worker Task\n\n");
    if report.deferred_tasks.is_empty() {
        out.push_str("No deferred keyword/topic review task generated.\n");
    } else {
        for task in &report.deferred_tasks {
            out.push_str(&format!("### {}\n\n", task.target_agent));
            out.push_str(&format!("- Goal: {}\n", task.goal));
            out.push_str("- Requirements:\n");
            for requirement in &task.requirements {
                out.push_str(&format!("  - {}\n", requirement));
            }
            out.push_str("- Files:\n");
            for file in &task.files {
                out.push_str(&format!("  - `{}`\n", file));
            }
            out.push_str("- Evidence:\n");
            for item in &task.evidence {
                out.push_str(&format!("  - {}\n", item));
            }
            out.push('\n');
        }
    }

    out
}

fn print_report(report: &KeywordsReport) {
    println!("kb keywords");
    println!("Search path: {}", report.search_path);
    println!("Mode: {}", report.mode);
    println!("Files scanned: {}", report.files_scanned);
    println!("Terms found: {}", report.term_count);
    println!("Hits: {}", report.hit_count);
    println!("Candidate relations: {}", report.relation_count);
    if let Some(path) = &report.report_path {
        println!("Report written: {}", path);
    }
    if report.dry_run {
        println!("Dry run: no processing/keywords report was written.");
    }
    println!();

    if report.relations.is_empty() {
        println!("No keyword/topic relation candidates found.");
        return;
    }

    for relation in &report.relations {
        println!(
            "{} <-> {} [{}] shared terms: {}",
            relation.source_file,
            relation.target_file,
            relation.status,
            relation.shared_terms.join(", ")
        );
    }

    if !report.deferred_tasks.is_empty() {
        println!();
        println!("Deferred Manager/Worker task hints:");
        for task in &report.deferred_tasks {
            println!("  - {}", task.target_agent);
        }
        println!("Use `kb tasks` or the generated report to create bounded Worker LLM work.");
    }
}

fn collect_evidence_lines(content: &str, term: &str, limit: usize) -> Vec<String> {
    let needle = term.to_lowercase();
    let mut evidence = Vec::new();
    for (line_idx, line) in content.lines().enumerate() {
        if line.to_lowercase().contains(&needle) {
            evidence.push(format!("L{}: {}", line_idx + 1, line.trim()));
            if evidence.len() >= limit {
                break;
            }
        }
    }
    evidence
}

fn count_occurrences(haystack: &str, needle: &str) -> usize {
    if needle.is_empty() {
        return 0;
    }
    haystack.match_indices(needle).count()
}

fn tokenize_words(content: &str) -> Vec<String> {
    normalize_phrase(content)
        .split_whitespace()
        .map(|s| s.to_string())
        .collect()
}

fn normalize_phrase(input: &str) -> String {
    input
        .to_lowercase()
        .chars()
        .map(|ch| if ch.is_alphanumeric() { ch } else { ' ' })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn stopwords() -> HashSet<&'static str> {
    [
        "about",
        "above",
        "after",
        "again",
        "against",
        "also",
        "analysis",
        "based",
        "between",
        "could",
        "during",
        "effect",
        "effects",
        "figure",
        "from",
        "have",
        "into",
        "journal",
        "method",
        "methods",
        "model",
        "models",
        "paper",
        "research",
        "result",
        "results",
        "section",
        "show",
        "shown",
        "study",
        "system",
        "systems",
        "that",
        "their",
        "there",
        "these",
        "this",
        "those",
        "through",
        "using",
        "were",
        "with",
    ]
    .into_iter()
    .collect()
}

fn is_text_like(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_ascii_lowercase()),
        Some(ext) if matches!(ext.as_str(), "md" | "txt" | "json" | "csv" | "tsv")
    )
}

fn relative_path_string(base: &Path, path: &Path) -> String {
    path.strip_prefix(base)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn escape_table_cell(input: &str) -> String {
    input.replace('|', "\\|").replace('\n', "<br>")
}
