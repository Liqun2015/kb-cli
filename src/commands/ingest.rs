use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use clap::Args;
use walkdir::{DirEntry, WalkDir};

#[derive(Debug, Clone, Args)]
pub struct IngestArgs {
    #[arg(
        long,
        help = "Copy source files into raw/ subfolders. This is the default safe mode."
    )]
    pub copy: bool,

    #[arg(
        long = "move",
        help = "Move source files into raw/ subfolders instead of copying them."
    )]
    pub move_files: bool,

    #[arg(
        long,
        help = "Scan subdirectories recursively. Generated/project folders such as raw, wiki, .git, and target are skipped."
    )]
    pub recursive: bool,

    #[arg(
        long,
        help = "Print planned ingest actions without copying or moving files."
    )]
    pub dry_run: bool,

    #[arg(long, help = "Alias for --dry-run.")]
    pub preview: bool,
}

impl IngestArgs {
    pub fn is_dry_run(&self) -> bool {
        self.dry_run || self.preview
    }
}

#[derive(Debug, Clone, Args)]
pub struct BootstrapArgs {
    #[command(flatten)]
    pub ingest: IngestArgs,

    #[arg(
        long = "force-init",
        help = "Pass --force to init before ingesting files."
    )]
    pub force_init: bool,

    #[arg(
        long = "force-metadata",
        help = "Overwrite existing papers_metadata.json during metadata extraction."
    )]
    pub force_metadata: bool,

    #[arg(
        long = "no-ingest",
        help = "Run init, metadata extraction, and wiki build without organizing root files."
    )]
    pub no_ingest: bool,

    #[arg(long = "skip-metadata", help = "Skip PDF metadata extraction.")]
    pub skip_metadata: bool,

    #[arg(long = "skip-build", help = "Skip wiki generation.")]
    pub skip_build: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IngestMode {
    Copy,
    Move,
}

#[derive(Debug, Default)]
struct IngestSummary {
    papers: usize,
    notes: usize,
    images: usize,
    datasets: usize,
    archives: usize,
    repos: usize,
    other: usize,
    skipped: usize,
}

impl IngestSummary {
    fn total(&self) -> usize {
        self.papers
            + self.notes
            + self.images
            + self.datasets
            + self.archives
            + self.repos
            + self.other
    }

    fn record(&mut self, kind: SourceKind) {
        match kind {
            SourceKind::Paper => self.papers += 1,
            SourceKind::Note => self.notes += 1,
            SourceKind::Image => self.images += 1,
            SourceKind::Dataset => self.datasets += 1,
            SourceKind::Archive => self.archives += 1,
            SourceKind::Repo => self.repos += 1,
            SourceKind::Other => self.other += 1,
        }
    }

    fn print(&self, dry_run: bool) {
        let label = if dry_run { "Planned" } else { "Ingested" };
        println!("\n{label} files:");
        println!("  papers   : {}", self.papers);
        println!("  notes    : {}", self.notes);
        println!("  images   : {}", self.images);
        println!("  datasets : {}", self.datasets);
        println!("  archives : {}", self.archives);
        println!("  repos    : {}", self.repos);
        println!("  other    : {}", self.other);
        println!("  skipped  : {}", self.skipped);
        println!("  total    : {}", self.total());
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SourceKind {
    Paper,
    Note,
    Image,
    Dataset,
    Archive,
    Repo,
    Other,
}

impl SourceKind {
    fn destination_dir(self) -> &'static str {
        match self {
            SourceKind::Paper => "raw/papers",
            SourceKind::Note => "raw/notes",
            SourceKind::Image => "raw/images",
            SourceKind::Dataset => "raw/datasets",
            SourceKind::Archive => "raw/archives",
            SourceKind::Repo => "raw/repos",
            SourceKind::Other => "raw/other",
        }
    }
}

pub fn execute(custom_kb: Option<&Path>, args: &IngestArgs) -> Result<()> {
    validate_mode(args)?;

    let kb_path = crate::commands::init::get_kb_path(custom_kb);
    let mode = ingest_mode(args);

    if !kb_path.exists() {
        return Err(anyhow!(
            "knowledge base path does not exist: {}. Run `kb init` first.",
            kb_path.display()
        ));
    }

    ensure_raw_dirs(&kb_path)?;

    println!("Ingesting source files from: {}", kb_path.display());
    println!(
        "Mode: {}{}",
        match mode {
            IngestMode::Copy => "copy",
            IngestMode::Move => "move",
        },
        if args.is_dry_run() { " (dry run)" } else { "" }
    );
    println!("Recursive: {}", if args.recursive { "yes" } else { "no" });

    let mut summary = IngestSummary::default();
    let files = collect_source_files(&kb_path, args.recursive)?;

    for file in files {
        if should_skip_file(&kb_path, &file) {
            summary.skipped += 1;
            continue;
        }

        let kind = classify_file(&file);
        let dest_dir = kb_path.join(kind.destination_dir());
        fs::create_dir_all(&dest_dir)?;

        let file_name = match file.file_name().and_then(|s| s.to_str()) {
            Some(name) => name.to_string(),
            None => {
                summary.skipped += 1;
                continue;
            }
        };

        let dest_path = match resolve_destination_path(&file, &dest_dir, &file_name)? {
            Some(path) => path,
            None => {
                let primary_dest = dest_dir.join(&file_name);
                println!(
                    "  Skipping duplicate content: {} -> {}",
                    file.display(),
                    primary_dest
                        .strip_prefix(&kb_path)
                        .unwrap_or(&primary_dest)
                        .display()
                );
                summary.skipped += 1;
                continue;
            }
        };
        let relative_dest = dest_path.strip_prefix(&kb_path).unwrap_or(&dest_path);

        println!("  {} -> {}", file.display(), relative_dest.display());

        if !args.is_dry_run() {
            match mode {
                IngestMode::Copy => {
                    fs::copy(&file, &dest_path)?;
                }
                IngestMode::Move => {
                    fs::rename(&file, &dest_path).or_else(|_| {
                        fs::copy(&file, &dest_path)?;
                        fs::remove_file(&file)
                    })?;
                }
            }
        }

        summary.record(kind);
    }

    summary.print(args.is_dry_run());

    if args.is_dry_run() {
        println!("\nDry run: manifest not refreshed.");
    } else {
        let manifest_summary = crate::commands::manifest::refresh_for_path(&kb_path, false)?;
        crate::commands::manifest::print_summary(&manifest_summary, false);
    }

    Ok(())
}

pub fn execute_bootstrap(custom_kb: Option<&Path>, args: &BootstrapArgs) -> Result<()> {
    validate_mode(&args.ingest)?;

    println!("Bootstrap started.");

    crate::commands::init::execute(custom_kb, args.force_init)?;

    if args.no_ingest {
        println!("\nSkipping ingest because --no-ingest was provided.");
    } else {
        println!("\nRunning ingest...");
        execute(custom_kb, &args.ingest)?;

        if args.ingest.is_dry_run() {
            println!("\nDry run complete. Skipping metadata extraction and wiki build.");
            return Ok(());
        }
    }

    if args.skip_metadata {
        println!("\nSkipping metadata extraction because --skip-metadata was provided.");
    } else {
        println!("\nRunning metadata extraction...");
        crate::commands::extract_metadata::execute(custom_kb, args.force_metadata)?;
    }

    if args.skip_build {
        println!("\nSkipping wiki build because --skip-build was provided.");
    } else {
        println!("\nBuilding wiki...");
        crate::commands::build_wiki::execute(custom_kb)?;
    }

    let kb_path = crate::commands::init::get_kb_path(custom_kb);
    println!("\nBootstrap complete.");
    println!("Wiki home: {}", kb_path.join("wiki/Home.md").display());
    println!("\nNext steps:");
    println!("  1. Review rules/LLM_WIKI_SCHEMA.md");
    println!("  2. Open wiki/Home.md in Obsidian");
    println!(
        "  3. Run: kb --kb-path \"{}\" status --unprocessed",
        kb_path.display()
    );
    Ok(())
}

fn validate_mode(args: &IngestArgs) -> Result<()> {
    if args.copy && args.move_files {
        return Err(anyhow!("use either --copy or --move, not both"));
    }
    Ok(())
}

fn ingest_mode(args: &IngestArgs) -> IngestMode {
    if args.move_files {
        IngestMode::Move
    } else if args.copy {
        IngestMode::Copy
    } else {
        IngestMode::Copy
    }
}

fn ensure_raw_dirs(kb_path: &Path) -> Result<()> {
    for dir in [
        "raw/papers",
        "raw/notes",
        "raw/images",
        "raw/datasets",
        "raw/archives",
        "raw/repos",
        "raw/other",
    ] {
        fs::create_dir_all(kb_path.join(dir))?;
    }
    Ok(())
}

fn collect_source_files(kb_path: &Path, recursive: bool) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    if recursive {
        let walker = WalkDir::new(kb_path)
            .into_iter()
            .filter_entry(|entry| should_descend(entry));
        for entry in walker.filter_map(|entry| entry.ok()) {
            if entry.file_type().is_file() {
                files.push(entry.path().to_path_buf());
            }
        }
    } else {
        for entry in fs::read_dir(kb_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                files.push(path);
            }
        }
    }

    files.sort();
    Ok(files)
}

fn should_descend(entry: &DirEntry) -> bool {
    if entry.depth() == 0 {
        return true;
    }

    let name = entry.file_name().to_string_lossy().to_lowercase();
    !matches!(
        name.as_str(),
        "raw"
            | "wiki"
            | "rules"
            | "processing"
            | "references"
            | "topics"
            | "outputs"
            | "logs"
            | ".git"
            | ".obsidian"
            | "target"
            | "node_modules"
            | ".venv"
            | "venv"
            | "__pycache__"
    )
}

fn should_skip_file(kb_path: &Path, file: &Path) -> bool {
    if is_inside_managed_dir(kb_path, file) {
        return true;
    }

    let filename = file
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    if matches!(
        filename.as_str(),
        "readme.md"
            | "license"
            | "license.md"
            | "cargo.toml"
            | "cargo.lock"
            | ".gitignore"
            | ".model_config.json"
            | ".model_config.example.json"
            | ".model_switch_input.json"
            | ".model_switch_output.json"
    ) {
        return true;
    }

    let ext = extension(file);
    matches!(ext.as_str(), "exe" | "dll" | "bat" | "cmd" | "ps1" | "sh")
}

fn is_inside_managed_dir(kb_path: &Path, file: &Path) -> bool {
    let relative = match file.strip_prefix(kb_path) {
        Ok(path) => path,
        Err(_) => return false,
    };

    let first = match relative.components().next() {
        Some(component) => component.as_os_str().to_string_lossy().to_lowercase(),
        None => return false,
    };

    matches!(
        first.as_str(),
        "raw"
            | "wiki"
            | "rules"
            | "processing"
            | "references"
            | "topics"
            | "outputs"
            | "logs"
            | ".git"
            | ".obsidian"
            | "target"
    )
}

fn classify_file(path: &Path) -> SourceKind {
    let ext = extension(path);
    match ext.as_str() {
        "pdf" => SourceKind::Paper,
        "md" | "markdown" | "txt" | "doc" | "docx" | "rtf" | "html" | "htm" => SourceKind::Note,
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" | "bmp" | "tif" | "tiff" => {
            SourceKind::Image
        }
        "csv" | "tsv" | "xlsx" | "xls" | "json" | "jsonl" | "xml" | "yaml" | "yml" | "parquet" => {
            SourceKind::Dataset
        }
        "zip" | "rar" | "7z" | "tar" | "gz" | "bz2" | "xz" => SourceKind::Archive,
        "rs" | "py" | "js" | "ts" | "tsx" | "jsx" | "java" | "c" | "cpp" | "h" | "hpp" | "go"
        | "rb" | "php" | "swift" | "kt" | "scala" | "r" | "m" | "jl" | "lua" | "pl" | "sql"
        | "toml" => SourceKind::Repo,
        _ => SourceKind::Other,
    }
}

fn extension(path: &Path) -> String {
    path.extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .trim_start_matches('.')
        .to_lowercase()
}

fn resolve_destination_path(
    source: &Path,
    dest_dir: &Path,
    file_name: &str,
) -> Result<Option<PathBuf>> {
    let primary_dest = dest_dir.join(file_name);
    if !primary_dest.exists() {
        return Ok(Some(primary_dest));
    }

    let source_hash = crate::commands::manifest::hash_file(source)?;
    let dest_hash = crate::commands::manifest::hash_file(&primary_dest)?;
    if source_hash == dest_hash {
        return Ok(None);
    }

    Ok(Some(unique_destination_path(dest_dir, file_name)))
}

fn unique_destination_path(dest_dir: &Path, file_name: &str) -> PathBuf {
    let candidate = dest_dir.join(file_name);
    if !candidate.exists() {
        return candidate;
    }

    let original = Path::new(file_name);
    let stem = original
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("file");
    let ext = original.extension().and_then(|s| s.to_str()).unwrap_or("");

    for i in 1.. {
        let name = if ext.is_empty() {
            format!("{stem}_{i}")
        } else {
            format!("{stem}_{i}.{ext}")
        };
        let path = dest_dir.join(name);
        if !path.exists() {
            return path;
        }
    }

    unreachable!("unbounded duplicate-name search should always return")
}
