use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use anyhow::{anyhow, Result};
use clap::Args;
use serde::Serialize;
use walkdir::WalkDir;

#[derive(Debug, Clone, Args)]
pub struct QueryArgs {
    #[arg(value_name = "QUERY", required = true, num_args = 1.., help = "Keyword query to search under wiki/. Multiple words are matched with AND semantics.")]
    pub query: Vec<String>,

    #[arg(
        long,
        default_value_t = 10,
        help = "Maximum number of matching pages to show. Use 0 for no limit."
    )]
    pub limit: usize,

    #[arg(
        long,
        default_value_t = 2,
        help = "Maximum number of matching line snippets per page. Use 0 to hide snippets."
    )]
    pub snippets: usize,

    #[arg(long, help = "Print a machine-readable JSON query report")]
    pub json: bool,

    #[arg(
        long = "title-only",
        help = "Search only page titles and relative paths, not full Markdown bodies"
    )]
    pub title_only: bool,
}

#[derive(Debug, Clone)]
struct WikiPage {
    rel_path: String,
    title: String,
    body: String,
}

#[derive(Debug, Clone, Serialize)]
struct QuerySnippet {
    line: usize,
    text: String,
}

#[derive(Debug, Clone, Serialize)]
struct QueryResult {
    path: String,
    title: String,
    score: usize,
    matched_terms: Vec<String>,
    snippets: Vec<QuerySnippet>,
}

#[derive(Debug, Clone, Serialize)]
struct QueryReport {
    schema_version: String,
    generated_by: String,
    generated_at: String,
    query: String,
    terms: Vec<String>,
    title_only: bool,
    pages_scanned: usize,
    match_count: usize,
    returned_count: usize,
    results: Vec<QueryResult>,
}

pub fn execute(custom_kb: Option<&Path>, args: &QueryArgs) -> Result<()> {
    let kb_path = crate::commands::init::get_kb_path(custom_kb);
    if !kb_path.exists() {
        return Err(anyhow!(
            "knowledge base path does not exist: {}. Run `kb init` first.",
            kb_path.display()
        ));
    }

    let report = run_query(&kb_path, args)?;

    if args.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print_report(&report);
    }

    Ok(())
}

fn run_query(kb_path: &Path, args: &QueryArgs) -> Result<QueryReport> {
    let raw_query = args.query.join(" ");
    let terms = normalize_query_terms(&raw_query);
    if terms.is_empty() {
        return Err(anyhow!(
            "query must contain at least one non-empty search term"
        ));
    }

    let pages = scan_wiki_pages(kb_path)?;
    let pages_scanned = pages.len();
    let mut results = pages
        .into_iter()
        .filter_map(|page| score_page(&page, &terms, args.title_only, args.snippets))
        .collect::<Vec<_>>();

    results.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then_with(|| a.path.cmp(&b.path))
            .then_with(|| a.title.cmp(&b.title))
    });

    let match_count = results.len();
    if args.limit > 0 && results.len() > args.limit {
        results.truncate(args.limit);
    }

    Ok(QueryReport {
        schema_version: "0.5.0".to_string(),
        generated_by: "kb-cli query".to_string(),
        generated_at: chrono::Utc::now().to_rfc3339(),
        query: raw_query,
        terms,
        title_only: args.title_only,
        pages_scanned,
        match_count,
        returned_count: results.len(),
        results,
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
        let body = markdown_body_without_front_matter(&content).to_string();
        let title = extract_title(&body).unwrap_or_else(|| {
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Untitled")
                .to_string()
        });
        pages.push(WikiPage {
            rel_path: relative_path_string(kb_path, &path),
            title,
            body,
        });
    }

    pages.sort_by(|a, b| a.rel_path.cmp(&b.rel_path));
    Ok(pages)
}

fn score_page(
    page: &WikiPage,
    terms: &[String],
    title_only: bool,
    snippet_limit: usize,
) -> Option<QueryResult> {
    let title_lower = page.title.to_lowercase();
    let path_lower = page.rel_path.to_lowercase();
    let body_lower = page.body.to_lowercase();
    let searchable = if title_only {
        format!("{title_lower}\n{path_lower}")
    } else {
        format!("{title_lower}\n{path_lower}\n{body_lower}")
    };

    let mut matched_terms = Vec::new();
    let mut score = 0usize;

    for term in terms {
        if !searchable.contains(term) {
            return None;
        }

        matched_terms.push(term.clone());
        score += count_occurrences(&title_lower, term) * 8;
        score += count_occurrences(&path_lower, term) * 3;
        if !title_only {
            score += count_occurrences(&body_lower, term);
        }
    }

    let snippets = if snippet_limit == 0 || title_only {
        Vec::new()
    } else {
        find_snippets(&page.body, terms, snippet_limit)
    };

    score += snippets.len();

    Some(QueryResult {
        path: page.rel_path.clone(),
        title: page.title.clone(),
        score,
        matched_terms,
        snippets,
    })
}

fn normalize_query_terms(input: &str) -> Vec<String> {
    let mut seen = BTreeSet::new();
    input
        .split_whitespace()
        .map(|term| clean_query_term(&term.to_lowercase()))
        .filter(|term| !term.is_empty())
        .filter(|term| seen.insert(term.clone()))
        .collect()
}

fn clean_query_term(input: &str) -> String {
    let mut value = input.trim().to_string();
    loop {
        let cleaned = value
            .trim_matches(|ch: char| {
                matches!(ch, '"' | '\'' | '`')
                    || (ch.is_ascii_punctuation() && ch != '-' && ch != '_')
            })
            .trim()
            .to_string();

        if cleaned.len() == value.len() {
            return cleaned;
        }
        value = cleaned;
    }
}

fn count_occurrences(haystack: &str, needle: &str) -> usize {
    if needle.is_empty() {
        return 0;
    }
    haystack.matches(needle).count()
}

fn find_snippets(body: &str, terms: &[String], limit: usize) -> Vec<QuerySnippet> {
    let mut snippets = Vec::new();
    for (line_index, line) in body.lines().enumerate() {
        let lower = line.to_lowercase();
        if terms.iter().any(|term| lower.contains(term)) {
            snippets.push(QuerySnippet {
                line: line_index + 1,
                text: truncate_line(line.trim(), 180),
            });
            if snippets.len() >= limit {
                break;
            }
        }
    }
    snippets
}

fn truncate_line(input: &str, max_chars: usize) -> String {
    let char_count = input.chars().count();
    if char_count <= max_chars {
        return input.to_string();
    }

    let mut value = input.chars().take(max_chars).collect::<String>();
    value.push('…');
    value
}

fn extract_title(content: &str) -> Option<String> {
    for line in content.lines() {
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

fn print_report(report: &QueryReport) {
    println!("\nWiki query summary:");
    println!("  query         : {}", report.query);
    println!("  terms         : {}", report.terms.join(", "));
    println!("  pages scanned : {}", report.pages_scanned);
    println!("  matches       : {}", report.match_count);
    println!("  returned      : {}", report.returned_count);

    if report.results.is_empty() {
        println!("\nNo matching wiki pages found.");
        println!("\nNotes:");
        println!("  - v0.5.0 query is plain local keyword search only.");
        println!("  - It does not use embeddings, vector search, RAG, or an LLM.");
        return;
    }

    println!("\nResults:");
    for (index, result) in report.results.iter().enumerate() {
        println!(
            "\n{}. {}  [score: {}]",
            index + 1,
            result.title,
            result.score
        );
        println!("   path : {}", result.path);
        println!("   terms: {}", result.matched_terms.join(", "));

        for snippet in &result.snippets {
            println!("   L{}: {}", snippet.line, snippet.text);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_test_kb(name: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "kb_cli_query_{}_{}_{}",
            name,
            std::process::id(),
            nanos
        ))
    }

    #[test]
    fn normalize_query_terms_deduplicates_and_cleans() {
        assert_eq!(
            normalize_query_terms("  Thermal thermal `cloak`,  "),
            vec!["thermal".to_string(), "cloak".to_string()]
        );
    }

    #[test]
    fn query_finds_matching_wiki_pages_with_and_semantics() -> Result<()> {
        let kb_path = unique_test_kb("finds_pages");
        fs::create_dir_all(kb_path.join("wiki/concepts"))?;
        fs::create_dir_all(kb_path.join("wiki/papers"))?;
        fs::write(
            kb_path.join("wiki/concepts/thermal_cloak.md"),
            "# Thermal Cloak\n\nA thermal cloak controls heat flow around a protected region.\n",
        )?;
        fs::write(
            kb_path.join("wiki/papers/unrelated.md"),
            "# Unrelated\n\nThis page is about protein crystallization.\n",
        )?;

        let args = QueryArgs {
            query: vec!["thermal".to_string(), "cloak".to_string()],
            limit: 10,
            snippets: 2,
            json: false,
            title_only: false,
        };
        let report = run_query(&kb_path, &args)?;

        assert_eq!(report.pages_scanned, 2);
        assert_eq!(report.match_count, 1);
        assert_eq!(report.results[0].path, "wiki/concepts/thermal_cloak.md");
        assert!(!report.results[0].snippets.is_empty());

        let _ = fs::remove_dir_all(&kb_path);
        Ok(())
    }

    #[test]
    fn title_only_query_ignores_body_text() -> Result<()> {
        let kb_path = unique_test_kb("title_only");
        fs::create_dir_all(kb_path.join("wiki/notes"))?;
        fs::write(
            kb_path.join("wiki/notes/body_match.md"),
            "# Ordinary Note\n\nThis body mentions hidden_keyword.\n",
        )?;

        let args = QueryArgs {
            query: vec!["hidden_keyword".to_string()],
            limit: 10,
            snippets: 2,
            json: false,
            title_only: true,
        };
        let report = run_query(&kb_path, &args)?;

        assert_eq!(report.match_count, 0);

        let _ = fs::remove_dir_all(&kb_path);
        Ok(())
    }
}
