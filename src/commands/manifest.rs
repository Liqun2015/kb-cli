use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use anyhow::Result;
use clap::Args;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

#[derive(Debug, Clone, Args)]
pub struct StatusArgs {
    #[arg(long, help = "Scan raw/ and print status without writing processing/manifest.json")]
    pub dry_run: bool,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Manifest {
    pub schema_version: String,
    pub generated_by: String,
    pub updated_at: String,
    pub root: String,
    pub entries: Vec<ManifestEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestEntry {
    pub id: String,
    pub path: String,
    pub kind: String,
    pub extension: Option<String>,
    pub size_bytes: u64,
    pub content_hash: String,
    pub status: String,
    pub wiki_pages: Vec<String>,
    pub first_seen_at: String,
    pub last_seen_at: String,
}

#[derive(Debug, Default)]
pub struct ManifestSummary {
    pub total: usize,
    pub new_files: usize,
    pub changed_files: usize,
    pub removed_files: usize,
    pub by_kind: BTreeMap<String, usize>,
    pub manifest_path: PathBuf,
}

pub fn execute(custom_kb: Option<&Path>, args: &StatusArgs) -> Result<()> {
    let kb_path = crate::commands::init::get_kb_path(custom_kb);
    let summary = refresh_for_path(&kb_path, args.dry_run)?;
    print_summary(&summary, args.dry_run);
    Ok(())
}

pub fn refresh_for_path(kb_path: &Path, dry_run: bool) -> Result<ManifestSummary> {
    let manifest_path = kb_path.join("processing/manifest.json");
    let old_manifest = load_manifest(&manifest_path).unwrap_or_default();

    let mut old_by_path: HashMap<String, ManifestEntry> = old_manifest
        .entries
        .into_iter()
        .map(|entry| (entry.path.clone(), entry))
        .collect();

    let now = chrono::Utc::now().to_rfc3339();
    let raw_dir = kb_path.join("raw");
    let mut entries = Vec::new();
    let mut summary = ManifestSummary {
        manifest_path: manifest_path.clone(),
        ..ManifestSummary::default()
    };

    if raw_dir.exists() {
        for item in WalkDir::new(&raw_dir)
            .into_iter()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().is_file())
        {
            let path = item.path();
            let rel_path = relative_path_string(kb_path, path);
            let kind = classify_raw_file(kb_path, path);
            let size_bytes = fs::metadata(path)?.len();
            let content_hash = hash_file(path)?;
            let extension = path
                .extension()
                .and_then(|s| s.to_str())
                .map(|s| s.to_lowercase());

            let old = old_by_path.remove(&rel_path);
            let (first_seen_at, status, wiki_pages) = match old {
                Some(old_entry) => {
                    if old_entry.content_hash == content_hash {
                        (old_entry.first_seen_at, old_entry.status, old_entry.wiki_pages)
                    } else {
                        summary.changed_files += 1;
                        (old_entry.first_seen_at, "raw_changed".to_string(), old_entry.wiki_pages)
                    }
                }
                None => {
                    summary.new_files += 1;
                    (now.clone(), "raw_registered".to_string(), Vec::new())
                }
            };

            *summary.by_kind.entry(kind.clone()).or_insert(0) += 1;

            entries.push(ManifestEntry {
                id: make_entry_id(&rel_path, &content_hash),
                path: rel_path,
                kind,
                extension,
                size_bytes,
                content_hash,
                status,
                wiki_pages,
                first_seen_at,
                last_seen_at: now.clone(),
            });
        }
    }

    entries.sort_by(|a, b| a.path.cmp(&b.path));
    summary.total = entries.len();
    summary.removed_files = old_by_path.len();

    if !dry_run {
        fs::create_dir_all(kb_path.join("processing"))?;
        let manifest = Manifest {
            schema_version: "0.3.0".to_string(),
            generated_by: "kb-cli".to_string(),
            updated_at: now,
            root: kb_path.display().to_string(),
            entries,
        };
        fs::write(&manifest_path, serde_json::to_string_pretty(&manifest)?)?;
    }

    Ok(summary)
}

pub fn print_summary(summary: &ManifestSummary, dry_run: bool) {
    let label = if dry_run { "Manifest preview" } else { "Manifest refreshed" };
    println!("\n{label}:");
    println!("  total raw files : {}", summary.total);
    println!("  new files       : {}", summary.new_files);
    println!("  changed files   : {}", summary.changed_files);
    println!("  removed files   : {}", summary.removed_files);

    if !summary.by_kind.is_empty() {
        println!("  by kind:");
        for (kind, count) in &summary.by_kind {
            println!("    {:<8} : {}", kind, count);
        }
    }

    if dry_run {
        println!("  manifest        : not written (--dry-run)");
    } else {
        println!("  manifest        : {}", summary.manifest_path.display());
    }
}

fn load_manifest(path: &Path) -> Result<Manifest> {
    let content = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&content)?)
}

fn classify_raw_file(kb_path: &Path, path: &Path) -> String {
    let raw_dir = kb_path.join("raw");
    let relative_to_raw = match path.strip_prefix(&raw_dir) {
        Ok(relative) => relative,
        Err(_) => return "other".to_string(),
    };

    let first = relative_to_raw
        .components()
        .next()
        .map(|component| component.as_os_str().to_string_lossy().to_lowercase())
        .unwrap_or_else(|| "other".to_string());

    match first.as_str() {
        "papers" => "paper",
        "notes" => "note",
        "images" => "image",
        "datasets" => "dataset",
        "archives" => "archive",
        "repos" => "repo",
        _ => "other",
    }
    .to_string()
}

fn relative_path_string(kb_path: &Path, path: &Path) -> String {
    path.strip_prefix(kb_path)
        .unwrap_or(path)
        .components()
        .map(|component| component.as_os_str().to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join("/")
}

fn make_entry_id(path: &str, content_hash: &str) -> String {
    let path_hash = fnv1a64_bytes(path.as_bytes());
    let content_short = content_hash
        .strip_prefix("fnv1a64:")
        .unwrap_or(content_hash)
        .chars()
        .take(8)
        .collect::<String>();
    format!("{:016x}-{}", path_hash, content_short)
}

fn hash_file(path: &Path) -> Result<String> {
    let mut file = fs::File::open(path)?;
    let mut buffer = [0_u8; 8192];
    let mut hash = FNV_OFFSET_BASIS;

    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        for byte in &buffer[..read] {
            hash ^= *byte as u64;
            hash = hash.wrapping_mul(FNV_PRIME);
        }
    }

    Ok(format!("fnv1a64:{:016x}", hash))
}

fn fnv1a64_bytes(bytes: &[u8]) -> u64 {
    let mut hash = FNV_OFFSET_BASIS;
    for byte in bytes {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x00000100000001b3;
