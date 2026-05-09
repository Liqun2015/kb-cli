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
    #[arg(
        long,
        help = "Scan raw/ and print status without writing processing/manifest.json"
    )]
    pub dry_run: bool,

    #[arg(long, help = "Alias for --dry-run.")]
    pub preview: bool,

    #[arg(long, help = "Print a machine-readable JSON status summary")]
    pub json: bool,

    #[arg(
        long,
        help = "Print raw files that are not yet compiled into wiki pages"
    )]
    pub unprocessed: bool,
}

impl StatusArgs {
    pub fn is_dry_run(&self) -> bool {
        self.dry_run || self.preview
    }
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
    pub manifest_entries: usize,
    pub new_files: usize,
    pub changed_files: usize,
    pub removed_files: usize,
    pub compiled_files: usize,
    pub unprocessed_files: usize,
    pub unprocessed_paths: Vec<String>,
    pub by_kind: BTreeMap<String, usize>,
    pub manifest_path: PathBuf,
}

#[derive(Debug, Serialize)]
struct ManifestSummaryReport {
    total_raw_files: usize,
    manifest_entries: usize,
    new_files: usize,
    changed_files: usize,
    removed_files: usize,
    compiled_files: usize,
    unprocessed_files: usize,
    unprocessed_paths: Vec<String>,
    by_kind: BTreeMap<String, usize>,
    manifest_path: String,
    dry_run: bool,
}

pub fn execute(custom_kb: Option<&Path>, args: &StatusArgs) -> Result<()> {
    let kb_path = crate::commands::init::get_kb_path(custom_kb);
    let summary = refresh_for_path(&kb_path, args.is_dry_run())?;

    if args.json {
        print_json_summary(&summary, args.is_dry_run())?;
    } else if args.unprocessed {
        print_unprocessed(&summary);
    } else {
        print_summary(&summary, args.is_dry_run());
    }

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
                    if old_entry.status == "raw_missing" {
                        (
                            old_entry.first_seen_at,
                            "raw_registered".to_string(),
                            old_entry.wiki_pages,
                        )
                    } else if old_entry.content_hash == content_hash
                        || old_entry.content_hash.starts_with("fnv1a64:")
                    {
                        (
                            old_entry.first_seen_at,
                            old_entry.status,
                            old_entry.wiki_pages,
                        )
                    } else {
                        summary.changed_files += 1;
                        (
                            old_entry.first_seen_at,
                            "raw_changed".to_string(),
                            old_entry.wiki_pages,
                        )
                    }
                }
                None => {
                    summary.new_files += 1;
                    (now.clone(), "raw_registered".to_string(), Vec::new())
                }
            };

            if status == "compiled" && !wiki_pages.is_empty() {
                summary.compiled_files += 1;
            }

            if is_unprocessed_status(&status, &wiki_pages) {
                summary.unprocessed_files += 1;
                summary.unprocessed_paths.push(rel_path.clone());
            }

            *summary.by_kind.entry(kind.clone()).or_insert(0) += 1;
            summary.total += 1;

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

    let mut removed_entries = old_by_path.into_values().collect::<Vec<_>>();
    removed_entries.sort_by(|a, b| a.path.cmp(&b.path));
    summary.removed_files = removed_entries.len();

    for mut entry in removed_entries {
        entry.status = "raw_missing".to_string();
        entries.push(entry);
    }

    entries.sort_by(|a, b| a.path.cmp(&b.path));
    summary.manifest_entries = entries.len();
    summary.unprocessed_paths.sort();

    if !dry_run {
        fs::create_dir_all(kb_path.join("processing"))?;
        let manifest = Manifest {
            schema_version: "0.5.0".to_string(),
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
    let label = if dry_run {
        "Manifest preview"
    } else {
        "Manifest refreshed"
    };
    println!("\n{label}:");
    println!("  total raw files : {}", summary.total);
    println!("  manifest entries: {}", summary.manifest_entries);
    println!("  new files       : {}", summary.new_files);
    println!("  changed files   : {}", summary.changed_files);
    println!("  removed files   : {}", summary.removed_files);
    println!("  compiled        : {}", summary.compiled_files);
    println!("  unprocessed     : {}", summary.unprocessed_files);

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

    if summary.unprocessed_files > 0 {
        println!(
            "\nNext: run `kb status --unprocessed` to list files awaiting future prepare work."
        );
    }
}

fn print_json_summary(summary: &ManifestSummary, dry_run: bool) -> Result<()> {
    let report = ManifestSummaryReport {
        total_raw_files: summary.total,
        manifest_entries: summary.manifest_entries,
        new_files: summary.new_files,
        changed_files: summary.changed_files,
        removed_files: summary.removed_files,
        compiled_files: summary.compiled_files,
        unprocessed_files: summary.unprocessed_files,
        unprocessed_paths: summary.unprocessed_paths.clone(),
        by_kind: summary.by_kind.clone(),
        manifest_path: summary.manifest_path.display().to_string(),
        dry_run,
    };

    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

pub fn print_unprocessed(summary: &ManifestSummary) {
    if summary.unprocessed_paths.is_empty() {
        println!("No unprocessed raw files found.");
        return;
    }

    println!("Unprocessed raw files:");
    for path in &summary.unprocessed_paths {
        println!("  {path}");
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
        .strip_prefix("sha256:")
        .unwrap_or(content_hash)
        .chars()
        .take(12)
        .collect::<String>();
    format!("{:016x}-{}", path_hash, content_short)
}

fn is_unprocessed_status(status: &str, wiki_pages: &[String]) -> bool {
    status != "raw_missing" && status != "compiled" && wiki_pages.is_empty()
}

pub fn hash_file(path: &Path) -> Result<String> {
    let mut file = fs::File::open(path)?;
    let mut buffer = [0_u8; 8192];
    let mut hasher = Sha256::new();

    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }

    Ok(format!("sha256:{}", hasher.finalize_hex()))
}

struct Sha256 {
    state: [u32; 8],
    buffer: [u8; 64],
    buffer_len: usize,
    message_len_bytes: u64,
}

impl Sha256 {
    fn new() -> Self {
        Self {
            state: [
                0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
                0x5be0cd19,
            ],
            buffer: [0; 64],
            buffer_len: 0,
            message_len_bytes: 0,
        }
    }

    fn update(&mut self, mut data: &[u8]) {
        self.message_len_bytes = self.message_len_bytes.wrapping_add(data.len() as u64);

        if self.buffer_len > 0 {
            let needed = 64 - self.buffer_len;
            let take = needed.min(data.len());
            self.buffer[self.buffer_len..self.buffer_len + take].copy_from_slice(&data[..take]);
            self.buffer_len += take;
            data = &data[take..];

            if self.buffer_len == 64 {
                let block = self.buffer;
                self.transform(&block);
                self.buffer_len = 0;
            }
        }

        while data.len() >= 64 {
            let mut block = [0_u8; 64];
            block.copy_from_slice(&data[..64]);
            self.transform(&block);
            data = &data[64..];
        }

        if !data.is_empty() {
            self.buffer[..data.len()].copy_from_slice(data);
            self.buffer_len = data.len();
        }
    }

    fn finalize_hex(mut self) -> String {
        let bit_len = self.message_len_bytes.wrapping_mul(8);

        self.buffer[self.buffer_len] = 0x80;
        self.buffer_len += 1;

        if self.buffer_len > 56 {
            for byte in &mut self.buffer[self.buffer_len..] {
                *byte = 0;
            }
            let block = self.buffer;
            self.transform(&block);
            self.buffer_len = 0;
        }

        for byte in &mut self.buffer[self.buffer_len..56] {
            *byte = 0;
        }
        self.buffer[56..64].copy_from_slice(&bit_len.to_be_bytes());
        let block = self.buffer;
        self.transform(&block);

        let mut output = String::with_capacity(64);
        for word in self.state {
            output.push_str(&format!("{word:08x}"));
        }
        output
    }

    fn transform(&mut self, block: &[u8; 64]) {
        let mut w = [0_u32; 64];

        for (i, chunk) in block.chunks_exact(4).take(16).enumerate() {
            w[i] = u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        }

        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }

        let mut a = self.state[0];
        let mut b = self.state[1];
        let mut c = self.state[2];
        let mut d = self.state[3];
        let mut e = self.state[4];
        let mut f = self.state[5];
        let mut g = self.state[6];
        let mut h = self.state[7];

        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = h
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(SHA256_K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);

            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }

        self.state[0] = self.state[0].wrapping_add(a);
        self.state[1] = self.state[1].wrapping_add(b);
        self.state[2] = self.state[2].wrapping_add(c);
        self.state[3] = self.state[3].wrapping_add(d);
        self.state[4] = self.state[4].wrapping_add(e);
        self.state[5] = self.state[5].wrapping_add(f);
        self.state[6] = self.state[6].wrapping_add(g);
        self.state[7] = self.state[7].wrapping_add(h);
    }
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

const SHA256_K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];
