use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use clap::Args;
use regex::Regex;
use serde::Serialize;
use walkdir::WalkDir;

#[derive(Debug, Clone, Args)]
pub struct LinksArgs {
    #[arg(
        long,
        value_name = "PATH",
        help = "Path relative to the knowledge base to scan. Defaults to wiki/."
    )]
    pub path: Option<PathBuf>,

    #[arg(
        long,
        default_value_t = 100,
        help = "Maximum number of link records to return. Use 0 for no limit."
    )]
    pub limit: usize,

    #[arg(long, help = "Show only unresolved WikiLinks")]
    pub unresolved: bool,

    #[arg(long, help = "Show only ambiguous WikiLinks")]
    pub ambiguous: bool,

    #[arg(long, help = "Show only resolved WikiLinks")]
    pub resolved: bool,

    #[arg(long, help = "Print a machine-readable JSON link report")]
    pub json: bool,
}

#[derive(Debug, Clone)]
struct WikiPage {
    rel_path: String,
    title: String,
    content: String,
}

#[derive(Debug, Clone, Serialize)]
struct LinkRecord {
    source_path: String,
    line: usize,
    raw: String,
    target: String,
    alias: Option<String>,
    heading: Option<String>,
    status: String,
    resolved_paths: Vec<String>,
    context: String,
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
struct LinksReport {
    schema_version: String,
    generated_by: String,
    generated_at: String,
    search_path: String,
    pages_scanned: usize,
    link_count: usize,
    resolved_count: usize,
    unresolved_count: usize,
    ambiguous_count: usize,
    selected_count: usize,
    returned_count: usize,
    limit: usize,
    links: Vec<LinkRecord>,
    deferred_tasks: Vec<DeferredTask>,
}

pub fn execute(custom_kb: Option<&Path>, args: &LinksArgs) -> Result<()> {
    let kb_path = crate::commands::init::get_kb_path(custom_kb);
    if !kb_path.exists() {
        return Err(anyhow!(
            "knowledge base path does not exist: {}. Run `kb init` first.",
            kb_path.display()
        ));
    }

    let report = run_links_scan(&kb_path, args)?;
    if args.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print_report(&report);
    }
    Ok(())
}

fn run_links_scan(kb_path: &Path, args: &LinksArgs) -> Result<LinksReport> {
    let search_root = match &args.path {
        Some(path) if path.is_absolute() => path.clone(),
        Some(path) => kb_path.join(path),
        None => kb_path.join("wiki"),
    };

    if !search_root.exists() {
        return Err(anyhow!(
            "link scan path does not exist: {}",
            search_root.display()
        ));
    }

    let pages = scan_markdown_pages(kb_path, &search_root)?;
    let mut page_keys: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for page in &pages {
        for key in page_lookup_keys(page) {
            page_keys
                .entry(key)
                .or_default()
                .insert(page.rel_path.clone());
        }
    }

    let link_re = Regex::new(r"\[\[([^\]\n]+)\]\]")?;
    let mut links = Vec::new();

    for page in &pages {
        for (line_index, line) in page.content.lines().enumerate() {
            for captures in link_re.captures_iter(line) {
                let Some(raw_match) = captures.get(1) else {
                    continue;
                };
                let raw = raw_match.as_str().trim().to_string();
                let parsed = parse_wikilink(&raw);
                let lookup_key = normalize_page_key(&parsed.target);
                let resolved_paths = page_keys
                    .get(&lookup_key)
                    .map(|paths| paths.iter().cloned().collect::<Vec<_>>())
                    .unwrap_or_default();
                let status = match resolved_paths.len() {
                    0 => "unresolved",
                    1 => "resolved",
                    _ => "ambiguous",
                };

                links.push(LinkRecord {
                    source_path: page.rel_path.clone(),
                    line: line_index + 1,
                    raw,
                    target: parsed.target,
                    alias: parsed.alias,
                    heading: parsed.heading,
                    status: status.to_string(),
                    resolved_paths,
                    context: line.trim().chars().take(220).collect::<String>(),
                });
            }
        }
    }

    let link_count = links.len();
    let resolved_count = links
        .iter()
        .filter(|link| link.status == "resolved")
        .count();
    let unresolved_count = links
        .iter()
        .filter(|link| link.status == "unresolved")
        .count();
    let ambiguous_count = links
        .iter()
        .filter(|link| link.status == "ambiguous")
        .count();

    let deferred_tasks = build_deferred_tasks(&links, unresolved_count, ambiguous_count);
    let filtered = filter_links(links, args);
    let selected_count = filtered.len();
    let returned_links = if args.limit > 0 {
        filtered.into_iter().take(args.limit).collect::<Vec<_>>()
    } else {
        filtered
    };

    Ok(LinksReport {
        schema_version: "0.5.9".to_string(),
        generated_by: "kb-cli links".to_string(),
        generated_at: chrono::Utc::now().to_rfc3339(),
        search_path: relative_path_string(kb_path, &search_root),
        pages_scanned: source_pages.len(),
        link_count,
        resolved_count,
        unresolved_count,
        ambiguous_count,
        selected_count,
        returned_count: returned_links.len(),
        limit: args.limit,
        links: returned_links,
        deferred_tasks,
    })
}

fn filter_links(links: Vec<LinkRecord>, args: &LinksArgs) -> Vec<LinkRecord> {
    let filters = [args.resolved, args.unresolved, args.ambiguous]
        .into_iter()
        .filter(|enabled| *enabled)
        .count();

    if filters == 0 {
        return links;
    }

    links
        .into_iter()
        .filter(|link| {
            (args.resolved && link.status == "resolved")
                || (args.unresolved && link.status == "unresolved")
                || (args.ambiguous && link.status == "ambiguous")
        })
        .collect()
}

fn build_deferred_tasks(
    links: &[LinkRecord],
    total_unresolved: usize,
    total_ambiguous: usize,
) -> Vec<DeferredTask> {
    let mut tasks = Vec::new();

    let unresolved = links
        .iter()
        .filter(|link| link.status == "unresolved")
        .collect::<Vec<_>>();
    if !unresolved.is_empty() {
        let files = unique_source_files(&unresolved);
        let evidence = unresolved
            .iter()
            .take(20)
            .map(|link| format!("{}:{}: [[{}]]", link.source_path, link.line, link.raw))
            .collect::<Vec<_>>();
        tasks.push(DeferredTask {
            id: "wiki-link-repair-unresolved-001".to_string(),
            category: "wiki_link_repair".to_string(),
            target_agent: "Wiki Link Repair Agent".to_string(),
            goal: "Review unresolved WikiLinks and decide whether each target should be created, renamed, redirected to an existing page, or converted to plain text.".to_string(),
            requirements: vec![
                "Use kb links evidence as the starting point; do not invent missing pages without checking nearby wiki pages.".to_string(),
                "For each unresolved link, return the source page, line, original link, chosen action, and confidence.".to_string(),
                "Do not rewrite broad wiki structure without Manager LLM approval.".to_string(),
                "Preserve aliases when they carry useful display text.".to_string(),
            ],
            files,
            evidence,
            source_command: "kb links --unresolved".to_string(),
            priority: if total_unresolved > 0 { "high" } else { "normal" }.to_string(),
        });
    }

    let ambiguous = links
        .iter()
        .filter(|link| link.status == "ambiguous")
        .collect::<Vec<_>>();
    if !ambiguous.is_empty() {
        let files = unique_source_files(&ambiguous);
        let evidence = ambiguous
            .iter()
            .take(20)
            .map(|link| {
                format!(
                    "{}:{}: [[{}]] -> {:?}",
                    link.source_path, link.line, link.raw, link.resolved_paths
                )
            })
            .collect::<Vec<_>>();
        tasks.push(DeferredTask {
            id: "wiki-link-repair-ambiguous-001".to_string(),
            category: "wiki_link_disambiguation".to_string(),
            target_agent: "Wiki Link Repair Agent".to_string(),
            goal: "Resolve ambiguous WikiLinks that match multiple pages by selecting the intended target or rewriting the link to a more specific path.".to_string(),
            requirements: vec![
                "Use page context and surrounding concepts to select the intended target.".to_string(),
                "Return a table with source page, line, original link, candidate targets, selected target, and confidence.".to_string(),
                "If intent is unclear, mark it unresolved instead of guessing.".to_string(),
            ],
            files,
            evidence,
            source_command: "kb links --ambiguous".to_string(),
            priority: if total_ambiguous > 0 { "normal" } else { "low" }.to_string(),
        });
    }

    tasks
}

fn unique_source_files(links: &[&LinkRecord]) -> Vec<String> {
    let mut files = BTreeSet::new();
    for link in links {
        files.insert(link.source_path.clone());
    }
    files.into_iter().collect()
}

struct ParsedWikiLink {
    target: String,
    alias: Option<String>,
    heading: Option<String>,
}

fn parse_wikilink(raw: &str) -> ParsedWikiLink {
    let mut parts = raw.splitn(2, '|');
    let target_part = parts.next().unwrap_or(raw).trim();
    let alias = parts
        .next()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    let mut target_parts = target_part.splitn(2, '#');
    let target = clean_link_part(target_parts.next().unwrap_or(target_part));
    let heading = target_parts
        .next()
        .map(clean_link_part)
        .filter(|value| !value.is_empty());

    ParsedWikiLink {
        target,
        alias,
        heading,
    }
}

fn clean_link_part(input: &str) -> String {
    input
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .trim_matches('`')
        .replace('\\', "/")
}

fn scan_markdown_pages(kb_path: &Path, search_root: &Path) -> Result<Vec<WikiPage>> {
    let mut pages = Vec::new();

    for entry in WalkDir::new(search_root)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| entry.path().extension().and_then(|s| s.to_str()) == Some("md"))
    {
        let path = entry.path().to_path_buf();
        let content = fs::read_to_string(&path).unwrap_or_default();
        let title = extract_title(&content).unwrap_or_else(|| {
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Untitled")
                .to_string()
        });
        pages.push(WikiPage {
            rel_path: relative_path_string(kb_path, &path),
            title,
            content,
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
    let Some(rest) = content
        .strip_prefix("---\n")
        .or_else(|| content.strip_prefix("---\r\n"))
    else {
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

fn relative_path_string(kb_path: &Path, path: &Path) -> String {
    path.strip_prefix(kb_path)
        .unwrap_or(path)
        .components()
        .map(|component| component.as_os_str().to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join("/")
}

fn print_report(report: &LinksReport) {
    println!("kb links");
    println!("Search path: {}", report.search_path);
    println!("Pages scanned: {}", report.pages_scanned);
    println!("WikiLinks found: {}", report.link_count);
    println!("Resolved: {}", report.resolved_count);
    println!("Unresolved: {}", report.unresolved_count);
    println!("Ambiguous: {}", report.ambiguous_count);
    println!();

    if report.links.is_empty() {
        println!("No WikiLinks matched the selected filters.");
        return;
    }

    for link in &report.links {
        let target = if link.resolved_paths.is_empty() {
            "-".to_string()
        } else {
            link.resolved_paths.join(", ")
        };
        println!(
            "{}:{} [{}] [[{}]] -> {}",
            link.source_path, link.line, link.status, link.raw, target
        );
    }

    if report.limit > 0 && report.selected_count > report.returned_count {
        println!();
        println!(
            "Showing {} of {} selected links. Use --limit 0 to return all selected records.",
            report.returned_count, report.selected_count
        );
    }

    if !report.deferred_tasks.is_empty() {
        println!();
        println!("Deferred LLM/agent task hints:");
        for task in &report.deferred_tasks {
            println!(
                "  - {} -> {} ({})",
                task.id, task.target_agent, task.priority
            );
        }
        println!("Use `kb tasks` to generate a consolidated LLM/tasks/ handoff list.");
    }
}
