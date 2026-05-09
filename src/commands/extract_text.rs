use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use clap::Args;
use lopdf::Document;
use serde::Serialize;
use walkdir::WalkDir;

#[derive(Debug, Clone, Args)]
pub struct ExtractTextArgs {
    #[arg(long, value_name = "PATH", help = "A file or directory to extract. Defaults to raw/papers/.")]
    pub path: Option<PathBuf>,

    #[arg(long = "output-dir", value_name = "DIR", help = "Output directory relative to the knowledge base. Defaults to processing/text/.")]
    pub output_dir: Option<PathBuf>,

    #[arg(long, help = "Preview extraction plan without writing text files")]
    pub dry_run: bool,

    #[arg(long, help = "Alias for --dry-run")]
    pub preview: bool,

    #[arg(long, help = "Overwrite existing extracted text files")]
    pub force: bool,

    #[arg(long, default_value_t = 0, help = "Maximum number of files to process. Use 0 for no limit.")]
    pub limit: usize,

    #[arg(long, help = "Print a machine-readable JSON extraction report")]
    pub json: bool,
}

#[derive(Debug, Clone, Serialize)]
struct ExtractTextItem {
    source_path: String,
    output_path: Option<String>,
    status: String,
    bytes_written: usize,
    message: String,
}

#[derive(Debug, Clone, Serialize)]
struct ExtractTextReport {
    schema_version: String,
    generated_by: String,
    generated_at: String,
    input_path: String,
    output_dir: String,
    dry_run: bool,
    force: bool,
    files_seen: usize,
    files_processed: usize,
    files_written: usize,
    files_skipped: usize,
    files_failed: usize,
    items: Vec<ExtractTextItem>,
}

pub fn execute(custom_kb: Option<&Path>, args: &ExtractTextArgs) -> Result<()> {
    let kb_path = crate::commands::init::get_kb_path(custom_kb);
    if !kb_path.exists() {
        return Err(anyhow!(
            "knowledge base path does not exist: {}. Run `kb init` first.",
            kb_path.display()
        ));
    }

    let report = run_extract_text(&kb_path, args)?;
    if args.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print_report(&report);
    }
    Ok(())
}

fn run_extract_text(kb_path: &Path, args: &ExtractTextArgs) -> Result<ExtractTextReport> {
    let dry_run = args.dry_run || args.preview;
    let input_path = match &args.path {
        Some(path) if path.is_absolute() => path.clone(),
        Some(path) => kb_path.join(path),
        None => kb_path.join("raw/papers"),
    };
    let output_dir = match &args.output_dir {
        Some(path) if path.is_absolute() => path.clone(),
        Some(path) => kb_path.join(path),
        None => kb_path.join("processing/text"),
    };

    if !input_path.exists() {
        return Err(anyhow!("input path does not exist: {}", input_path.display()));
    }

    let sources = collect_sources(&input_path, args.limit)?;
    if !dry_run {
        fs::create_dir_all(&output_dir)?;
    }

    let mut items = Vec::new();
    let mut files_written = 0usize;
    let mut files_skipped = 0usize;
    let mut files_failed = 0usize;

    for source in &sources {
        let output_path = output_path_for(kb_path, &output_dir, source);
        if output_path.exists() && !args.force {
            files_skipped += 1;
            items.push(ExtractTextItem {
                source_path: relative_path_string(kb_path, source),
                output_path: Some(relative_path_string(kb_path, &output_path)),
                status: "skipped".to_string(),
                bytes_written: 0,
                message: "output exists; use --force to overwrite".to_string(),
            });
            continue;
        }

        match extract_source_text(source) {
            Ok(text) if text.trim().is_empty() => {
                files_failed += 1;
                items.push(ExtractTextItem {
                    source_path: relative_path_string(kb_path, source),
                    output_path: Some(relative_path_string(kb_path, &output_path)),
                    status: "failed".to_string(),
                    bytes_written: 0,
                    message: "no extractable text found; file may be scanned/image-based and should be left for future explicit OCR/LLM-assisted processing".to_string(),
                });
            }
            Ok(text) => {
                let bytes = text.as_bytes().len();
                if !dry_run {
                    if let Some(parent) = output_path.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    fs::write(&output_path, text)?;
                }
                files_written += 1;
                items.push(ExtractTextItem {
                    source_path: relative_path_string(kb_path, source),
                    output_path: Some(relative_path_string(kb_path, &output_path)),
                    status: if dry_run { "planned" } else { "written" }.to_string(),
                    bytes_written: if dry_run { 0 } else { bytes },
                    message: if dry_run {
                        "would extract text".to_string()
                    } else {
                        "text extracted with deterministic best-effort method".to_string()
                    },
                });
            }
            Err(err) => {
                files_failed += 1;
                items.push(ExtractTextItem {
                    source_path: relative_path_string(kb_path, source),
                    output_path: Some(relative_path_string(kb_path, &output_path)),
                    status: "failed".to_string(),
                    bytes_written: 0,
                    message: format!("{}; leave for future explicit OCR/LLM-assisted stage if needed", err),
                });
            }
        }
    }

    Ok(ExtractTextReport {
        schema_version: "0.5.6.1".to_string(),
        generated_by: "kb-cli extract-text".to_string(),
        generated_at: chrono::Utc::now().to_rfc3339(),
        input_path: relative_path_string(kb_path, &input_path),
        output_dir: relative_path_string(kb_path, &output_dir),
        dry_run,
        force: args.force,
        files_seen: sources.len(),
        files_processed: sources.len(),
        files_written,
        files_skipped,
        files_failed,
        items,
    })
}

fn collect_sources(input_path: &Path, limit: usize) -> Result<Vec<PathBuf>> {
    let mut sources = Vec::new();
    if input_path.is_file() {
        if is_supported_source(input_path) {
            sources.push(input_path.to_path_buf());
        }
        return Ok(sources);
    }

    for entry in WalkDir::new(input_path)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
    {
        let path = entry.path();
        if is_supported_source(path) {
            sources.push(path.to_path_buf());
            if limit > 0 && sources.len() >= limit {
                break;
            }
        }
    }
    sources.sort();
    Ok(sources)
}

fn is_supported_source(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|s| s.to_str()).map(|s| s.to_ascii_lowercase()),
        Some(ext) if matches!(ext.as_str(), "pdf" | "txt" | "md")
    )
}

fn extract_source_text(path: &Path) -> Result<String> {
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    match ext.as_str() {
        "txt" | "md" => fs::read_to_string(path).with_context(|| format!("could not read {}", path.display())),
        "pdf" => extract_pdf_text(path),
        _ => Err(anyhow!("unsupported source type")),
    }
}

fn extract_pdf_text(path: &Path) -> Result<String> {
    let doc = Document::load(path).with_context(|| format!("could not load PDF {}", path.display()))?;
    let pages = doc.get_pages().keys().copied().collect::<Vec<_>>();
    if pages.is_empty() {
        return Ok(String::new());
    }
    let text = doc
        .extract_text(&pages)
        .with_context(|| format!("could not extract PDF text from {}", path.display()))?;
    Ok(text)
}

fn output_path_for(kb_path: &Path, output_dir: &Path, source: &Path) -> PathBuf {
    let rel = source.strip_prefix(kb_path).unwrap_or(source);
    let mut filename = rel.to_string_lossy().replace('\\', "__").replace('/', "__");
    if filename.ends_with(".pdf") || filename.ends_with(".PDF") || filename.ends_with(".md") || filename.ends_with(".txt") {
        if let Some(idx) = filename.rfind('.') {
            filename.truncate(idx);
        }
    }
    output_dir.join(format!("{filename}.txt"))
}

fn relative_path_string(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn print_report(report: &ExtractTextReport) {
    println!("kb extract-text");
    println!("Input: {}", report.input_path);
    println!("Output: {}", report.output_dir);
    println!("Dry run: {}", report.dry_run);
    println!("Files seen: {}", report.files_seen);
    println!("Written/planned: {}", report.files_written);
    println!("Skipped: {}", report.files_skipped);
    println!("Failed: {}", report.files_failed);
    println!();

    for item in &report.items {
        let output = item.output_path.as_deref().unwrap_or("-");
        println!("[{}] {} -> {}", item.status, item.source_path, output);
        if !item.message.is_empty() {
            println!("  {}", item.message);
        }
    }

    if report.items.is_empty() {
        println!("No supported source files found. Supported input types: .pdf, .txt, .md");
    }
}
