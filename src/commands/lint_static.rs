use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use clap::Args;
use regex::Regex;
use serde::Serialize;
use walkdir::WalkDir;

#[derive(Debug, Clone, Args)]
pub struct LintStaticArgs {
    #[arg(long, help = "Preview lint results without writing outputs/reports/lint_static_*.md")]
    pub dry_run: bool,

    #[arg(long, help = "Alias for --dry-run.")]
    pub preview: bool,

    #[arg(long = "no-report", help = "Alias for --dry-run in lint-static: check only, do not write a report file.")]
    pub no_report: bool,

    #[arg(long, help = "Print a machine-readable JSON lint summary")]
    pub json: bool,

    #[arg(long, help = "Return a non-zero exit code when static lint issues are found")]
    pub strict: bool,
}

impl LintStaticArgs {
    pub fn is_dry_run(&self) -> bool {
        self.dry_run || self.preview || self.no_report
    }
}

#[derive(Debug, Clone)]
struct WikiPage {
    rel_path: String,
    title: String,
    content: String,
    source_refs: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct BrokenLinkIssue {
    page: String,
    line: usize,
    target: String,
}

#[derive(Debug, Clone, Serialize)]
struct DuplicateTitleIssue {
    title: String,
    pages: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct LintStaticReport {
    schema_version: String,
    generated_by: String,
    generated_at: String,
    pages_scanned: usize,
    wiki_links_found: usize,
    broken_links: Vec<BrokenLinkIssue>,
    orphan_pages: Vec<String>,
    missing_source_pages: Vec<String>,
    empty_pages: Vec<String>,
    duplicate_titles: Vec<DuplicateTitleIssue>,
    report_path: Option<String>,
    dry_run: bool,
}

impl LintStaticReport {
    fn issue_count(&self) -> usize {
        self.broken_links.len()
            + self.orphan_pages.len()
            + self.missing_source_pages.len()
            + self.empty_pages.len()
            + self.duplicate_titles.len()
    }
}

pub fn execute(custom_kb: Option<&Path>, args: &LintStaticArgs) -> Result<()> {
    let kb_path = crate::commands::init::get_kb_path(custom_kb);
    if !kb_path.exists() {
        return Err(anyhow!(
            "knowledge base path does not exist: {}. Run `kb init` first.",
            kb_path.display()
        ));
    }

    let mut report = run_lint(&kb_path)?;

    if !args.is_dry_run() {
        let report_path = write_report(&kb_path, &report)?;
        report.report_path = Some(report_path.display().to_string());
    }
    report.dry_run = args.is_dry_run();

    if args.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print_report(&report);
    }

    if args.strict && report.issue_count() > 0 {
        return Err(anyhow!(
            "static lint found {} issue group(s). See report output above.",
            report.issue_count()
        ));
    }

    Ok(())
}

fn run_lint(kb_path: &Path) -> Result<LintStaticReport> {
    let pages = scan_wiki_pages(kb_path)?;
    let generated_at = chrono::Utc::now().to_rfc3339();
    let link_re = Regex::new(r"\[\[([^\]\n]+)\]\]")?;

    let mut page_keys: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut incoming_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut titles: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

    for page in &pages {
        incoming_counts.entry(page.rel_path.clone()).or_insert(0);
        for key in page_lookup_keys(page) {
            page_keys.entry(key).or_default().insert(page.rel_path.clone());
        }
        titles
            .entry(normalize_title(&page.title))
            .or_default()
            .insert(page.rel_path.clone());
    }

    let mut broken_links = Vec::new();
    let mut wiki_links_found = 0;

    for page in &pages {
        for (line_index, line) in page.content.lines().enumerate() {
            for captures in link_re.captures_iter(line) {
                let Some(target_match) = captures.get(1) else {
                    continue;
                };
                wiki_links_found += 1;
                let target = clean_wikilink_target(target_match.as_str());
                if target.is_empty() {
                    continue;
                }

                let lookup_key = normalize_page_key(&target);
                let matches = page_keys.get(&lookup_key);
                match matches {
                    Some(target_pages) if !target_pages.is_empty() => {
                        for target_page in target_pages {
                            if target_page != &page.rel_path {
                                *incoming_counts.entry(target_page.clone()).or_insert(0) += 1;
                            }
                        }
                    }
                    _ => broken_links.push(BrokenLinkIssue {
                        page: page.rel_path.clone(),
                        line: line_index + 1,
                        target,
                    }),
                }
            }
        }
    }

    let mut orphan_pages = pages
        .iter()
        .filter(|page| !is_orphan_exempt(&page.rel_path))
        .filter(|page| incoming_counts.get(&page.rel_path).copied().unwrap_or(0) == 0)
        .map(|page| page.rel_path.clone())
        .collect::<Vec<_>>();
    orphan_pages.sort();

    let mut missing_source_pages = pages
        .iter()
        .filter(|page| requires_source_front_matter(&page.rel_path))
        .filter(|page| page.source_refs.is_empty())
        .map(|page| page.rel_path.clone())
        .collect::<Vec<_>>();
    missing_source_pages.sort();

    let mut empty_pages = pages
        .iter()
        .filter(|page| !is_orphan_exempt(&page.rel_path))
        .filter(|page| markdown_body_without_front_matter(&page.content).trim().len() < 20)
        .map(|page| page.rel_path.clone())
        .collect::<Vec<_>>();
    empty_pages.sort();

    let mut duplicate_titles = titles
        .into_iter()
        .filter_map(|(title, pages)| {
            if title.is_empty() || pages.len() < 2 {
                return None;
            }
            Some(DuplicateTitleIssue {
                title,
                pages: pages.into_iter().collect(),
            })
        })
        .collect::<Vec<_>>();
    duplicate_titles.sort_by(|a, b| a.title.cmp(&b.title));

    Ok(LintStaticReport {
        schema_version: "0.4.5".to_string(),
        generated_by: "kb-cli".to_string(),
        generated_at,
        pages_scanned: pages.len(),
        wiki_links_found,
        broken_links,
        orphan_pages,
        missing_source_pages,
        empty_pages,
        duplicate_titles,
        report_path: None,
        dry_run: false,
    })
}

fn scan_wiki_pages(kb_path: &Path) -> Result<Vec<WikiPage>> {
    let wiki_dir = kb_path.join("wiki");
    if !wiki_dir.exists() {
        return Ok(Vec::new());
    }

    let mut pages = Vec::new();
    for item in WalkDir::new(&wiki_dir)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| entry.path().extension().and_then(|s| s.to_str()) == Some("md"))
    {
        let path = item.path().to_path_buf();
        let content = fs::read_to_string(&path).unwrap_or_default();
        let rel_path = relative_path_string(kb_path, &path);
        let title = extract_title(&content).unwrap_or_else(|| {
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Untitled")
                .to_string()
        });
        let source_refs = source_refs_from_markdown(&content);
        pages.push(WikiPage {
            rel_path,
            title,
            content,
            source_refs,
        });
    }

    pages.sort_by(|a, b| a.rel_path.cmp(&b.rel_path));
    Ok(pages)
}

fn page_lookup_keys(page: &WikiPage) -> Vec<String> {
    let mut keys = BTreeSet::new();
    let rel_no_ext = strip_md_extension(&page.rel_path);
    let without_wiki = rel_no_ext.strip_prefix("wiki/").unwrap_or(&rel_no_ext);
    let stem = Path::new(&page.rel_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(&page.title);

    keys.insert(normalize_page_key(&page.rel_path));
    keys.insert(normalize_page_key(&rel_no_ext));
    keys.insert(normalize_page_key(without_wiki));
    keys.insert(normalize_page_key(stem));
    keys.insert(normalize_page_key(&page.title));

    keys.into_iter().filter(|key| !key.is_empty()).collect()
}

fn clean_wikilink_target(input: &str) -> String {
    let before_alias = input.split('|').next().unwrap_or(input);
    let before_heading = before_alias.split('#').next().unwrap_or(before_alias);
    before_heading
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .trim_matches('`')
        .replace('\\', "/")
}

fn normalize_page_key(input: &str) -> String {
    let mut value = input
        .trim()
        .replace('\\', "/")
        .trim_start_matches("./")
        .trim_start_matches('/')
        .to_string();

    if value.ends_with(".md") {
        value.truncate(value.len() - 3);
    }

    value.to_lowercase()
}

fn normalize_title(input: &str) -> String {
    input.trim().to_lowercase()
}

fn strip_md_extension(input: &str) -> String {
    input.strip_suffix(".md").unwrap_or(input).to_string()
}

fn extract_title(content: &str) -> Option<String> {
    for line in markdown_body_without_front_matter(content).lines() {
        let trimmed = line.trim();
        if let Some(title) = trimmed.strip_prefix("# ") {
            let title = title.trim();
            if !title.is_empty() {
                return Some(title.to_string());
            }
        }
    }
    None
}

fn markdown_body_without_front_matter(content: &str) -> &str {
    let Some(rest) = content.strip_prefix("---\n").or_else(|| content.strip_prefix("---\r\n")) else {
        return content;
    };

    if let Some(end) = rest.find("\n---\n") {
        return &rest[end + 5..];
    }
    if let Some(end) = rest.find("\r\n---\r\n") {
        return &rest[end + 7..];
    }
    content
}

fn source_refs_from_markdown(content: &str) -> Vec<String> {
    let Some(front_matter) = extract_front_matter(content) else {
        return Vec::new();
    };

    let mut refs = Vec::new();
    for key in ["source_ids", "source_files", "sources", "raw_files"] {
        refs.extend(front_matter_values(front_matter, key));
    }

    let mut seen = BTreeSet::new();
    refs.into_iter()
        .map(|value| clean_ref(&value))
        .filter(|value| !value.is_empty())
        .filter(|value| seen.insert(value.clone()))
        .collect()
}

fn extract_front_matter(content: &str) -> Option<&str> {
    let content = content.strip_prefix("---\n").or_else(|| content.strip_prefix("---\r\n"))?;
    let end_lf = content.find("\n---\n");
    let end_crlf = content.find("\r\n---\r\n");

    match (end_lf, end_crlf) {
        (Some(a), Some(b)) => Some(&content[..a.min(b)]),
        (Some(a), None) => Some(&content[..a]),
        (None, Some(b)) => Some(&content[..b]),
        (None, None) => None,
    }
}

fn front_matter_values(front_matter: &str, key: &str) -> Vec<String> {
    let mut values = Vec::new();
    let mut in_block = false;
    let prefix = format!("{key}:");

    for line in front_matter.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with(&prefix) {
            in_block = true;
            let rest = trimmed[prefix.len()..].trim();
            if !rest.is_empty() {
                values.extend(parse_inline_values(rest));
                in_block = false;
            }
            continue;
        }

        if in_block {
            if trimmed.starts_with("- ") {
                values.push(trimmed.trim_start_matches("- ").trim().to_string());
            } else if trimmed.is_empty() || line.starts_with(' ') || line.starts_with('\t') {
                continue;
            } else {
                in_block = false;
            }
        }
    }

    values
}

fn parse_inline_values(input: &str) -> Vec<String> {
    let trimmed = input.trim();
    if trimmed.starts_with('[') && trimmed.ends_with(']') {
        let inner = &trimmed[1..trimmed.len() - 1];
        return inner.split(',').map(clean_ref).filter(|s| !s.is_empty()).collect();
    }
    vec![clean_ref(trimmed)].into_iter().filter(|s| !s.is_empty()).collect()
}

fn clean_ref(input: &str) -> String {
    input
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .trim_matches('`')
        .trim()
        .replace('\\', "/")
}

fn requires_source_front_matter(rel_path: &str) -> bool {
    if is_orphan_exempt(rel_path) {
        return false;
    }

    matches!(
        first_wiki_subdir(rel_path).as_deref(),
        Some("papers")
            | Some("notes")
            | Some("concepts")
            | Some("topics")
            | Some("people")
            | Some("methods")
            | Some("comparisons")
            | Some("timelines")
            | Some("questions")
    )
}

fn first_wiki_subdir(rel_path: &str) -> Option<String> {
    let without_prefix = rel_path.strip_prefix("wiki/")?;
    let first = without_prefix.split('/').next()?;
    Some(first.to_string())
}

fn is_orphan_exempt(rel_path: &str) -> bool {
    rel_path == "wiki/Home.md" || rel_path.starts_with("wiki/indexes/")
}

fn relative_path_string(kb_path: &Path, path: &Path) -> String {
    path.strip_prefix(kb_path)
        .unwrap_or(path)
        .components()
        .map(|component| component.as_os_str().to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join("/")
}

fn write_report(kb_path: &Path, report: &LintStaticReport) -> Result<PathBuf> {
    let reports_dir = kb_path.join("outputs/reports");
    fs::create_dir_all(&reports_dir)?;
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();
    let report_path = reports_dir.join(format!("lint_static_{timestamp}.md"));
    fs::write(&report_path, render_markdown_report(report))?;
    Ok(report_path)
}

fn print_report(report: &LintStaticReport) {
    println!("\nStatic lint summary:");
    println!("  pages scanned        : {}", report.pages_scanned);
    println!("  wiki links found     : {}", report.wiki_links_found);
    println!("  broken links         : {}", report.broken_links.len());
    println!("  orphan pages         : {}", report.orphan_pages.len());
    println!("  missing source pages : {}", report.missing_source_pages.len());
    println!("  empty pages          : {}", report.empty_pages.len());
    println!("  duplicate titles     : {}", report.duplicate_titles.len());

    if let Some(path) = &report.report_path {
        println!("  report               : {path}");
    } else if report.dry_run {
        println!("  report               : not written (--dry-run)");
    }

    if report.issue_count() == 0 {
        println!("\nNo static lint issues found.");
        return;
    }

    println!("\nTop issues:");
    for issue in report.broken_links.iter().take(5) {
        println!(
            "  broken link: {}:{} -> [[{}]]",
            issue.page, issue.line, issue.target
        );
    }
    for page in report.orphan_pages.iter().take(5) {
        println!("  orphan page: {page}");
    }
    for page in report.missing_source_pages.iter().take(5) {
        println!("  missing source front matter: {page}");
    }

    println!("\nNext steps:");
    println!("  1. Review the lint report.");
    println!("  2. Fix broken links and missing source front matter first.");
    println!("  3. Use Git diff before committing wiki cleanup edits.");
}

fn render_markdown_report(report: &LintStaticReport) -> String {
    let mut out = String::new();
    out.push_str("# Static Wiki Lint Report\n\n");
    out.push_str("This report is generated by `kb lint-static`. It does not modify `wiki/`.\n\n");
    out.push_str("## Summary\n\n");
    out.push_str(&format!("- Generated at: `{}`\n", report.generated_at));
    out.push_str(&format!("- Pages scanned: `{}`\n", report.pages_scanned));
    out.push_str(&format!("- Wiki links found: `{}`\n", report.wiki_links_found));
    out.push_str(&format!("- Broken links: `{}`\n", report.broken_links.len()));
    out.push_str(&format!("- Orphan pages: `{}`\n", report.orphan_pages.len()));
    out.push_str(&format!(
        "- Pages missing source front matter: `{}`\n",
        report.missing_source_pages.len()
    ));
    out.push_str(&format!("- Empty pages: `{}`\n", report.empty_pages.len()));
    out.push_str(&format!("- Duplicate title groups: `{}`\n\n", report.duplicate_titles.len()));

    out.push_str("## Broken WikiLinks\n\n");
    if report.broken_links.is_empty() {
        out.push_str("None.\n\n");
    } else {
        for issue in &report.broken_links {
            out.push_str(&format!(
                "- `{}` line {} -> `[[{}]]`\n",
                issue.page, issue.line, issue.target
            ));
        }
        out.push('\n');
    }

    push_path_section(&mut out, "Orphan Pages", &report.orphan_pages);
    push_path_section(
        &mut out,
        "Pages Missing Source Front Matter",
        &report.missing_source_pages,
    );
    push_path_section(&mut out, "Empty or Near-Empty Pages", &report.empty_pages);

    out.push_str("## Duplicate Titles\n\n");
    if report.duplicate_titles.is_empty() {
        out.push_str("None.\n\n");
    } else {
        for issue in &report.duplicate_titles {
            out.push_str(&format!("### `{}`\n\n", issue.title));
            for page in &issue.pages {
                out.push_str(&format!("- `{page}`\n"));
            }
            out.push('\n');
        }
    }

    out.push_str("## Recommended Cleanup Order\n\n");
    out.push_str("1. Fix broken links first.\n");
    out.push_str("2. Add `source_ids` / `source_files` front matter to source-derived pages.\n");
    out.push_str("3. Link useful orphan pages from indexes, topic pages, or concept pages.\n");
    out.push_str("4. Merge or rename duplicate concepts only after human review.\n");

    out
}

fn push_path_section(out: &mut String, title: &str, paths: &[String]) {
    out.push_str(&format!("## {title}\n\n"));
    if paths.is_empty() {
        out.push_str("None.\n\n");
    } else {
        for path in paths {
            out.push_str(&format!("- `{path}`\n"));
        }
        out.push('\n');
    }
}
