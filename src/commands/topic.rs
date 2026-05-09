use anyhow::{anyhow, Result};
use clap::{Args, Subcommand};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Args)]
pub struct TopicArgs {
    #[command(subcommand)]
    pub command: TopicCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum TopicCommand {
    #[command(about = "Initialize a topic-specific literature relationship workspace")]
    Init(TopicInitArgs),
}

#[derive(Debug, Clone, Args)]
pub struct TopicInitArgs {
    #[arg(
        value_name = "TOPIC",
        help = "Topic name or slug, such as thermal-metamaterials"
    )]
    pub topic: String,

    #[arg(
        long,
        value_name = "TITLE",
        help = "Human-readable topic title. Defaults to the topic argument."
    )]
    pub title: Option<String>,

    #[arg(
        long,
        help = "Overwrite generated topic template files when they already exist"
    )]
    pub force: bool,

    #[arg(long, help = "Preview topic workspace creation without writing files")]
    pub dry_run: bool,

    #[arg(long, help = "Alias for --dry-run")]
    pub preview: bool,
}

#[derive(Debug, Default)]
struct InitSummary {
    topic_slug: String,
    topic_title: String,
    topic_path: PathBuf,
    dry_run: bool,
    dirs_created: usize,
    dirs_skipped: usize,
    files_created: usize,
    files_skipped: usize,
}

pub fn execute(custom_kb: Option<&Path>, args: &TopicArgs) -> Result<()> {
    match &args.command {
        TopicCommand::Init(init_args) => execute_init(custom_kb, init_args),
    }
}

fn execute_init(custom_kb: Option<&Path>, args: &TopicInitArgs) -> Result<()> {
    let kb_path = crate::commands::init::get_kb_path(custom_kb);
    if !kb_path.exists() {
        return Err(anyhow!(
            "knowledge base path does not exist: {}. Run `kb init` first.",
            kb_path.display()
        ));
    }

    let slug = topic_slug(&args.topic);
    if slug.is_empty() {
        return Err(anyhow!(
            "topic name must contain at least one letter or digit"
        ));
    }
    let title = args
        .title
        .clone()
        .unwrap_or_else(|| topic_title(&args.topic));
    let topic_root = kb_path.join("topics").join(&slug);
    let dry_run = args.dry_run || args.preview;

    let mut summary = InitSummary {
        topic_slug: slug.clone(),
        topic_title: title.clone(),
        topic_path: topic_root.clone(),
        dry_run,
        ..Default::default()
    };

    let dirs = [
        "importance",
        "relations",
        "review",
        "graph",
        "tasks",
        "memory",
    ];

    if dry_run {
        println!("Topic init preview:");
        println!("  topic slug : {slug}");
        println!("  topic title: {title}");
        println!("  path       : {}", topic_root.display());
        println!("  directories:");
        println!("    topics/{slug}/");
        for dir in dirs {
            println!("    topics/{slug}/{dir}/");
        }
        println!("  template files:");
        for template in topic_templates(&slug, &title) {
            println!("    topics/{slug}/{}", template.path.display());
        }
        println!("  no files written");
        return Ok(());
    }

    ensure_dir(&topic_root, &mut summary)?;
    for dir in dirs {
        ensure_dir(&topic_root.join(dir), &mut summary)?;
    }

    for template in topic_templates(&slug, &title) {
        let path = topic_root.join(&template.path);
        write_template_file(&path, template.content, args.force, &mut summary)?;
    }

    print_summary(&summary);
    Ok(())
}

struct TopicTemplate {
    path: PathBuf,
    content: String,
}

fn topic_templates(slug: &str, title: &str) -> Vec<TopicTemplate> {
    vec![
        TopicTemplate {
            path: PathBuf::from("README.md"),
            content: render_topic_readme(slug, title),
        },
        TopicTemplate {
            path: PathBuf::from("scope.md"),
            content: render_scope(slug, title),
        },
        TopicTemplate {
            path: PathBuf::from("literature.md"),
            content: render_literature(slug, title),
        },
        TopicTemplate {
            path: PathBuf::from("importance/importance_candidates.md"),
            content: render_importance_candidates(slug, title),
        },
        TopicTemplate {
            path: PathBuf::from("importance/importance_review.md"),
            content: render_importance_review(slug, title),
        },
        TopicTemplate {
            path: PathBuf::from("importance/confirmed_importance.md"),
            content: render_confirmed_importance(slug, title),
        },
        TopicTemplate {
            path: PathBuf::from("relations/causal_relations.md"),
            content: render_relation_file(slug, title, "causal_or_motivates"),
        },
        TopicTemplate {
            path: PathBuf::from("relations/method_relations.md"),
            content: render_relation_file(slug, title, "uses_method_or_improves"),
        },
        TopicTemplate {
            path: PathBuf::from("relations/evidence_relations.md"),
            content: render_relation_file(slug, title, "supports_or_contradicts"),
        },
        TopicTemplate {
            path: PathBuf::from("relations/idea_relations.md"),
            content: render_relation_file(slug, title, "scientific_idea_relation"),
        },
        TopicTemplate {
            path: PathBuf::from("review/relation_review.md"),
            content: render_review_file(slug, title, "relation_review"),
        },
        TopicTemplate {
            path: PathBuf::from("review/importance_review.md"),
            content: render_review_file(slug, title, "importance_review"),
        },
        TopicTemplate {
            path: PathBuf::from("review/unresolved.md"),
            content: render_review_file(slug, title, "unresolved"),
        },
        TopicTemplate {
            path: PathBuf::from("graph/README.md"),
            content: render_graph_readme(slug, title),
        },
        TopicTemplate {
            path: PathBuf::from("tasks/README.md"),
            content: render_tasks_readme(slug, title),
        },
        TopicTemplate {
            path: PathBuf::from("memory/confirmed_relations.md"),
            content: render_memory_file(slug, title, "confirmed_relations"),
        },
        TopicTemplate {
            path: PathBuf::from("memory/completed_tasks.md"),
            content: render_memory_file(slug, title, "completed_tasks"),
        },
    ]
}

fn render_topic_readme(slug: &str, title: &str) -> String {
    format!(
        r#"# {title}

Topic slug: `{slug}`

This directory stores topic-specific literature relationship work.

Global bibliographic index relations stay under `processing/refs/`.
Topic-local importance, causal, method, evidence, and idea relations stay here.

## Directory map

- `scope.md` defines the research-topic boundary.
- `literature.md` lists literature included in this topic.
- `importance/` records topic-local importance candidates, reviews, and confirmed labels.
- `relations/` records topic-local explanatory relation candidates and confirmed relations.
- `review/` stores human-review tables for uncertain topic-local claims.
- `graph/` stores topic graph exports.
- `tasks/` stores Worker LLM / human task handoff files for this topic.
- `memory/` stores completed task records and confirmed relation memories.

## Boundary

This directory must not be treated as a global citation index. Topic-local relations depend on the research question and require evidence.

High-impact claims such as `causal_or_motivates`, `improves`, `contradicts`, and `core` importance should normally be marked `needs_human` until reviewed.
"#
    )
}

fn render_scope(slug: &str, title: &str) -> String {
    format!(
        r#"# Scope: {title}

Topic slug: `{slug}`

## Research question

TODO: Define the central research question for this topic.

## Inclusion criteria

- TODO: Which papers are inside this topic?
- TODO: Which methods, mechanisms, materials, or models count as relevant?

## Exclusion criteria

- TODO: Which nearby topics should not be mixed into this topic?

## Review rule

If the scope is unclear, Manager LLM should ask a human reviewer before assigning Worker LLM relation tasks.
"#
    )
}

fn render_literature(slug: &str, title: &str) -> String {
    format!(
        r#"# Literature: {title}

Topic slug: `{slug}`

List literature included in this topic. Prefer stable references to `wiki/papers/` pages or `raw/papers/` source files.

| paper | role in topic | status | notes |
|---|---|---|---|
| TODO | unknown | candidate | Add topic-relevant papers here. |

## Notes

Use `processing/refs/` for global bibliographic index evidence. Use this file only for topic membership and local role.
"#
    )
}

fn render_importance_candidates(slug: &str, title: &str) -> String {
    format!(
        r#"# Importance Candidates: {title}

Topic slug: `{slug}`

This file records candidate topic-local importance labels. These are not final until reviewed.

| paper | importance_level | evidence | status | needs_human_review |
|---|---|---|---|---|
| TODO | unknown | TODO | candidate | true |

Allowed `importance_level` values: `core`, `important`, `background`, `peripheral`, `unknown`.
"#
    )
}

fn render_importance_review(slug: &str, title: &str) -> String {
    format!(
        r#"# Importance Review: {title}

Topic slug: `{slug}`

Use this table for human review of topic-local importance.

| paper | proposed_level | reviewer_decision | reviewer | evidence | date |
|---|---|---|---|---|---|
| TODO | unknown | pending | TODO | TODO | TODO |
"#
    )
}

fn render_confirmed_importance(slug: &str, title: &str) -> String {
    format!(
        r#"# Confirmed Importance: {title}

Topic slug: `{slug}`

Confirmed topic-local literature importance records.

| paper | importance_level | confirmed_by | evidence | date |
|---|---|---|---|---|
| TODO | unknown | TODO | TODO | TODO |
"#
    )
}

fn render_relation_file(slug: &str, title: &str, relation_family: &str) -> String {
    format!(
        r#"# {relation_family}: {title}

Topic slug: `{slug}`

This file stores topic-local relation records for `{relation_family}`.

| source | relation_type | target | status | evidence | needs_human_review |
|---|---|---|---|---|---|
| TODO | {relation_family} | TODO | candidate | TODO | true |

Allowed status values: `confirmed`, `candidate`, `ambiguous`, `needs_human`, `rejected`.

Do not use this file for global citation identity. Global citation/index evidence belongs under `processing/refs/`.
"#
    )
}

fn render_review_file(slug: &str, title: &str, review_kind: &str) -> String {
    format!(
        r#"# {review_kind}: {title}

Topic slug: `{slug}`

Use this file for topic-local human review.

| item | proposed_decision | final_decision | reviewer | evidence | date |
|---|---|---|---|---|---|
| TODO | pending | pending | TODO | TODO | TODO |
"#
    )
}

fn render_graph_readme(slug: &str, title: &str) -> String {
    format!(
        r#"# Topic Graph: {title}

Topic slug: `{slug}`

Future topic graph exports should be written here, for example:

- `topic_graph.json`
- `topic_graph.mmd`
- `topic_graph.dot`

Visual protocol:

- confirmed topic relation -> solid arrow
- candidate / needs_human topic relation -> dashed arrow
- missing / unresolved target -> hollow node
- topic-local importance -> node size
"#
    )
}

fn render_tasks_readme(slug: &str, title: &str) -> String {
    format!(
        r#"# Topic Tasks: {title}

Topic slug: `{slug}`

Store topic-specific Worker LLM / human handoff tasks here.

Task files should include:

- goal
- requirements
- file list
- evidence
- expected output
- review rule

Worker LLM must not expand scope beyond this topic without Manager LLM approval.
"#
    )
}

fn render_memory_file(slug: &str, title: &str, memory_kind: &str) -> String {
    format!(
        r#"# {memory_kind}: {title}

Topic slug: `{slug}`

Record completed topic-local work here.

| task_id | summary | files | unresolved_follow_up | date |
|---|---|---|---|---|
| TODO | TODO | TODO | TODO | TODO |
"#
    )
}

fn ensure_dir(path: &Path, summary: &mut InitSummary) -> Result<()> {
    if path.exists() {
        summary.dirs_skipped += 1;
    } else {
        fs::create_dir_all(path)?;
        summary.dirs_created += 1;
    }
    Ok(())
}

fn write_template_file(
    path: &Path,
    content: String,
    force: bool,
    summary: &mut InitSummary,
) -> Result<()> {
    if path.exists() && !force {
        summary.files_skipped += 1;
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content)?;
    summary.files_created += 1;
    Ok(())
}

fn print_summary(summary: &InitSummary) {
    println!("Topic workspace initialized:");
    println!("  slug       : {}", summary.topic_slug);
    println!("  title      : {}", summary.topic_title);
    println!("  path       : {}", summary.topic_path.display());
    println!("  dry run    : {}", summary.dry_run);
    println!(
        "  dirs       : {} created, {} skipped",
        summary.dirs_created, summary.dirs_skipped
    );
    println!(
        "  files      : {} created, {} skipped",
        summary.files_created, summary.files_skipped
    );
    println!();
    println!("Next:");
    println!("  1. Edit scope.md to define the research-topic boundary.");
    println!("  2. Add topic literature to literature.md.");
    println!("  3. Record topic-local importance and relation candidates under importance/ and relations/.");
    println!("  4. Use review/ for human-confirmed decisions when claims are important.");
}

fn topic_slug(input: &str) -> String {
    let mut output = String::new();
    let mut last_was_sep = false;
    for ch in input.trim().chars() {
        if ch.is_alphanumeric() {
            for lower in ch.to_lowercase() {
                output.push(lower);
            }
            last_was_sep = false;
        } else if matches!(
            ch,
            '-' | '_' | ' ' | '/' | '\\' | ':' | ';' | ',' | '(' | ')' | '[' | ']'
        ) {
            if !last_was_sep && !output.is_empty() {
                output.push('-');
                last_was_sep = true;
            }
        }
    }
    output.trim_matches('-').to_string()
}

fn topic_title(input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        "Untitled Topic".to_string()
    } else {
        trimmed.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn topic_slug_is_stable() {
        assert_eq!(topic_slug("Thermal Metamaterials"), "thermal-metamaterials");
        assert_eq!(
            topic_slug(" microfluidic/dialysis "),
            "microfluidic-dialysis"
        );
        assert_eq!(topic_slug("!!!"), "");
    }
}
