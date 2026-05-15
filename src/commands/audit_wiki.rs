use anyhow::{anyhow, Result};
use clap::Args;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Args)]
pub struct AuditWikiArgs {
    #[arg(long, help = "Preview the audit without writing report files")]
    pub dry_run: bool,

    #[arg(long, help = "Alias for --dry-run")]
    pub preview: bool,

    #[arg(long, help = "Print the audit report as JSON")]
    pub json: bool,

    #[arg(
        long,
        help = "Return non-zero when the proof score is lower than --min-score"
    )]
    pub strict: bool,

    #[arg(
        long,
        default_value_t = 70,
        help = "Minimum proof score required by --strict"
    )]
    pub min_score: u8,

    #[arg(long, help = "Do not write the LLM review prompt under LLM/tasks/")]
    pub no_llm_prompt: bool,
}

#[derive(Debug, serde::Serialize)]
struct AuditWikiReport {
    schema_version: String,
    generated_by: String,
    generated_at: String,
    kb_path: String,
    dry_run: bool,
    written: bool,
    report_path: Option<String>,
    llm_prompt_path: Option<String>,
    proof_score: u8,
    max_score: u8,
    verdict: String,
    claim: String,
    checks: Vec<AuditCheck>,
    review_queue_audits: Vec<ReviewQueueAudit>,
    warnings: Vec<String>,
    next_steps: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
struct AuditCheck {
    id: String,
    layer: String,
    status: String,
    score: u8,
    max_score: u8,
    evidence: Vec<String>,
    remediation: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
struct ReviewQueueAudit {
    topic_slug: String,
    queue_path: String,
    status: String,
    score: u8,
    max_score: u8,
    rows: usize,
    missing_columns: Vec<String>,
    evidence_rows: usize,
    weak_evidence_rows: usize,
    pending_decision_rows: usize,
    recommendations: Vec<String>,
}

pub fn execute(custom_kb: Option<&Path>, args: &AuditWikiArgs) -> Result<()> {
    let kb_path = crate::commands::init::get_kb_path(custom_kb);
    let mut report = build_audit_report(&kb_path, args)?;
    let dry_run = args.dry_run || args.preview;

    if !dry_run {
        let reports_dir = kb_path.join("outputs/reports");
        fs::create_dir_all(&reports_dir)?;
        let stamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let report_path = reports_dir.join(format!("wiki_audit_{stamp}.md"));
        report.report_path = Some(relative_path_string(&kb_path, &report_path));

        let llm_prompt_path = if !args.no_llm_prompt {
            let tasks_dir = kb_path.join("LLM/tasks");
            fs::create_dir_all(&tasks_dir)?;
            let path = tasks_dir.join(format!("wiki_audit_llm_review_{stamp}.md"));
            report.llm_prompt_path = Some(relative_path_string(&kb_path, &path));
            Some(path)
        } else {
            None
        };

        report.written = true;
        fs::write(&report_path, render_audit_markdown(&report))?;
        if let Some(path) = llm_prompt_path {
            fs::write(&path, render_llm_audit_prompt(&report))?;
        }
    }

    if args.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print_audit_report(&report);
    }

    if args.strict && report.proof_score < args.min_score {
        return Err(anyhow!(
            "wiki audit score {} is below required minimum {}",
            report.proof_score,
            args.min_score
        ));
    }
    Ok(())
}

fn build_audit_report(kb_path: &Path, args: &AuditWikiArgs) -> Result<AuditWikiReport> {
    let dry_run = args.dry_run || args.preview;
    let mut checks = Vec::new();
    checks.push(check_source_pile(kb_path));
    checks.push(check_rules_layer(kb_path));
    checks.push(check_manifest_layer(kb_path));
    checks.push(check_processing_layer(kb_path));
    checks.push(check_wiki_layer(kb_path));
    checks.push(check_traceability_layer(kb_path));
    checks.push(check_literature_relation_layer(kb_path));
    checks.push(check_topic_layer(kb_path));
    checks.push(check_llm_workbench_layer(kb_path));
    checks.push(check_review_outputs_layer(kb_path));

    let review_queue_audits = audit_review_queues(kb_path)?;
    let review_queue_score = review_queue_audits
        .iter()
        .map(|audit| audit.score as usize)
        .sum::<usize>();
    let review_queue_max = review_queue_audits
        .iter()
        .map(|audit| audit.max_score as usize)
        .sum::<usize>();

    let checks_score = checks
        .iter()
        .map(|check| check.score as usize)
        .sum::<usize>();
    let checks_max = checks
        .iter()
        .map(|check| check.max_score as usize)
        .sum::<usize>();
    let total_score = checks_score + review_queue_score;
    let total_max = checks_max + review_queue_max;
    let proof_score = if total_max == 0 {
        0
    } else {
        ((total_score * 100) / total_max).min(100) as u8
    };
    let max_score = 100;
    let verdict = audit_verdict(proof_score).to_string();
    let warnings = collect_warnings(&checks, &review_queue_audits);
    let next_steps = build_next_steps(&checks, &review_queue_audits, args.no_llm_prompt);

    Ok(AuditWikiReport {
        schema_version: "wiki-audit.v0.7.2".to_string(),
        generated_by: "kb-cli audit-wiki".to_string(),
        generated_at: chrono::Utc::now().to_rfc3339(),
        kb_path: kb_path.display().to_string(),
        dry_run,
        written: false,
        report_path: None,
        llm_prompt_path: None,
        proof_score,
        max_score,
        verdict,
        claim: "kb-cli can turn a pile of materials into a reviewable, human/AI-maintainable research Markdown knowledge base when the deterministic layers, review queues, and explicit LLM handoff records are present.".to_string(),
        checks,
        review_queue_audits,
        warnings,
        next_steps,
    })
}

fn check_source_pile(kb_path: &Path) -> AuditCheck {
    let raw_dir = kb_path.join("raw");
    let total = count_files(&raw_dir, None);
    let paper_count = count_files(&raw_dir.join("papers"), None);
    let status = if total > 0 { "pass" } else { "fail" };
    let score = if total > 0 { 10 } else { 0 };
    AuditCheck {
        id: "source_pile".to_string(),
        layer: "raw".to_string(),
        status: status.to_string(),
        score,
        max_score: 10,
        evidence: vec![
            format!("raw files: {total}"),
            format!("raw/papers files: {paper_count}"),
        ],
        remediation: if total == 0 {
            vec!["Add source materials under raw/ or run `kb ingest --copy`.".to_string()]
        } else {
            Vec::new()
        },
    }
}

fn check_rules_layer(kb_path: &Path) -> AuditCheck {
    let required = [
        "rules/LLM_WIKI_SCHEMA.md",
        "rules/PAPER_PAGE_TEMPLATE.md",
        "rules/CONCEPT_PAGE_TEMPLATE.md",
        "rules/QUERY_POLICY.md",
        "rules/LINT_POLICY.md",
    ];
    check_required_files(
        "rules_layer",
        "rules",
        8,
        kb_path,
        &required,
        "Run `kb init` to create the rules layer.",
    )
}

fn check_manifest_layer(kb_path: &Path) -> AuditCheck {
    let path = kb_path.join("processing/manifest.json");
    let exists = path.is_file();
    let text = fs::read_to_string(&path).unwrap_or_default();
    let has_raw_paths = text.contains("raw/") || text.contains("raw\\");
    let score = match (exists, has_raw_paths) {
        (true, true) => 10,
        (true, false) => 5,
        _ => 0,
    };
    AuditCheck {
        id: "manifest_registry".to_string(),
        layer: "processing".to_string(),
        status: status_from_score(score, 10).to_string(),
        score,
        max_score: 10,
        evidence: vec![
            format!("processing/manifest.json exists: {exists}"),
            format!("manifest mentions raw paths: {has_raw_paths}"),
        ],
        remediation: if score < 10 {
            vec!["Run `kb status` after ingesting raw files.".to_string()]
        } else {
            Vec::new()
        },
    }
}

fn check_processing_layer(kb_path: &Path) -> AuditCheck {
    let text_count = count_files(&kb_path.join("processing/text"), Some("txt"));
    let proposal_count = count_files(&kb_path.join("processing/proposals"), Some("md"));
    let keyword_count = count_files(&kb_path.join("processing/keywords"), None);
    let score = if text_count > 0 && (proposal_count > 0 || keyword_count > 0) {
        10
    } else if text_count > 0 || proposal_count > 0 || keyword_count > 0 {
        6
    } else {
        0
    };
    AuditCheck {
        id: "processing_outputs".to_string(),
        layer: "processing".to_string(),
        status: status_from_score(score, 10).to_string(),
        score,
        max_score: 10,
        evidence: vec![
            format!("processing/text .txt files: {text_count}"),
            format!("processing/proposals .md files: {proposal_count}"),
            format!("processing/keywords files: {keyword_count}"),
        ],
        remediation: if score < 10 {
            vec![
                "Run `kb extract-text` to create deterministic text inputs.".to_string(),
                "Run `kb build <topic>` or `kb keywords` to create reviewable intermediate artifacts.".to_string(),
            ]
        } else {
            Vec::new()
        },
    }
}

fn check_wiki_layer(kb_path: &Path) -> AuditCheck {
    let wiki_count = count_files(&kb_path.join("wiki"), Some("md"));
    let score = if wiki_count >= 5 {
        10
    } else if wiki_count > 0 {
        6
    } else {
        0
    };
    AuditCheck {
        id: "markdown_wiki".to_string(),
        layer: "wiki".to_string(),
        status: status_from_score(score, 10).to_string(),
        score,
        max_score: 10,
        evidence: vec![format!("wiki Markdown files: {wiki_count}")],
        remediation: if score < 10 {
            vec![
                "Run `kb build-wiki`, then let human/LLM reviewers improve pages under Git diff."
                    .to_string(),
            ]
        } else {
            Vec::new()
        },
    }
}

fn check_traceability_layer(kb_path: &Path) -> AuditCheck {
    let wiki_dir = kb_path.join("wiki");
    let total = count_files(&wiki_dir, Some("md"));
    let with_source = count_markdown_with_any(&wiki_dir, &["source_ids", "source_files"]);
    let score = if total > 0 && with_source == total {
        10
    } else if with_source > 0 {
        7
    } else if total > 0 {
        3
    } else {
        0
    };
    AuditCheck {
        id: "source_traceability".to_string(),
        layer: "wiki".to_string(),
        status: status_from_score(score, 10).to_string(),
        score,
        max_score: 10,
        evidence: vec![
            format!("wiki Markdown files: {total}"),
            format!("wiki files with source_ids/source_files: {with_source}"),
        ],
        remediation: if score < 10 {
            vec![
                "Add source_ids or source_files front matter to source-derived wiki pages."
                    .to_string(),
                "Run `kb sync-wiki` after accepted wiki edits.".to_string(),
            ]
        } else {
            Vec::new()
        },
    }
}

fn check_literature_relation_layer(kb_path: &Path) -> AuditCheck {
    let refs_dir = kb_path.join("processing/refs");
    let refs = count_files_with_prefix(&refs_dir, "refs_", None)
        + count_files_with_prefix(&refs_dir, "refs_index_", None)
        + count_files_with_prefix(&refs_dir, "refs_graph_", None);
    let graph = count_files_with_prefix(&refs_dir, "refs_graph_", None);
    let score = if refs > 0 && graph > 0 {
        10
    } else if refs > 0 {
        6
    } else {
        0
    };
    AuditCheck {
        id: "literature_relations".to_string(),
        layer: "processing/refs".to_string(),
        status: status_from_score(score, 10).to_string(),
        score,
        max_score: 10,
        evidence: vec![
            format!("reference/index/graph files: {refs}"),
            format!("refs graph files: {graph}"),
        ],
        remediation: if score < 10 {
            vec![
                "Run `kb refs`, `kb refs-index`, and optionally `kb refs-graph`.".to_string(),
                "Keep uncertain bibliographic relations as reviewable candidates.".to_string(),
            ]
        } else {
            Vec::new()
        },
    }
}

fn check_topic_layer(kb_path: &Path) -> AuditCheck {
    let topics = collect_topic_slugs(kb_path);
    let mut with_importance = 0usize;
    let mut with_review = 0usize;
    for slug in &topics {
        let root = kb_path.join("topics").join(slug);
        if count_files(&root.join("importance"), None) > 0 {
            with_importance += 1;
        }
        if root.join("review/review_queue.md").is_file() {
            with_review += 1;
        }
    }
    let score = if !topics.is_empty() && with_importance > 0 && with_review > 0 {
        12
    } else if !topics.is_empty() && with_importance > 0 {
        8
    } else if !topics.is_empty() {
        5
    } else {
        0
    };
    AuditCheck {
        id: "topic_workspaces".to_string(),
        layer: "topics".to_string(),
        status: status_from_score(score, 12).to_string(),
        score,
        max_score: 12,
        evidence: vec![
            format!("topic workspaces: {}", topics.len()),
            format!("topics with importance files: {with_importance}"),
            format!("topics with review_queue.md: {with_review}"),
        ],
        remediation: if score < 12 {
            vec![
                "Run `kb topic init <topic>`.".to_string(),
                "Edit topics/<topic>/literature.md, then run `kb topic rank <topic>` and `kb topic review <topic>`.".to_string(),
            ]
        } else {
            Vec::new()
        },
    }
}

fn check_llm_workbench_layer(kb_path: &Path) -> AuditCheck {
    let tasks = count_files(&kb_path.join("LLM/tasks"), Some("md"));
    let memory = count_files(&kb_path.join("LLM/memory"), Some("md"));
    let score = if tasks > 0 && memory > 0 {
        10
    } else if tasks > 0 || memory > 0 {
        6
    } else if kb_path.join("LLM/tasks").is_dir() && kb_path.join("LLM/memory").is_dir() {
        3
    } else {
        0
    };
    AuditCheck {
        id: "llm_workbench".to_string(),
        layer: "LLM".to_string(),
        status: status_from_score(score, 10).to_string(),
        score,
        max_score: 10,
        evidence: vec![
            format!("LLM/tasks markdown files: {tasks}"),
            format!("LLM/memory markdown files: {memory}"),
        ],
        remediation: if score < 10 {
            vec![
                "Run `kb tasks` to create explicit handoff tasks.".to_string(),
                "Use `kb memory --task-id ... --summary ...` after accepted work.".to_string(),
            ]
        } else {
            Vec::new()
        },
    }
}

fn check_review_outputs_layer(kb_path: &Path) -> AuditCheck {
    let lint =
        count_files_with_prefix(&kb_path.join("outputs/reports"), "lint_static_", Some("md"));
    let health = count_files_with_prefix(&kb_path.join("outputs/reports"), "health_", Some("md"));
    let viewer = kb_path.join("outputs/html/index.html").is_file();
    let score = if lint > 0 && health > 0 && viewer {
        10
    } else if lint > 0 || health > 0 || viewer {
        6
    } else {
        0
    };
    AuditCheck {
        id: "review_outputs".to_string(),
        layer: "outputs".to_string(),
        status: status_from_score(score, 10).to_string(),
        score,
        max_score: 10,
        evidence: vec![
            format!("lint reports: {lint}"),
            format!("health reports: {health}"),
            format!("static viewer exists: {viewer}"),
        ],
        remediation: if score < 10 {
            vec![
                "Run `kb lint-static`, `kb health`, and `kb view --no-open` to create review outputs.".to_string(),
            ]
        } else {
            Vec::new()
        },
    }
}

fn check_required_files(
    id: &str,
    layer: &str,
    max_score: u8,
    kb_path: &Path,
    required: &[&str],
    remediation: &str,
) -> AuditCheck {
    let present = required
        .iter()
        .filter(|relative| kb_path.join(relative).is_file())
        .count();
    let score = if required.is_empty() {
        max_score
    } else {
        ((present * max_score as usize) / required.len()) as u8
    };
    AuditCheck {
        id: id.to_string(),
        layer: layer.to_string(),
        status: status_from_score(score, max_score).to_string(),
        score,
        max_score,
        evidence: vec![
            format!("required files present: {present}/{}", required.len()),
            format!("required files: {}", required.join(", ")),
        ],
        remediation: if present < required.len() {
            vec![remediation.to_string()]
        } else {
            Vec::new()
        },
    }
}

fn audit_review_queues(kb_path: &Path) -> Result<Vec<ReviewQueueAudit>> {
    let mut audits = Vec::new();
    for slug in collect_topic_slugs(kb_path) {
        let queue_path = kb_path
            .join("topics")
            .join(&slug)
            .join("review/review_queue.md");
        if queue_path.is_file() {
            audits.push(audit_one_review_queue(kb_path, &slug, &queue_path)?);
        }
    }
    Ok(audits)
}

fn audit_one_review_queue(
    kb_path: &Path,
    slug: &str,
    queue_path: &Path,
) -> Result<ReviewQueueAudit> {
    let text = fs::read_to_string(queue_path)?;
    let required_columns = [
        "review_id",
        "candidate_id",
        "source_item",
        "proposed_decision",
        "final_decision",
        "status",
        "reviewer",
        "confidence",
        "evidence",
        "uncertainty",
        "review_notes",
        "accepted_record_path",
    ];
    let (header, rows) = parse_first_markdown_table(&text);
    let missing_columns = required_columns
        .iter()
        .filter(|column| !header.iter().any(|h| h == **column))
        .map(|column| column.to_string())
        .collect::<Vec<_>>();
    let evidence_index = header.iter().position(|h| h == "evidence");
    let final_decision_index = header.iter().position(|h| h == "final_decision");
    let mut evidence_rows = 0usize;
    let mut weak_evidence_rows = 0usize;
    let mut pending_decision_rows = 0usize;
    for row in &rows {
        if let Some(index) = evidence_index {
            let evidence = row.get(index).map(|v| v.trim()).unwrap_or_default();
            if evidence_contains_file_hint(evidence) {
                evidence_rows += 1;
            } else {
                weak_evidence_rows += 1;
            }
        }
        if let Some(index) = final_decision_index {
            let decision = row.get(index).map(|v| v.trim()).unwrap_or_default();
            if decision.is_empty() || decision.eq_ignore_ascii_case("pending") || decision == "TODO"
            {
                pending_decision_rows += 1;
            }
        }
    }

    let rows_count = rows.len();
    let mut score = 0u8;
    if missing_columns.is_empty() {
        score += 8;
    } else if missing_columns.len() <= 3 {
        score += 5;
    } else if !header.is_empty() {
        score += 2;
    }
    if rows_count > 0 {
        score += 4;
    }
    if rows_count > 0 && evidence_rows == rows_count {
        score += 5;
    } else if evidence_rows > 0 {
        score += 3;
    }
    if rows_count > 0 && final_decision_index.is_some() {
        score += 3;
    }

    let status = status_from_score(score, 20).to_string();
    let mut recommendations = Vec::new();
    if !missing_columns.is_empty() {
        recommendations.push(format!(
            "Add missing queue columns: {}.",
            missing_columns.join(", ")
        ));
    }
    if rows_count == 0 {
        recommendations.push(
            "Run `kb topic rank <topic>` and regenerate the review queue after candidates exist."
                .to_string(),
        );
    }
    if weak_evidence_rows > 0 {
        recommendations.push("Strengthen evidence cells with concrete raw/, wiki/, processing/, or topics/ file references.".to_string());
    }
    if final_decision_index.is_none() {
        recommendations.push(
            "Add final_decision so the queue can be used as an editable review workbench."
                .to_string(),
        );
    }

    Ok(ReviewQueueAudit {
        topic_slug: slug.to_string(),
        queue_path: relative_path_string(kb_path, queue_path),
        status,
        score,
        max_score: 20,
        rows: rows_count,
        missing_columns,
        evidence_rows,
        weak_evidence_rows,
        pending_decision_rows,
        recommendations,
    })
}

fn parse_first_markdown_table(text: &str) -> (Vec<String>, Vec<Vec<String>>) {
    let mut header = Vec::new();
    let mut rows = Vec::new();
    let mut in_table = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with('|') || !trimmed.ends_with('|') {
            if in_table && !rows.is_empty() {
                break;
            }
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
            in_table = true;
            continue;
        }
        if header.is_empty() {
            header = cells.iter().map(|cell| normalize_header(cell)).collect();
            in_table = true;
        } else {
            let normalized_first = cells
                .first()
                .map(|c| normalize_header(c))
                .unwrap_or_default();
            if normalized_first == "_none_" || normalized_first == "review_id" {
                continue;
            }
            rows.push(cells);
        }
    }
    (header, rows)
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

fn evidence_contains_file_hint(input: &str) -> bool {
    let lower = input.to_ascii_lowercase();
    lower.contains("raw/")
        || lower.contains("wiki/")
        || lower.contains("processing/")
        || lower.contains("topics/")
        || lower.contains("llm/")
        || lower.contains("doi")
}

fn count_files(dir: &Path, extension: Option<&str>) -> usize {
    if !dir.exists() {
        return 0;
    }
    walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| {
            if let Some(ext) = extension {
                entry
                    .path()
                    .extension()
                    .and_then(|value| value.to_str())
                    .map(|value| value.eq_ignore_ascii_case(ext))
                    .unwrap_or(false)
            } else {
                true
            }
        })
        .count()
}

fn count_files_with_prefix(dir: &Path, prefix: &str, extension: Option<&str>) -> usize {
    if !dir.exists() {
        return 0;
    }
    walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| {
            entry
                .path()
                .file_name()
                .and_then(|value| value.to_str())
                .map(|name| name.starts_with(prefix))
                .unwrap_or(false)
        })
        .filter(|entry| {
            if let Some(ext) = extension {
                entry
                    .path()
                    .extension()
                    .and_then(|value| value.to_str())
                    .map(|value| value.eq_ignore_ascii_case(ext))
                    .unwrap_or(false)
            } else {
                true
            }
        })
        .count()
}

fn count_markdown_with_any(dir: &Path, needles: &[&str]) -> usize {
    if !dir.exists() {
        return 0;
    }
    walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| entry.path().extension().and_then(|ext| ext.to_str()) == Some("md"))
        .filter(|entry| {
            fs::read_to_string(entry.path())
                .map(|text| needles.iter().any(|needle| text.contains(needle)))
                .unwrap_or(false)
        })
        .count()
}

fn collect_topic_slugs(kb_path: &Path) -> Vec<String> {
    let topics_dir = kb_path.join("topics");
    let mut topics = Vec::new();
    if let Ok(entries) = fs::read_dir(topics_dir) {
        for entry in entries.flatten() {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                if let Some(name) = entry.file_name().to_str() {
                    topics.push(name.to_string());
                }
            }
        }
    }
    topics.sort();
    topics
}

fn status_from_score(score: u8, max_score: u8) -> &'static str {
    if score == 0 {
        "fail"
    } else if score >= max_score {
        "pass"
    } else {
        "partial"
    }
}

fn audit_verdict(score: u8) -> &'static str {
    if score >= 85 {
        "strong evidence"
    } else if score >= 70 {
        "sufficient evidence"
    } else if score >= 45 {
        "partial evidence"
    } else {
        "insufficient evidence"
    }
}

fn collect_warnings(checks: &[AuditCheck], queues: &[ReviewQueueAudit]) -> Vec<String> {
    let mut warnings = Vec::new();
    for check in checks {
        if check.status != "pass" {
            warnings.push(format!("{} is {}", check.id, check.status));
        }
    }
    if queues.is_empty() {
        warnings.push(
            "no topic review queues found; run `kb topic review <topic>` after ranking candidates"
                .to_string(),
        );
    }
    for queue in queues {
        if queue.status != "pass" {
            warnings.push(format!(
                "review queue `{}` is {}",
                queue.queue_path, queue.status
            ));
        }
    }
    warnings
}

fn build_next_steps(
    checks: &[AuditCheck],
    queues: &[ReviewQueueAudit],
    no_llm_prompt: bool,
) -> Vec<String> {
    let mut steps = Vec::new();
    for check in checks {
        for item in &check.remediation {
            if !steps.contains(item) {
                steps.push(item.clone());
            }
        }
    }
    for queue in queues {
        for item in &queue.recommendations {
            if !steps.contains(item) {
                steps.push(item.clone());
            }
        }
    }
    if !no_llm_prompt {
        steps.push("Give the generated LLM audit prompt to a Manager LLM for an independent review of whether the proof chain is convincing.".to_string());
    }
    steps.truncate(12);
    steps
}

fn render_audit_markdown(report: &AuditWikiReport) -> String {
    let mut md = String::new();
    md.push_str("# LLM Wiki Transformation Audit\n\n");
    md.push_str(&format!("Generated at: `{}`\n\n", report.generated_at));
    md.push_str(&format!("Knowledge base: `{}`\n\n", report.kb_path));
    md.push_str("## Claim under audit\n\n");
    md.push_str(&report.claim);
    md.push_str("\n\n");
    md.push_str("## Verdict\n\n");
    md.push_str(&format!(
        "- Proof score: `{}/{}`\n",
        report.proof_score, report.max_score
    ));
    md.push_str(&format!("- Verdict: `{}`\n", report.verdict));
    md.push_str(&format!("- Dry run: `{}`\n", report.dry_run));
    md.push_str(&format!(
        "- LLM prompt: `{}`\n",
        report
            .llm_prompt_path
            .clone()
            .unwrap_or_else(|| "not written yet".to_string())
    ));

    md.push_str("\n## Deterministic checks\n\n");
    md.push_str("| id | layer | status | score | evidence | remediation |\n");
    md.push_str("|---|---|---|---:|---|---|\n");
    for check in &report.checks {
        md.push_str(&format!(
            "| {} | {} | {} | {}/{} | {} | {} |\n",
            escape_table_cell(&check.id),
            escape_table_cell(&check.layer),
            check.status,
            check.score,
            check.max_score,
            escape_table_cell(&check.evidence.join("; ")),
            escape_table_cell(&check.remediation.join("; "))
        ));
    }

    md.push_str("\n## Review queue format audit\n\n");
    if report.review_queue_audits.is_empty() {
        md.push_str("No review queue files were found.\n");
    } else {
        md.push_str("| topic | status | score | rows | missing_columns | weak_evidence_rows | pending_decision_rows |\n");
        md.push_str("|---|---|---:|---:|---|---:|---:|\n");
        for queue in &report.review_queue_audits {
            md.push_str(&format!(
                "| {} | {} | {}/{} | {} | {} | {} | {} |\n",
                escape_table_cell(&queue.topic_slug),
                queue.status,
                queue.score,
                queue.max_score,
                queue.rows,
                escape_table_cell(&queue.missing_columns.join(", ")),
                queue.weak_evidence_rows,
                queue.pending_decision_rows
            ));
        }
    }

    if !report.warnings.is_empty() {
        md.push_str("\n## Warnings\n\n");
        for warning in &report.warnings {
            md.push_str(&format!("- {warning}\n"));
        }
    }

    md.push_str("\n## Next steps\n\n");
    if report.next_steps.is_empty() {
        md.push_str(
            "- No immediate remediation found. Use the LLM audit prompt for independent review.\n",
        );
    } else {
        for step in &report.next_steps {
            md.push_str(&format!("- {step}\n"));
        }
    }
    md
}

fn render_llm_audit_prompt(report: &AuditWikiReport) -> String {
    let mut md = String::new();
    md.push_str("# LLM Audit Task: Verify Markdown LLM Wiki Transformation\n\n");
    md.push_str("You are a Manager LLM reviewing whether this kb-cli knowledge base has genuinely moved from a loose pile of materials to a reviewable research Markdown knowledge base.\n\n");
    md.push_str("## Claim to evaluate\n\n");
    md.push_str(&report.claim);
    md.push_str("\n\n");
    md.push_str("## Deterministic audit summary\n\n");
    md.push_str(&format!(
        "- Proof score: `{}/{}`\n",
        report.proof_score, report.max_score
    ));
    md.push_str(&format!("- Verdict: `{}`\n", report.verdict));
    if let Some(path) = &report.report_path {
        md.push_str(&format!("- Deterministic report: `{path}`\n"));
    }
    md.push_str("\n## Files to inspect first\n\n");
    md.push_str("- `raw/`\n");
    md.push_str("- `processing/manifest.json`\n");
    md.push_str("- `wiki/`\n");
    md.push_str("- `rules/`\n");
    md.push_str("- `processing/refs/`\n");
    md.push_str("- `topics/*/importance/`\n");
    md.push_str("- `topics/*/review/review_queue.md`\n");
    md.push_str("- `LLM/tasks/`\n");
    md.push_str("- `LLM/memory/`\n\n");
    md.push_str("## Review questions\n\n");
    md.push_str("1. Does the KB preserve original source materials instead of overwriting them?\n");
    md.push_str("2. Are generated wiki pages traceable to raw/source files?\n");
    md.push_str("3. Are uncertain bibliographic, topic, and importance claims stored as candidates rather than final truth?\n");
    md.push_str("4. Is there a usable review queue with evidence and editable decision columns?\n");
    md.push_str("5. Are LLM tasks explicit and bounded rather than hidden model behavior?\n");
    md.push_str("6. What is the strongest missing link in the proof chain?\n\n");
    md.push_str("## Required response format\n\n");
    md.push_str("```text\n");
    md.push_str("verdict: PASS | PARTIAL | FAIL\n");
    md.push_str("confidence: high | medium | low\n");
    md.push_str("strongest_evidence:\n");
    md.push_str("- ...\n");
    md.push_str("weakest_link:\n");
    md.push_str("- ...\n");
    md.push_str("review_queue_usability:\n");
    md.push_str("- ...\n");
    md.push_str("required_next_actions:\n");
    md.push_str("- ...\n");
    md.push_str("```\n\n");
    md.push_str("Do not overclaim semantic correctness. The question is whether the workflow is reviewable and maintainable, not whether every scientific claim is true.\n");
    md
}

fn print_audit_report(report: &AuditWikiReport) {
    println!("LLM Wiki transformation audit:");
    println!("  score   : {}/{}", report.proof_score, report.max_score);
    println!("  verdict : {}", report.verdict);
    println!("  dry run : {}", report.dry_run);
    println!("  written : {}", report.written);
    if let Some(path) = &report.report_path {
        println!("  report  : {path}");
    }
    if let Some(path) = &report.llm_prompt_path {
        println!("  LLM task: {path}");
    }

    println!();
    println!("Checks:");
    for check in &report.checks {
        println!(
            "  - {} [{}]: {}/{} ({})",
            check.id, check.layer, check.score, check.max_score, check.status
        );
    }

    println!();
    println!("Review queue format:");
    if report.review_queue_audits.is_empty() {
        println!("  - no review queues found");
    } else {
        for queue in &report.review_queue_audits {
            println!(
                "  - {}: {}/{} ({}) rows={}, missing_columns={}",
                queue.topic_slug,
                queue.score,
                queue.max_score,
                queue.status,
                queue.rows,
                if queue.missing_columns.is_empty() {
                    "none".to_string()
                } else {
                    queue.missing_columns.join(",")
                }
            );
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

fn escape_table_cell(input: &str) -> String {
    input.replace('|', "\\|").replace('\n', " ")
}

fn relative_path_string(kb_path: &Path, path: &Path) -> String {
    path.strip_prefix(kb_path)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
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
            "kb_cli_audit_{}_{}_{}",
            name,
            std::process::id(),
            nanos
        ))
    }

    #[test]
    fn review_queue_audit_requires_decision_columns() -> Result<()> {
        let kb_path = unique_test_kb("queue_columns");
        let queue_path = kb_path.join("topics/thermal/review/review_queue.md");
        fs::create_dir_all(queue_path.parent().unwrap())?;
        fs::write(
            &queue_path,
            "| review_id | candidate_id | source_item | proposed_decision | final_decision | status | reviewer | confidence | evidence | uncertainty | review_notes | accepted_record_path |\n|---|---|---|---|---|---|---|---|---|---|---|---|\n| tr-0001 | c1 | topics/thermal/importance/a.md | core | pending | candidate | unassigned | medium | raw/papers/a.pdf | TODO | TODO | topics/thermal/reviewed/a.md |\n",
        )?;
        let audit = audit_one_review_queue(&kb_path, "thermal", &queue_path)?;
        assert_eq!(audit.status, "pass");
        assert_eq!(audit.missing_columns.len(), 0);
        assert_eq!(audit.rows, 1);
        let _ = fs::remove_dir_all(&kb_path);
        Ok(())
    }

    #[test]
    fn audit_report_scores_a_complete_minimal_kb() -> Result<()> {
        let kb_path = unique_test_kb("minimal");
        fs::create_dir_all(kb_path.join("raw/papers"))?;
        fs::write(kb_path.join("raw/papers/a.pdf"), "not really a pdf")?;
        fs::create_dir_all(kb_path.join("rules"))?;
        for file in [
            "LLM_WIKI_SCHEMA.md",
            "PAPER_PAGE_TEMPLATE.md",
            "CONCEPT_PAGE_TEMPLATE.md",
            "QUERY_POLICY.md",
            "LINT_POLICY.md",
        ] {
            fs::write(kb_path.join("rules").join(file), "rule")?;
        }
        fs::create_dir_all(kb_path.join("processing/text"))?;
        fs::write(
            kb_path.join("processing/manifest.json"),
            "{\"path\":\"raw/papers/a.pdf\"}",
        )?;
        fs::write(kb_path.join("processing/text/a.txt"), "text")?;
        fs::create_dir_all(kb_path.join("processing/proposals"))?;
        fs::write(kb_path.join("processing/proposals/task_plan.md"), "plan")?;
        fs::create_dir_all(kb_path.join("wiki/papers"))?;
        for i in 0..5 {
            fs::write(
                kb_path.join(format!("wiki/papers/p{i}.md")),
                "---\nsource_files: [raw/papers/a.pdf]\n---\nbody",
            )?;
        }
        fs::create_dir_all(kb_path.join("processing/refs"))?;
        fs::write(kb_path.join("processing/refs/refs_index_1.md"), "refs")?;
        fs::write(kb_path.join("processing/refs/refs_graph_1.json"), "{}")?;
        fs::create_dir_all(kb_path.join("topics/thermal/importance"))?;
        fs::write(
            kb_path.join("topics/thermal/importance/importance_candidates_1.md"),
            "candidate",
        )?;
        fs::create_dir_all(kb_path.join("topics/thermal/review"))?;
        fs::write(
            kb_path.join("topics/thermal/review/review_queue.md"),
            "| review_id | candidate_id | source_item | proposed_decision | final_decision | status | reviewer | confidence | evidence | uncertainty | review_notes | accepted_record_path |\n|---|---|---|---|---|---|---|---|---|---|---|---|\n| tr-0001 | c1 | topics/thermal/importance/a.md | core | pending | candidate | unassigned | medium | raw/papers/a.pdf | TODO | TODO | topics/thermal/reviewed/a.md |\n",
        )?;
        fs::create_dir_all(kb_path.join("LLM/tasks"))?;
        fs::write(kb_path.join("LLM/tasks/task.md"), "task")?;
        fs::create_dir_all(kb_path.join("LLM/memory"))?;
        fs::write(kb_path.join("LLM/memory/completed_tasks.md"), "memory")?;
        fs::create_dir_all(kb_path.join("outputs/reports"))?;
        fs::write(kb_path.join("outputs/reports/lint_static_1.md"), "lint")?;
        fs::write(kb_path.join("outputs/reports/health_1.md"), "health")?;
        fs::create_dir_all(kb_path.join("outputs/html"))?;
        fs::write(kb_path.join("outputs/html/index.html"), "html")?;

        let args = AuditWikiArgs {
            dry_run: true,
            preview: false,
            json: false,
            strict: false,
            min_score: 70,
            no_llm_prompt: false,
        };
        let report = build_audit_report(&kb_path, &args)?;
        assert!(report.proof_score >= 85, "score was {}", report.proof_score);
        let _ = fs::remove_dir_all(&kb_path);
        Ok(())
    }
}
