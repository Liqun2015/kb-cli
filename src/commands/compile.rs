use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use clap::Args;
use serde::Serialize;

use crate::commands::manifest::{Manifest, ManifestEntry};

#[derive(Debug, Clone, Args)]
pub struct CompileArgs {
    #[arg(long, help = "Plan compilation for raw files not yet linked to wiki pages. This is the default when --file is not provided.")]
    pub new: bool,

    #[arg(long, value_name = "PATH", help = "Plan compilation for one raw file. Accepts a path relative to the KB root, such as raw/papers/example.pdf, or an absolute path inside the KB.")]
    pub file: Option<PathBuf>,

    #[arg(long, default_value_t = 20, help = "Maximum number of files to include. Use 0 for no limit.")]
    pub limit: usize,

    #[arg(long, help = "Preview the compile plan without writing processing/compile_queue.json or a proposal file.")]
    pub dry_run: bool,

    #[arg(long, help = "Alias for --dry-run.")]
    pub preview: bool,

    #[arg(long, help = "Do not refresh processing/manifest.json before planning. Non-dry-run compile refreshes it by default.")]
    pub no_refresh: bool,
}

impl CompileArgs {
    pub fn is_dry_run(&self) -> bool {
        self.dry_run || self.preview
    }
}

#[derive(Debug, Serialize)]
struct CompileQueue {
    schema_version: String,
    generated_by: String,
    generated_at: String,
    mode: String,
    item_count: usize,
    items: Vec<CompileQueueItem>,
}

#[derive(Debug, Clone, Serialize)]
struct CompileQueueItem {
    manifest_id: String,
    raw_path: String,
    kind: String,
    status: String,
    content_hash: String,
    proposed_wiki_pages: Vec<String>,
    instructions: Vec<String>,
}

pub fn execute(custom_kb: Option<&Path>, args: &CompileArgs) -> Result<()> {
    validate_args(args)?;

    let kb_path = crate::commands::init::get_kb_path(custom_kb);
    if !kb_path.exists() {
        return Err(anyhow!(
            "knowledge base path does not exist: {}. Run `kb init` first.",
            kb_path.display()
        ));
    }

    if !args.is_dry_run() && !args.no_refresh {
        println!("Refreshing manifest before compile planning...");
        crate::commands::manifest::refresh_for_path(&kb_path, false)?;
    }

    let manifest = load_manifest(&kb_path)?;
    let mode = compile_mode(args);
    let selected = select_entries(&kb_path, manifest.entries, args)?;
    let queue = build_queue(mode, selected);

    if args.is_dry_run() {
        print_queue_preview(&queue);
        println!("\nDry run: no compile queue or proposal file was written.");
        return Ok(());
    }

    write_queue_and_proposal(&kb_path, &queue)?;
    Ok(())
}

fn validate_args(args: &CompileArgs) -> Result<()> {
    if args.new && args.file.is_some() {
        return Err(anyhow!("use either --new or --file, not both"));
    }
    Ok(())
}

fn compile_mode(args: &CompileArgs) -> &'static str {
    if args.file.is_some() {
        "file"
    } else {
        "new"
    }
}

fn load_manifest(kb_path: &Path) -> Result<Manifest> {
    let manifest_path = kb_path.join("processing/manifest.json");
    if !manifest_path.exists() {
        return Err(anyhow!(
            "manifest not found: {}. Run `kb status` first, or run non-dry-run `kb compile --new` so the manifest can be refreshed.",
            manifest_path.display()
        ));
    }

    let content = fs::read_to_string(&manifest_path)?;
    Ok(serde_json::from_str(&content)?)
}

fn select_entries(
    kb_path: &Path,
    mut entries: Vec<ManifestEntry>,
    args: &CompileArgs,
) -> Result<Vec<ManifestEntry>> {
    entries.sort_by(|a, b| a.path.cmp(&b.path));

    if let Some(file) = &args.file {
        let target = normalize_requested_path(kb_path, file);
        let entry = entries
            .into_iter()
            .find(|entry| entry.path == target)
            .ok_or_else(|| anyhow!("raw file not found in manifest: {target}"))?;

        if entry.status == "raw_missing" {
            return Err(anyhow!("cannot compile missing raw file: {}", entry.path));
        }

        return Ok(vec![entry]);
    }

    let mut selected = entries
        .into_iter()
        .filter(is_new_compile_candidate)
        .collect::<Vec<_>>();

    if args.limit > 0 && selected.len() > args.limit {
        selected.truncate(args.limit);
    }

    Ok(selected)
}

fn is_new_compile_candidate(entry: &ManifestEntry) -> bool {
    if entry.status == "raw_missing" {
        return false;
    }

    entry.wiki_pages.is_empty()
        && matches!(
            entry.status.as_str(),
            "raw_registered" | "raw_changed" | "raw_ingested"
        )
}

fn normalize_requested_path(kb_path: &Path, requested: &Path) -> String {
    let absolute = if requested.is_absolute() {
        requested.to_path_buf()
    } else {
        kb_path.join(requested)
    };

    absolute
        .strip_prefix(kb_path)
        .unwrap_or(requested)
        .components()
        .map(|component| component.as_os_str().to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join("/")
}

fn build_queue(mode: &str, entries: Vec<ManifestEntry>) -> CompileQueue {
    let items = entries
        .into_iter()
        .map(|entry| CompileQueueItem {
            manifest_id: entry.id.clone(),
            raw_path: entry.path.clone(),
            kind: entry.kind.clone(),
            status: entry.status.clone(),
            content_hash: entry.content_hash.clone(),
            proposed_wiki_pages: proposed_wiki_pages(&entry),
            instructions: compile_instructions(&entry),
        })
        .collect::<Vec<_>>();

    CompileQueue {
        schema_version: "0.4.5".to_string(),
        generated_by: "kb-cli".to_string(),
        generated_at: chrono::Utc::now().to_rfc3339(),
        mode: mode.to_string(),
        item_count: items.len(),
        items,
    }
}

fn proposed_wiki_pages(entry: &ManifestEntry) -> Vec<String> {
    let stem = Path::new(&entry.path)
        .file_stem()
        .and_then(|s| s.to_str())
        .map(sanitize_wiki_stem)
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "source".to_string());

    let primary = match entry.kind.as_str() {
        "paper" => format!("wiki/papers/{stem}.md"),
        "note" => format!("wiki/notes/{stem}.md"),
        _ => format!("wiki/notes/{stem}.md"),
    };

    vec![primary]
}

fn compile_instructions(entry: &ManifestEntry) -> Vec<String> {
    let mut instructions = vec![
        "Read the raw source without modifying it.".to_string(),
        "Read rules/LLM_WIKI_SCHEMA.md before proposing wiki edits.".to_string(),
        "Create or update the proposed source-summary Markdown page.".to_string(),
        "Add YAML front matter with source_ids and source_files so `kb sync-wiki` can update the manifest.".to_string(),
        "Extract durable concepts, methods, people, topics, and cross-links when supported by the source.".to_string(),
        "Preserve uncertainty and cite the raw source path instead of inventing unsupported conclusions.".to_string(),
        "Use Git diff or proposal review before accepting changes into wiki/.".to_string(),
    ];

    if entry.kind == "paper" {
        instructions.push("Follow rules/PAPER_PAGE_TEMPLATE.md for the paper summary page.".to_string());
    } else {
        instructions.push("Use the closest matching template under rules/ and keep the page concise.".to_string());
    }

    instructions
}

fn sanitize_wiki_stem(input: &str) -> String {
    let mut output = String::new();
    let mut last_was_sep = false;

    for ch in input.chars() {
        if ch.is_alphanumeric() || matches!(ch, '-' | '_') {
            output.push(ch);
            last_was_sep = false;
        } else if ch.is_whitespace() || matches!(ch, '.' | '/' | '\\' | ':' | ';' | ',' | '(' | ')' | '[' | ']') {
            if !last_was_sep && !output.is_empty() {
                output.push('_');
                last_was_sep = true;
            }
        }
    }

    output.trim_matches('_').to_string()
}

fn print_queue_preview(queue: &CompileQueue) {
    println!("\nCompile plan preview:");
    println!("  mode       : {}", queue.mode);
    println!("  items      : {}", queue.item_count);
    println!("  generated  : {}", queue.generated_at);

    if queue.items.is_empty() {
        println!("\nNo raw files selected for compile planning.");
        return;
    }

    println!("\nPlanned items:");
    for item in &queue.items {
        println!("  - {}", item.raw_path);
        println!("    kind   : {}", item.kind);
        println!("    status : {}", item.status);
        println!("    pages  : {}", item.proposed_wiki_pages.join(", "));
    }
}

fn write_queue_and_proposal(kb_path: &Path, queue: &CompileQueue) -> Result<()> {
    fs::create_dir_all(kb_path.join("processing/proposals"))?;

    let queue_path = kb_path.join("processing/compile_queue.json");
    fs::write(&queue_path, serde_json::to_string_pretty(queue)?)?;

    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();
    let proposal_path = kb_path.join(format!("processing/proposals/compile_plan_{timestamp}.md"));
    fs::write(&proposal_path, render_proposal(queue))?;

    let prompt_path = kb_path.join(format!("processing/proposals/compile_agent_prompt_{timestamp}.md"));
    fs::write(&prompt_path, render_agent_prompt(queue))?;

    println!("\nCompile planning complete.");
    println!("Queue written to   : {}", queue_path.display());
    println!("Proposal written to: {}", proposal_path.display());
    println!("Agent prompt       : {}", prompt_path.display());

    if queue.item_count == 0 {
        println!("\nNo raw files currently need compile planning.");
    } else {
        println!("\nNext steps:");
        println!("  1. Review the proposal file.");
        println!("  2. Copy the agent prompt into Claude Code, ChatGPT, or another knowledge agent.");
        println!("  3. Let the agent edit only wiki/ and processing/ outputs, never raw/.");
        println!("  4. Review all changes with git diff before committing.");
        println!("  5. Run `kb sync-wiki` after accepted wiki edits.");
    }

    Ok(())
}

fn render_proposal(queue: &CompileQueue) -> String {
    let mut out = String::new();
    out.push_str("# Compile Plan Proposal\n\n");
    out.push_str("This file is generated by `kb compile`. It is a review artifact, not an automatic wiki edit.\n\n");
    out.push_str("## Summary\n\n");
    out.push_str(&format!("- Mode: `{}`\n", queue.mode));
    out.push_str(&format!("- Generated at: `{}`\n", queue.generated_at));
    out.push_str(&format!("- Item count: `{}`\n\n", queue.item_count));

    if queue.items.is_empty() {
        out.push_str("No raw files were selected for compile planning.\n");
        return out;
    }

    out.push_str("## Global Instructions for the LLM Maintainer\n\n");
    out.push_str("1. Do not modify files under `raw/`.\n");
    out.push_str("2. Read `rules/LLM_WIKI_SCHEMA.md` before editing `wiki/`.\n");
    out.push_str("3. Prefer small, reviewable Markdown edits.\n");
    out.push_str("4. Add Obsidian-style links such as `[[Concept Name]]` only when useful.\n");
    out.push_str("5. Preserve uncertainty and cite the raw source path.\n");
    out.push_str("6. Every source-derived wiki page must include YAML front matter with `source_ids` and `source_files`.\n");
    out.push_str("7. Let the user review changes through `git diff`, then run `kb sync-wiki`.\n\n");

    out.push_str("## Planned Source Items\n\n");
    for item in &queue.items {
        out.push_str(&format!("### `{}`\n\n", item.raw_path));
        out.push_str(&format!("- Kind: `{}`\n", item.kind));
        out.push_str(&format!("- Status: `{}`\n", item.status));
        out.push_str(&format!("- Manifest ID: `{}`\n", item.manifest_id));
        out.push_str(&format!("- Content hash: `{}`\n", item.content_hash));
        out.push_str("- Required source front matter values:\n");
        out.push_str(&format!("  - `source_ids`: `{}`\n", item.manifest_id));
        out.push_str(&format!("  - `source_files`: `{}`\n", item.raw_path));
        out.push_str("- Proposed wiki pages:\n");
        for page in &item.proposed_wiki_pages {
            out.push_str(&format!("  - `{page}`\n"));
        }
        out.push_str("- Instructions:\n");
        for instruction in &item.instructions {
            out.push_str(&format!("  - {instruction}\n"));
        }
        out.push('\n');
    }

    out
}


fn render_agent_prompt(queue: &CompileQueue) -> String {
    let mut out = String::new();
    out.push_str("# LLM Wiki Compile Agent Prompt\n\n");
    out.push_str("You are maintaining a local LLM Wiki. Follow the project rules exactly.\n\n");
    out.push_str("## Non-negotiable constraints\n\n");
    out.push_str("1. Do not edit, rename, move, or delete anything under `raw/`.\n");
    out.push_str("2. Read `rules/LLM_WIKI_SCHEMA.md` before editing wiki pages.\n");
    out.push_str("3. Use `rules/PAPER_PAGE_TEMPLATE.md` for paper summary pages.\n");
    out.push_str("4. Use `rules/CONCEPT_PAGE_TEMPLATE.md` for concept pages.\n");
    out.push_str("5. Prefer small, reviewable Markdown edits.\n");
    out.push_str("6. Preserve uncertainty; do not invent claims not supported by the source.\n");
    out.push_str("7. Add backlinks and Obsidian-style links only when they help future navigation.\n");
    out.push_str("8. Add YAML front matter to every source-derived page. Include both `source_ids` and `source_files`.\n");
    out.push_str("9. After editing, summarize every changed file and why it changed.\n");
    out.push_str("10. Tell the user to run `kb sync-wiki` after review so the manifest records the new wiki links.\n\n");

    out.push_str("## Compile scope\n\n");
    out.push_str(&format!("- Mode: `{}`\n", queue.mode));
    out.push_str(&format!("- Generated at: `{}`\n", queue.generated_at));
    out.push_str(&format!("- Item count: `{}`\n\n", queue.item_count));

    if queue.items.is_empty() {
        out.push_str("There are no raw files to compile in this queue. Do not edit the wiki.\n");
        return out;
    }

    out.push_str("## Source files to compile\n\n");
    for item in &queue.items {
        out.push_str(&format!("### `{}`\n\n", item.raw_path));
        out.push_str(&format!("- Kind: `{}`\n", item.kind));
        out.push_str(&format!("- Status: `{}`\n", item.status));
        out.push_str(&format!("- Manifest ID: `{}`\n", item.manifest_id));
        out.push_str(&format!("- Content hash: `{}`\n", item.content_hash));
        out.push_str("- Required source front matter values:\n");
        out.push_str(&format!("  - `source_ids`: `{}`\n", item.manifest_id));
        out.push_str(&format!("  - `source_files`: `{}`\n", item.raw_path));
        out.push_str("- Proposed wiki pages:\n");
        for page in &item.proposed_wiki_pages {
            out.push_str(&format!("  - `{page}`\n"));
        }
        out.push_str("- Required actions:\n");
        for instruction in &item.instructions {
            out.push_str(&format!("  - {instruction}\n"));
        }
        out.push('\n');
    }

    out.push_str("## Expected output from the agent\n\n");
    out.push_str("When finished, report:\n\n");
    out.push_str("1. New wiki pages created.\n");
    out.push_str("2. Existing wiki pages updated.\n");
    out.push_str("3. Concepts or topics that should be reviewed by the user.\n");
    out.push_str("4. Any source files that could not be read or confidently summarized.\n");
    out.push_str("5. Suggested next command: `git diff`, then `kb sync-wiki` after accepted wiki edits.\n");

    out
}
