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
