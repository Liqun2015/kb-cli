use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use clap::Args;
use serde::Serialize;
use walkdir::WalkDir;

#[derive(Debug, Clone, Args)]
pub struct HealthArgs {
    #[arg(
        long,
        help = "Preview health report without writing interfaces/reports/health_*.md"
    )]
    pub dry_run: bool,

    #[arg(long, help = "Alias for --dry-run")]
    pub preview: bool,

    #[arg(long, help = "Print a machine-readable JSON health report")]
    pub json: bool,

    #[arg(
        long,
        help = "Return a non-zero exit code when blocker-level issues are found"
    )]
    pub strict: bool,
}

#[derive(Debug, Clone, Serialize)]
struct HealthCheck {
    id: String,
    category: String,
    status: String,
    summary: String,
    evidence: Vec<String>,
    recommended_next: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct DeferredHealthTask {
    id: String,
    category: String,
    target_agent: String,
    goal: String,
    requirements: Vec<String>,
    files: Vec<String>,
    evidence: Vec<String>,
    priority: String,
}

#[derive(Debug, Clone, Serialize)]
struct HealthReport {
    schema_version: String,
    generated_by: String,
    generated_at: String,
    kb_path: String,
    status: String,
    score: i32,
    dry_run: bool,
    report_path: Option<String>,
    counts: BTreeMap<String, usize>,
    checks: Vec<HealthCheck>,
    deferred_tasks: Vec<DeferredHealthTask>,
}

pub fn execute(custom_kb: Option<&Path>, args: &HealthArgs) -> Result<()> {
    let kb_path = crate::commands::init::get_kb_path(custom_kb);
    if !kb_path.exists() {
        return Err(anyhow!(
            "knowledge base path does not exist: {}. Run `kb init` first.",
            kb_path.display()
        ));
    }

    let mut report = run_health(&kb_path, args)?;
    let dry_run = args.dry_run || args.preview;

    if !dry_run {
        let report_path = write_markdown_report(&kb_path, &report)?;
        report.report_path = Some(relative_path_string(&kb_path, &report_path));
    }

    if args.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print_report(&report);
    }

    if args.strict && report.status == "blocker" {
        std::process::exit(1);
    }

    Ok(())
}

fn run_health(kb_path: &Path, args: &HealthArgs) -> Result<HealthReport> {
    let raw_papers = list_files_with_ext(&kb_path.join("raw/papers"), &["pdf"]);
    let text_files = list_files_with_ext(&kb_path.join("processing/text"), &["txt", "md"]);
    let wiki_pages = list_files_with_ext(&kb_path.join("wiki"), &["md"]);
    let refs_reports = list_named_reports(&kb_path.join("processing/refs"), "refs_scan_", "md");
    let refs_index_reports =
        list_named_reports(&kb_path.join("processing/refs"), "refs_index_", "md");
    let refs_graph_json =
        list_named_reports(&kb_path.join("processing/refs"), "refs_graph_", "json");
    let refs_graph_mermaid =
        list_named_reports(&kb_path.join("processing/refs"), "refs_graph_", "mmd");
    let refs_graph_dot = list_named_reports(&kb_path.join("processing/refs"), "refs_graph_", "dot");
    let keyword_reports =
        list_named_reports(&kb_path.join("processing/keywords"), "keywords_", "md");
    let task_reports = list_files_with_ext(&kb_path.join("LLM/tasks"), &["md", "json"]);
    let memory_files = list_files_with_ext(&kb_path.join("LLM/memory"), &["md", "json"]);
    let lint_reports =
        list_named_reports(&kb_path.join("interfaces/reports"), "lint_static_", "md");

    let empty_text_files = text_files
        .iter()
        .filter(|path| file_len(path).unwrap_or(0) == 0)
        .cloned()
        .collect::<Vec<_>>();

    let uncertain_relation_evidence = scan_uncertain_refs_index(kb_path, &refs_index_reports);
    let keyword_relation_evidence = scan_keyword_candidates(kb_path, &keyword_reports);

    let mut counts = BTreeMap::new();
    counts.insert("raw_papers".to_string(), raw_papers.len());
    counts.insert("extracted_text_files".to_string(), text_files.len());
    counts.insert("empty_text_files".to_string(), empty_text_files.len());
    counts.insert("wiki_pages".to_string(), wiki_pages.len());
    counts.insert("refs_reports".to_string(), refs_reports.len());
    counts.insert("refs_index_reports".to_string(), refs_index_reports.len());
    counts.insert("refs_graph_json".to_string(), refs_graph_json.len());
    counts.insert("refs_graph_mermaid".to_string(), refs_graph_mermaid.len());
    counts.insert("refs_graph_dot".to_string(), refs_graph_dot.len());
    counts.insert("keyword_reports".to_string(), keyword_reports.len());
    counts.insert("llm_task_reports".to_string(), task_reports.len());
    counts.insert("llm_memory_files".to_string(), memory_files.len());
    counts.insert("lint_reports".to_string(), lint_reports.len());
    counts.insert(
        "uncertain_bibliographic_relation_hints".to_string(),
        uncertain_relation_evidence.len(),
    );
    counts.insert(
        "keyword_topic_relation_hints".to_string(),
        keyword_relation_evidence.len(),
    );

    let mut checks = Vec::new();
    checks.push(check_raw_papers(&raw_papers));
    checks.push(check_text_coverage(
        &raw_papers,
        &text_files,
        &empty_text_files,
    ));
    checks.push(check_wiki_pages(&wiki_pages));
    checks.push(check_reference_pipeline(&refs_reports, &refs_index_reports));
    checks.push(check_refs_graph(
        &refs_index_reports,
        &refs_graph_json,
        &refs_graph_mermaid,
        &refs_graph_dot,
    ));
    checks.push(check_keywords(&keyword_reports));
    checks.push(check_uncertain_relations(&uncertain_relation_evidence));
    checks.push(check_task_memory(&task_reports, &memory_files));
    checks.push(check_lint_reports(&lint_reports));
    checks.push(check_third_party_skills(kb_path));

    let deferred_tasks = build_deferred_tasks(
        kb_path,
        &raw_papers,
        &text_files,
        &empty_text_files,
        &refs_index_reports,
        &uncertain_relation_evidence,
        &keyword_reports,
        &keyword_relation_evidence,
        &wiki_pages,
    );

    let (status, score) = summarize_status(&checks);

    Ok(HealthReport {
        schema_version: "0.5.13".to_string(),
        generated_by: "kb-cli health".to_string(),
        generated_at: chrono::Utc::now().to_rfc3339(),
        kb_path: kb_path.display().to_string(),
        status,
        score,
        dry_run: args.dry_run || args.preview,
        report_path: None,
        counts,
        checks,
        deferred_tasks,
    })
}

fn check_raw_papers(raw_papers: &[PathBuf]) -> HealthCheck {
    if raw_papers.is_empty() {
        HealthCheck {
            id: "raw-papers".to_string(),
            category: "source_materials".to_string(),
            status: "warn".to_string(),
            summary: "No PDF papers found under raw/papers/.".to_string(),
            evidence: vec!["raw/papers/ contains no PDF files".to_string()],
            recommended_next: vec!["Run `kb ingest` or add PDFs under raw/papers/.".to_string()],
        }
    } else {
        HealthCheck {
            id: "raw-papers".to_string(),
            category: "source_materials".to_string(),
            status: "ok".to_string(),
            summary: format!("{} PDF paper(s) found under raw/papers/.", raw_papers.len()),
            evidence: raw_papers
                .iter()
                .take(5)
                .map(|p| p.display().to_string())
                .collect(),
            recommended_next: vec![
                "Run `kb extract-text` to create searchable text when needed.".to_string(),
            ],
        }
    }
}

fn check_text_coverage(
    raw_papers: &[PathBuf],
    text_files: &[PathBuf],
    empty_text_files: &[PathBuf],
) -> HealthCheck {
    if !raw_papers.is_empty() && text_files.is_empty() {
        HealthCheck {
            id: "text-coverage".to_string(),
            category: "text_extraction".to_string(),
            status: "warn".to_string(),
            summary: "Papers exist, but no extracted text files were found under processing/text/.".to_string(),
            evidence: vec![format!("raw_papers={}, extracted_text_files=0", raw_papers.len())],
            recommended_next: vec!["Run `kb extract-text` before refs, refs-index, keywords, or grep over extracted text.".to_string()],
        }
    } else if !empty_text_files.is_empty() {
        HealthCheck {
            id: "text-coverage".to_string(),
            category: "text_extraction".to_string(),
            status: "warn".to_string(),
            summary: format!("{} extracted text file(s) are empty.", empty_text_files.len()),
            evidence: empty_text_files.iter().take(5).map(|p| p.display().to_string()).collect(),
            recommended_next: vec!["Run `kb tasks` to hand empty or scanned PDFs to a future PDF Text Conversion Worker.".to_string()],
        }
    } else {
        HealthCheck {
            id: "text-coverage".to_string(),
            category: "text_extraction".to_string(),
            status: "ok".to_string(),
            summary: format!("{} extracted text file(s) found.", text_files.len()),
            evidence: text_files
                .iter()
                .take(5)
                .map(|p| p.display().to_string())
                .collect(),
            recommended_next: vec![
                "Use `kb grep`, `kb refs`, and `kb keywords` over processing/text/.".to_string(),
            ],
        }
    }
}

fn check_wiki_pages(wiki_pages: &[PathBuf]) -> HealthCheck {
    if wiki_pages.is_empty() {
        HealthCheck {
            id: "wiki-pages".to_string(),
            category: "wiki".to_string(),
            status: "warn".to_string(),
            summary: "No Markdown wiki pages found under wiki/.".to_string(),
            evidence: vec!["wiki/ contains no Markdown pages".to_string()],
            recommended_next: vec![
                "Run `kb build <topic>` to create topic tasks and handoff files, or `kb build-wiki` for deterministic wiki skeletons."
                    .to_string(),
            ],
        }
    } else {
        HealthCheck {
            id: "wiki-pages".to_string(),
            category: "wiki".to_string(),
            status: "ok".to_string(),
            summary: format!("{} wiki Markdown page(s) found.", wiki_pages.len()),
            evidence: wiki_pages
                .iter()
                .take(5)
                .map(|p| p.display().to_string())
                .collect(),
            recommended_next: vec![
                "Run `kb links` and `kb lint-static` to inspect structural health.".to_string(),
            ],
        }
    }
}

fn check_reference_pipeline(
    refs_reports: &[PathBuf],
    refs_index_reports: &[PathBuf],
) -> HealthCheck {
    if refs_reports.is_empty() && refs_index_reports.is_empty() {
        HealthCheck {
            id: "reference-pipeline".to_string(),
            category: "literature_relationships".to_string(),
            status: "warn".to_string(),
            summary: "No refs or refs-index reports found.".to_string(),
            evidence: vec![
                "processing/refs/ has no refs_scan_*.md or refs_index_*.md reports".to_string(),
            ],
            recommended_next: vec!["Run `kb refs`, then `kb refs-index`.".to_string()],
        }
    } else if refs_index_reports.is_empty() {
        HealthCheck {
            id: "reference-pipeline".to_string(),
            category: "literature_relationships".to_string(),
            status: "warn".to_string(),
            summary: "Reference hints exist, but no bibliographic index report was found."
                .to_string(),
            evidence: refs_reports
                .iter()
                .take(5)
                .map(|p| p.display().to_string())
                .collect(),
            recommended_next: vec![
                "Run `kb refs-index` to build bibliographic index relation candidates.".to_string(),
            ],
        }
    } else {
        HealthCheck {
            id: "reference-pipeline".to_string(),
            category: "literature_relationships".to_string(),
            status: "ok".to_string(),
            summary: format!("{} refs-index report(s) found.", refs_index_reports.len()),
            evidence: refs_index_reports
                .iter()
                .take(5)
                .map(|p| p.display().to_string())
                .collect(),
            recommended_next: vec![
                "Run `kb refs-graph` to export relation candidates for visualization.".to_string(),
            ],
        }
    }
}

fn check_refs_graph(
    refs_index_reports: &[PathBuf],
    json: &[PathBuf],
    mermaid: &[PathBuf],
    dot: &[PathBuf],
) -> HealthCheck {
    let graph_count = json.len() + mermaid.len() + dot.len();
    if refs_index_reports.is_empty() {
        HealthCheck {
            id: "refs-graph".to_string(),
            category: "third_party_skills".to_string(),
            status: "ok".to_string(),
            summary: "No refs-index report yet, so graph export is not expected.".to_string(),
            evidence: vec!["refs-index report count is 0".to_string()],
            recommended_next: vec!["Run `kb refs-index` before `kb refs-graph`.".to_string()],
        }
    } else if graph_count == 0 {
        HealthCheck {
            id: "refs-graph".to_string(),
            category: "third_party_skills".to_string(),
            status: "warn".to_string(),
            summary: "refs-index reports exist, but no refs-graph export was found.".to_string(),
            evidence: refs_index_reports
                .iter()
                .take(5)
                .map(|p| p.display().to_string())
                .collect(),
            recommended_next: vec![
                "Run `kb refs-graph --json`, optionally with `--mermaid` or `--dot`.".to_string(),
            ],
        }
    } else {
        HealthCheck {
            id: "refs-graph".to_string(),
            category: "third_party_skills".to_string(),
            status: "ok".to_string(),
            summary: format!("{} refs-graph export file(s) found.", graph_count),
            evidence: json
                .iter()
                .chain(mermaid.iter())
                .chain(dot.iter())
                .take(5)
                .map(|p| p.display().to_string())
                .collect(),
            recommended_next: vec![
                "Use docs/third-party-skills/ to render graph certainty correctly.".to_string(),
            ],
        }
    }
}

fn check_keywords(keyword_reports: &[PathBuf]) -> HealthCheck {
    if keyword_reports.is_empty() {
        HealthCheck {
            id: "keywords".to_string(),
            category: "keyword_topic_relations".to_string(),
            status: "warn".to_string(),
            summary: "No keyword/topic relation candidate report was found.".to_string(),
            evidence: vec!["processing/keywords/ has no keywords_*.md reports".to_string()],
            recommended_next: vec!["Run `kb keywords` after text extraction.".to_string()],
        }
    } else {
        HealthCheck {
            id: "keywords".to_string(),
            category: "keyword_topic_relations".to_string(),
            status: "ok".to_string(),
            summary: format!("{} keyword report(s) found.", keyword_reports.len()),
            evidence: keyword_reports
                .iter()
                .take(5)
                .map(|p| p.display().to_string())
                .collect(),
            recommended_next: vec![
                "Review keyword/topic candidates before claiming scientific idea relations."
                    .to_string(),
            ],
        }
    }
}

fn check_uncertain_relations(evidence: &[String]) -> HealthCheck {
    if evidence.is_empty() {
        HealthCheck {
            id: "uncertain-bibliographic-relations".to_string(),
            category: "human_review".to_string(),
            status: "ok".to_string(),
            summary: "No uncertain bibliographic relation hints detected in refs-index reports."
                .to_string(),
            evidence: vec![],
            recommended_next: vec![
                "Continue using humans as the final guarantee when uncertain matches appear."
                    .to_string(),
            ],
        }
    } else {
        HealthCheck {
            id: "uncertain-bibliographic-relations".to_string(),
            category: "human_review".to_string(),
            status: "warn".to_string(),
            summary: format!("{} uncertain bibliographic relation hint(s) need human review.", evidence.len()),
            evidence: evidence.iter().take(10).cloned().collect(),
            recommended_next: vec!["Run `kb tasks` and assign the bibliographic-index review task to a human reviewer.".to_string()],
        }
    }
}

fn check_task_memory(task_reports: &[PathBuf], memory_files: &[PathBuf]) -> HealthCheck {
    if task_reports.is_empty() {
        HealthCheck {
            id: "task-memory".to_string(),
            category: "llm_handoff".to_string(),
            status: "ok".to_string(),
            summary: "No LLM task files found yet.".to_string(),
            evidence: vec!["LLM/tasks/ has no task reports".to_string()],
            recommended_next: vec![
                "Run `kb tasks` when deterministic commands uncover deferred work.".to_string(),
            ],
        }
    } else if memory_files.is_empty() {
        HealthCheck {
            id: "task-memory".to_string(),
            category: "llm_handoff".to_string(),
            status: "warn".to_string(),
            summary: "Task reports exist, but no completed memory file was found.".to_string(),
            evidence: task_reports
                .iter()
                .take(5)
                .map(|p| p.display().to_string())
                .collect(),
            recommended_next: vec![
                "Use `kb memory --task-id ... --summary ...` after accepted work.".to_string(),
            ],
        }
    } else {
        HealthCheck {
            id: "task-memory".to_string(),
            category: "llm_handoff".to_string(),
            status: "ok".to_string(),
            summary: "LLM task and memory areas are both present.".to_string(),
            evidence: memory_files
                .iter()
                .take(5)
                .map(|p| p.display().to_string())
                .collect(),
            recommended_next: vec![
                "Keep completed task records reviewable under LLM/memory/.".to_string()
            ],
        }
    }
}

fn check_lint_reports(lint_reports: &[PathBuf]) -> HealthCheck {
    if lint_reports.is_empty() {
        HealthCheck {
            id: "lint-reports".to_string(),
            category: "wiki_structure".to_string(),
            status: "warn".to_string(),
            summary: "No lint-static report was found under interfaces/reports/.".to_string(),
            evidence: vec!["interfaces/reports/ has no lint_static_*.md reports".to_string()],
            recommended_next: vec![
                "Run `kb lint-static` to check broken links, orphans, and source front matter."
                    .to_string(),
            ],
        }
    } else {
        HealthCheck {
            id: "lint-reports".to_string(),
            category: "wiki_structure".to_string(),
            status: "ok".to_string(),
            summary: format!("{} lint-static report(s) found.", lint_reports.len()),
            evidence: lint_reports
                .iter()
                .take(5)
                .map(|p| p.display().to_string())
                .collect(),
            recommended_next: vec![
                "Use `kb tasks` for unresolved link or source-traceability handoffs.".to_string(),
            ],
        }
    }
}

fn check_third_party_skills(kb_path: &Path) -> HealthCheck {
    let skills_dir = kb_path.join("docs/third-party-skills");
    if skills_dir.exists() {
        HealthCheck {
            id: "third-party-skills".to_string(),
            category: "third_party_skills".to_string(),
            status: "ok".to_string(),
            summary: "Third-party skills documentation is present inside this workspace."
                .to_string(),
            evidence: vec![relative_path_string(kb_path, &skills_dir)],
            recommended_next: vec![
                "Use these skills when generating graph viewers or review dashboards.".to_string(),
            ],
        }
    } else {
        HealthCheck {
            id: "third-party-skills".to_string(),
            category: "third_party_skills".to_string(),
            status: "ok".to_string(),
            summary: "Third-party skills docs are package-level guidance and are not required inside every knowledge base.".to_string(),
            evidence: vec!["docs/third-party-skills/ not found in KB root; this is acceptable for generated knowledge bases".to_string()],
            recommended_next: vec!["Consult the kb-cli package docs when building graph viewers or review dashboards.".to_string()],
        }
    }
}

fn build_deferred_tasks(
    kb_path: &Path,
    raw_papers: &[PathBuf],
    text_files: &[PathBuf],
    empty_text_files: &[PathBuf],
    refs_index_reports: &[PathBuf],
    uncertain_relation_evidence: &[String],
    keyword_reports: &[PathBuf],
    keyword_relation_evidence: &[String],
    wiki_pages: &[PathBuf],
) -> Vec<DeferredHealthTask> {
    let mut tasks = Vec::new();

    if !raw_papers.is_empty() && (text_files.is_empty() || !empty_text_files.is_empty()) {
        tasks.push(DeferredHealthTask {
            id: "health-pdf-text-conversion-001".to_string(),
            category: "text_extraction".to_string(),
            target_agent: "PDF Text Conversion Worker".to_string(),
            goal: "Review PDFs that lack usable extracted text after deterministic best-effort extraction.".to_string(),
            requirements: vec![
                "Do not overwrite raw PDFs.".to_string(),
                "Use OCR or LLM cleanup only when explicitly authorized by the Manager LLM or human.".to_string(),
                "Write accepted text outputs under processing/text/ and record completion with kb memory.".to_string(),
            ],
            files: raw_papers.iter().take(20).map(|p| relative_path_string(kb_path, p)).collect(),
            evidence: empty_text_files.iter().take(20).map(|p| relative_path_string(kb_path, p)).collect(),
            priority: "high".to_string(),
        });
    }

    if !uncertain_relation_evidence.is_empty() {
        tasks.push(DeferredHealthTask {
            id: "health-bibliographic-index-review-001".to_string(),
            category: "literature_relationships".to_string(),
            target_agent: "Human Reference Index Reviewer".to_string(),
            goal: "Confirm, reject, or mark missing uncertain bibliographic index relation candidates.".to_string(),
            requirements: vec![
                "Use DOI, title, author, journal, volume, pages, and date evidence where available.".to_string(),
                "Do not let a Worker LLM serve as the final guarantee for bibliographic identity.".to_string(),
                "Record accepted decisions with kb memory.".to_string(),
            ],
            files: refs_index_reports.iter().take(20).map(|p| relative_path_string(kb_path, p)).collect(),
            evidence: uncertain_relation_evidence.iter().take(20).cloned().collect(),
            priority: "high".to_string(),
        });
    }

    if !keyword_relation_evidence.is_empty() {
        tasks.push(DeferredHealthTask {
            id: "health-keyword-topic-review-001".to_string(),
            category: "keyword_topic_relations".to_string(),
            target_agent: "Keyword Topic Relation Worker".to_string(),
            goal: "Review candidate keyword/topic relations and decide whether they indicate meaningful research relationships.".to_string(),
            requirements: vec![
                "Treat shared terms as evidence, not proof.".to_string(),
                "Preserve source files and evidence lines in any proposed relationship note.".to_string(),
                "Escalate scientific idea claims to Manager LLM review.".to_string(),
            ],
            files: keyword_reports.iter().take(20).map(|p| relative_path_string(kb_path, p)).collect(),
            evidence: keyword_relation_evidence.iter().take(20).cloned().collect(),
            priority: "medium".to_string(),
        });
    }

    if wiki_pages.is_empty() {
        tasks.push(DeferredHealthTask {
            id: "health-wiki-drafting-001".to_string(),
            category: "wiki".to_string(),
            target_agent: "Wiki Drafting Worker".to_string(),
            goal: "Create initial reviewable wiki pages only after Manager LLM selects bounded source files.".to_string(),
            requirements: vec![
                "Use `kb topic tasks <topic>` or an explicit task file before editing wiki/.".to_string(),
                "Include source_files and source_ids front matter when possible.".to_string(),
                "Do not invent claims that are not supported by evidence.".to_string(),
            ],
            files: raw_papers.iter().take(20).map(|p| relative_path_string(kb_path, p)).collect(),
            evidence: vec!["wiki/ has no Markdown pages".to_string()],
            priority: "medium".to_string(),
        });
    }

    tasks
}

fn summarize_status(checks: &[HealthCheck]) -> (String, i32) {
    let warn_count = checks.iter().filter(|check| check.status == "warn").count() as i32;
    let blocker_count = checks
        .iter()
        .filter(|check| check.status == "blocker")
        .count() as i32;
    let score = (100 - warn_count * 8 - blocker_count * 25).max(0);
    let status = if blocker_count > 0 {
        "blocker"
    } else if warn_count > 0 {
        "warn"
    } else {
        "ok"
    };
    (status.to_string(), score)
}

fn write_markdown_report(kb_path: &Path, report: &HealthReport) -> Result<PathBuf> {
    let out_dir = kb_path.join("interfaces/reports");
    fs::create_dir_all(&out_dir)?;
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let out_path = out_dir.join(format!("health_{timestamp}.md"));
    fs::write(&out_path, render_markdown_report(report))?;
    Ok(out_path)
}

fn render_markdown_report(report: &HealthReport) -> String {
    let mut out = String::new();
    out.push_str("# LLM Wiki Health Report\n\n");
    out.push_str(&format!("- Generated at: `{}`\n", report.generated_at));
    out.push_str(&format!("- Status: `{}`\n", report.status));
    out.push_str(&format!("- Score: `{}`\n", report.score));
    out.push_str(&format!("- Knowledge base: `{}`\n\n", report.kb_path));

    out.push_str("## Counts\n\n");
    for (key, value) in &report.counts {
        out.push_str(&format!("- `{key}`: `{value}`\n"));
    }

    out.push_str("\n## Checks\n\n");
    for check in &report.checks {
        out.push_str(&format!("### {} — `{}`\n\n", check.id, check.status));
        out.push_str(&format!("{}\n\n", check.summary));
        if !check.evidence.is_empty() {
            out.push_str("Evidence:\n");
            for item in &check.evidence {
                out.push_str(&format!("- `{}`\n", item));
            }
            out.push('\n');
        }
        if !check.recommended_next.is_empty() {
            out.push_str("Recommended next:\n");
            for item in &check.recommended_next {
                out.push_str(&format!("- {}\n", item));
            }
            out.push('\n');
        }
    }

    out.push_str("## Deferred Task Hints\n\n");
    if report.deferred_tasks.is_empty() {
        out.push_str("No deferred task hints were generated by this health report.\n");
    } else {
        for task in &report.deferred_tasks {
            out.push_str(&format!("### {}\n\n", task.id));
            out.push_str(&format!("- Target agent: `{}`\n", task.target_agent));
            out.push_str(&format!("- Category: `{}`\n", task.category));
            out.push_str(&format!("- Priority: `{}`\n", task.priority));
            out.push_str(&format!("- Goal: {}\n", task.goal));
            out.push_str("- Requirements:\n");
            for item in &task.requirements {
                out.push_str(&format!("  - {}\n", item));
            }
            out.push_str("- Files:\n");
            for file in &task.files {
                out.push_str(&format!("  - `{}`\n", file));
            }
            if !task.evidence.is_empty() {
                out.push_str("- Evidence:\n");
                for item in &task.evidence {
                    out.push_str(&format!("  - `{}`\n", item));
                }
            }
            out.push('\n');
        }
    }

    out
}

fn print_report(report: &HealthReport) {
    println!(
        "LLM Wiki health: {} (score {})",
        report.status, report.score
    );
    println!("Counts:");
    for (key, value) in &report.counts {
        println!("  {key}: {value}");
    }
    println!();
    println!("Checks:");
    for check in &report.checks {
        println!("  [{}] {}: {}", check.status, check.id, check.summary);
    }
    if !report.deferred_tasks.is_empty() {
        println!();
        println!("Deferred task hints:");
        for task in &report.deferred_tasks {
            println!(
                "  - {} -> {} ({})",
                task.id, task.target_agent, task.priority
            );
        }
        println!("Use `kb tasks` for a consolidated LLM/tasks/ handoff list.");
    }
    if let Some(path) = &report.report_path {
        println!();
        println!("Health report written: {path}");
    } else if report.dry_run {
        println!();
        println!("Dry run: no health report file was written.");
    }
}

fn list_files_with_ext(root: &Path, exts: &[&str]) -> Vec<PathBuf> {
    if !root.exists() {
        return Vec::new();
    }
    let exts = exts
        .iter()
        .map(|ext| ext.to_ascii_lowercase())
        .collect::<BTreeSet<_>>();
    let mut files = Vec::new();
    for entry in WalkDir::new(root)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
    {
        let path = entry.path();
        let Some(ext) = path.extension().and_then(|s| s.to_str()) else {
            continue;
        };
        if exts.contains(&ext.to_ascii_lowercase()) {
            files.push(path.to_path_buf());
        }
    }
    files.sort();
    files
}

fn list_named_reports(root: &Path, prefix: &str, ext: &str) -> Vec<PathBuf> {
    if !root.exists() {
        return Vec::new();
    }
    list_files_with_ext(root, &[ext])
        .into_iter()
        .filter(|path| {
            path.file_name()
                .and_then(|s| s.to_str())
                .map(|name| name.starts_with(prefix))
                .unwrap_or(false)
        })
        .collect()
}

fn file_len(path: &Path) -> Option<u64> {
    fs::metadata(path).ok().map(|metadata| metadata.len())
}

fn scan_uncertain_refs_index(kb_path: &Path, reports: &[PathBuf]) -> Vec<String> {
    let mut evidence = Vec::new();
    for report in reports {
        let Ok(content) = fs::read_to_string(report) else {
            continue;
        };
        for line in content.lines() {
            if line.contains("| `candidate` |")
                || line.contains("| `ambiguous` |")
                || line.contains("| `missing` |")
            {
                evidence.push(format!(
                    "{}: {}",
                    relative_path_string(kb_path, report),
                    line.trim()
                ));
            }
        }
    }
    evidence
}

fn scan_keyword_candidates(kb_path: &Path, reports: &[PathBuf]) -> Vec<String> {
    let mut evidence = Vec::new();
    for report in reports {
        let Ok(content) = fs::read_to_string(report) else {
            continue;
        };
        for line in content.lines() {
            if line.contains("| `candidate` |") || line.contains("candidate keyword/topic") {
                evidence.push(format!(
                    "{}: {}",
                    relative_path_string(kb_path, report),
                    line.trim()
                ));
            }
        }
    }
    evidence
}

fn relative_path_string(kb_path: &Path, path: &Path) -> String {
    path.strip_prefix(kb_path)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}
