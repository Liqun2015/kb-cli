use anyhow::{anyhow, Result};
use chrono::Utc;
use clap::{Args, Subcommand};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone, Args)]
pub struct BatchArgs {
    #[command(subcommand)]
    pub command: BatchCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum BatchCommand {
    #[command(
        name = "paper-profile",
        about = "Create a 3-5 paper Worker batch for paper-profile extraction"
    )]
    PaperProfile(PaperProfileBatchArgs),
}

#[derive(Debug, Clone, Args)]
pub struct PaperProfileBatchArgs {
    #[arg(long, help = "Topic slug/title this batch should support")]
    pub topic: String,
    #[arg(
        long,
        default_value_t = 5,
        help = "Maximum papers in this batch. Recommended range: 3-5"
    )]
    pub limit: usize,
    #[arg(long, help = "Do not mark the generated batch task as assigned")]
    pub no_mark: bool,
}

#[derive(Debug, Clone)]
struct PaperCandidate {
    pdf: PathBuf,
    profile: PathBuf,
    wiki_page: PathBuf,
}

pub fn execute(custom_kb: Option<&Path>, args: &BatchArgs) -> Result<()> {
    let kb_path = crate::commands::init::get_kb_path(custom_kb);
    if !kb_path.exists() {
        return Err(anyhow!(
            "knowledge base path does not exist: {}. Run `kb create --wiki <A> --from <B> --about <topic>` first.",
            kb_path.display()
        ));
    }

    match &args.command {
        BatchCommand::PaperProfile(args) => execute_paper_profile_batch(&kb_path, args),
    }
}

fn execute_paper_profile_batch(kb_path: &Path, args: &PaperProfileBatchArgs) -> Result<()> {
    if args.topic.trim().is_empty() {
        return Err(anyhow!("--topic cannot be empty"));
    }
    let limit = args.limit.clamp(1, 8);
    let candidates = collect_paper_profile_candidates(kb_path)?;
    let selected = candidates.into_iter().take(limit).collect::<Vec<_>>();
    if selected.is_empty() {
        println!("No paper-profile batch created. All detected paper profiles look present, or no PDFs were found under raw/papers/.");
        return Ok(());
    }

    let stamp = Utc::now().format("%Y%m%d_%H%M%S").to_string();
    let batch_id = format!("paper-profile-batch-{}", stamp);
    let batch_dir = kb_path.join("LLM/tasks/batches");
    fs::create_dir_all(&batch_dir)?;
    let batch_path = batch_dir.join(format!("{}.md", batch_id));
    fs::write(
        &batch_path,
        render_batch_file(kb_path, &batch_id, args, &selected),
    )?;

    if !args.no_mark {
        let note = format!(
            "Batch created for topic `{}` with {} papers.",
            args.topic,
            selected.len()
        );
        crate::commands::task::mark_task_state(
            kb_path,
            &batch_id,
            "assigned",
            Some("openclaw-manager"),
            Some(&note),
        )?;
        let _ = crate::commands::task::mark_task_state(
            kb_path,
            "paper-key-info-extraction-001",
            "assigned",
            Some("openclaw-manager"),
            Some("At least one paper-profile batch has been created."),
        );
    }

    println!("Paper-profile batch created:");
    println!("  id    : {}", batch_id);
    println!("  topic : {}", args.topic);
    println!("  papers: {}", selected.len());
    println!("  file  : {}", relative_path_string(kb_path, &batch_path));
    println!();
    println!("Suggested Worker prompt:");
    println!(
        "  kb --wiki \"{}\" prompt paper-profile --task {} --topic \"{}\"",
        kb_path.display(),
        batch_id,
        args.topic
    );
    Ok(())
}

fn collect_paper_profile_candidates(kb_path: &Path) -> Result<Vec<PaperCandidate>> {
    let raw_papers = kb_path.join("raw/papers");
    let mut candidates = Vec::new();
    if !raw_papers.exists() {
        return Ok(candidates);
    }

    for entry in WalkDir::new(&raw_papers)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        if path
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.eq_ignore_ascii_case("pdf"))
            .unwrap_or(false)
        {
            let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("paper");
            let slug = sanitize_path_component(stem);
            let profile = kb_path
                .join("processing/paper_profiles")
                .join(format!("{}.md", slug));
            let wiki_page = kb_path.join("wiki/papers").join(format!("{}.md", slug));
            let needs_work = !profile.exists()
                || fs::read_to_string(&profile)
                    .map(|content| {
                        content.contains("Needs LLM extraction")
                            || content.contains("TODO")
                            || content.len() < 600
                    })
                    .unwrap_or(true);
            if needs_work {
                candidates.push(PaperCandidate {
                    pdf: path.to_path_buf(),
                    profile,
                    wiki_page,
                });
            }
        }
    }
    candidates.sort_by(|a, b| a.pdf.cmp(&b.pdf));
    Ok(candidates)
}

fn render_batch_file(
    kb_path: &Path,
    batch_id: &str,
    args: &PaperProfileBatchArgs,
    papers: &[PaperCandidate],
) -> String {
    let mut out = String::new();
    out.push_str(&format!("# Worker Batch: {}\n\n", batch_id));
    out.push_str(
        "This is a bounded small-batch Worker task generated by `kb batch paper-profile`.\n\n",
    );
    out.push_str("## Task Metadata\n\n");
    out.push_str(&format!("- Task ID: `{}`\n", batch_id));
    out.push_str("- Status: `pending`\n");
    out.push_str("- Priority: `high`\n");
    out.push_str("- Category: `paper_key_information_batch`\n");
    out.push_str("- Target agent: `Paper Key-Info Extraction Worker LLM`\n");
    out.push_str("- Source command: `kb batch paper-profile`\n");
    out.push_str(&format!("- Topic: `{}`\n", args.topic));
    out.push_str(&format!("- Paper count: `{}`\n\n", papers.len()));

    out.push_str("## Goal\n\n");
    out.push_str("Extract title, authors, journal/year/DOI, abstract, keywords, and introduction summary for this small batch. Update reviewable profiles and paper pages with evidence-backed information.\n\n");

    out.push_str("## Papers\n\n");
    out.push_str("| # | source_pdf | profile_target | wiki_page_target |\n");
    out.push_str("|---|---|---|---|\n");
    for (idx, paper) in papers.iter().enumerate() {
        out.push_str(&format!(
            "| {} | `{}` | `{}` | `{}` |\n",
            idx + 1,
            relative_path_string(kb_path, &paper.pdf),
            relative_path_string(kb_path, &paper.profile),
            relative_path_string(kb_path, &paper.wiki_page),
        ));
    }
    out.push_str("\n");

    out.push_str("## Worker Rules\n\n");
    out.push_str(
        "- Work only on the papers listed in this batch unless the Manager expands scope.\n",
    );
    out.push_str("- Use PDFs, extracted text, and extracted sections as evidence.\n");
    out.push_str("- Do not invent metadata or citations.\n");
    out.push_str("- Mark uncertain fields explicitly.\n");
    out.push_str("- Do not modify `raw/`.\n");
    out.push_str("- List every changed or created file in the completion report.\n\n");

    out.push_str("## Standard Prompt\n\n");
    out.push_str("Run this to print a reusable Worker prompt:\n\n");
    out.push_str("```bash\n");
    out.push_str(&format!(
        "kb --wiki \"{}\" prompt paper-profile --task {} --topic \"{}\"\n",
        kb_path.display(),
        batch_id,
        args.topic
    ));
    out.push_str("```\n");
    out
}

fn sanitize_path_component(value: &str) -> String {
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
        "paper".to_string()
    } else {
        trimmed.to_string()
    }
}

fn relative_path_string(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}
