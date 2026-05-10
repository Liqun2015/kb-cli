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
    #[command(about = "List existing topic-specific relationship workspaces")]
    List(TopicListArgs),
    #[command(about = "Check one topic workspace for expected files and directories")]
    Status(TopicStatusArgs),
    #[command(about = "Generate topic-local literature importance candidates")]
    Rank(TopicRankArgs),
    #[command(about = "Build a deterministic review queue from topic-local importance candidates")]
    Review(TopicReviewArgs),
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
        TopicCommand::Review(review_args) => execute_review(custom_kb, review_args),
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
