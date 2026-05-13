use anyhow::{anyhow, Result};
use clap::{Args, Subcommand};
use std::collections::BTreeMap;
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
    #[command(about = "List existing topic-specific relationship workspaces")]
    List(TopicListArgs),
    #[command(about = "Check one topic workspace for expected files and directories")]
    Status(TopicStatusArgs),
    #[command(about = "Generate topic-local literature importance candidates")]
    Rank(TopicRankArgs),
    #[command(about = "Compile topic-local directed relation records into graph artifacts")]
    Relations(TopicRelationsArgs),
    #[command(about = "Prepare a topic graph by running topic rank and topic relations")]
    Prepare(TopicPrepareArgs),
    #[command(about = "Build a deterministic review queue from topic-local importance candidates")]
    Review(TopicReviewArgs),
    #[command(
        name = "tasks",
        visible_alias = "task",
        about = "Generate topic-specific Manager/Worker LLM handoff tasks"
    )]
    Tasks(TopicTasksArgs),
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

#[derive(Debug, Clone, Args)]
pub struct TopicListArgs {
    #[arg(long, help = "Print a machine-readable JSON topic list")]
    pub json: bool,
}

#[derive(Debug, Clone, Args)]
pub struct TopicStatusArgs {
    #[arg(value_name = "TOPIC", help = "Topic name or slug to inspect")]
    pub topic: String,

    #[arg(long, help = "Print a machine-readable JSON topic status report")]
    pub json: bool,

    #[arg(
        long,
        help = "Return non-zero when required topic files or directories are missing"
    )]
    pub strict: bool,
}

#[derive(Debug, Clone, Args)]
pub struct TopicRankArgs {
    #[arg(value_name = "TOPIC", help = "Topic name or slug to rank within")]
    pub topic: String,

    #[arg(
        long,
        default_value_t = 50,
        help = "Maximum number of topic-local importance candidates to return. Use 0 for no limit."
    )]
    pub limit: usize,

    #[arg(
        long,
        help = "Preview topic-local importance ranking without writing an importance report"
    )]
    pub dry_run: bool,

    #[arg(long, help = "Alias for --dry-run")]
    pub preview: bool,

    #[arg(
        long,
        help = "Print a machine-readable JSON topic-local importance report"
    )]
    pub json: bool,
}

#[derive(Debug, Clone, Args)]
pub struct TopicRelationsArgs {
    #[arg(
        value_name = "TOPIC",
        help = "Topic name or slug to compile relations for"
    )]
    pub topic: String,

    #[arg(
        long,
        help = "Preview relation graph generation without writing graph artifacts"
    )]
    pub dry_run: bool,

    #[arg(long, help = "Alias for --dry-run")]
    pub preview: bool,

    #[arg(
        long,
        help = "Print a machine-readable JSON topic relation graph report"
    )]
    pub json: bool,
}

#[derive(Debug, Clone, Args)]
pub struct TopicPrepareArgs {
    #[arg(value_name = "TOPIC", help = "Topic name or slug to prepare")]
    pub topic: String,

    #[arg(
        long,
        value_name = "TITLE",
        help = "Human-readable topic title when a missing workspace is initialized"
    )]
    pub title: Option<String>,

    #[arg(
        long,
        default_value_t = 50,
        help = "Maximum number of topic-local importance candidates to include. Use 0 for no limit."
    )]
    pub limit: usize,

    #[arg(
        long,
        help = "Do not initialize a missing topic workspace before running rank and relations"
    )]
    pub no_init: bool,

    #[arg(long, help = "Preview topic preparation without writing files")]
    pub dry_run: bool,

    #[arg(long, help = "Alias for --dry-run")]
    pub preview: bool,

    #[arg(long, help = "Print a machine-readable JSON topic preparation report")]
    pub json: bool,
}

#[derive(Debug, Clone, Args)]
pub struct TopicReviewArgs {
    #[arg(value_name = "TOPIC", help = "Topic name or slug to review")]
    pub topic: String,

    #[arg(long, help = "Preview review queue generation without writing files")]
    pub dry_run: bool,

    #[arg(long, help = "Alias for --dry-run")]
    pub preview: bool,

    #[arg(long, help = "Print a machine-readable JSON topic review report")]
    pub json: bool,

    #[arg(
        long,
        help = "Overwrite existing review_queue.md and review_summary.md files"
    )]
    pub force: bool,
}

#[derive(Debug, Clone, Args)]
pub struct TopicTasksArgs {
    #[arg(
        value_name = "TOPIC",
        help = "Topic name or slug to generate tasks for"
    )]
    pub topic: String,

    #[arg(long, help = "Preview topic task generation without writing files")]
    pub dry_run: bool,

    #[arg(long, help = "Alias for --dry-run")]
    pub preview: bool,

    #[arg(
        long,
        default_value_t = 0,
        help = "Maximum number of topic task items to return. Use 0 for no limit."
    )]
    pub limit: usize,

    #[arg(long, help = "Print a machine-readable JSON topic task report")]
    pub json: bool,
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
        TopicCommand::List(list_args) => execute_list(custom_kb, list_args),
        TopicCommand::Status(status_args) => execute_status(custom_kb, status_args),
        TopicCommand::Rank(rank_args) => execute_rank(custom_kb, rank_args),
        TopicCommand::Relations(relations_args) => execute_relations(custom_kb, relations_args),
        TopicCommand::Prepare(prepare_args) => execute_prepare(custom_kb, prepare_args),
        TopicCommand::Review(review_args) => execute_review(custom_kb, review_args),
        TopicCommand::Tasks(tasks_args) => execute_tasks(custom_kb, tasks_args),
    }
}

#[derive(Debug, serde::Serialize)]
struct TopicRankReport {
    schema_version: String,
    generated_by: String,
    generated_at: String,
    topic_slug: String,
    topic_path: String,
    dry_run: bool,
    report_path: Option<String>,
    candidates_total: usize,
    returned_count: usize,
    limit: usize,
    candidates: Vec<TopicImportanceCandidate>,
    deferred_review: Vec<TopicRankReviewHint>,
    next_steps: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
struct TopicRelationsReport {
    schema_version: String,
    generated_by: String,
    generated_at: String,
    topic_slug: String,
    topic_path: String,
    relation_dir: String,
    graph_dir: String,
    dry_run: bool,
    written: bool,
    graph_json_path: String,
    graph_markdown_path: String,
    relation_files: usize,
    relation_records: usize,
    node_count: usize,
    edge_count: usize,
    nodes: Vec<TopicGraphNode>,
    edges: Vec<TopicGraphEdge>,
    warnings: Vec<String>,
    next_steps: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
struct TopicGraphNode {
    id: String,
    label: String,
    kind: String,
    path: Option<String>,
    importance_level: Option<String>,
    status: String,
    evidence: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
struct TopicGraphEdge {
    id: String,
    source: String,
    target: String,
    relation_type: String,
    status: String,
    evidence: Vec<String>,
    needs_human_review: bool,
    source_file: String,
    topic: String,
}

#[derive(Debug, serde::Serialize)]
struct TopicPrepareReport {
    schema_version: String,
    generated_by: String,
    generated_at: String,
    topic_slug: String,
    topic_path: String,
    dry_run: bool,
    initialized_missing_workspace: bool,
    rank_report_path: Option<String>,
    relations_graph_json_path: String,
    relations_graph_markdown_path: String,
    importance_candidates: usize,
    relation_records: usize,
    graph_nodes: usize,
    graph_edges: usize,
    warnings: Vec<String>,
    next_steps: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
struct TopicImportanceCandidate {
    paper: String,
    proposed_importance_level: String,
    score: i32,
    status: String,
    needs_human_review: bool,
    reasons: Vec<String>,
    evidence: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
struct TopicRankReviewHint {
    target_agent: String,
    goal: String,
    requirements: Vec<String>,
    files: Vec<String>,
    evidence: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
struct TopicReviewReport {
    schema_version: String,
    generated_by: String,
    generated_at: String,
    topic_slug: String,
    topic_path: String,
    candidate_dir: String,
    review_dir: String,
    queue_path: String,
    summary_path: String,
    dry_run: bool,
    force: bool,
    written: bool,
    candidate_files: usize,
    review_items: usize,
    empty_or_unreadable_candidates: usize,
    warnings: Vec<String>,
    items: Vec<TopicReviewItem>,
    next_steps: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
struct TopicReviewItem {
    review_id: String,
    candidate_id: String,
    source_item: String,
    proposed_decision: String,
    status: String,
    reviewer: String,
    evidence: String,
}

#[derive(Debug, serde::Serialize)]
struct TopicTasksReport {
    schema_version: String,
    generated_by: String,
    generated_at: String,
    topic_slug: String,
    topic_path: String,
    dry_run: bool,
    task_count: usize,
    returned_count: usize,
    limit: usize,
    index_path: String,
    task_item_dir: String,
    task_item_count: usize,
    warnings: Vec<String>,
    tasks: Vec<TopicTaskItem>,
    next_steps: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
struct TopicTaskItem {
    id: String,
    priority: String,
    category: String,
    target_agent: String,
    goal: String,
    requirements: Vec<String>,
    files: Vec<String>,
    evidence: Vec<String>,
    source_command: String,
    expected_output: Vec<String>,
    review_rule: String,
}

#[derive(Debug, serde::Serialize)]
struct TopicListReport {
    topics_dir: String,
    topic_count: usize,
    topics: Vec<TopicListItem>,
}

#[derive(Debug, serde::Serialize)]
struct TopicListItem {
    slug: String,
    title: String,
    path: String,
    has_scope: bool,
    has_literature: bool,
}

#[derive(Debug, serde::Serialize)]
struct TopicStatusReport {
    topic_slug: String,
    topic_path: String,
    status: String,
    required_dirs_present: usize,
    required_dirs_missing: usize,
    required_files_present: usize,
    required_files_missing: usize,
    missing_dirs: Vec<String>,
    missing_files: Vec<String>,
    next_steps: Vec<String>,
}

fn execute_list(custom_kb: Option<&Path>, args: &TopicListArgs) -> Result<()> {
    let kb_path = crate::commands::init::get_kb_path(custom_kb);
    let topics_dir = kb_path.join("topics");
    let topics = collect_topics(&kb_path, &topics_dir)?;
    let report = TopicListReport {
        topics_dir: relative_path_string(&kb_path, &topics_dir),
        topic_count: topics.len(),
        topics,
    };

    if args.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
        return Ok(());
    }

    println!("Topic workspaces:");
    println!("  topics dir : {}", report.topics_dir);
    println!("  count      : {}", report.topic_count);
    if report.topics.is_empty() {
        println!();
        println!("No topic workspaces found. Create one with `kb topic init <topic>`.");
        return Ok(());
    }

    println!();
    for topic in report.topics {
        println!("- {}", topic.slug);
        println!("  title     : {}", topic.title);
        println!("  path      : {}", topic.path);
        println!(
            "  scope     : {}",
            if topic.has_scope { "yes" } else { "missing" }
        );
        println!(
            "  literature: {}",
            if topic.has_literature {
                "yes"
            } else {
                "missing"
            }
        );
    }
    Ok(())
}

fn execute_status(custom_kb: Option<&Path>, args: &TopicStatusArgs) -> Result<()> {
    let kb_path = crate::commands::init::get_kb_path(custom_kb);
    let slug = topic_slug(&args.topic);
    if slug.is_empty() {
        return Err(anyhow!(
            "topic name must contain at least one letter or digit"
        ));
    }
    let topic_root = kb_path.join("topics").join(&slug);
    let report = build_topic_status(&kb_path, &slug, &topic_root);

    if args.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print_status_report(&report);
    }

    if args.strict && report.status != "ok" {
        return Err(anyhow!(
            "topic workspace `{}` is incomplete: {} missing dirs, {} missing files",
            report.topic_slug,
            report.required_dirs_missing,
            report.required_files_missing
        ));
    }
    Ok(())
}

fn execute_rank(custom_kb: Option<&Path>, args: &TopicRankArgs) -> Result<()> {
    let kb_path = crate::commands::init::get_kb_path(custom_kb);
    let slug = topic_slug(&args.topic);
    if slug.is_empty() {
        return Err(anyhow!(
            "topic name must contain at least one letter or digit"
        ));
    }
    let topic_root = kb_path.join("topics").join(&slug);
    if !topic_root.exists() {
        return Err(anyhow!(
            "topic workspace does not exist: {}. Run `kb topic init {}` first.",
            topic_root.display(),
            slug
        ));
    }

    let mut report = build_topic_rank_report(&kb_path, &slug, &topic_root, args)?;
    let dry_run = args.dry_run || args.preview;
    if !dry_run {
        let out_path = write_topic_rank_report(&kb_path, &slug, &report)?;
        report.report_path = Some(relative_path_string(&kb_path, &out_path));
    }

    if args.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print_rank_report(&report);
    }
    Ok(())
}

fn build_topic_rank_report(
    kb_path: &Path,
    slug: &str,
    topic_root: &Path,
    args: &TopicRankArgs,
) -> Result<TopicRankReport> {
    let dry_run = args.dry_run || args.preview;
    let literature_path = topic_root.join("literature.md");
    let literature = fs::read_to_string(&literature_path).unwrap_or_default();
    let mut candidates = parse_topic_literature(&literature);

    let topic_text = collect_topic_text(topic_root)?;
    let refs_index_text = collect_refs_index_text(&kb_path.join("processing/refs"))?;

    for candidate in &mut candidates {
        score_candidate(candidate, &topic_text, &refs_index_text);
    }

    candidates.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.paper.cmp(&b.paper)));
    let candidates_total = candidates.len();
    let returned = if args.limit > 0 {
        candidates.into_iter().take(args.limit).collect::<Vec<_>>()
    } else {
        candidates
    };

    let deferred_review = build_rank_review_hints(&kb_path, topic_root, &returned);
    let next_steps = if returned.is_empty() {
        vec![
            format!("Add topic literature rows to topics/{slug}/literature.md."),
            "Then rerun `kb topic rank <topic>` to generate topic-local importance candidates."
                .to_string(),
        ]
    } else {
        vec![
            format!("Review generated candidates under topics/{slug}/importance/."),
            format!("Record accepted decisions in topics/{slug}/importance/confirmed_importance.md."),
            format!("Use topics/{slug}/review/importance_review.md for human review of core/important claims."),
        ]
    };

    Ok(TopicRankReport {
        schema_version: "topic-rank.v0.6.7".to_string(),
        generated_by: "kb-cli topic rank".to_string(),
        generated_at: chrono::Utc::now().to_rfc3339(),
        topic_slug: slug.to_string(),
        topic_path: relative_path_string(kb_path, topic_root),
        dry_run,
        report_path: None,
        candidates_total,
        returned_count: returned.len(),
        limit: args.limit,
        candidates: returned,
        deferred_review,
        next_steps,
    })
}

fn parse_topic_literature(content: &str) -> Vec<TopicImportanceCandidate> {
    let mut out = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with('|') || trimmed.contains("|---") {
            continue;
        }
        let cells = trimmed
            .trim_matches('|')
            .split('|')
            .map(|cell| cell.trim())
            .collect::<Vec<_>>();
        if cells.is_empty() {
            continue;
        }
        let paper = cells[0].trim();
        if paper.is_empty()
            || paper.eq_ignore_ascii_case("paper")
            || paper.eq_ignore_ascii_case("todo")
            || paper.eq_ignore_ascii_case("TODO")
        {
            continue;
        }
        let role = cells.get(1).copied().unwrap_or("unknown");
        let status = cells.get(2).copied().unwrap_or("candidate");
        let mut candidate = TopicImportanceCandidate {
            paper: paper.to_string(),
            proposed_importance_level: "unknown".to_string(),
            score: 0,
            status: "candidate".to_string(),
            needs_human_review: true,
            reasons: Vec::new(),
            evidence: vec![format!(
                "literature.md row: paper=`{paper}`, role=`{role}`, status=`{status}`"
            )],
        };
        candidate.score += 20;
        candidate
            .reasons
            .push("listed in topic literature.md".to_string());
        let role_lower = role.to_lowercase();
        if contains_any(
            &role_lower,
            &[
                "core",
                "central",
                "main",
                "classic",
                "source",
                "origin",
                "foundation",
            ],
        ) {
            candidate.score += 35;
            candidate
                .reasons
                .push("topic literature role suggests core/source paper".to_string());
        } else if contains_any(
            &role_lower,
            &[
                "method",
                "model",
                "framework",
                "review",
                "survey",
                "important",
            ],
        ) {
            candidate.score += 25;
            candidate
                .reasons
                .push("topic literature role suggests important method/review paper".to_string());
        } else if contains_any(&role_lower, &["background", "context"]) {
            candidate.score += 10;
            candidate
                .reasons
                .push("topic literature role suggests background paper".to_string());
        } else if contains_any(&role_lower, &["peripheral", "weak", "nearby"]) {
            candidate.score += 3;
            candidate
                .reasons
                .push("topic literature role suggests peripheral paper".to_string());
        }
        out.push(candidate);
    }
    out
}

fn score_candidate(
    candidate: &mut TopicImportanceCandidate,
    topic_text: &str,
    refs_index_text: &str,
) {
    let needle = normalize_for_search(&candidate.paper);
    if needle.is_empty() {
        candidate.proposed_importance_level = importance_level(candidate.score).to_string();
        return;
    }

    let topic_mentions = count_occurrences(&normalize_for_search(topic_text), &needle);
    if topic_mentions > 1 {
        let bonus = ((topic_mentions as i32 - 1) * 5).min(30);
        candidate.score += bonus;
        candidate.reasons.push(format!(
            "mentioned {topic_mentions} times inside the topic workspace"
        ));
        candidate
            .evidence
            .push(format!("topic workspace mention count: {topic_mentions}"));
    }

    let refs_mentions = count_occurrences(&normalize_for_search(refs_index_text), &needle);
    if refs_mentions > 0 {
        let bonus = (refs_mentions as i32 * 5).min(20);
        candidate.score += bonus;
        candidate
            .reasons
            .push("appears in global bibliographic index outputs".to_string());
        candidate
            .evidence
            .push(format!("processing/refs mention count: {refs_mentions}"));
    }

    let paper_lower = candidate.paper.to_lowercase();
    if contains_any(
        &paper_lower,
        &["review", "survey", "overview", "perspective"],
    ) {
        candidate.score += 8;
        candidate
            .reasons
            .push("title/path looks like a review or survey".to_string());
    }
    if contains_any(&paper_lower, &["classic", "seminal", "landmark"]) {
        candidate.score += 15;
        candidate
            .reasons
            .push("title/path contains classic/seminal marker".to_string());
    }

    candidate.proposed_importance_level = importance_level(candidate.score).to_string();
    candidate.needs_human_review = matches!(
        candidate.proposed_importance_level.as_str(),
        "core" | "important"
    );
}

fn build_rank_review_hints(
    kb_path: &Path,
    topic_root: &Path,
    candidates: &[TopicImportanceCandidate],
) -> Vec<TopicRankReviewHint> {
    let high_impact = candidates
        .iter()
        .filter(|candidate| {
            matches!(
                candidate.proposed_importance_level.as_str(),
                "core" | "important"
            )
        })
        .take(20)
        .collect::<Vec<_>>();
    if high_impact.is_empty() {
        return Vec::new();
    }
    let files = vec![
        relative_path_string(kb_path, &topic_root.join("literature.md")),
        relative_path_string(
            kb_path,
            &topic_root.join("importance/importance_candidates.md"),
        ),
        relative_path_string(
            kb_path,
            &topic_root.join("importance/confirmed_importance.md"),
        ),
        relative_path_string(kb_path, &topic_root.join("review/importance_review.md")),
    ];
    let evidence = high_impact
        .iter()
        .map(|candidate| {
            format!(
                "{} -> {} (score {})",
                candidate.paper, candidate.proposed_importance_level, candidate.score
            )
        })
        .collect::<Vec<_>>();

    vec![TopicRankReviewHint {
        target_agent: "Human Topic Importance Reviewer / Manager LLM".to_string(),
        goal: "Review topic-local literature importance candidates before treating node size as meaningful.".to_string(),
        requirements: vec![
            "Confirm whether proposed core/important papers are actually central to this topic.".to_string(),
            "Record accepted decisions in importance/confirmed_importance.md.".to_string(),
            "Do not treat this deterministic score as final authority.".to_string(),
        ],
        files,
        evidence,
    }]
}

fn write_topic_rank_report(
    kb_path: &Path,
    slug: &str,
    report: &TopicRankReport,
) -> Result<PathBuf> {
    let out_dir = kb_path.join("topics").join(slug).join("importance");
    fs::create_dir_all(&out_dir)?;
    let stamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let out_path = out_dir.join(format!("importance_candidates_{stamp}.md"));
    fs::write(&out_path, render_topic_rank_markdown(report))?;
    Ok(out_path)
}

fn render_topic_rank_markdown(report: &TopicRankReport) -> String {
    let mut md = String::new();
    md.push_str(&format!(
        "# Topic Importance Candidates: {}\n\n",
        report.topic_slug
    ));
    md.push_str(&format!("Generated at: `{}`\n\n", report.generated_at));
    md.push_str("These are deterministic, topic-local candidates. They are not final literature-importance decisions.\n\n");
    md.push_str("| paper | proposed_importance_level | score | status | needs_human_review | reasons | evidence |\n");
    md.push_str("|---|---:|---:|---|---|---|---|\n");
    for candidate in &report.candidates {
        md.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} |\n",
            escape_table_cell(&candidate.paper),
            candidate.proposed_importance_level,
            candidate.score,
            candidate.status,
            candidate.needs_human_review,
            escape_table_cell(&candidate.reasons.join("; ")),
            escape_table_cell(&candidate.evidence.join("; "))
        ));
    }
    if report.candidates.is_empty() {
        md.push_str("| _none_ | unknown | 0 | candidate | true | Add topic literature first. | topics/<topic>/literature.md |\n");
    }
    md.push_str("\n## Review rule\n\nCore/important labels should be reviewed by a human before they drive graph node sizes or topic conclusions.\n\n");
    md.push_str("## Next steps\n\n");
    for step in &report.next_steps {
        md.push_str(&format!("- {step}\n"));
    }
    md
}

fn print_rank_report(report: &TopicRankReport) {
    println!("Topic importance candidates:");
    println!("  topic       : {}", report.topic_slug);
    println!("  path        : {}", report.topic_path);
    println!(
        "  candidates  : {} total, {} returned",
        report.candidates_total, report.returned_count
    );
    println!("  dry run     : {}", report.dry_run);
    if let Some(path) = &report.report_path {
        println!("  report      : {path}");
    }
    println!();
    if report.candidates.is_empty() {
        println!(
            "No topic literature rows found. Add papers to topics/{}/literature.md.",
            report.topic_slug
        );
    } else {
        for candidate in &report.candidates {
            println!(
                "- {} -> {} (score {}, review: {})",
                candidate.paper,
                candidate.proposed_importance_level,
                candidate.score,
                candidate.needs_human_review
            );
            for reason in &candidate.reasons {
                println!("  reason: {reason}");
            }
        }
    }
    if !report.next_steps.is_empty() {
        println!();
        println!("Next steps:");
        for step in &report.next_steps {
            println!("  - {step}");
        }
    }
}

fn execute_relations(custom_kb: Option<&Path>, args: &TopicRelationsArgs) -> Result<()> {
    let kb_path = crate::commands::init::get_kb_path(custom_kb);
    let slug = topic_slug(&args.topic);
    if slug.is_empty() {
        return Err(anyhow!(
            "topic name must contain at least one letter or digit"
        ));
    }
    let topic_root = kb_path.join("topics").join(&slug);
    if !topic_root.exists() {
        return Err(anyhow!(
            "topic workspace does not exist: {}. Run `kb topic init {}` first.",
            topic_root.display(),
            slug
        ));
    }

    let dry_run = args.dry_run || args.preview;
    let mut report = build_topic_relations_report(&kb_path, &slug, &topic_root, dry_run)?;
    if !dry_run {
        write_topic_relations_report(&kb_path, &slug, &mut report)?;
    }

    if args.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print_relations_report(&report);
    }
    Ok(())
}

fn execute_prepare(custom_kb: Option<&Path>, args: &TopicPrepareArgs) -> Result<()> {
    let kb_path = crate::commands::init::get_kb_path(custom_kb);
    let slug = topic_slug(&args.topic);
    if slug.is_empty() {
        return Err(anyhow!(
            "topic name must contain at least one letter or digit"
        ));
    }
    let topic_root = kb_path.join("topics").join(&slug);
    let dry_run = args.dry_run || args.preview;
    let mut initialized_missing_workspace = false;
    let mut warnings = Vec::new();

    if !topic_root.exists() {
        if args.no_init {
            return Err(anyhow!(
                "topic workspace does not exist: {}. Run `kb topic init {}` first, or omit --no-init.",
                topic_root.display(),
                slug
            ));
        }
        initialized_missing_workspace = true;
        if dry_run {
            warnings.push(format!(
                "topic workspace would be initialized before preparation: topics/{slug}/"
            ));
        } else {
            let title = args
                .title
                .clone()
                .unwrap_or_else(|| topic_title(&args.topic));
            ensure_topic_workspace_templates(&kb_path, &slug, &title)?;
        }
    }

    if dry_run {
        let report = TopicPrepareReport {
            schema_version: "topic-prepare.v0.7.10".to_string(),
            generated_by: "kb-cli topic prepare".to_string(),
            generated_at: chrono::Utc::now().to_rfc3339(),
            topic_slug: slug.clone(),
            topic_path: relative_path_string(&kb_path, &topic_root),
            dry_run,
            initialized_missing_workspace,
            rank_report_path: None,
            relations_graph_json_path: format!("topics/{slug}/graph/topic_graph.json"),
            relations_graph_markdown_path: format!("topics/{slug}/graph/topic_graph.md"),
            importance_candidates: 0,
            relation_records: 0,
            graph_nodes: 0,
            graph_edges: 0,
            warnings,
            next_steps: vec![
                format!("Would run `kb topic rank {slug} --limit {}`.", args.limit),
                format!("Would run `kb topic relations {slug}`."),
                format!("Then inspect the result with `kb view --relations --topic {slug}`."),
            ],
        };
        if args.json {
            println!("{}", serde_json::to_string_pretty(&report)?);
        } else {
            print_prepare_report(&report);
        }
        return Ok(());
    }

    let rank_args = TopicRankArgs {
        topic: slug.clone(),
        limit: args.limit,
        dry_run: false,
        preview: false,
        json: false,
    };
    let mut rank_report = build_topic_rank_report(&kb_path, &slug, &topic_root, &rank_args)?;
    let rank_report_path = write_topic_rank_report(&kb_path, &slug, &rank_report)?;
    rank_report.report_path = Some(relative_path_string(&kb_path, &rank_report_path));

    let mut relations_report = build_topic_relations_report(&kb_path, &slug, &topic_root, false)?;
    write_topic_relations_report(&kb_path, &slug, &mut relations_report)?;
    warnings.extend(relations_report.warnings.clone());

    let report = TopicPrepareReport {
        schema_version: "topic-prepare.v0.7.10".to_string(),
        generated_by: "kb-cli topic prepare".to_string(),
        generated_at: chrono::Utc::now().to_rfc3339(),
        topic_slug: slug.clone(),
        topic_path: relative_path_string(&kb_path, &topic_root),
        dry_run,
        initialized_missing_workspace,
        rank_report_path: rank_report.report_path.clone(),
        relations_graph_json_path: relations_report.graph_json_path.clone(),
        relations_graph_markdown_path: relations_report.graph_markdown_path.clone(),
        importance_candidates: rank_report.returned_count,
        relation_records: relations_report.relation_records,
        graph_nodes: relations_report.node_count,
        graph_edges: relations_report.edge_count,
        warnings,
        next_steps: vec![
            format!("Review topic-local importance under topics/{slug}/importance/."),
            format!("Review directed topic relations under topics/{slug}/relations/."),
            format!("Inspect the graph with `kb view --relations --topic {slug}`."),
        ],
    };

    if args.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print_prepare_report(&report);
    }
    Ok(())
}

fn ensure_topic_workspace_templates(kb_path: &Path, slug: &str, title: &str) -> Result<()> {
    let topic_root = kb_path.join("topics").join(slug);
    fs::create_dir_all(&topic_root)?;
    for dir in required_topic_dirs() {
        fs::create_dir_all(topic_root.join(dir))?;
    }
    for template in topic_templates(slug, title) {
        let path = topic_root.join(&template.path);
        if path.exists() {
            continue;
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, template.content)?;
    }
    Ok(())
}

fn build_topic_relations_report(
    kb_path: &Path,
    slug: &str,
    topic_root: &Path,
    dry_run: bool,
) -> Result<TopicRelationsReport> {
    let relation_dir = topic_root.join("relations");
    let graph_dir = topic_root.join("graph");
    let graph_json_path = graph_dir.join("topic_graph.json");
    let graph_markdown_path = graph_dir.join("topic_graph.md");
    let mut nodes_by_id: BTreeMap<String, TopicGraphNode> = BTreeMap::new();
    let mut edges = Vec::new();
    let mut warnings = Vec::new();

    let topic_title = read_topic_title(topic_root).unwrap_or_else(|| slug.to_string());
    insert_topic_graph_node(
        &mut nodes_by_id,
        TopicGraphNode {
            id: format!("topic:{slug}"),
            label: topic_title,
            kind: "topic".to_string(),
            path: Some(relative_path_string(kb_path, topic_root)),
            importance_level: None,
            status: "indexed".to_string(),
            evidence: vec![relative_path_string(kb_path, topic_root)],
        },
    );

    add_topic_graph_literature(kb_path, slug, topic_root, &mut nodes_by_id, &mut edges)?;
    add_topic_graph_importance(kb_path, slug, topic_root, &mut nodes_by_id, &mut edges)?;
    let relation_files = add_topic_graph_relation_records(
        kb_path,
        slug,
        topic_root,
        &mut nodes_by_id,
        &mut edges,
        &mut warnings,
    )?;

    dedupe_topic_graph_edges(&mut edges);
    let mut nodes = nodes_by_id.into_values().collect::<Vec<_>>();
    nodes.sort_by(|a, b| a.id.cmp(&b.id));
    edges.sort_by(|a, b| {
        a.status
            .cmp(&b.status)
            .then_with(|| a.source.cmp(&b.source))
            .then_with(|| a.target.cmp(&b.target))
            .then_with(|| a.relation_type.cmp(&b.relation_type))
    });

    let relation_records = edges
        .iter()
        .filter(|edge| edge.source_file.contains("/relations/"))
        .count();
    if relation_records == 0 {
        warnings.push(format!(
            "no paper-to-paper topic relation records found under topics/{slug}/relations/; the graph will show membership and importance only"
        ));
    }
    if !relation_dir.exists() {
        warnings.push(format!(
            "relation directory missing: topics/{slug}/relations/"
        ));
    }

    let next_steps = vec![
        format!("Edit topics/{slug}/relations/*.md to add evidence-backed directed paper-to-paper relations."),
        format!("Run `kb topic prepare {slug}` after changing literature, importance, or relation files."),
        format!("Inspect with `kb view --relations --topic {slug}`."),
    ];

    Ok(TopicRelationsReport {
        schema_version: "topic-relations.v0.7.10".to_string(),
        generated_by: "kb-cli topic relations".to_string(),
        generated_at: chrono::Utc::now().to_rfc3339(),
        topic_slug: slug.to_string(),
        topic_path: relative_path_string(kb_path, topic_root),
        relation_dir: relative_path_string(kb_path, &relation_dir),
        graph_dir: relative_path_string(kb_path, &graph_dir),
        dry_run,
        written: false,
        graph_json_path: relative_path_string(kb_path, &graph_json_path),
        graph_markdown_path: relative_path_string(kb_path, &graph_markdown_path),
        relation_files,
        relation_records,
        node_count: nodes.len(),
        edge_count: edges.len(),
        nodes,
        edges,
        warnings,
        next_steps,
    })
}

fn write_topic_relations_report(
    kb_path: &Path,
    slug: &str,
    report: &mut TopicRelationsReport,
) -> Result<()> {
    let graph_dir = kb_path.join("topics").join(slug).join("graph");
    fs::create_dir_all(&graph_dir)?;
    let json_path = graph_dir.join("topic_graph.json");
    let md_path = graph_dir.join("topic_graph.md");
    report.written = true;
    report.graph_json_path = relative_path_string(kb_path, &json_path);
    report.graph_markdown_path = relative_path_string(kb_path, &md_path);
    fs::write(&json_path, serde_json::to_string_pretty(&report)?)?;
    fs::write(&md_path, render_topic_relations_markdown(report))?;
    Ok(())
}

fn add_topic_graph_literature(
    kb_path: &Path,
    slug: &str,
    topic_root: &Path,
    nodes_by_id: &mut BTreeMap<String, TopicGraphNode>,
    edges: &mut Vec<TopicGraphEdge>,
) -> Result<()> {
    let path = topic_root.join("literature.md");
    if !path.exists() {
        return Ok(());
    }
    let rel_path = relative_path_string(kb_path, &path);
    let content = fs::read_to_string(&path).unwrap_or_default();
    let topic_id = format!("topic:{slug}");
    for (idx, line) in content.lines().enumerate() {
        let cells = split_markdown_table_row(line);
        if cells.len() < 2 || is_topic_table_header_or_rule(&cells) {
            continue;
        }
        let paper = cells[0].trim();
        if !is_real_candidate_value(paper) {
            continue;
        }
        let role = cells
            .get(1)
            .cloned()
            .unwrap_or_else(|| "topic member".to_string());
        let status = normalize_topic_relation_status(
            cells.get(2).map(String::as_str).unwrap_or("candidate"),
        );
        let paper_id = topic_graph_node_id("paper", paper);
        insert_topic_graph_node(
            nodes_by_id,
            TopicGraphNode {
                id: paper_id.clone(),
                label: topic_graph_label(paper),
                kind: "paper".to_string(),
                path: Some(paper.to_string()),
                importance_level: None,
                status: status.clone(),
                evidence: vec![format!("{}:{}", rel_path, idx + 1)],
            },
        );
        edges.push(TopicGraphEdge {
            id: format!("edge:{topic_id}->{paper_id}:topic_membership:{}", idx + 1),
            source: topic_id.clone(),
            target: paper_id,
            relation_type: "topic_membership".to_string(),
            status: status.clone(),
            evidence: vec![format!("{}:{} | role={}", rel_path, idx + 1, role)],
            needs_human_review: !matches!(status.as_str(), "confirmed" | "accepted"),
            source_file: rel_path.clone(),
            topic: slug.to_string(),
        });
    }
    Ok(())
}

fn add_topic_graph_importance(
    kb_path: &Path,
    slug: &str,
    topic_root: &Path,
    nodes_by_id: &mut BTreeMap<String, TopicGraphNode>,
    edges: &mut Vec<TopicGraphEdge>,
) -> Result<()> {
    let importance_dir = topic_root.join("importance");
    if !importance_dir.exists() {
        return Ok(());
    }
    let mut files = collect_markdown_files(&importance_dir);
    files.sort();
    let topic_id = format!("topic:{slug}");
    for file in files {
        let name = file
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default();
        if name == "importance_review.md" {
            continue;
        }
        let rel_path = relative_path_string(kb_path, &file);
        let base_status = if name.contains("confirmed") {
            "confirmed"
        } else {
            "candidate"
        };
        let content = fs::read_to_string(&file).unwrap_or_default();
        for (idx, line) in content.lines().enumerate() {
            let cells = split_markdown_table_row(line);
            if cells.len() < 2 || is_topic_table_header_or_rule(&cells) {
                continue;
            }
            let paper = cells[0].trim();
            if !is_real_candidate_value(paper) {
                continue;
            }
            let importance = cells
                .get(1)
                .cloned()
                .unwrap_or_else(|| "unknown".to_string());
            let status = if cells.iter().any(|cell| {
                let lower = cell.to_ascii_lowercase();
                lower.contains("confirm") || lower.contains("accept")
            }) {
                "confirmed".to_string()
            } else {
                base_status.to_string()
            };
            let paper_id = topic_graph_node_id("paper", paper);
            insert_topic_graph_node(
                nodes_by_id,
                TopicGraphNode {
                    id: paper_id.clone(),
                    label: topic_graph_label(paper),
                    kind: "paper".to_string(),
                    path: Some(paper.to_string()),
                    importance_level: Some(importance.clone()),
                    status: status.clone(),
                    evidence: vec![format!("{}:{}", rel_path, idx + 1)],
                },
            );
            edges.push(TopicGraphEdge {
                id: format!(
                    "edge:{topic_id}->{paper_id}:topic_importance:{}:{}",
                    topic_slug(&rel_path),
                    idx + 1
                ),
                source: topic_id.clone(),
                target: paper_id,
                relation_type: format!("topic_importance:{importance}"),
                status: status.clone(),
                evidence: vec![format!(
                    "{}:{} | importance={}",
                    rel_path,
                    idx + 1,
                    importance
                )],
                needs_human_review: status != "confirmed",
                source_file: rel_path.clone(),
                topic: slug.to_string(),
            });
        }
    }
    Ok(())
}

fn add_topic_graph_relation_records(
    kb_path: &Path,
    slug: &str,
    topic_root: &Path,
    nodes_by_id: &mut BTreeMap<String, TopicGraphNode>,
    edges: &mut Vec<TopicGraphEdge>,
    warnings: &mut Vec<String>,
) -> Result<usize> {
    let relation_dir = topic_root.join("relations");
    if !relation_dir.exists() {
        return Ok(0);
    }
    let mut files = collect_markdown_files(&relation_dir);
    files.sort();
    let mut files_seen = 0usize;
    for file in files {
        files_seen += 1;
        let rel_path = relative_path_string(kb_path, &file);
        let content = match fs::read_to_string(&file) {
            Ok(content) => content,
            Err(err) => {
                warnings.push(format!("could not read {rel_path}: {err}"));
                continue;
            }
        };
        for (idx, line) in content.lines().enumerate() {
            let cells = split_markdown_table_row(line);
            if cells.len() < 3 || is_topic_table_header_or_rule(&cells) {
                continue;
            }
            let source = cells[0].trim();
            let relation_type = cells.get(1).map(|cell| cell.trim()).unwrap_or("related_to");
            let target = cells.get(2).map(|cell| cell.trim()).unwrap_or_default();
            if !is_real_candidate_value(source)
                || !is_real_candidate_value(target)
                || !is_real_candidate_value(relation_type)
            {
                continue;
            }
            let status = normalize_topic_relation_status(
                cells.get(3).map(String::as_str).unwrap_or("candidate"),
            );
            let evidence = cells
                .get(4)
                .filter(|value| is_real_candidate_value(value))
                .cloned()
                .unwrap_or_else(|| "TODO evidence required".to_string());
            let needs_human_review = cells
                .get(5)
                .map(|value| parse_topic_bool(value))
                .unwrap_or(!matches!(status.as_str(), "confirmed" | "accepted"));
            let source_id = topic_graph_node_id("paper", source);
            let target_id = topic_graph_node_id("paper", target);
            insert_topic_graph_node(
                nodes_by_id,
                TopicGraphNode {
                    id: source_id.clone(),
                    label: topic_graph_label(source),
                    kind: "paper".to_string(),
                    path: Some(source.to_string()),
                    importance_level: None,
                    status: "indexed".to_string(),
                    evidence: vec![format!("{}:{}", rel_path, idx + 1)],
                },
            );
            insert_topic_graph_node(
                nodes_by_id,
                TopicGraphNode {
                    id: target_id.clone(),
                    label: topic_graph_label(target),
                    kind: "paper".to_string(),
                    path: Some(target.to_string()),
                    importance_level: None,
                    status: "indexed".to_string(),
                    evidence: vec![format!("{}:{}", rel_path, idx + 1)],
                },
            );
            edges.push(TopicGraphEdge {
                id: format!(
                    "edge:{source_id}->{target_id}:{}:{}",
                    topic_slug(relation_type),
                    idx + 1
                ),
                source: source_id,
                target: target_id,
                relation_type: relation_type.to_string(),
                status: status.clone(),
                evidence: vec![format!("{}:{} | {}", rel_path, idx + 1, evidence)],
                needs_human_review,
                source_file: rel_path.clone(),
                topic: slug.to_string(),
            });
        }
    }
    Ok(files_seen)
}

fn insert_topic_graph_node(nodes: &mut BTreeMap<String, TopicGraphNode>, node: TopicGraphNode) {
    nodes
        .entry(node.id.clone())
        .and_modify(|existing| {
            if existing.importance_level.is_none() && node.importance_level.is_some() {
                existing.importance_level = node.importance_level.clone();
            }
            if existing.path.is_none() && node.path.is_some() {
                existing.path = node.path.clone();
            }
            for evidence in &node.evidence {
                if !existing.evidence.contains(evidence) {
                    existing.evidence.push(evidence.clone());
                }
            }
        })
        .or_insert(node);
}

fn dedupe_topic_graph_edges(edges: &mut Vec<TopicGraphEdge>) {
    let mut seen = BTreeMap::new();
    edges.retain(|edge| {
        let key = format!(
            "{}|{}|{}|{}|{}",
            edge.source, edge.target, edge.relation_type, edge.status, edge.source_file
        );
        if seen.contains_key(&key) {
            false
        } else {
            seen.insert(key, true);
            true
        }
    });
}

fn render_topic_relations_markdown(report: &TopicRelationsReport) -> String {
    let mut md = String::new();
    md.push_str(&format!(
        "# Topic Relation Graph: {}\n\n",
        report.topic_slug
    ));
    md.push_str(&format!("Generated by: `{}`\n\n", report.generated_by));
    md.push_str(&format!("Generated at: `{}`\n\n", report.generated_at));
    md.push_str("This is a deterministic graph export. It compiles topic membership, topic-local importance, and manually/LLM-proposed directed relation records. It does not make final scholarly claims.\n\n");
    md.push_str("## Summary\n\n");
    md.push_str(&format!("- Nodes: `{}`\n", report.node_count));
    md.push_str(&format!("- Edges: `{}`\n", report.edge_count));
    md.push_str(&format!("- Relation files: `{}`\n", report.relation_files));
    md.push_str(&format!(
        "- Paper-to-paper relation records: `{}`\n",
        report.relation_records
    ));
    md.push_str(&format!("- JSON: `{}`\n", report.graph_json_path));

    if !report.warnings.is_empty() {
        md.push_str("\n## Warnings\n\n");
        for warning in &report.warnings {
            md.push_str(&format!("- {}\n", warning));
        }
    }

    md.push_str("\n## Edges\n\n");
    md.push_str("| source | relation_type | target | status | needs_human_review | evidence |\n");
    md.push_str("|---|---|---|---|---|---|\n");
    if report.edges.is_empty() {
        md.push_str(
            "| _none_ | related_to | _none_ | candidate | true | Add rows under relations/*.md |\n",
        );
    } else {
        for edge in &report.edges {
            md.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} |\n",
                escape_table_cell(&edge.source),
                escape_table_cell(&edge.relation_type),
                escape_table_cell(&edge.target),
                edge.status,
                edge.needs_human_review,
                escape_table_cell(&edge.evidence.join("; "))
            ));
        }
    }

    md.push_str("\n## Next steps\n\n");
    for step in &report.next_steps {
        md.push_str(&format!("- {step}\n"));
    }
    md
}

fn print_relations_report(report: &TopicRelationsReport) {
    println!("Topic relation graph:");
    println!("  topic           : {}", report.topic_slug);
    println!("  path            : {}", report.topic_path);
    println!("  relation files  : {}", report.relation_files);
    println!("  relation records: {}", report.relation_records);
    println!("  nodes           : {}", report.node_count);
    println!("  edges           : {}", report.edge_count);
    println!("  dry run         : {}", report.dry_run);
    println!("  written         : {}", report.written);
    println!("  graph json      : {}", report.graph_json_path);
    println!("  graph markdown  : {}", report.graph_markdown_path);
    if !report.warnings.is_empty() {
        println!();
        println!("Warnings:");
        for warning in &report.warnings {
            println!("  - {warning}");
        }
    }
    if !report.next_steps.is_empty() {
        println!();
        println!("Next steps:");
        for step in &report.next_steps {
            println!("  - {step}");
        }
    }
}

fn print_prepare_report(report: &TopicPrepareReport) {
    println!("Topic prepare:");
    println!("  topic             : {}", report.topic_slug);
    println!("  path              : {}", report.topic_path);
    println!(
        "  initialized       : {}",
        report.initialized_missing_workspace
    );
    println!("  dry run           : {}", report.dry_run);
    println!(
        "  rank report       : {}",
        report
            .rank_report_path
            .as_deref()
            .unwrap_or("<not written>")
    );
    println!("  graph json        : {}", report.relations_graph_json_path);
    println!(
        "  graph markdown    : {}",
        report.relations_graph_markdown_path
    );
    println!("  importance rows   : {}", report.importance_candidates);
    println!("  relation records  : {}", report.relation_records);
    println!("  graph nodes       : {}", report.graph_nodes);
    println!("  graph edges       : {}", report.graph_edges);
    if !report.warnings.is_empty() {
        println!();
        println!("Warnings:");
        for warning in &report.warnings {
            println!("  - {warning}");
        }
    }
    if !report.next_steps.is_empty() {
        println!();
        println!("Next steps:");
        for step in &report.next_steps {
            println!("  - {step}");
        }
    }
}

fn execute_tasks(custom_kb: Option<&Path>, args: &TopicTasksArgs) -> Result<()> {
    let kb_path = crate::commands::init::get_kb_path(custom_kb);
    let slug = topic_slug(&args.topic);
    if slug.is_empty() {
        return Err(anyhow!(
            "topic name must contain at least one letter or digit"
        ));
    }

    let topic_root = kb_path.join("topics").join(&slug);
    if !topic_root.exists() {
        return Err(anyhow!(
            "topic workspace does not exist: {}. Run `kb topic init {}` first.",
            topic_root.display(),
            slug
        ));
    }

    let mut report = build_topic_tasks_report(&kb_path, &slug, &topic_root, args)?;
    let dry_run = args.dry_run || args.preview;
    if !dry_run {
        let item_dir = write_topic_task_item_files(&kb_path, &slug, &report)?;
        let index_path = write_topic_task_index(&kb_path, &slug, &report)?;
        report.index_path = relative_path_string(&kb_path, &index_path);
        report.task_item_dir = relative_path_string(&kb_path, &item_dir);
        report.task_item_count = report.tasks.len();
    }

    if args.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print_topic_tasks_report(&report);
    }
    Ok(())
}

fn build_topic_tasks_report(
    kb_path: &Path,
    slug: &str,
    topic_root: &Path,
    args: &TopicTasksArgs,
) -> Result<TopicTasksReport> {
    let dry_run = args.dry_run || args.preview;
    let mut tasks = Vec::new();
    let mut warnings = Vec::new();

    let status_report = build_topic_status(kb_path, slug, topic_root);
    if status_report.status != "ok" {
        tasks.push(TopicTaskItem {
            id: format!("topic-{slug}-workspace-repair"),
            priority: "high".to_string(),
            category: "topic_workspace".to_string(),
            target_agent: "Manager LLM / human maintainer".to_string(),
            goal: "Repair the topic workspace structure before assigning semantic Worker LLM tasks.".to_string(),
            requirements: vec![
                format!("Run `kb topic status {slug}` and inspect missing files/directories."),
                format!("Run `kb topic init {slug}` to create missing templates without overwriting existing files."),
                "Do not edit scholarly relation files until the workspace structure is complete.".to_string(),
            ],
            files: status_report
                .missing_dirs
                .iter()
                .chain(status_report.missing_files.iter())
                .cloned()
                .collect(),
            evidence: vec![format!(
                "missing dirs: {}; missing files: {}",
                status_report.required_dirs_missing, status_report.required_files_missing
            )],
            source_command: format!("kb topic status {slug}"),
            expected_output: vec![
                "Complete topic directory skeleton under topics/<topic>/.".to_string(),
                "No semantic claims accepted or rejected during structure repair.".to_string(),
            ],
            review_rule: "Human or Manager LLM checks `kb topic status <topic>` before further topic work.".to_string(),
        });
    }

    let scope_path = topic_root.join("scope.md");
    if file_missing_empty_or_todo(&scope_path) {
        tasks.push(TopicTaskItem {
            id: format!("topic-{slug}-scope-definition"),
            priority: "high".to_string(),
            category: "topic_scope".to_string(),
            target_agent: "Manager LLM / human researcher".to_string(),
            goal: "Turn the topic scope into a usable research boundary before relation extraction.".to_string(),
            requirements: vec![
                "Define the central research question in concrete terms.".to_string(),
                "Write inclusion criteria and exclusion criteria for the topic.".to_string(),
                "Keep nearby-but-outside topics separate to avoid graph drift.".to_string(),
                "Do not summarize papers here; this file defines the topic boundary only.".to_string(),
            ],
            files: vec![relative_path_string(kb_path, &scope_path)],
            evidence: vec![scope_evidence(kb_path, &scope_path)],
            source_command: format!("kb topic tasks {slug}"),
            expected_output: vec![
                format!("A revised `topics/{slug}/scope.md` with research question, inclusion criteria, and exclusion criteria."),
                "Any unresolved boundary questions listed explicitly.".to_string(),
            ],
            review_rule: "High-level topic scope should be checked by a human researcher before Worker LLM relation tasks proceed.".to_string(),
        });
    }

    let literature_path = topic_root.join("literature.md");
    let literature_text = fs::read_to_string(&literature_path).unwrap_or_default();
    let literature_candidates = parse_topic_literature(&literature_text);
    if literature_candidates.is_empty() || file_missing_empty_or_todo(&literature_path) {
        tasks.push(TopicTaskItem {
            id: format!("topic-{slug}-literature-curation"),
            priority: "high".to_string(),
            category: "topic_literature".to_string(),
            target_agent: "Manager LLM / literature curator".to_string(),
            goal: "Curate the initial paper list for this topic so ranking and relation work has a bounded input set.".to_string(),
            requirements: vec![
                "Add topic-relevant papers as rows in literature.md.".to_string(),
                "Use stable paths such as `wiki/papers/...` or `raw/papers/...` when available.".to_string(),
                "Describe each paper's role in the topic without claiming final importance.".to_string(),
                "Mark uncertain inclusions as `candidate`, not confirmed.".to_string(),
            ],
            files: vec![relative_path_string(kb_path, &literature_path)],
            evidence: vec![format!(
                "topic literature rows parsed: {}; file status: {}",
                literature_candidates.len(),
                if literature_path.exists() { "present" } else { "missing" }
            )],
            source_command: format!("kb topic rank {slug}"),
            expected_output: vec![
                format!("A usable `topics/{slug}/literature.md` table with paper, role, status, and notes."),
                format!("Then run `kb topic prepare {slug}`."),
            ],
            review_rule: "A human or Manager LLM should reject papers that do not match scope.md before running importance review.".to_string(),
        });
    }

    let importance_review_items = collect_importance_review_items(kb_path, topic_root)?;
    let generated_importance_files = collect_generated_importance_files(kb_path, topic_root)?;
    let review_queue = topic_root.join("review/review_queue.md");
    if !literature_candidates.is_empty()
        && generated_importance_files.is_empty()
        && importance_review_items.is_empty()
    {
        tasks.push(TopicTaskItem {
            id: format!("topic-{slug}-importance-generation"),
            priority: "medium".to_string(),
            category: "topic_importance".to_string(),
            target_agent: "Manager LLM / deterministic CLI operator".to_string(),
            goal: "Generate topic-local importance candidates from the curated literature table."
                .to_string(),
            requirements: vec![
                format!("Run `kb topic rank {slug}` or `kb topic prepare {slug}`."),
                "Treat generated scores as candidates only.".to_string(),
                "Do not use ranking output as final scholarly authority.".to_string(),
            ],
            files: vec![relative_path_string(kb_path, &literature_path)],
            evidence: vec![format!(
                "literature rows parsed: {}; generated importance reports found: {}",
                literature_candidates.len(),
                generated_importance_files.len()
            )],
            source_command: format!("kb topic rank {slug}"),
            expected_output: vec![format!(
                "A timestamped importance candidate report under `topics/{slug}/importance/`."
            )],
            review_rule:
                "Core/important labels require later review before affecting topic conclusions."
                    .to_string(),
        });
    }

    if !importance_review_items.is_empty() && !review_queue.exists() {
        tasks.push(TopicTaskItem {
            id: format!("topic-{slug}-importance-review-queue"),
            priority: "high".to_string(),
            category: "topic_importance_review".to_string(),
            target_agent: "Manager LLM / human topic reviewer".to_string(),
            goal: "Build a review queue for topic-local importance candidates before using them as graph semantics.".to_string(),
            requirements: vec![
                format!("Run `kb topic review {slug}` to create review/review_queue.md."),
                "Review one candidate row at a time.".to_string(),
                "Confirm only when there is topic-specific evidence.".to_string(),
                "Record accepted/rejected/deferred outcomes under reviewed/.".to_string(),
            ],
            files: generated_importance_files.clone(),
            evidence: importance_review_items
                .iter()
                .take(10)
                .map(|item| format!("{} -> {}", item.candidate_id, item.proposed_decision))
                .collect(),
            source_command: format!("kb topic review {slug}"),
            expected_output: vec![
                format!("`topics/{slug}/review/review_queue.md`."),
                format!("`topics/{slug}/review/review_summary.md`."),
            ],
            review_rule: "The queue only creates candidate review rows; it does not accept claims automatically.".to_string(),
        });
    }

    let relation_scan = scan_topic_relation_tables(kb_path, slug, topic_root)?;
    if relation_scan.real_records == 0 {
        tasks.push(TopicTaskItem {
            id: format!("topic-{slug}-relation-authoring"),
            priority: "high".to_string(),
            category: "topic_relations".to_string(),
            target_agent: "Worker LLM / human literature relation reviewer".to_string(),
            goal: "Author the first evidence-backed directed relation records for this topic.".to_string(),
            requirements: vec![
                "Use scope.md and literature.md as the topic boundary.".to_string(),
                "Inspect Introduction and References sections before proposing relations.".to_string(),
                "Write only evidence-backed directed records under relations/*.md.".to_string(),
                "Keep uncertain relations as `candidate` or `needs_human`; do not mark them confirmed.".to_string(),
                "Do not use global citation identity as a substitute for topic-local idea relation.".to_string(),
            ],
            files: relation_scan.relation_files.clone(),
            evidence: vec![format!(
                "real topic relation records parsed: {}; template/TODO rows ignored: {}",
                relation_scan.real_records, relation_scan.todo_rows
            )],
            source_command: format!("kb topic relations {slug}"),
            expected_output: vec![
                format!("Evidence-backed rows in `topics/{slug}/relations/*.md`."),
                format!("Then run `kb topic prepare {slug}` and inspect `kb view --relations --topic {slug}`."),
            ],
            review_rule: "Worker output remains candidate-level until reviewed; confirmed relations require explicit acceptance.".to_string(),
        });
    } else if relation_scan.needs_review_records > 0 || relation_scan.weak_evidence_records > 0 {
        tasks.push(TopicTaskItem {
            id: format!("topic-{slug}-relation-review"),
            priority: "high".to_string(),
            category: "topic_relation_review".to_string(),
            target_agent: "Manager LLM / human relation reviewer".to_string(),
            goal: "Review uncertain topic-local relation records and strengthen weak evidence before graph conclusions are used.".to_string(),
            requirements: vec![
                "Review candidate, ambiguous, needs_human, or weak-evidence relation rows one at a time.".to_string(),
                "Promote a relation to confirmed only when source, target, relation type, and evidence are all clear.".to_string(),
                "Move rejected/deferred decisions into reviewed/ when appropriate.".to_string(),
                "Preserve uncertainty when evidence is only suggestive.".to_string(),
            ],
            files: relation_scan.relation_files.clone(),
            evidence: relation_scan.evidence.iter().take(20).cloned().collect(),
            source_command: format!("kb topic relations {slug}"),
            expected_output: vec![
                format!("Reviewed relation decisions under `topics/{slug}/reviewed/`."),
                format!("Updated relation rows under `topics/{slug}/relations/*.md` where evidence supports the edit."),
            ],
            review_rule: "Do not let a Worker LLM be the final guarantee for high-impact `improves`, `contradicts`, or `causal_or_motivates` claims.".to_string(),
        });
    }

    let graph_json = topic_root.join("graph/topic_graph.json");
    let graph_md = topic_root.join("graph/topic_graph.md");
    if !graph_json.exists() || !graph_md.exists() {
        tasks.push(TopicTaskItem {
            id: format!("topic-{slug}-graph-export"),
            priority: "medium".to_string(),
            category: "topic_graph".to_string(),
            target_agent: "Manager LLM / deterministic CLI operator".to_string(),
            goal: "Compile the topic graph artifacts so the Manager LLM and human reviewer can inspect the relation structure visually.".to_string(),
            requirements: vec![
                format!("Run `kb topic relations {slug}` or `kb topic prepare {slug}`."),
                format!("Inspect the result with `kb view --relations --topic {slug}`."),
                "Treat dashed/candidate edges as unresolved review items.".to_string(),
            ],
            files: vec![
                relative_path_string(kb_path, &graph_json),
                relative_path_string(kb_path, &graph_md),
            ],
            evidence: vec![format!(
                "graph json exists: {}; graph markdown exists: {}",
                graph_json.exists(),
                graph_md.exists()
            )],
            source_command: format!("kb topic prepare {slug}"),
            expected_output: vec![
                format!("`topics/{slug}/graph/topic_graph.json`."),
                format!("`topics/{slug}/graph/topic_graph.md`."),
            ],
            review_rule: "Graph export is a review surface, not a source of final claims.".to_string(),
        });
    }

    if relation_scan.confirmed_records > 0 && importance_review_items.is_empty() {
        warnings.push("confirmed relations exist but no parsed importance review items were found; check whether node importance is intentionally absent".to_string());
    }

    if relation_scan.confirmed_records > 0 {
        tasks.push(TopicTaskItem {
            id: format!("topic-{slug}-synthesis-outline"),
            priority: "low".to_string(),
            category: "topic_synthesis".to_string(),
            target_agent: "Manager LLM / synthesis Worker LLM".to_string(),
            goal: "Draft a topic synthesis outline from confirmed or explicitly reviewed relation evidence.".to_string(),
            requirements: vec![
                "Use only confirmed or explicitly reviewed relations as the backbone.".to_string(),
                "Separate evidence-backed statements from hypotheses and open questions.".to_string(),
                "Point to source relation files and review decisions for every major claim.".to_string(),
                "Do not overwrite wiki concept pages without human/Git review.".to_string(),
            ],
            files: vec![
                relative_path_string(kb_path, &topic_root.join("relations")),
                relative_path_string(kb_path, &topic_root.join("reviewed")),
                relative_path_string(kb_path, &topic_root.join("graph/topic_graph.md")),
            ],
            evidence: vec![format!(
                "confirmed relation records parsed: {}",
                relation_scan.confirmed_records
            )],
            source_command: format!("kb topic tasks {slug}"),
            expected_output: vec![
                format!("A proposed synthesis outline under `topics/{slug}/tasks/` or a reviewed wiki draft proposal."),
                "A list of unresolved claims that need further evidence.".to_string(),
            ],
            review_rule: "Synthesis is allowed only as a proposal until accepted by human/Git review.".to_string(),
        });
    }

    let task_count = tasks.len();
    let returned_tasks = if args.limit > 0 {
        tasks.into_iter().take(args.limit).collect::<Vec<_>>()
    } else {
        tasks
    };

    let next_steps = if returned_tasks.is_empty() {
        vec![
            format!("No topic-specific handoff tasks detected for `{slug}`."),
            format!("Run `kb topic prepare {slug}` and `kb view --relations --topic {slug}` when topic materials change."),
        ]
    } else {
        vec![
            format!("Open `topics/{slug}/tasks/index.md` as the topic Manager LLM dashboard."),
            "Assign exactly one bounded task item to a Worker LLM or human reviewer at a time.".to_string(),
            format!("After accepted work, record topic-local completion in `topics/{slug}/memory/completed_tasks.md` or global `kb memory`."),
        ]
    };

    Ok(TopicTasksReport {
        schema_version: "topic-tasks.v0.7.11".to_string(),
        generated_by: "kb-cli topic tasks".to_string(),
        generated_at: chrono::Utc::now().to_rfc3339(),
        topic_slug: slug.to_string(),
        topic_path: relative_path_string(kb_path, topic_root),
        dry_run,
        task_count,
        returned_count: returned_tasks.len(),
        limit: args.limit,
        index_path: format!("topics/{slug}/tasks/index.md"),
        task_item_dir: format!("topics/{slug}/tasks/items"),
        task_item_count: 0,
        warnings,
        tasks: returned_tasks,
        next_steps,
    })
}

#[derive(Debug, Default)]
struct TopicRelationScan {
    relation_files: Vec<String>,
    real_records: usize,
    confirmed_records: usize,
    needs_review_records: usize,
    weak_evidence_records: usize,
    todo_rows: usize,
    evidence: Vec<String>,
}

fn file_missing_empty_or_todo(path: &Path) -> bool {
    let Ok(content) = fs::read_to_string(path) else {
        return true;
    };
    let trimmed = content.trim();
    trimmed.is_empty() || trimmed.contains("TODO") || trimmed.contains("todo")
}

fn scope_evidence(kb_path: &Path, scope_path: &Path) -> String {
    if !scope_path.exists() {
        return format!("{} is missing", relative_path_string(kb_path, scope_path));
    }
    let content = fs::read_to_string(scope_path).unwrap_or_default();
    if content.trim().is_empty() {
        format!("{} is empty", relative_path_string(kb_path, scope_path))
    } else if content.contains("TODO") || content.contains("todo") {
        format!(
            "{} still contains TODO markers",
            relative_path_string(kb_path, scope_path)
        )
    } else {
        format!(
            "{} may need human boundary review",
            relative_path_string(kb_path, scope_path)
        )
    }
}

fn collect_generated_importance_files(kb_path: &Path, topic_root: &Path) -> Result<Vec<String>> {
    let importance_dir = topic_root.join("importance");
    let mut files = Vec::new();
    if !importance_dir.exists() {
        return Ok(files);
    }
    for entry in walkdir::WalkDir::new(&importance_dir)
        .min_depth(1)
        .into_iter()
        .filter_map(|entry| entry.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if name.starts_with("importance_candidates_") && name.ends_with(".md") {
            files.push(relative_path_string(kb_path, path));
        }
    }
    files.sort();
    Ok(files)
}

fn collect_importance_review_items(
    kb_path: &Path,
    topic_root: &Path,
) -> Result<Vec<TopicReviewItem>> {
    let importance_dir = topic_root.join("importance");
    let candidate_files = discover_topic_review_candidate_files(&importance_dir)?;
    let mut items = Vec::new();
    for file in &candidate_files {
        let Ok(text) = fs::read_to_string(file) else {
            continue;
        };
        if looks_like_empty_topic_template(file, &text) {
            continue;
        }
        let relative_file = relative_path_string(kb_path, file);
        let parsed = parse_review_items_from_candidate_text(kb_path, file, &relative_file, &text);
        if parsed.is_empty() {
            continue;
        }
        items.extend(parsed);
    }
    Ok(items)
}

fn scan_topic_relation_tables(
    kb_path: &Path,
    _slug: &str,
    topic_root: &Path,
) -> Result<TopicRelationScan> {
    let relation_dir = topic_root.join("relations");
    let mut scan = TopicRelationScan::default();
    if !relation_dir.exists() {
        return Ok(scan);
    }

    let mut files = collect_markdown_files(&relation_dir);
    files.sort();
    for path in files {
        let rel_path = relative_path_string(kb_path, &path);
        scan.relation_files.push(rel_path.clone());
        let Ok(content) = fs::read_to_string(&path) else {
            continue;
        };
        let mut header: Option<Vec<String>> = None;
        for (line_idx, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if !trimmed.starts_with('|') || !trimmed.ends_with('|') {
                continue;
            }
            let cells = split_markdown_table_row(trimmed);
            if cells.is_empty()
                || cells
                    .iter()
                    .all(|cell| cell.chars().all(|ch| matches!(ch, '-' | ':' | ' ')))
            {
                continue;
            }
            if header.is_none()
                && cells.iter().any(|cell| {
                    matches!(
                        normalize_header(cell).as_str(),
                        "source" | "relation_type" | "target"
                    )
                })
            {
                header = Some(cells.iter().map(|cell| normalize_header(cell)).collect());
                continue;
            }
            let Some(header_cells) = &header else {
                continue;
            };
            let source = value_for_header(&cells, header_cells, &["source", "paper", "from"])
                .unwrap_or_default();
            let target =
                value_for_header(&cells, header_cells, &["target", "to"]).unwrap_or_default();
            let status = value_for_header(&cells, header_cells, &["status"])
                .unwrap_or_else(|| "candidate".to_string());
            let evidence = value_for_header(&cells, header_cells, &["evidence", "source_evidence"])
                .unwrap_or_default();
            let needs_human =
                value_for_header(&cells, header_cells, &["needs_human_review", "needs_human"])
                    .map(|value| parse_topic_bool(&value))
                    .unwrap_or(false);

            if !is_real_candidate_value(&source) || !is_real_candidate_value(&target) {
                scan.todo_rows += 1;
                continue;
            }

            scan.real_records += 1;
            let normalized_status = normalize_topic_relation_status(&status);
            if normalized_status == "confirmed" {
                scan.confirmed_records += 1;
            }
            if normalized_status != "confirmed" || needs_human {
                scan.needs_review_records += 1;
            }
            if !is_real_candidate_value(&evidence) || evidence.to_ascii_lowercase().contains("todo")
            {
                scan.weak_evidence_records += 1;
            }
            if scan.evidence.len() < 40 {
                scan.evidence.push(format!(
                    "{}:{} | {} -> {} | status={} | evidence={}",
                    rel_path,
                    line_idx + 1,
                    source,
                    target,
                    normalized_status,
                    if evidence.is_empty() {
                        "<missing>"
                    } else {
                        evidence.as_str()
                    }
                ));
            }
        }
    }
    Ok(scan)
}

fn write_topic_task_item_files(
    kb_path: &Path,
    slug: &str,
    report: &TopicTasksReport,
) -> Result<PathBuf> {
    let item_dir = kb_path.join("topics").join(slug).join("tasks/items");
    fs::create_dir_all(&item_dir)?;
    for task in &report.tasks {
        let path = item_dir.join(format!("{}.md", sanitize_topic_task_id(&task.id)));
        fs::write(path, render_topic_task_item(report, task))?;
    }
    Ok(item_dir)
}

fn write_topic_task_index(
    kb_path: &Path,
    slug: &str,
    report: &TopicTasksReport,
) -> Result<PathBuf> {
    let tasks_dir = kb_path.join("topics").join(slug).join("tasks");
    fs::create_dir_all(&tasks_dir)?;
    let path = tasks_dir.join("index.md");
    fs::write(&path, render_topic_task_index(report))?;
    Ok(path)
}

fn render_topic_task_index(report: &TopicTasksReport) -> String {
    let mut out = String::new();
    out.push_str(&format!("# Topic Task Index: {}\n\n", report.topic_slug));
    out.push_str("This is the topic-local Manager LLM dashboard. It routes bounded Worker LLM / human tasks for one topic only.\n\n");
    out.push_str("## Metadata\n\n");
    out.push_str(&format!("- Generated by: `{}`\n", report.generated_by));
    out.push_str(&format!("- Generated at: `{}`\n", report.generated_at));
    out.push_str(&format!("- Schema version: `{}`\n", report.schema_version));
    out.push_str(&format!("- Topic: `{}`\n", report.topic_slug));
    out.push_str(&format!("- Topic path: `{}`\n", report.topic_path));
    out.push_str("- Intended reader: `Manager LLM / human maintainer`\n");
    out.push_str("- Worker task directory: `tasks/items/`\n\n");

    out.push_str("## Operating Rules\n\n");
    out.push_str("1. Stay inside this topic unless a human expands the scope.\n");
    out.push_str("2. Assign one task item at a time.\n");
    out.push_str("3. Worker LLMs must cite the listed files and preserve uncertainty.\n");
    out.push_str("4. Candidate relations remain candidate-level until reviewed.\n");
    out.push_str("5. Do not modify `raw/` source materials.\n\n");

    out.push_str("## Pending Topic Tasks\n\n");
    if report.tasks.is_empty() {
        out.push_str("No pending topic-specific tasks were detected.\n\n");
    } else {
        out.push_str(
            "| priority | task_id | category | target_agent | source_command | task_file |\n",
        );
        out.push_str("|---|---|---|---|---|---|\n");
        for task in &report.tasks {
            let task_file = format!(
                "topics/{}/tasks/items/{}.md",
                report.topic_slug,
                sanitize_topic_task_id(&task.id)
            );
            out.push_str(&format!(
                "| `{}` | `{}` | `{}` | {} | `{}` | `{}` |\n",
                escape_table_cell(&task.priority),
                escape_table_cell(&task.id),
                escape_table_cell(&task.category),
                escape_table_cell(&task.target_agent),
                escape_table_cell(&task.source_command),
                escape_table_cell(&task_file)
            ));
        }
        out.push('\n');
    }

    if !report.warnings.is_empty() {
        out.push_str("## Warnings\n\n");
        for warning in &report.warnings {
            out.push_str(&format!("- {}\n", warning));
        }
        out.push('\n');
    }

    out.push_str("## Recommended Topic Loop\n\n");
    out.push_str("```text\n");
    out.push_str("scope.md + literature.md\n");
    out.push_str("        ↓\n");
    out.push_str("kb topic prepare <topic>\n");
    out.push_str("        ↓\n");
    out.push_str("kb topic tasks <topic>\n");
    out.push_str("        ↓\n");
    out.push_str("assign one bounded task\n");
    out.push_str("        ↓\n");
    out.push_str("review changed files with Git diff\n");
    out.push_str("        ↓\n");
    out.push_str("record accepted work in memory/\n");
    out.push_str("```\n\n");

    out.push_str("## Next Steps\n\n");
    for step in &report.next_steps {
        out.push_str(&format!("- {step}\n"));
    }

    out
}

fn render_topic_task_item(report: &TopicTasksReport, task: &TopicTaskItem) -> String {
    let mut out = String::new();
    out.push_str(&format!("# Topic Worker Task: {}\n\n", task.id));
    out.push_str(
        "This is a bounded topic-local Worker LLM / human task generated by `kb topic tasks`.\n\n",
    );
    out.push_str("## Task Metadata\n\n");
    out.push_str(&format!("- Topic: `{}`\n", report.topic_slug));
    out.push_str(&format!("- Task ID: `{}`\n", task.id));
    out.push_str("- Status: `pending`\n");
    out.push_str(&format!("- Priority: `{}`\n", task.priority));
    out.push_str(&format!("- Category: `{}`\n", task.category));
    out.push_str(&format!("- Target agent: `{}`\n", task.target_agent));
    out.push_str(&format!("- Source command: `{}`\n\n", task.source_command));

    out.push_str("## Goal\n\n");
    out.push_str(&task.goal);
    out.push_str("\n\n");

    out.push_str("## Requirements\n\n");
    for requirement in &task.requirements {
        out.push_str(&format!("- {}\n", requirement));
    }
    out.push('\n');

    out.push_str("## Input Files\n\n");
    if task.files.is_empty() {
        out.push_str("No specific files were detected. Inspect the topic workspace manually before editing.\n");
    } else {
        for file in &task.files {
            out.push_str(&format!("- `{}`\n", file));
        }
    }
    out.push('\n');

    out.push_str("## Evidence\n\n");
    if task.evidence.is_empty() {
        out.push_str(
            "No evidence lines were collected. Treat this task as a planning hint, not as proof.\n",
        );
    } else {
        for item in &task.evidence {
            out.push_str(&format!("- {}\n", item));
        }
    }
    out.push('\n');

    out.push_str("## Expected Output\n\n");
    for item in &task.expected_output {
        out.push_str(&format!("- {}\n", item));
    }
    out.push('\n');

    out.push_str("## Review Rule\n\n");
    out.push_str(&task.review_rule);
    out.push_str("\n\n");

    out.push_str("## Worker Output Rules\n\n");
    out.push_str("- Work only on the files listed above unless the Manager LLM or human maintainer expands the scope.\n");
    out.push_str("- Preserve uncertainty; do not turn candidate evidence into final knowledge without review.\n");
    out.push_str("- List every changed or created file.\n");
    out.push_str("- Return unresolved issues and follow-up recommendations.\n");
    out.push_str("- Do not modify `raw/` source materials.\n");
    out.push_str("- Do not commit changes directly; leave the result for Git diff review.\n\n");

    out.push_str("## Completion\n\n");
    out.push_str("After the work is reviewed and accepted, record the result in topic memory or global memory.\n");
    out
}

fn print_topic_tasks_report(report: &TopicTasksReport) {
    println!("Topic tasks:");
    println!("  topic       : {}", report.topic_slug);
    println!("  path        : {}", report.topic_path);
    println!(
        "  tasks       : {} total, {} returned",
        report.task_count, report.returned_count
    );
    println!("  dry run     : {}", report.dry_run);
    println!(
        "  index       : {}",
        if report.dry_run {
            "<not written>"
        } else {
            report.index_path.as_str()
        }
    );
    println!(
        "  task items  : {}",
        if report.dry_run {
            "<not written>"
        } else {
            report.task_item_dir.as_str()
        }
    );

    if report.tasks.is_empty() {
        println!();
        println!("No pending topic-specific tasks detected.");
    } else {
        println!();
        println!("Pending topic tasks:");
        for task in &report.tasks {
            println!(
                "- {} | {} | {} | {}",
                task.priority, task.id, task.category, task.target_agent
            );
            println!("  goal: {}", task.goal);
        }
    }

    if !report.warnings.is_empty() {
        println!();
        println!("Warnings:");
        for warning in &report.warnings {
            println!("  - {warning}");
        }
    }

    if !report.next_steps.is_empty() {
        println!();
        println!("Next steps:");
        for step in &report.next_steps {
            println!("  - {step}");
        }
    }
}

fn sanitize_topic_task_id(value: &str) -> String {
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

fn is_topic_table_header_or_rule(cells: &[String]) -> bool {
    if cells.is_empty() {
        return true;
    }
    if cells
        .iter()
        .all(|cell| cell.chars().all(|ch| matches!(ch, '-' | ':' | ' ')))
    {
        return true;
    }
    let first = normalize_header(&cells[0]);
    matches!(
        first.as_str(),
        "paper" | "source" | "item" | "todo" | "_none_" | "source_item"
    )
}

fn normalize_topic_relation_status(raw: &str) -> String {
    let lower = raw.trim().to_ascii_lowercase();
    if lower.contains("confirm") || lower.contains("accept") || lower == "done" {
        "confirmed".to_string()
    } else if lower.contains("ambiguous") || lower.contains("uncertain") {
        "ambiguous".to_string()
    } else if lower.contains("missing") || lower.contains("unresolved") {
        "missing".to_string()
    } else if lower.contains("reject") {
        "rejected".to_string()
    } else {
        "candidate".to_string()
    }
}

fn parse_topic_bool(raw: &str) -> bool {
    let lower = raw.trim().to_ascii_lowercase();
    matches!(
        lower.as_str(),
        "true" | "yes" | "y" | "1" | "needs_human" | "needs_human_review"
    )
}

fn topic_graph_node_id(kind: &str, value: &str) -> String {
    let slug = topic_slug(value);
    if slug.is_empty() {
        format!("{kind}:unknown")
    } else {
        format!("{kind}:{slug}")
    }
}

fn topic_graph_label(value: &str) -> String {
    Path::new(value)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .filter(|stem| !stem.is_empty())
        .unwrap_or(value)
        .to_string()
}

fn execute_review(custom_kb: Option<&Path>, args: &TopicReviewArgs) -> Result<()> {
    let kb_path = crate::commands::init::get_kb_path(custom_kb);
    let slug = topic_slug(&args.topic);
    if slug.is_empty() {
        return Err(anyhow!(
            "topic name must contain at least one letter or digit"
        ));
    }

    let topic_root = kb_path.join("topics").join(&slug);
    if !topic_root.exists() {
        return Err(anyhow!(
            "topic workspace not found: {}. Hint: run `kb topic init {}` first.",
            relative_path_string(&kb_path, &topic_root),
            slug
        ));
    }

    let importance_dir = topic_root.join("importance");
    if !importance_dir.is_dir() {
        return Err(anyhow!(
            "importance candidate directory not found: {}. Hint: run `kb topic rank {}` first.",
            relative_path_string(&kb_path, &importance_dir),
            slug
        ));
    }

    let mut report = build_topic_review_report(&kb_path, &slug, &topic_root, args)?;
    let dry_run = args.dry_run || args.preview;

    if !dry_run {
        let queue_path = topic_root.join("review").join("review_queue.md");
        let summary_path = topic_root.join("review").join("review_summary.md");
        if !args.force {
            if queue_path.exists() {
                return Err(anyhow!(
                    "review_queue.md already exists. Use --force to overwrite: {}",
                    relative_path_string(&kb_path, &queue_path)
                ));
            }
            if summary_path.exists() {
                return Err(anyhow!(
                    "review_summary.md already exists. Use --force to overwrite: {}",
                    relative_path_string(&kb_path, &summary_path)
                ));
            }
        }

        fs::create_dir_all(topic_root.join("review"))?;
        report.written = true;
        fs::write(&queue_path, render_review_queue_markdown(&report))?;
        fs::write(&summary_path, render_review_summary_markdown(&report))?;
    }

    if args.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print_review_report(&report);
    }
    Ok(())
}

fn build_topic_review_report(
    kb_path: &Path,
    slug: &str,
    topic_root: &Path,
    args: &TopicReviewArgs,
) -> Result<TopicReviewReport> {
    let dry_run = args.dry_run || args.preview;
    let importance_dir = topic_root.join("importance");
    let review_dir = topic_root.join("review");
    let queue_path = review_dir.join("review_queue.md");
    let summary_path = review_dir.join("review_summary.md");
    let candidate_files = discover_topic_review_candidate_files(&importance_dir)?;

    let mut warnings = Vec::new();
    let mut empty_or_unreadable_candidates = 0usize;
    let mut items = Vec::new();

    for file in &candidate_files {
        let relative_file = relative_path_string(kb_path, file);
        let text = match fs::read_to_string(file) {
            Ok(text) => text,
            Err(err) => {
                empty_or_unreadable_candidates += 1;
                warnings.push(format!("could not read {relative_file}: {err}"));
                continue;
            }
        };

        if text.trim().is_empty() {
            empty_or_unreadable_candidates += 1;
            warnings.push(format!("empty candidate file skipped: {relative_file}"));
            continue;
        }

        let parsed = parse_review_items_from_candidate_text(kb_path, file, &relative_file, &text);
        if parsed.is_empty() {
            if looks_like_empty_topic_template(file, &text) {
                warnings.push(format!(
                    "template-like candidate file has no reviewable rows: {relative_file}"
                ));
                continue;
            }
            items.push(fallback_review_item(
                kb_path,
                file,
                &relative_file,
                &text,
                items.len() + 1,
            ));
        } else {
            items.extend(parsed);
        }
    }

    for (index, item) in items.iter_mut().enumerate() {
        item.review_id = format!("tr-{index:04}", index = index + 1);
    }

    if candidate_files.is_empty() {
        warnings.push(format!(
            "no topic-local importance candidate files found under topics/{slug}/importance/"
        ));
    }
    if items.is_empty() {
        warnings.push(
            "no review items generated; run `kb topic rank <topic>` after editing literature.md"
                .to_string(),
        );
    }
    if dry_run {
        if queue_path.exists() && !args.force {
            warnings.push(format!(
                "existing queue would not be overwritten without --force: {}",
                relative_path_string(kb_path, &queue_path)
            ));
        }
        if summary_path.exists() && !args.force {
            warnings.push(format!(
                "existing summary would not be overwritten without --force: {}",
                relative_path_string(kb_path, &summary_path)
            ));
        }
    }

    let next_steps = if items.is_empty() {
        vec![
            format!("Add papers to topics/{slug}/literature.md."),
            format!("Run `kb topic rank {slug}` to generate topic-local importance candidates."),
            format!("Run `kb topic review {slug}` again to build a review queue."),
        ]
    } else {
        vec![
            format!("Review one item at a time in topics/{slug}/review/review_queue.md."),
            "Do not accept a candidate without evidence.".to_string(),
            format!(
                "Record accepted, rejected, or deferred decisions under topics/{slug}/reviewed/."
            ),
        ]
    };

    Ok(TopicReviewReport {
        schema_version: "topic-review.v0.7.1".to_string(),
        generated_by: "kb-cli topic review".to_string(),
        generated_at: chrono::Utc::now().to_rfc3339(),
        topic_slug: slug.to_string(),
        topic_path: relative_path_string(kb_path, topic_root),
        candidate_dir: relative_path_string(kb_path, &importance_dir),
        review_dir: relative_path_string(kb_path, &review_dir),
        queue_path: relative_path_string(kb_path, &queue_path),
        summary_path: relative_path_string(kb_path, &summary_path),
        dry_run,
        force: args.force,
        written: false,
        candidate_files: candidate_files.len(),
        review_items: items.len(),
        empty_or_unreadable_candidates,
        warnings,
        items,
        next_steps,
    })
}

fn discover_topic_review_candidate_files(importance_dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    if !importance_dir.exists() {
        return Ok(files);
    }
    for entry in walkdir::WalkDir::new(importance_dir)
        .min_depth(1)
        .into_iter()
        .filter_map(|entry| entry.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if !is_review_candidate_extension(path) || is_non_candidate_importance_file(path) {
            continue;
        }
        files.push(path.to_path_buf());
    }
    files.sort_by(|a, b| a.to_string_lossy().cmp(&b.to_string_lossy()));
    Ok(files)
}

fn is_review_candidate_extension(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|ext| ext.to_str()).map(|ext| ext.to_ascii_lowercase()),
        Some(ext) if matches!(ext.as_str(), "md" | "markdown" | "json" | "toml" | "yaml" | "yml")
    )
}

fn is_non_candidate_importance_file(path: &Path) -> bool {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    name == "confirmed_importance.md" || name == "importance_review.md"
}

fn parse_review_items_from_candidate_text(
    kb_path: &Path,
    path: &Path,
    _relative_file: &str,
    text: &str,
) -> Vec<TopicReviewItem> {
    let mut items = Vec::new();
    let mut header: Option<Vec<String>> = None;
    let mut row_index = 0usize;

    for line in text.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with('|') || !trimmed.ends_with('|') {
            continue;
        }
        let cells = split_markdown_table_row(trimmed);
        if cells.is_empty() {
            continue;
        }
        if cells
            .iter()
            .all(|cell| cell.chars().all(|ch| matches!(ch, '-' | ':' | ' ')))
        {
            continue;
        }
        if header.is_none() && cells.iter().any(|cell| normalize_header(cell) == "paper") {
            header = Some(cells.iter().map(|cell| normalize_header(cell)).collect());
            continue;
        }
        let Some(header_cells) = &header else {
            continue;
        };
        let paper = value_for_header(&cells, header_cells, &["paper", "source_item", "item"])
            .unwrap_or_default();
        if !is_real_candidate_value(&paper) {
            continue;
        }

        row_index += 1;
        let proposed_decision = value_for_header(
            &cells,
            header_cells,
            &[
                "proposed_importance_level",
                "importance_level",
                "proposed_decision",
                "importance",
                "level",
            ],
        )
        .filter(|value| is_real_candidate_value(value))
        .unwrap_or_else(|| extract_proposed_decision(text));
        let evidence = value_for_header(&cells, header_cells, &["evidence", "source", "sources"])
            .filter(|value| is_real_candidate_value(value))
            .unwrap_or_else(|| extract_evidence(text));
        let source_item = format!("{}#row-{}", relative_path_string(kb_path, path), row_index);
        items.push(TopicReviewItem {
            review_id: String::new(),
            candidate_id: candidate_id_from_value(&paper, row_index),
            source_item,
            proposed_decision,
            status: "candidate".to_string(),
            reviewer: "unassigned".to_string(),
            evidence,
        });
    }

    items
}

fn fallback_review_item(
    _kb_path: &Path,
    path: &Path,
    relative_file: &str,
    text: &str,
    index: usize,
) -> TopicReviewItem {
    TopicReviewItem {
        review_id: String::new(),
        candidate_id: extract_candidate_id(text, path, index),
        source_item: relative_file.to_string(),
        proposed_decision: extract_proposed_decision(text),
        status: "candidate".to_string(),
        reviewer: "unassigned".to_string(),
        evidence: extract_evidence(text),
    }
}

fn split_markdown_table_row(line: &str) -> Vec<String> {
    line.trim_matches('|')
        .split('|')
        .map(|cell| cell.trim().replace("\\|", "|"))
        .collect()
}

fn normalize_header(input: &str) -> String {
    input
        .trim()
        .trim_matches('`')
        .to_ascii_lowercase()
        .replace(' ', "_")
        .replace('-', "_")
}

fn value_for_header(cells: &[String], header: &[String], names: &[&str]) -> Option<String> {
    header.iter().enumerate().find_map(|(index, name)| {
        if names.iter().any(|candidate| name == candidate) {
            cells.get(index).map(|value| value.trim().to_string())
        } else {
            None
        }
    })
}

fn is_real_candidate_value(value: &str) -> bool {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return false;
    }
    !matches!(
        trimmed.to_ascii_lowercase().as_str(),
        "todo" | "_none_" | "none" | "unknown" | "paper" | "item"
    )
}

fn extract_field(text: &str, keys: &[&str]) -> Option<String> {
    for line in text.lines() {
        let trimmed = line.trim().trim_start_matches('-').trim();
        let Some((key, value)) = trimmed.split_once(':') else {
            continue;
        };
        let normalized_key = normalize_header(key);
        if keys.iter().any(|candidate| normalized_key == *candidate) {
            let cleaned = value
                .trim()
                .trim_matches('`')
                .trim_matches('"')
                .trim_matches('\'')
                .trim()
                .to_string();
            if !cleaned.is_empty() {
                return Some(cleaned);
            }
        }
    }
    None
}

fn extract_candidate_id(text: &str, path: &Path, index: usize) -> String {
    extract_field(text, &["candidate_id", "id", "review_candidate_id"])
        .filter(|value| is_real_candidate_value(value))
        .unwrap_or_else(|| {
            let stem = path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .map(topic_slug)
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| format!("candidate-{index:04}"));
            format!("importance-{stem}")
        })
}

fn candidate_id_from_value(value: &str, index: usize) -> String {
    let slug = topic_slug(value);
    if slug.is_empty() {
        format!("candidate-{index:04}")
    } else {
        format!("importance-{slug}")
    }
}

fn extract_proposed_decision(text: &str) -> String {
    if let Some(value) = extract_field(
        text,
        &[
            "proposed_decision",
            "proposed_importance_level",
            "importance_level",
            "importance",
        ],
    ) {
        return value;
    }
    let lower = text.to_ascii_lowercase();
    for label in [
        "core",
        "supporting",
        "important",
        "background",
        "peripheral",
        "unknown",
    ] {
        if lower.contains(label) {
            return label.to_string();
        }
    }
    "unknown".to_string()
}

fn extract_evidence(text: &str) -> String {
    if let Some(value) = extract_field(text, &["evidence", "source", "sources"]) {
        return value;
    }
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.contains("processing/")
            || trimmed.contains("wiki/")
            || trimmed.contains("raw/")
            || trimmed.contains("topics/")
        {
            return trimmed.trim_matches('|').trim().to_string();
        }
    }
    "see candidate file".to_string()
}

fn looks_like_empty_topic_template(path: &Path, text: &str) -> bool {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    name == "importance_candidates.md" && text.to_ascii_lowercase().contains("| todo |")
}

fn render_review_queue_markdown(report: &TopicReviewReport) -> String {
    let mut md = String::new();
    md.push_str(&format!("# Topic Review Queue: {}\n\n", report.topic_slug));
    md.push_str(&format!("Generated by: `{}`\n\n", report.generated_by));
    md.push_str(&format!("Generated at: `{}`\n\n", report.generated_at));
    md.push_str("Candidate source directory:\n\n");
    md.push_str("```text\n");
    md.push_str(&report.candidate_dir);
    md.push_str("/\n```\n\n");
    md.push_str("## Review Items\n\n");
    md.push_str("This table is an editable review workbench. Keep the source columns unchanged, then fill the decision columns during human / Manager LLM / Worker LLM review.\n\n");
    md.push_str("| review_id | candidate_id | source_item | proposed_decision | final_decision | status | reviewer | confidence | evidence | uncertainty | review_notes | accepted_record_path |\n");
    md.push_str("|---|---|---|---|---|---|---|---|---|---|---|---|\n");
    if report.items.is_empty() {
        md.push_str("| _none_ | unknown | _none_ | unknown | pending | candidate | unassigned | low | run `kb topic rank <topic>` after editing literature.md | no candidate rows generated | TODO | topics/<topic>/reviewed/ |\n");
    } else {
        for item in &report.items {
            md.push_str(&format!(
                "| {} | {} | {} | {} | pending | {} | {} | medium | {} | TODO | TODO | topics/{}/reviewed/ |\n",
                escape_table_cell(&item.review_id),
                escape_table_cell(&item.candidate_id),
                escape_table_cell(&item.source_item),
                escape_table_cell(&item.proposed_decision),
                escape_table_cell(&item.status),
                escape_table_cell(&item.reviewer),
                escape_table_cell(&item.evidence),
                escape_table_cell(&report.topic_slug)
            ));
        }
    }
    md.push_str("\n## Decision vocabulary\n\n");
    md.push_str(
        "- `final_decision`: `accepted`, `rejected`, `deferred`, or `needs_more_evidence`.\n",
    );
    md.push_str("- `confidence`: `high`, `medium`, or `low`.\n");
    md.push_str(
        "- `accepted_record_path`: where the final reviewed record should be copied if accepted.\n",
    );
    md.push_str("\n## Review Rules\n\n");
    md.push_str("- Treat all generated rows as candidates.\n");
    md.push_str("- Do not accept a candidate without evidence.\n");
    md.push_str("- Preserve uncertainty instead of forcing a decision.\n");
    md.push_str(
        "- Store accepted, rejected, or deferred decisions under `topics/<topic>/reviewed/`.\n",
    );
    md.push_str("- This queue does not perform scientific judgment and does not call an LLM.\n");
    md
}

fn render_review_summary_markdown(report: &TopicReviewReport) -> String {
    let mut md = String::new();
    md.push_str(&format!(
        "# Topic Review Summary: {}\n\n",
        report.topic_slug
    ));
    md.push_str(&format!("- Topic: `{}`\n", report.topic_slug));
    md.push_str(&format!(
        "- Candidate directory: `{}/`\n",
        report.candidate_dir
    ));
    md.push_str(&format!("- Review directory: `{}/`\n", report.review_dir));
    md.push_str(&format!(
        "- Candidate files found: {}\n",
        report.candidate_files
    ));
    md.push_str(&format!(
        "- Review items generated: {}\n",
        report.review_items
    ));
    md.push_str(&format!(
        "- Empty or unreadable candidates: {}\n",
        report.empty_or_unreadable_candidates
    ));
    md.push_str(&format!("- Generated queue: `{}`\n", report.queue_path));
    md.push_str(&format!("- Generated summary: `{}`\n", report.summary_path));
    md.push_str(&format!("- Dry run: `{}`\n", report.dry_run));
    md.push_str(&format!("- Written: `{}`\n", report.written));

    if !report.warnings.is_empty() {
        md.push_str("\n## Warnings\n\n");
        for warning in &report.warnings {
            md.push_str(&format!("- {}\n", warning));
        }
    }

    md.push_str("\n## Next Step\n\n");
    for step in &report.next_steps {
        md.push_str(&format!("- {step}\n"));
    }
    md
}

fn print_review_report(report: &TopicReviewReport) {
    println!("Topic review queue:");
    println!("  topic       : {}", report.topic_slug);
    println!("  candidates  : {} files", report.candidate_files);
    println!("  review items: {}", report.review_items);
    println!("  dry run     : {}", report.dry_run);
    println!("  written     : {}", report.written);
    println!("  queue       : {}", report.queue_path);
    println!("  summary     : {}", report.summary_path);

    if report.items.is_empty() {
        println!();
        println!("No topic-local importance candidates found.");
    } else {
        println!();
        println!("Review items:");
        for item in &report.items {
            println!(
                "- {} | {} | {} | {}",
                item.review_id, item.candidate_id, item.proposed_decision, item.source_item
            );
        }
    }

    if !report.warnings.is_empty() {
        println!();
        println!("Warnings:");
        for warning in &report.warnings {
            println!("  - {warning}");
        }
    }

    if !report.next_steps.is_empty() {
        println!();
        println!("Next steps:");
        for step in &report.next_steps {
            println!("  - {step}");
        }
    }
}

fn collect_topic_text(topic_root: &Path) -> Result<String> {
    let mut text = String::new();
    if !topic_root.exists() {
        return Ok(text);
    }
    for entry in walkdir::WalkDir::new(topic_root)
        .into_iter()
        .filter_map(|entry| entry.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|ext| ext.to_str()) != Some("md") {
            continue;
        }
        if let Ok(content) = fs::read_to_string(entry.path()) {
            text.push_str(&content);
            text.push('\n');
        }
    }
    Ok(text)
}

fn collect_refs_index_text(refs_dir: &Path) -> Result<String> {
    let mut text = String::new();
    if !refs_dir.exists() {
        return Ok(text);
    }
    for entry in walkdir::WalkDir::new(refs_dir)
        .into_iter()
        .filter_map(|entry| entry.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let is_refs_index = path
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.starts_with("refs_index_"))
            .unwrap_or(false);
        if !is_refs_index {
            continue;
        }
        if let Ok(content) = fs::read_to_string(path) {
            text.push_str(&content);
            text.push('\n');
        }
    }
    Ok(text)
}

fn contains_any(input: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| input.contains(needle))
}

fn normalize_for_search(input: &str) -> String {
    input
        .chars()
        .map(|ch| {
            if ch.is_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn count_occurrences(haystack: &str, needle: &str) -> usize {
    if needle.is_empty() {
        return 0;
    }
    haystack.matches(needle).count()
}

fn importance_level(score: i32) -> &'static str {
    if score >= 70 {
        "core"
    } else if score >= 45 {
        "important"
    } else if score >= 20 {
        "background"
    } else {
        "peripheral"
    }
}

fn escape_table_cell(input: &str) -> String {
    input.replace('|', "\\|").replace('\n', " ")
}

fn collect_markdown_files(root: &Path) -> Vec<PathBuf> {
    if !root.exists() {
        return Vec::new();
    }
    walkdir::WalkDir::new(root)
        .min_depth(1)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .map(|entry| entry.into_path())
        .filter(|path| {
            matches!(
                path.extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext.to_ascii_lowercase()),
                Some(ext) if matches!(ext.as_str(), "md" | "markdown")
            )
        })
        .collect()
}

fn collect_topics(kb_path: &Path, topics_dir: &Path) -> Result<Vec<TopicListItem>> {
    if !topics_dir.exists() {
        return Ok(Vec::new());
    }
    let mut items = Vec::new();
    for entry in fs::read_dir(topics_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let path = entry.path();
        let slug = entry.file_name().to_string_lossy().to_string();
        let title = read_topic_title(&path).unwrap_or_else(|| slug.clone());
        items.push(TopicListItem {
            slug,
            title,
            path: relative_path_string(kb_path, &path),
            has_scope: path.join("scope.md").exists(),
            has_literature: path.join("literature.md").exists(),
        });
    }
    items.sort_by(|a, b| a.slug.cmp(&b.slug));
    Ok(items)
}

fn build_topic_status(kb_path: &Path, slug: &str, topic_root: &Path) -> TopicStatusReport {
    let mut missing_dirs = Vec::new();
    let mut missing_files = Vec::new();

    if !topic_root.exists() {
        missing_dirs.push(relative_path_string(kb_path, topic_root));
    }

    for dir in required_topic_dirs() {
        let path = topic_root.join(dir);
        if !path.is_dir() {
            missing_dirs.push(format!("topics/{slug}/{dir}"));
        }
    }

    for file in required_topic_files() {
        let path = topic_root.join(file);
        if !path.is_file() {
            missing_files.push(format!("topics/{slug}/{file}"));
        }
    }

    let required_dirs_missing = missing_dirs.len();
    let required_files_missing = missing_files.len();
    let required_dirs_total = required_topic_dirs().len() + 1;
    let required_files_total = required_topic_files().len();
    let status = if required_dirs_missing == 0 && required_files_missing == 0 {
        "ok".to_string()
    } else {
        "incomplete".to_string()
    };

    let mut next_steps = Vec::new();
    if status != "ok" {
        next_steps.push(
            "Run `kb topic init <topic>` to create missing topic workspace files.".to_string(),
        );
        next_steps.push("Use `kb topic init <topic> --force` only when you intentionally want to refresh template files.".to_string());
    } else {
        next_steps.push(
            "Edit scope.md and literature.md before assigning topic relation tasks.".to_string(),
        );
        next_steps.push(
            "Keep topic-local importance and explanatory relations under this topic directory."
                .to_string(),
        );
    }

    TopicStatusReport {
        topic_slug: slug.to_string(),
        topic_path: relative_path_string(kb_path, topic_root),
        status,
        required_dirs_present: required_dirs_total.saturating_sub(required_dirs_missing),
        required_dirs_missing,
        required_files_present: required_files_total.saturating_sub(required_files_missing),
        required_files_missing,
        missing_dirs,
        missing_files,
        next_steps,
    }
}

fn print_status_report(report: &TopicStatusReport) {
    println!("Topic status:");
    println!("  slug : {}", report.topic_slug);
    println!("  path : {}", report.topic_path);
    println!("  state: {}", report.status);
    println!(
        "  dirs : {} present, {} missing",
        report.required_dirs_present, report.required_dirs_missing
    );
    println!(
        "  files: {} present, {} missing",
        report.required_files_present, report.required_files_missing
    );

    if !report.missing_dirs.is_empty() {
        println!();
        println!("Missing directories:");
        for dir in &report.missing_dirs {
            println!("  - {dir}");
        }
    }

    if !report.missing_files.is_empty() {
        println!();
        println!("Missing files:");
        for file in &report.missing_files {
            println!("  - {file}");
        }
    }

    println!();
    println!("Next steps:");
    for step in &report.next_steps {
        println!("  - {step}");
    }
}

fn required_topic_dirs() -> Vec<&'static str> {
    vec![
        "importance",
        "relations",
        "review",
        "reviewed",
        "graph",
        "tasks",
        "memory",
    ]
}

fn required_topic_files() -> Vec<&'static str> {
    vec![
        "README.md",
        "scope.md",
        "literature.md",
        "importance/importance_candidates.md",
        "importance/importance_review.md",
        "importance/confirmed_importance.md",
        "relations/causal_relations.md",
        "relations/method_relations.md",
        "relations/evidence_relations.md",
        "relations/idea_relations.md",
        "review/relation_review.md",
        "review/importance_review.md",
        "review/unresolved.md",
        "reviewed/README.md",
        "graph/README.md",
        "tasks/README.md",
        "memory/confirmed_relations.md",
        "memory/completed_tasks.md",
    ]
}

fn read_topic_title(topic_root: &Path) -> Option<String> {
    let readme = fs::read_to_string(topic_root.join("README.md")).ok()?;
    readme
        .lines()
        .find_map(|line| {
            line.strip_prefix("# ")
                .map(|title| title.trim().to_string())
        })
        .filter(|title| !title.is_empty())
}

fn relative_path_string(kb_path: &Path, path: &Path) -> String {
    path.strip_prefix(kb_path)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
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
        "reviewed",
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
            path: PathBuf::from("reviewed/README.md"),
            content: render_reviewed_readme(slug, title),
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
- `review/` stores review queues and working review tables for uncertain topic-local claims.
- `reviewed/` stores accepted, rejected, or deferred topic-review decisions.
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

fn render_reviewed_readme(slug: &str, title: &str) -> String {
    format!(
        r#"# Reviewed Topic Decisions: {title}

Topic slug: `{slug}`

Store accepted, rejected, or deferred topic-review decisions here after human or explicit Manager LLM review.

Recommended files for later versions:

```text
accepted.md
rejected.md
deferred.md
```

Do not write automatic decisions here from `kb topic review`. That command only builds a queue under `review/`.
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
    use anyhow::Result;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_test_kb(name: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "kb_cli_topic_{}_{}_{}",
            name,
            std::process::id(),
            nanos
        ))
    }

    #[test]
    fn topic_slug_is_stable() {
        assert_eq!(topic_slug("Thermal Metamaterials"), "thermal-metamaterials");
        assert_eq!(
            topic_slug(" microfluidic/dialysis "),
            "microfluidic-dialysis"
        );
        assert_eq!(topic_slug("!!!"), "");
    }

    #[test]
    fn topic_review_parses_rank_table_rows() {
        let text = r#"# Topic Importance Candidates: thermal

| paper | proposed_importance_level | score | status | needs_human_review | reasons | evidence |
|---|---:|---:|---|---|---|---|
| Classic Thermal Cloak | core | 80 | candidate | true | listed in topic literature.md | processing/refs/refs_index.md |
| Background Heat Paper | background | 25 | candidate | false | listed in topic literature.md | wiki/papers/background.md |
"#;
        let path = PathBuf::from("topics/thermal/importance/importance_candidates_20260511.md");
        let items = parse_review_items_from_candidate_text(
            Path::new("."),
            &path,
            "topics/thermal/importance/importance_candidates_20260511.md",
            text,
        );
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].candidate_id, "importance-classic-thermal-cloak");
        assert_eq!(items[0].proposed_decision, "core");
        assert_eq!(items[0].status, "candidate");
        assert_eq!(items[0].reviewer, "unassigned");
        assert!(items[0].source_item.ends_with("#row-1"));
        assert!(items[1].evidence.contains("wiki/papers/background.md"));
    }

    #[test]
    fn topic_review_discovers_candidate_files_conservatively() -> Result<()> {
        let kb_path = unique_test_kb("review_discovery");
        let importance_dir = kb_path.join("topics/thermal/importance");
        fs::create_dir_all(&importance_dir)?;
        fs::write(
            importance_dir.join("importance_candidates_1.md"),
            "candidate_id: one",
        )?;
        fs::write(
            importance_dir.join("importance_review.md"),
            "not a candidate",
        )?;
        fs::write(
            importance_dir.join("confirmed_importance.md"),
            "not a candidate",
        )?;
        fs::write(
            importance_dir.join("candidate.json"),
            r#"{"candidate_id":"two"}"#,
        )?;
        fs::write(importance_dir.join("ignore.txt"), "not accepted")?;

        let files = discover_topic_review_candidate_files(&importance_dir)?;
        let names = files
            .iter()
            .filter_map(|path| path.file_name().and_then(|name| name.to_str()))
            .collect::<Vec<_>>();
        assert_eq!(names, vec!["candidate.json", "importance_candidates_1.md"]);

        let _ = fs::remove_dir_all(&kb_path);
        Ok(())
    }

    #[test]
    fn topic_review_report_builds_stable_ids() -> Result<()> {
        let kb_path = unique_test_kb("review_report");
        let topic_root = kb_path.join("topics/thermal");
        fs::create_dir_all(topic_root.join("importance"))?;
        fs::write(
            topic_root.join("importance/importance_candidates_1.md"),
            "| paper | proposed_importance_level | evidence |
|---|---|---|
| Paper A | core | raw/papers/a.pdf |
| Paper B | background | raw/papers/b.pdf |
",
        )?;
        let args = TopicReviewArgs {
            topic: "thermal".to_string(),
            dry_run: true,
            preview: false,
            json: false,
            force: false,
        };
        let report = build_topic_review_report(&kb_path, "thermal", &topic_root, &args)?;
        assert_eq!(report.review_items, 2);
        assert_eq!(report.items[0].review_id, "tr-0001");
        assert_eq!(report.items[1].review_id, "tr-0002");
        assert!(!report.written);

        let _ = fs::remove_dir_all(&kb_path);
        Ok(())
    }
}
