use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use clap::Args;
use regex::Regex;
use serde::Serialize;
use walkdir::WalkDir;

#[derive(Debug, Clone, Args)]
pub struct GrepArgs {
    #[arg(value_name = "PATTERN", help = "Text or regex pattern to search for")]
    pub pattern: String,

    #[arg(long, value_name = "PATH", help = "Path relative to the knowledge base to search. Defaults to wiki/.")]
    pub path: Option<PathBuf>,

    #[arg(long, default_value_t = 50, help = "Maximum number of matching lines to show. Use 0 for no limit.")]
    pub limit: usize,

    #[arg(long, help = "Treat PATTERN as a regular expression")]
    pub regex: bool,

    #[arg(long = "case-sensitive", help = "Use case-sensitive matching")]
    pub case_sensitive: bool,

    #[arg(long, help = "Search all files, not only text-like files")]
    pub all: bool,

    #[arg(long, help = "Print a machine-readable JSON grep report")]
    pub json: bool,
}

#[derive(Debug, Clone, Serialize)]
struct GrepMatch {
    path: String,
    line: usize,
    text: String,
}

#[derive(Debug, Clone, Serialize)]
struct GrepReport {
    schema_version: String,
    generated_by: String,
    generated_at: String,
    pattern: String,
    search_path: String,
    regex: bool,
    case_sensitive: bool,
    files_scanned: usize,
    match_count: usize,
    returned_count: usize,
    limit: usize,
    matches: Vec<GrepMatch>,
}

pub fn execute(custom_kb: Option<&Path>, args: &GrepArgs) -> Result<()> {
    let kb_path = crate::commands::init::get_kb_path(custom_kb);
    if !kb_path.exists() {
        return Err(anyhow!(
            "knowledge base path does not exist: {}. Run `kb init` first.",
            kb_path.display()
        ));
    }

    let report = run_grep(&kb_path, args)?;
    if args.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print_report(&report);
    }
    Ok(())
}

fn run_grep(kb_path: &Path, args: &GrepArgs) -> Result<GrepReport> {
    if args.pattern.trim().is_empty() {
        return Err(anyhow!("grep pattern must not be empty"));
    }

    let search_root = match &args.path {
        Some(path) if path.is_absolute() => path.clone(),
        Some(path) => kb_path.join(path),
        None => kb_path.join("wiki"),
    };

    if !search_root.exists() {
        return Err(anyhow!("search path does not exist: {}", search_root.display()));
    }

    let matcher = Matcher::new(&args.pattern, args.regex, args.case_sensitive)?;
    let mut files_scanned = 0usize;
    let mut matches = Vec::new();
    let mut match_count = 0usize;

    for entry in WalkDir::new(&search_root)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
    {
        let path = entry.path();
        if !args.all && !is_text_like(path) {
            continue;
        }

        let content = match fs::read_to_string(path) {
            Ok(content) => content,
            Err(_) => continue,
        };
        files_scanned += 1;

        for (line_idx, line) in content.lines().enumerate() {
            if matcher.is_match(line) {
                match_count += 1;
                if args.limit == 0 || matches.len() < args.limit {
                    matches.push(GrepMatch {
                        path: relative_path_string(kb_path, path),
                        line: line_idx + 1,
                        text: line.trim_end().to_string(),
                    });
                }
            }
        }
    }

    Ok(GrepReport {
        schema_version: "0.5.6.1".to_string(),
        generated_by: "kb-cli grep".to_string(),
        generated_at: chrono::Utc::now().to_rfc3339(),
        pattern: args.pattern.clone(),
        search_path: relative_path_string(kb_path, &search_root),
        regex: args.regex,
        case_sensitive: args.case_sensitive,
        files_scanned,
        match_count,
        returned_count: matches.len(),
        limit: args.limit,
        matches,
    })
}

fn print_report(report: &GrepReport) {
    println!("kb grep");
    println!("Pattern: {}", report.pattern);
    println!("Search path: {}", report.search_path);
    println!("Files scanned: {}", report.files_scanned);
    println!("Matches: {}", report.match_count);
    println!();

    if report.matches.is_empty() {
        println!("No matches found.");
        return;
    }

    for item in &report.matches {
        println!("{}:{}: {}", item.path, item.line, item.text);
    }

    if report.limit > 0 && report.match_count > report.returned_count {
        println!();
        println!(
            "Showing {} of {} matches. Use --limit 0 to show all matches.",
            report.returned_count, report.match_count
        );
    }
}

struct Matcher {
    regex: Option<Regex>,
    needle: Option<String>,
    case_sensitive: bool,
}

impl Matcher {
    fn new(pattern: &str, regex: bool, case_sensitive: bool) -> Result<Self> {
        if regex {
            let pattern = if case_sensitive {
                pattern.to_string()
            } else {
                format!("(?i:{pattern})")
            };
            Ok(Self {
                regex: Some(Regex::new(&pattern)?),
                needle: None,
                case_sensitive,
            })
        } else {
            Ok(Self {
                regex: None,
                needle: Some(if case_sensitive {
                    pattern.to_string()
                } else {
                    pattern.to_lowercase()
                }),
                case_sensitive,
            })
        }
    }

    fn is_match(&self, line: &str) -> bool {
        if let Some(regex) = &self.regex {
            return regex.is_match(line);
        }

        let needle = self.needle.as_deref().unwrap_or_default();
        if self.case_sensitive {
            line.contains(needle)
        } else {
            line.to_lowercase().contains(needle)
        }
    }
}

fn is_text_like(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|s| s.to_str()).map(|s| s.to_ascii_lowercase()),
        Some(ext) if matches!(
            ext.as_str(),
            "md" | "txt" | "json" | "yaml" | "yml" | "toml" | "csv" | "tsv" | "rs" | "py" | "js" | "ts" | "html" | "css"
        )
    )
}

fn relative_path_string(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}
