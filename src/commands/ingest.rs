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
pub struct SetupArgs {
    #[command(flatten)]
    pub ingest: IngestArgs,

    #[arg(
        long = "name",
        default_value = "knowledgebase",
        help = "Workspace directory name created under --source when --kb-path is not provided."
    )]
    pub name: String,

    #[arg(
        long,
        value_name = "TOPIC",
        help = "Default topic workspace to create. Defaults to the source directory name."
    )]
    pub topic: Option<String>,

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

pub fn execute(
    custom_kb: Option<&Path>,
    custom_source: Option<&Path>,
    args: &IngestArgs,
) -> Result<()> {
    validate_mode(args)?;

    let kb_path = resolve_kb_path_for_ingest(custom_kb, custom_source);
    let source_path = resolve_source_path(custom_source, &kb_path);
    let mode = ingest_mode(args);

    if !kb_path.exists() {
        return Err(anyhow!(
            "knowledge base path does not exist: {}. Run `kb init` first.",
            kb_path.display()
        ));
    }

    if !source_path.is_dir() {
        return Err(anyhow!(
            "source path is not a directory: {}",
            source_path.display()
        ));
    }

    ensure_raw_dirs(&kb_path)?;

    println!("Knowledge base: {}", kb_path.display());
    println!("Ingesting source files from: {}", source_path.display());
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
    let files = collect_source_files(&source_path, &kb_path, args.recursive)?;

    for file in files {
        if should_skip_file(&source_path, &kb_path, &file) {
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

pub fn execute_setup(
    custom_kb: Option<&Path>,
    custom_source: Option<&Path>,
    args: &SetupArgs,
) -> Result<()> {
    validate_mode(&args.ingest)?;
    if args.name.trim().is_empty() {
        return Err(anyhow!("workspace name cannot be empty"));
    }

    let (kb_path, source_path) = resolve_setup_paths(custom_kb, custom_source, args.name.trim());
    let topic = resolve_setup_topic(&source_path, args.topic.as_deref())?;

    println!("Setup started.");
    println!("Source directory: {}", source_path.display());
    println!("Knowledge base workspace: {}", kb_path.display());
    println!("Default topic: {}", topic);

    if !source_path.is_dir() {
        return Err(anyhow!(
            "source path is not a directory: {}",
            source_path.display()
        ));
    }

    crate::commands::init::execute_with_source(
        Some(&kb_path),
        Some(&source_path),
        Some(&topic),
        args.force_init,
    )?;

    if args.no_ingest {
        println!("\nSkipping ingest because --no-ingest was provided.");
    } else {
        println!("\nRunning ingest...");
        execute(Some(&kb_path), Some(&source_path), &args.ingest)?;

        if args.ingest.is_dry_run() {
            println!("\nDry run complete. Skipping metadata extraction and wiki build.");
            return Ok(());
        }
    }

    if args.skip_metadata {
        println!("\nSkipping metadata extraction because --skip-metadata was provided.");
    } else {
        println!("\nRunning metadata extraction...");
        crate::commands::extract_metadata::execute(Some(&kb_path), args.force_metadata)?;
    }

    if args.skip_build {
        println!("\nSkipping wiki build because --skip-build was provided.");
    } else {
        println!("\nBuilding wiki...");
        crate::commands::build_wiki::execute(Some(&kb_path))?;
    }

    println!("\nSetup complete.");
    println!("Wiki home: {}", kb_path.join("wiki/Home.md").display());
    println!(
        "Default topic workspace: {}",
        kb_path
            .join("topics")
            .join(crate::commands::topic::slugify_topic_name(&topic))
            .display()
    );
    println!("\nNext steps:");
    println!("  1. Review rules/LLM_WIKI_SCHEMA.md");
    println!("  2. Open wiki/Home.md in Obsidian");
    println!(
        "  3. Run: kb --kb-path \"{}\" status --unprocessed",
        kb_path.display()
    );
    Ok(())
}

fn resolve_setup_topic(source_path: &Path, explicit_topic: Option<&str>) -> Result<String> {
    if let Some(topic) = explicit_topic {
        let topic = topic.trim();
        if !topic.is_empty() {
            return Ok(topic.to_string());
        }
    }

    let Some(name) = source_path.file_name().and_then(|name| name.to_str()) else {
        return Err(anyhow!(
            "could not infer a default topic from source directory: {}. Please pass `--topic <topic>`.",
            source_path.display()
        ));
    };

    let name = name.trim();
    if name.is_empty() || name == "." || name == ".." {
        return Err(anyhow!(
            "could not infer a default topic from source directory: {}. Please pass `--topic <topic>`.",
            source_path.display()
        ));
    }

    Ok(name.to_string())
}

fn resolve_kb_path_for_ingest(custom_kb: Option<&Path>, custom_source: Option<&Path>) -> PathBuf {
    if let Some(path) = custom_kb {
        return crate::commands::init::resolve_user_path(path);
    }

    if let Some(source) = custom_source {
        return crate::commands::init::resolve_user_path(source)
            .join(crate::commands::init::DEFAULT_WORKSPACE_DIR);
    }

    crate::commands::init::get_kb_path(None)
}

fn resolve_source_path(custom_source: Option<&Path>, kb_path: &Path) -> PathBuf {
    custom_source
        .map(crate::commands::init::resolve_user_path)
        .unwrap_or_else(|| kb_path.to_path_buf())
}

fn resolve_setup_paths(
    custom_kb: Option<&Path>,
    custom_source: Option<&Path>,
    workspace_name: &str,
) -> (PathBuf, PathBuf) {
    let source_path = custom_source
        .map(crate::commands::init::resolve_user_path)
        .unwrap_or_else(|| {
            if custom_kb.is_some() {
                crate::commands::init::resolve_user_path(custom_kb.unwrap())
            } else {
                std::env::current_dir().unwrap()
            }
        });

    let kb_path = custom_kb
        .map(crate::commands::init::resolve_user_path)
        .unwrap_or_else(|| source_path.join(workspace_name));

    (kb_path, source_path)
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

fn collect_source_files(
    source_path: &Path,
    kb_path: &Path,
    recursive: bool,
) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    if recursive {
        let walker = WalkDir::new(source_path)
            .into_iter()
            .filter_entry(|entry| should_descend(entry, source_path, kb_path));
        for entry in walker.filter_map(|entry| entry.ok()) {
            if entry.file_type().is_file() {
                files.push(entry.path().to_path_buf());
            }
        }
    } else {
        for entry in fs::read_dir(source_path)? {
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

fn should_descend(entry: &DirEntry, source_path: &Path, kb_path: &Path) -> bool {
    if entry.depth() == 0 {
        return true;
    }

    if source_path != kb_path && entry.path().starts_with(kb_path) {
        return false;
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
            | "interfaces"
            | "logs"
            | ".git"
            | ".obsidian"
            | "target"
            | "knowledgebase"
            | "node_modules"
            | ".venv"
            | "venv"
            | "__pycache__"
    )
}

fn should_skip_file(source_path: &Path, kb_path: &Path, file: &Path) -> bool {
    if source_path != kb_path && file.starts_with(kb_path) {
        return true;
    }

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
            | "llm-wiki.toml"
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
            | "interfaces"
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
