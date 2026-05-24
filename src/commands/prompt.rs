use anyhow::{anyhow, Result};
use clap::{Args, Subcommand};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Args)]
pub struct PromptArgs {
    #[command(subcommand)]
    pub command: PromptCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum PromptCommand {
    #[command(
        name = "paper-profile",
        about = "Print a Worker prompt for paper key-info extraction"
    )]
    PaperProfile(PromptTaskArgs),
    #[command(
        name = "topic-narrative",
        about = "Print a Worker prompt for topic story narrative construction"
    )]
    TopicNarrative(PromptTopicArgs),
    #[command(
        name = "relation-review",
        about = "Print a Worker/Human prompt for relation candidate review"
    )]
    RelationReview(PromptTaskArgs),
    #[command(
        name = "wiki-link-repair",
        about = "Print a Worker prompt for WikiLink repair"
    )]
    WikiLinkRepair(PromptTaskArgs),
    #[command(name = "human-review", about = "Print a Human Review prompt/checklist")]
    HumanReview(PromptTaskArgs),
}

#[derive(Debug, Clone, Args)]
pub struct PromptTaskArgs {
    #[arg(
        long,
        help = "Optional task id from LLM/tasks/items or LLM/tasks/batches"
    )]
    pub task: Option<String>,
    #[arg(long, help = "Optional topic slug/title used to specialize the prompt")]
    pub topic: Option<String>,
}

#[derive(Debug, Clone, Args)]
pub struct PromptTopicArgs {
    #[arg(long, help = "Topic slug/title for the narrative prompt")]
    pub topic: Option<String>,
    #[arg(
        long,
        help = "Optional task id from LLM/tasks/items or LLM/tasks/batches"
    )]
    pub task: Option<String>,
}

pub fn execute(custom_kb: Option<&Path>, args: &PromptArgs) -> Result<()> {
    let kb_path = crate::commands::init::get_kb_path(custom_kb);
    if !kb_path.exists() {
        return Err(anyhow!(
            "knowledge base path does not exist: {}. Run `kb create --wiki <A> --from <B> --about <topic>` first.",
            kb_path.display()
        ));
    }

    let prompt = match &args.command {
        PromptCommand::PaperProfile(args) => {
            render_paper_profile_prompt(&kb_path, args.task.as_deref(), args.topic.as_deref())?
        }
        PromptCommand::TopicNarrative(args) => {
            render_topic_narrative_prompt(&kb_path, args.task.as_deref(), args.topic.as_deref())?
        }
        PromptCommand::RelationReview(args) => {
            render_relation_review_prompt(&kb_path, args.task.as_deref(), args.topic.as_deref())?
        }
        PromptCommand::WikiLinkRepair(args) => {
            render_wiki_link_repair_prompt(&kb_path, args.task.as_deref(), args.topic.as_deref())?
        }
        PromptCommand::HumanReview(args) => {
            render_human_review_prompt(&kb_path, args.task.as_deref(), args.topic.as_deref())?
        }
    };

    println!("{}", prompt);
    Ok(())
}

fn render_paper_profile_prompt(
    kb_path: &Path,
    task: Option<&str>,
    topic: Option<&str>,
) -> Result<String> {
    Ok(render_common_prompt(
        kb_path,
        "Paper Profile Worker",
        task,
        topic,
        "Extract evidence-backed paper key information and update reviewable paper profiles.",
        &["title", "authors", "journal/year/doi", "abstract", "keywords", "introduction summary", "evidence snippets"],
        &["processing/paper_profiles/<paper>.md", "wiki/papers/<paper>.md"],
        "Do not invent bibliographic fields. Mark unknown fields as `Needs LLM extraction` or `uncertain`.",
    )?)
}

fn render_topic_narrative_prompt(
    kb_path: &Path,
    task: Option<&str>,
    topic: Option<&str>,
) -> Result<String> {
    Ok(render_common_prompt(
        kb_path,
        "Topic Narrative Worker",
        task,
        topic,
        "Build a concise topic story narrative from topic-local literature and paper profiles.",
        &["background", "core problem", "method lineage", "key papers", "competing ideas", "open questions", "WikiLinks to papers"],
        &["wiki/topics/<topic>.md"],
        "Use topic relation files as hints, not final truth. Every important claim should link to supporting paper pages.",
    )?)
}

fn render_relation_review_prompt(
    kb_path: &Path,
    task: Option<&str>,
    topic: Option<&str>,
) -> Result<String> {
    Ok(render_common_prompt(
        kb_path,
        "Relation Review Worker / Human Reviewer",
        task,
        topic,
        "Review candidate literature relations and classify them as confirmed, rejected, missing, or needs_human.",
        &["source paper", "target/reference", "relation type", "evidence", "decision", "confidence", "unresolved issues"],
        &["processing/refs/*", "topics/<topic>/relations/*", "state/human_tasks/* if needed"],
        "Do not promote candidate edges to confirmed without DOI/title/author/context evidence or human approval.",
    )?)
}

fn render_wiki_link_repair_prompt(
    kb_path: &Path,
    task: Option<&str>,
    topic: Option<&str>,
) -> Result<String> {
    Ok(render_common_prompt(
        kb_path,
        "WikiLink Repair Worker",
        task,
        topic,
        "Repair broken WikiLinks and create only necessary concept stubs with clear source traceability.",
        &["broken link", "current page", "suggested target", "action taken", "evidence"],
        &["wiki/**/*.md"],
        "Prefer correcting link targets over creating empty pages. Do not create unsourced concept pages.",
    )?)
}

fn render_human_review_prompt(
    kb_path: &Path,
    task: Option<&str>,
    topic: Option<&str>,
) -> Result<String> {
    Ok(render_common_prompt(
        kb_path,
        "Human Review Checklist",
        task,
        topic,
        "Make the final judgment on high-value or uncertain knowledge decisions.",
        &["approve/reject/defer", "reason", "files reviewed", "required follow-up", "whether Manager may continue"],
        &["LLM/tasks/items/<task>.md", "wiki/**", "processing/**", "interfaces/html/index.html"],
        "Human decision is the final gate for anchor papers, confirmed relations, topic narratives, and bulk merges.",
    )?)
}

fn render_common_prompt(
    kb_path: &Path,
    role: &str,
    task: Option<&str>,
    topic: Option<&str>,
    mission: &str,
    output_fields: &[&str],
    writable_targets: &[&str],
    special_rule: &str,
) -> Result<String> {
    let task_context = match task {
        Some(id) => read_task_context(kb_path, id)?,
        None => "No concrete task id was supplied. The Manager must provide a bounded file list before Worker execution.".to_string(),
    };
    let topic = topic.unwrap_or("<topic>");
    let output_fields = output_fields
        .iter()
        .map(|v| format!("- {}", v))
        .collect::<Vec<_>>()
        .join("\n");
    let writable_targets = writable_targets
        .iter()
        .map(|v| format!("- `{}`", v))
        .collect::<Vec<_>>()
        .join("\n");

    Ok(format!(
        r#"# {role} Prompt

## Workspace

`{workspace}`

## Topic

`{topic}`

## Mission

{mission}

## Task Context

{task_context}

## Required Output Fields

{output_fields}

## Allowed Write Targets

{writable_targets}

## Hard Rules

1. Do not modify `raw/` source materials.
2. Do not browse or rewrite the entire workspace unless the task explicitly expands scope.
3. Preserve uncertainty. Do not invent citations, metadata, or scientific claims.
4. List every changed or created file.
5. Report unresolved issues and follow-up questions.
6. After completing work, ask the Manager to run `kb task status <task-id> --mark completed` only after review accepts the result.
7. {special_rule}

## Completion Report Format

```markdown
# Worker Completion Report

- Task ID:
- Topic:
- Files changed:
- Evidence used:
- Unresolved issues:
- Recommended status:
```
"#,
        role = role,
        workspace = kb_path.display(),
        topic = topic,
        mission = mission,
        task_context = task_context,
        output_fields = output_fields,
        writable_targets = writable_targets,
        special_rule = special_rule,
    ))
}

fn read_task_context(kb_path: &Path, id: &str) -> Result<String> {
    let id = sanitize_task_id(id);
    for path in [
        kb_path.join("LLM/tasks/items").join(format!("{}.md", id)),
        kb_path.join("LLM/tasks/batches").join(format!("{}.md", id)),
    ] {
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            let excerpt = content.lines().take(120).collect::<Vec<_>>().join("\n");
            return Ok(format!(
                "Task file: `{}`\n\n{}",
                relative_path_string(kb_path, &path),
                excerpt
            ));
        }
    }
    Ok(format!(
        "Task `{}` was not found under `LLM/tasks/items/` or `LLM/tasks/batches/`.",
        id
    ))
}

fn sanitize_task_id(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

fn relative_path_string(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}
