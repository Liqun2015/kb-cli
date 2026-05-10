use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use clap::Args;
use regex::Regex;
use serde::Serialize;
use walkdir::WalkDir;

#[derive(Debug, Clone, Args)]
pub struct ExtractSectionsArgs {
    #[arg(
        long,
        value_name = "PATH",
        help = "Text file or directory to section. Defaults to processing/text/."
    )]
    pub path: Option<PathBuf>,

    #[arg(
        long = "output-dir",
        value_name = "DIR",
        help = "Output directory relative to the knowledge base. Defaults to processing/sections/."
    )]
    pub output_dir: Option<PathBuf>,

    #[arg(long, help = "Preview section extraction without writing files")]
    pub dry_run: bool,

    #[arg(long, help = "Alias for --dry-run")]
    pub preview: bool,

    #[arg(long, help = "Overwrite existing section files")]
    pub force: bool,

    #[arg(
        long,
        default_value_t = 0,
        help = "Maximum number of text files to process. Use 0 for no limit."
    )]
    pub limit: usize,

    #[arg(
        long,
        help = "Do not write an LLM/OCR fallback task for missing sections"
    )]
    pub no_llm_task: bool,

    #[arg(long, help = "Print a machine-readable JSON section extraction report")]
    pub json: bool,
}

#[derive(Debug, Clone, Serialize)]
struct SectionSlice {
    found: bool,
    start_line: Option<usize>,
    end_line: Option<usize>,
    line_count: usize,
    char_count: usize,
    quality: String,
}

#[derive(Debug, Clone, Serialize)]
struct ExtractSectionsItem {
    source_path: String,
    article_dir: String,
    introduction_path: Option<String>,
    references_path: Option<String>,
    introduction: SectionSlice,
    references: SectionSlice,
    status: String,
    needs_llm_or_ocr: bool,
    message: String,
}

#[derive(Debug, Clone, Serialize)]
struct ExtractSectionsReport {
    schema_version: String,
    generated_by: String,
    generated_at: String,
    search_path: String,
    output_dir: String,
    dry_run: bool,
    force: bool,
    files_seen: usize,
    files_processed: usize,
    full_success_count: usize,
    partial_count: usize,
    missing_count: usize,
    llm_task_path: Option<String>,
    items: Vec<ExtractSectionsItem>,
}

struct SectionText {
    text: String,
    start_line: usize,
    end_line: usize,
}

struct SectionPatterns {
    introduction_heading: Regex,
    references_heading: Regex,
    generic_heading: Regex,
    appendix_heading: Regex,
}

pub fn execute(custom_kb: Option<&Path>, args: &ExtractSectionsArgs) -> Result<()> {
    let kb_path = crate::commands::init::get_kb_path(custom_kb);
    if !kb_path.exists() {
        return Err(anyhow!(
            "knowledge base path does not exist: {}. Run `kb init` first.",
            kb_path.display()
        ));
    }

    let report = run_extract_sections(&kb_path, args)?;
    if args.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print_report(&report);
    }
    Ok(())
}

fn run_extract_sections(
    kb_path: &Path,
    args: &ExtractSectionsArgs,
) -> Result<ExtractSectionsReport> {
    let dry_run = args.dry_run || args.preview;
    let search_path = match &args.path {
        Some(path) if path.is_absolute() => path.clone(),
        Some(path) => kb_path.join(path),
        None => kb_path.join("processing/text"),
    };
    let output_dir = match &args.output_dir {
        Some(path) if path.is_absolute() => path.clone(),
        Some(path) => kb_path.join(path),
        None => kb_path.join("processing/sections"),
    };

    if !search_path.exists() {
        return Err(anyhow!(
            "search path does not exist: {}. Run `kb extract-text` first or pass --path.",
            search_path.display()
        ));
    }

    let sources = collect_text_files(&search_path, args.limit)?;
    let patterns = SectionPatterns::new()?;
    if !dry_run {
        fs::create_dir_all(&output_dir)?;
    }

    let mut items = Vec::new();
    let mut missing_for_llm = Vec::new();
    let mut full_success_count = 0usize;
    let mut partial_count = 0usize;
    let mut missing_count = 0usize;

    for source in &sources {
        let content = match fs::read_to_string(source) {
            Ok(content) => content,
            Err(err) => {
                let article_dir = article_output_dir_for(kb_path, &output_dir, source);
                let item = ExtractSectionsItem {
                    source_path: relative_path_string(kb_path, source),
                    article_dir: relative_path_string(kb_path, &article_dir),
                    introduction_path: None,
                    references_path: None,
                    introduction: missing_slice(),
                    references: missing_slice(),
                    status: "failed".to_string(),
                    needs_llm_or_ocr: true,
                    message: format!("could not read text file: {err}"),
                };
                missing_for_llm.push(item.source_path.clone());
                missing_count += 1;
                items.push(item);
                continue;
            }
        };

        let article_dir = article_output_dir_for(kb_path, &output_dir, source);
        let intro_path = article_dir.join("introduction.txt");
        let refs_path = article_dir.join("references.txt");

        let intro = extract_introduction(&content, &patterns);
        let refs = extract_references(&content, &patterns);

        let intro_slice = slice_metadata(intro.as_ref());
        let refs_slice = slice_metadata(refs.as_ref());
        let intro_good = intro_slice.found && intro_slice.quality != "too_short";
        let refs_good = refs_slice.found && refs_slice.quality != "too_short";

        let status = match (intro_good, refs_good) {
            (true, true) => {
                full_success_count += 1;
                "written"
            }
            (true, false) | (false, true) => {
                partial_count += 1;
                "partial"
            }
            (false, false) => {
                missing_count += 1;
                "missing"
            }
        }
        .to_string();

        let needs_llm_or_ocr = !intro_good || !refs_good;
        if needs_llm_or_ocr {
            missing_for_llm.push(relative_path_string(kb_path, source));
        }

        if !dry_run {
            fs::create_dir_all(&article_dir)?;
            if let Some(section) = &intro {
                write_section_if_allowed(&intro_path, &section.text, args.force)?;
            }
            if let Some(section) = &refs {
                write_section_if_allowed(&refs_path, &section.text, args.force)?;
            }
            write_section_manifest(kb_path, &article_dir, source, &intro_slice, &refs_slice)?;
        }

        let message = if needs_llm_or_ocr {
            "one or both target sections are missing or too short; use LLM/OCR Worker for image-based PDFs, bad extraction order, or ambiguous headings".to_string()
        } else if dry_run {
            "would write introduction.txt and references.txt".to_string()
        } else {
            "introduction.txt and references.txt extracted deterministically".to_string()
        };

        items.push(ExtractSectionsItem {
            source_path: relative_path_string(kb_path, source),
            article_dir: relative_path_string(kb_path, &article_dir),
            introduction_path: intro
                .as_ref()
                .map(|_| relative_path_string(kb_path, &intro_path)),
            references_path: refs
                .as_ref()
                .map(|_| relative_path_string(kb_path, &refs_path)),
            introduction: intro_slice,
            references: refs_slice,
            status,
            needs_llm_or_ocr,
            message,
        });
    }

    let mut report = ExtractSectionsReport {
        schema_version: "0.7.4".to_string(),
        generated_by: "kb-cli extract-sections".to_string(),
        generated_at: chrono::Utc::now().to_rfc3339(),
        search_path: relative_path_string(kb_path, &search_path),
        output_dir: relative_path_string(kb_path, &output_dir),
        dry_run,
        force: args.force,
        files_seen: sources.len(),
        files_processed: sources.len(),
        full_success_count,
        partial_count,
        missing_count,
        llm_task_path: None,
        items,
    };

    if !dry_run && !args.no_llm_task && !missing_for_llm.is_empty() {
        let task_path = write_llm_section_task(kb_path, &report, &missing_for_llm)?;
        report.llm_task_path = Some(relative_path_string(kb_path, &task_path));
    }

    Ok(report)
}

impl SectionPatterns {
    fn new() -> Result<Self> {
        Ok(Self {
            introduction_heading: Regex::new(
                r"(?i)^\s*((\d+(\.\d+)*|[ivxlcdm]+)\s*[\.)]?\s*)?(introduction|intro|引言|绪论)\s*$",
            )?,
            references_heading: Regex::new(
                r"(?i)^\s*((\d+(\.\d+)*|[ivxlcdm]+)\s*[\.)]?\s*)?(references|bibliography|works cited|literature cited|参考文献|参考资料|文献)\s*$",
            )?,
            generic_heading: Regex::new(
                r"(?i)^\s*((\d+(\.\d+)*|[ivxlcdm]+)\s*[\.)]?\s*)?(abstract|introduction|intro|background|related work|materials and methods|materials|methods|methodology|model|models|results|discussion|conclusion|conclusions|acknowledg(e)?ments?|references|bibliography|works cited|literature cited|appendix|supplementary|supporting information|引言|绪论|背景|相关工作|材料与方法|材料|方法|模型|结果|讨论|结论|致谢|参考文献|参考资料|文献|附录)\s*$",
            )?,
            appendix_heading: Regex::new(
                r"(?i)^\s*((\d+(\.\d+)*|[ivxlcdm]+)\s*[\.)]?\s*)?(appendix|supplementary|supporting information|附录|补充材料)\s*$",
            )?,
        })
    }
}

fn extract_introduction(content: &str, patterns: &SectionPatterns) -> Option<SectionText> {
    let lines = content.lines().collect::<Vec<_>>();
    if lines.is_empty() {
        return None;
    }
    let max_start = std::cmp::max(lines.len() / 2, 1);
    let start = find_first_heading(&lines, &patterns.introduction_heading, 0, max_start)?;
    let end = find_next_generic_heading(&lines, start + 1, patterns).unwrap_or(lines.len());
    build_section_text(&lines, start, end)
}

fn extract_references(content: &str, patterns: &SectionPatterns) -> Option<SectionText> {
    let lines = content.lines().collect::<Vec<_>>();
    if lines.is_empty() {
        return None;
    }
    let min_start = lines.len() / 4;
    let start = find_last_heading(&lines, &patterns.references_heading, min_start, lines.len())?;
    let end = find_next_appendix_heading(&lines, start + 1, patterns).unwrap_or(lines.len());
    build_section_text(&lines, start, end)
}

fn find_first_heading(lines: &[&str], pattern: &Regex, start: usize, end: usize) -> Option<usize> {
    let end = end.min(lines.len());
    (start..end).find(|idx| pattern.is_match(lines[*idx].trim()))
}

fn find_last_heading(lines: &[&str], pattern: &Regex, start: usize, end: usize) -> Option<usize> {
    let end = end.min(lines.len());
    (start..end)
        .rev()
        .find(|idx| pattern.is_match(lines[*idx].trim()))
}

fn find_next_generic_heading(
    lines: &[&str],
    start: usize,
    patterns: &SectionPatterns,
) -> Option<usize> {
    (start..lines.len()).find(|idx| {
        let trimmed = lines[*idx].trim();
        if trimmed.len() > 80 {
            return false;
        }
        patterns.generic_heading.is_match(trimmed)
    })
}

fn find_next_appendix_heading(
    lines: &[&str],
    start: usize,
    patterns: &SectionPatterns,
) -> Option<usize> {
    (start..lines.len()).find(|idx| {
        let trimmed = lines[*idx].trim();
        if trimmed.len() > 80 {
            return false;
        }
        patterns.appendix_heading.is_match(trimmed)
    })
}

fn build_section_text(lines: &[&str], start: usize, end: usize) -> Option<SectionText> {
    if start >= end || start >= lines.len() {
        return None;
    }
    let end = end.min(lines.len());
    let text = lines[start..end]
        .join("\n")
        .trim()
        .trim_matches('\u{feff}')
        .to_string();
    if text.is_empty() {
        return None;
    }
    Some(SectionText {
        text,
        start_line: start + 1,
        end_line: end,
    })
}

fn slice_metadata(section: Option<&SectionText>) -> SectionSlice {
    match section {
        Some(section) => {
            let char_count = section.text.chars().count();
            SectionSlice {
                found: true,
                start_line: Some(section.start_line),
                end_line: Some(section.end_line),
                line_count: section.end_line.saturating_sub(section.start_line) + 1,
                char_count,
                quality: if char_count < 120 {
                    "too_short".to_string()
                } else {
                    "usable".to_string()
                },
            }
        }
        None => missing_slice(),
    }
}

fn missing_slice() -> SectionSlice {
    SectionSlice {
        found: false,
        start_line: None,
        end_line: None,
        line_count: 0,
        char_count: 0,
        quality: "missing".to_string(),
    }
}

fn collect_text_files(input_path: &Path, limit: usize) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    if input_path.is_file() {
        if is_text_file(input_path) {
            files.push(input_path.to_path_buf());
        }
        return Ok(files);
    }

    for entry in WalkDir::new(input_path)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
    {
        let path = entry.path();
        if is_text_file(path) {
            files.push(path.to_path_buf());
            if limit > 0 && files.len() >= limit {
                break;
            }
        }
    }
    files.sort();
    Ok(files)
}

fn is_text_file(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_ascii_lowercase()),
        Some(ext) if matches!(ext.as_str(), "txt" | "md")
    )
}

fn article_output_dir_for(kb_path: &Path, output_dir: &Path, source: &Path) -> PathBuf {
    let text_root = kb_path.join("processing/text");
    let rel = source
        .strip_prefix(&text_root)
        .or_else(|_| source.strip_prefix(kb_path))
        .unwrap_or(source);
    let mut stem = rel.to_string_lossy().replace('\\', "__").replace('/', "__");
    if let Some(idx) = stem.rfind('.') {
        stem.truncate(idx);
    }
    output_dir.join(sanitize_filename(&stem))
}
fn sanitize_filename(value: &str) -> String {
    let cleaned = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch == '.' {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>();
    let trimmed = cleaned.trim_matches('_');
    if trimmed.is_empty() {
        "source".to_string()
    } else {
        trimmed.to_string()
    }
}

fn write_section_if_allowed(path: &Path, content: &str, force: bool) -> Result<()> {
    if path.exists() && !force {
        return Ok(());
    }
    fs::write(path, content)?;
    Ok(())
}

fn write_section_manifest(
    kb_path: &Path,
    article_dir: &Path,
    source: &Path,
    introduction: &SectionSlice,
    references: &SectionSlice,
) -> Result<()> {
    let content = format!(
        "# Extracted Paper Sections\n\n- Source: `{}`\n- Generated by: `kb-cli extract-sections`\n- Generated at: `{}`\n\n## Sections\n\n| section | found | quality | start_line | end_line | char_count |\n|---|---:|---|---:|---:|---:|\n| introduction | `{}` | `{}` | `{}` | `{}` | `{}` |\n| references | `{}` | `{}` | `{}` | `{}` | `{}` |\n\n## Boundary\n\nThis file records deterministic section slicing only. Missing or suspiciously short sections should be assigned to an OCR/LLM Worker instead of being silently accepted.\n",
        relative_path_string(kb_path, source),
        chrono::Utc::now().to_rfc3339(),
        introduction.found,
        introduction.quality,
        introduction
            .start_line
            .map(|v| v.to_string())
            .unwrap_or_else(|| "".to_string()),
        introduction
            .end_line
            .map(|v| v.to_string())
            .unwrap_or_else(|| "".to_string()),
        introduction.char_count,
        references.found,
        references.quality,
        references
            .start_line
            .map(|v| v.to_string())
            .unwrap_or_else(|| "".to_string()),
        references
            .end_line
            .map(|v| v.to_string())
            .unwrap_or_else(|| "".to_string()),
        references.char_count,
    );
    fs::write(article_dir.join("section_manifest.md"), content)?;
    Ok(())
}

fn write_llm_section_task(
    kb_path: &Path,
    report: &ExtractSectionsReport,
    missing_sources: &[String],
) -> Result<PathBuf> {
    let tasks_dir = kb_path.join("LLM/tasks/items");
    fs::create_dir_all(&tasks_dir)?;
    let stamp = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();
    let path = tasks_dir.join(format!("paper-section-extraction-{stamp}.md"));

    let mut out = String::new();
    out.push_str("# Worker Task: Paper Section Extraction Fallback\n\n");
    out.push_str("This task was generated by `kb extract-sections` when deterministic slicing could not produce both `Introduction` and `References` reliably.\n\n");
    out.push_str("## Task Metadata\n\n");
    out.push_str("- Status: `pending`\n");
    out.push_str("- Priority: `high`\n");
    out.push_str("- Category: `paper_section_extraction`\n");
    out.push_str("- Target agent: `Paper Section OCR/LLM Worker`\n");
    out.push_str("- Source command: `kb extract-sections`\n");
    out.push_str(&format!("- Generated at: `{}`\n\n", report.generated_at));

    out.push_str("## Goal\n\n");
    out.push_str("Recover the `Introduction` and `References` sections for the listed papers. These two sections are high-value inputs for building literature relationships among papers.\n\n");

    out.push_str("## Input Files Needing Review\n\n");
    for source in missing_sources {
        out.push_str(&format!("- `{source}`\n"));
    }
    out.push_str("\n");

    out.push_str("## Required Output\n\n");
    out.push_str("For each source, write or repair these files under `processing/sections/<source-key>/`:\n\n");
    out.push_str("- `introduction.txt`\n");
    out.push_str("- `references.txt`\n");
    out.push_str("- `section_manifest.md` with extraction notes and uncertainty\n\n");

    out.push_str("## Requirements\n\n");
    out.push_str("- Prefer deterministic text extraction when possible.\n");
    out.push_str(
        "- If the PDF is scanned/image-based, use OCR or multimodal LLM reading explicitly.\n",
    );
    out.push_str("- Preserve the original section wording as much as possible.\n");
    out.push_str("- Do not summarize these sections in the section text files; the goal is section capture, not interpretation.\n");
    out.push_str(
        "- Mark garbled, uncertain, or incomplete extraction clearly in `section_manifest.md`.\n",
    );
    out.push_str("- Do not modify `raw/` source files.\n");
    out.push_str("- Leave final acceptance for human/Git diff review.\n\n");

    out.push_str("## Why This Matters\n\n");
    out.push_str("`Introduction` often states the research gap, motivation, and cited foundation. `References` exposes the explicit citation boundary. Together they are the primary source for later literature relationship reconstruction.\n");

    fs::write(&path, out)?;
    Ok(path)
}

fn print_report(report: &ExtractSectionsReport) {
    println!("kb extract-sections");
    println!("Generated at: {}", report.generated_at);
    println!("Search path: {}", report.search_path);
    println!("Output dir: {}", report.output_dir);
    println!("Dry run: {}", report.dry_run);
    println!("Files processed: {}", report.files_processed);
    println!("Full success: {}", report.full_success_count);
    println!("Partial: {}", report.partial_count);
    println!("Missing: {}", report.missing_count);
    if let Some(path) = &report.llm_task_path {
        println!("LLM fallback task: {path}");
    }
    println!();

    for item in &report.items {
        println!("- {} [{}]", item.source_path, item.status);
        println!("  article dir: {}", item.article_dir);
        println!(
            "  introduction: found={}, quality={}, chars={}",
            item.introduction.found, item.introduction.quality, item.introduction.char_count
        );
        println!(
            "  references: found={}, quality={}, chars={}",
            item.references.found, item.references.quality, item.references.char_count
        );
        if item.needs_llm_or_ocr {
            println!("  needs LLM/OCR: yes");
        }
    }
}

fn relative_path_string(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_introduction_and_references_from_simple_text() {
        let patterns = SectionPatterns::new().unwrap();
        let text = "Title\n\nAbstract\nsummary\n\n1 Introduction\nThis is the motivation.\nThis cites prior work.\n\n2 Methods\nmethod text\n\nReferences\n[1] Smith 2020.\n[2] Zhang 2021.";
        let intro = extract_introduction(text, &patterns).unwrap();
        let refs = extract_references(text, &patterns).unwrap();
        assert!(intro.text.contains("This is the motivation"));
        assert!(!intro.text.contains("2 Methods"));
        assert!(refs.text.contains("Smith 2020"));
    }
}
