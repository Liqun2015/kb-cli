use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::path::Path;

use anyhow::{anyhow, Result};
use clap::Args;
use serde::Serialize;
use walkdir::WalkDir;

use crate::commands::manifest::Manifest;

#[derive(Debug, Clone, Args)]
pub struct SyncWikiArgs {
    #[arg(long, help = "Scan wiki/ and preview manifest updates without writing processing/manifest.json")]
    pub dry_run: bool,

    #[arg(long, help = "Print a machine-readable JSON sync summary")]
    pub json: bool,
}

#[derive(Debug, Default, Serialize)]
struct SyncSummary {
    manifest_path: String,
    scanned_pages: usize,
    pages_with_sources: usize,
    source_references: usize,
    matched_sources: usize,
    unmatched_sources: Vec<String>,
    updated_entries: usize,
    dry_run: bool,
}

pub fn execute(custom_kb: Option<&Path>, args: &SyncWikiArgs) -> Result<()> {
    let kb_path = crate::commands::init::get_kb_path(custom_kb);
    if !kb_path.exists() {
        return Err(anyhow!(
            "knowledge base path does not exist: {}. Run `kb init` first.",
            kb_path.display()
        ));
    }

    let manifest_path = kb_path.join("processing/manifest.json");
    if !manifest_path.exists() {
        println!("Manifest not found. Refreshing manifest first...");
        crate::commands::manifest::refresh_for_path(&kb_path, false)?;
    }

    let mut manifest = load_manifest(&manifest_path)?;
    let summary = sync_manifest_from_wiki(&kb_path, &manifest_path, &mut manifest, args.dry_run)?;

    if args.json {
        println!("{}", serde_json::to_string_pretty(&summary)?);
    } else {
        print_summary(&summary);
    }

    Ok(())
}

fn sync_manifest_from_wiki(
    kb_path: &Path,
    manifest_path: &Path,
    manifest: &mut Manifest,
    dry_run: bool,
) -> Result<SyncSummary> {
    let wiki_dir = kb_path.join("wiki");
    let mut summary = SyncSummary {
        manifest_path: manifest_path.display().to_string(),
        dry_run,
        ..SyncSummary::default()
    };

    let mut refs_by_source: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

    if wiki_dir.exists() {
        for item in WalkDir::new(&wiki_dir)
            .into_iter()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().is_file())
            .filter(|entry| entry.path().extension().and_then(|s| s.to_str()).map(|s| s.eq_ignore_ascii_case("md")).unwrap_or(false))
        {
            summary.scanned_pages += 1;
            let page_path = relative_path_string(kb_path, item.path());
            let content = fs::read_to_string(item.path())?;
            let sources = source_refs_from_markdown(&content);

            if !sources.is_empty() {
                summary.pages_with_sources += 1;
            }

            for source in sources {
                summary.source_references += 1;
                refs_by_source.entry(source).or_default().insert(page_path.clone());
            }
        }
    }

    let mut by_path: HashMap<String, usize> = HashMap::new();
    let mut by_id: HashMap<String, usize> = HashMap::new();
    for (index, entry) in manifest.entries.iter().enumerate() {
        by_path.insert(normalize_ref(&entry.path), index);
        by_id.insert(normalize_ref(&entry.id), index);
    }

    let mut wiki_pages_by_entry: BTreeMap<usize, BTreeSet<String>> = BTreeMap::new();
    let mut unmatched = BTreeSet::new();

    for (source_ref, pages) in refs_by_source {
        let normalized = normalize_ref(&source_ref);
        let matched_index = by_path.get(&normalized).copied().or_else(|| by_id.get(&normalized).copied());

        if let Some(index) = matched_index {
            summary.matched_sources += 1;
            wiki_pages_by_entry.entry(index).or_default().extend(pages);
        } else {
            unmatched.insert(source_ref);
        }
    }

    for (index, pages) in wiki_pages_by_entry {
        if let Some(entry) = manifest.entries.get_mut(index) {
            let mut merged = entry.wiki_pages.iter().cloned().collect::<BTreeSet<_>>();
            let before = merged.len();
            merged.extend(pages);
            let after = merged.len();

            if after != before || entry.status != "compiled" {
                summary.updated_entries += 1;
            }

            entry.wiki_pages = merged.into_iter().collect();
            if entry.status != "raw_missing" && !entry.wiki_pages.is_empty() {
                entry.status = "compiled".to_string();
            }
        }
    }

    summary.unmatched_sources = unmatched.into_iter().collect();

    if !dry_run {
        manifest.schema_version = "0.4.3".to_string();
        manifest.generated_by = "kb-cli".to_string();
        manifest.updated_at = chrono::Utc::now().to_rfc3339();
        fs::write(manifest_path, serde_json::to_string_pretty(manifest)?)?;
    }

    Ok(summary)
}

fn print_summary(summary: &SyncSummary) {
    let label = if summary.dry_run { "Wiki sync preview" } else { "Wiki sync complete" };
    println!("\n{label}:");
    println!("  scanned pages      : {}", summary.scanned_pages);
    println!("  pages with sources : {}", summary.pages_with_sources);
    println!("  source references  : {}", summary.source_references);
    println!("  matched sources    : {}", summary.matched_sources);
    println!("  updated entries    : {}", summary.updated_entries);
    println!("  unmatched sources  : {}", summary.unmatched_sources.len());

    if !summary.unmatched_sources.is_empty() {
        println!("\nUnmatched source references:");
        for source in &summary.unmatched_sources {
            println!("  - {source}");
        }
    }

    if summary.dry_run {
        println!("\nDry run: processing/manifest.json was not written.");
    } else {
        println!("\nManifest updated: {}", summary.manifest_path);
        println!("Next: run `kb status --unprocessed` to see remaining raw files without wiki links.");
    }
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

fn normalize_ref(input: &str) -> String {
    let cleaned = clean_ref(input);
    cleaned
        .trim_start_matches("./")
        .trim_start_matches('/')
        .replace('\\', "/")
}

fn relative_path_string(kb_path: &Path, path: &Path) -> String {
    path.strip_prefix(kb_path)
        .unwrap_or(path)
        .components()
        .map(|component| component.as_os_str().to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join("/")
}

fn load_manifest(path: &Path) -> Result<Manifest> {
    let content = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&content)?)
}
