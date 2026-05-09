use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use clap::Args;
use regex::Regex;
use serde::Serialize;
use walkdir::WalkDir;

#[derive(Debug, Clone, Args)]
pub struct RefsArgs {
    #[arg(long, value_name = "PATH", help = "Text file or directory to scan. Defaults to processing/text/.")]
    pub path: Option<PathBuf>,

    #[arg(long, default_value_t = 200, help = "Maximum number of reference hints to print. Use 0 for no limit.")]
    pub limit: usize,

    #[arg(long, help = "Also scan for in-text citation markers such as [1] and (Smith 2020). This can be noisy.")]
    pub citations: bool,

    #[arg(long, help = "Print a machine-readable JSON reference scan report")]
    pub json: bool,
}

#[derive(Debug, Clone, Serialize)]
struct RefHint {
    kind: String,
    path: String,
    line: usize,
    value: Option<String>,
    text: String,
}

#[derive(Debug, Clone, Serialize)]
struct RefsReport {
    schema_version: String,
    generated_by: String,
    generated_at: String,
    search_path: String,
    include_citations: bool,
    files_scanned: usize,
    match_count: usize,
    returned_count: usize,
    limit: usize,
    counts: BTreeMap<String, usize>,
    matches: Vec<RefHint>,
}

pub fn execute(custom_kb: Option<&Path>, args: &RefsArgs) -> Result<()> {
    let kb_path = crate::commands::init::get_kb_path(custom_kb);
    if !kb_path.exists() {
        return Err(anyhow!(
            "knowledge base path does not exist: {}. Run `kb init` first.",
            kb_path.display()
        ));
    }

    let report = run_refs_scan(&kb_path, args)?;
    if args.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print_report(&report);
    }
    Ok(())
}

fn run_refs_scan(kb_path: &Path, args: &RefsArgs) -> Result<RefsReport> {
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

    let patterns = RefPatterns::new()?;
    let mut files_scanned = 0usize;
    let mut match_count = 0usize;
    let mut counts = BTreeMap::new();
    let mut matches = Vec::new();

    for path in collect_text_files(&search_root)? {
        let content = match fs::read_to_string(&path) {
            Ok(content) => content,
            Err(_) => continue,
        };
        files_scanned += 1;
        scan_content(
            kb_path,
            &path,
            &content,
            args,
            &patterns,
            &mut match_count,
            &mut counts,
            &mut matches,
        );
    }

    Ok(RefsReport {
        schema_version: "0.5.7".to_string(),
        generated_by: "kb-cli refs".to_string(),
        generated_at: chrono::Utc::now().to_rfc3339(),
        search_path: relative_path_string(kb_path, &search_root),
        include_citations: args.citations,
        files_scanned,
        match_count,
        returned_count: matches.len(),
        limit: args.limit,
        counts,
        matches,
    })
}

fn scan_content(
    kb_path: &Path,
    path: &Path,
    content: &str,
    args: &RefsArgs,
    patterns: &RefPatterns,
    match_count: &mut usize,
    counts: &mut BTreeMap<String, usize>,
    matches: &mut Vec<RefHint>,
) {
    for (line_idx, line) in content.lines().enumerate() {
        let line_no = line_idx + 1;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if patterns.heading.is_match(trimmed) {
            record_hint(
                kb_path,
                path,
                line_no,
                "reference_heading",
                None,
                trimmed,
                args.limit,
                match_count,
                counts,
                matches,
            );
        }

        if patterns.numbered_entry.is_match(trimmed) {
            record_hint(
                kb_path,
                path,
                line_no,
                "reference_entry",
                None,
                trimmed,
                args.limit,
                match_count,
                counts,
                matches,
            );
        }

        for hit in patterns.doi.find_iter(trimmed) {
            record_hint(
                kb_path,
                path,
                line_no,
                "doi",
                Some(hit.as_str().trim_end_matches('.').to_string()),
                trimmed,
                args.limit,
                match_count,
                counts,
                matches,
            );
        }

        if args.citations {
            for hit in patterns.numeric_citation.find_iter(trimmed) {
                record_hint(
                    kb_path,
                    path,
                    line_no,
                    "numeric_citation",
                    Some(hit.as_str().to_string()),
                    trimmed,
                    args.limit,
                    match_count,
                    counts,
                    matches,
                );
            }

            for hit in patterns.author_year.find_iter(trimmed) {
                record_hint(
                    kb_path,
                    path,
                    line_no,
                    "author_year_citation",
                    Some(hit.as_str().to_string()),
                    trimmed,
                    args.limit,
                    match_count,
                    counts,
                    matches,
                );
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn record_hint(
    kb_path: &Path,
    path: &Path,
    line: usize,
    kind: &str,
    value: Option<String>,
    text: &str,
    limit: usize,
    match_count: &mut usize,
    counts: &mut BTreeMap<String, usize>,
    matches: &mut Vec<RefHint>,
) {
    *match_count += 1;
    *counts.entry(kind.to_string()).or_insert(0) += 1;

    if limit == 0 || matches.len() < limit {
        matches.push(RefHint {
            kind: kind.to_string(),
            path: relative_path_string(kb_path, path),
            line,
            value,
            text: text.to_string(),
        });
    }
}

struct RefPatterns {
    heading: Regex,
    numbered_entry: Regex,
    doi: Regex,
    numeric_citation: Regex,
    author_year: Regex,
}

impl RefPatterns {
    fn new() -> Result<Self> {
        Ok(Self {
            heading: Regex::new(r"(?i)^\s*(references|bibliography|works cited|参考文献|参考资料|文献)\s*$")?,
            numbered_entry: Regex::new(r"^\s*(\[\s*\d+\s*\]|\d+\s*[\.)])\s+\S+")?,
            doi: Regex::new(r"(?i)\b10\.\d{4,9}/[-._;()/:A-Z0-9]+\b")?,
            numeric_citation: Regex::new(r"\[(?:\s*\d+\s*(?:[-,;]\s*\d+\s*)*)\]")?,
            author_year: Regex::new(r"\([A-Z][A-Za-zÀ-ÖØ-öø-ÿ'’-]+(?:\s+et\s+al\.)?(?:,\s*|\s+)(?:19|20)\d{2}[a-z]?\)")?,
        })
    }
}

fn collect_text_files(search_root: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    if search_root.is_file() {
        if is_text_like(search_root) {
            files.push(search_root.to_path_buf());
        }
        return Ok(files);
    }

    for entry in WalkDir::new(search_root)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
    {
        let path = entry.path();
        if is_text_like(path) {
            files.push(path.to_path_buf());
        }
    }
    files.sort();
    Ok(files)
}

fn is_text_like(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|s| s.to_str()).map(|s| s.to_ascii_lowercase()),
        Some(ext) if matches!(ext.as_str(), "txt" | "md")
    )
}

fn print_report(report: &RefsReport) {
    println!("kb refs");
    println!("Search path: {}", report.search_path);
    println!("Files scanned: {}", report.files_scanned);
    println!("Matches: {}", report.match_count);
    println!("Include in-text citations: {}", report.include_citations);
    println!();

    if !report.counts.is_empty() {
        println!("Counts by kind:");
        for (kind, count) in &report.counts {
            println!("  {kind}: {count}");
        }
        println!();
    }

    if report.matches.is_empty() {
        println!("No reference hints found.");
        println!("Run `kb extract-text` first, or pass --path to scan an existing text directory.");
        return;
    }

    for item in &report.matches {
        match &item.value {
            Some(value) => println!("{}:{} [{}] {}", item.path, item.line, item.kind, value),
            None => println!("{}:{} [{}]", item.path, item.line, item.kind),
        }
        println!("    {}", item.text);
    }

    if report.limit > 0 && report.match_count > report.returned_count {
        println!();
        println!(
            "Showing {} of {} reference hints. Use --limit 0 to show all matches.",
            report.returned_count, report.match_count
        );
    }
}

fn relative_path_string(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}
