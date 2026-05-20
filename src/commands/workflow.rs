use anyhow::Result;
use serde::Serialize;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

pub const RAG_THRESHOLD: usize = 200;

#[derive(Debug, Clone, Serialize)]
pub struct WorkflowStatus {
    pub schema_version: String,
    pub generated_by: String,
    pub generated_at: String,
    pub paper_count: usize,
    pub rag_threshold: usize,
    pub mode: String,
    pub mode_label: String,
    pub mode_reason: String,
    pub topic: Option<String>,
    pub recommended_next_step: String,
    pub generated_guides: Vec<String>,
}

pub fn refresh_workflow_guides(
    kb_path: &Path,
    topic_slug: Option<&str>,
    topic_title: Option<&str>,
    force: bool,
) -> Result<WorkflowStatus> {
    let paper_count = count_raw_papers(kb_path);
    let is_small = paper_count < RAG_THRESHOLD;
    let mode = if is_small {
        "karpathy_llm_wiki"
    } else {
        "rag_assisted_required"
    };
    let mode_label = if is_small {
        "Karpathy-style LLM Wiki workflow"
    } else {
        "RAG-assisted LLM Wiki workflow required"
    };
    let mode_reason = if is_small {
        format!(
            "paper_count {} < rag_threshold {}",
            paper_count, RAG_THRESHOLD
        )
    } else {
        format!(
            "paper_count {} >= rag_threshold {}",
            paper_count, RAG_THRESHOLD
        )
    };
    let recommended_next_step = if is_small {
        "Assign Manager/Worker LLM tasks for paper key-info extraction, topic story narrative, and WikiLink construction; then route key judgments to Human Review.".to_string()
    } else {
        "Do not ask LLM workers to read the whole corpus directly. Build retrieval/chunk/evidence-pack infrastructure before topic narrative work.".to_string()
    };

    let mut status = WorkflowStatus {
        schema_version: "workflow.v0.7.38".to_string(),
        generated_by: "kb-cli workflow".to_string(),
        generated_at: chrono::Utc::now().to_rfc3339(),
        paper_count,
        rag_threshold: RAG_THRESHOLD,
        mode: mode.to_string(),
        mode_label: mode_label.to_string(),
        mode_reason,
        topic: topic_slug.map(|s| s.to_string()),
        recommended_next_step,
        generated_guides: Vec::new(),
    };

    ensure_workflow_dirs(kb_path)?;
    let status_path = kb_path.join("processing/workflow_status.json");
    fs::write(&status_path, serde_json::to_string_pretty(&status)?)?;
    status
        .generated_guides
        .push(relative_path_string(kb_path, &status_path));

    let workflow_path = kb_path.join("LLM/workflow.md");
    write_if_allowed(
        &workflow_path,
        &render_workflow_overview(&status, topic_title),
        force,
    )?;
    status
        .generated_guides
        .push(relative_path_string(kb_path, &workflow_path));

    if is_small {
        let specs = vec![
            (
                "LLM/tasks/manager_karpathy_small_corpus.md",
                render_small_corpus_manager_guide(&status, topic_title),
            ),
            (
                "LLM/tasks/worker_paper_extraction_karpathy_small_corpus.md",
                render_small_corpus_paper_worker_guide(&status, topic_title),
            ),
            (
                "LLM/tasks/worker_topic_narrative_karpathy_small_corpus.md",
                render_small_corpus_topic_worker_guide(&status, topic_title),
            ),
            (
                "LLM/tasks/human_review_karpathy_small_corpus.md",
                render_small_corpus_human_review_guide(&status, topic_title),
            ),
            (
                "agents/shared/karpathy-small-corpus-workflow.md",
                render_small_corpus_shared_agent_protocol(&status, topic_title),
            ),
        ];
        for (rel, content) in specs {
            let path = kb_path.join(rel);
            write_if_allowed(&path, &content, force)?;
            status.generated_guides.push(rel.to_string());
        }
    } else {
        let path = kb_path.join("LLM/tasks/rag_required_placeholder.md");
        write_if_allowed(
            &path,
            &render_rag_required_placeholder(&status, topic_title),
            force,
        )?;
        status
            .generated_guides
            .push(relative_path_string(kb_path, &path));
    }

    // Re-write generated files after all guide paths are known.
    fs::write(
        &workflow_path,
        render_workflow_overview(&status, topic_title),
    )?;
    fs::write(&status_path, serde_json::to_string_pretty(&status)?)?;
    Ok(status)
}

pub fn count_raw_papers(kb_path: &Path) -> usize {
    let raw_papers = kb_path.join("raw/papers");
    if !raw_papers.exists() {
        return 0;
    }
    WalkDir::new(raw_papers)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| {
            entry
                .path()
                .extension()
                .and_then(|s| s.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("pdf"))
                .unwrap_or(false)
        })
        .count()
}

fn ensure_workflow_dirs(kb_path: &Path) -> Result<()> {
    for rel in [
        "LLM",
        "LLM/tasks",
        "processing",
        "agents/shared",
        "agents/openclaw",
        "agents/claude-code",
        "agents/codex",
    ] {
        fs::create_dir_all(kb_path.join(rel))?;
    }
    Ok(())
}

fn write_if_allowed(path: &Path, content: &str, _force: bool) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content)?;
    Ok(())
}

fn render_workflow_overview(status: &WorkflowStatus, topic_title: Option<&str>) -> String {
    let topic = topic_title
        .or(status.topic.as_deref())
        .unwrap_or("<unspecified>");
    format!(
        r#"# LLM Wiki Workflow

## Current Mode

- Corpus size: **{paper_count} papers**
- RAG threshold: **{threshold} papers**
- Workflow mode: **{mode_label}**
- Reason: `{reason}`
- Topic: `{topic}`

## Product Positioning

`kb-cli` is not a PDF understanding engine. It is a deterministic workflow generator for LLM-assisted Wiki construction.

It prepares the workspace, indexes, scaffolds, task site, and human-facing views. It also issues task guides and checklists for LLM Managers, LLM Workers, and Human Reviewers so that paper understanding, topic narratives, and knowledge links can be completed collaboratively.

## Recommended Next Step

{next_step}

## Generated Guides

{guides}
"#,
        paper_count = status.paper_count,
        threshold = status.rag_threshold,
        mode_label = status.mode_label.as_str(),
        reason = status.mode_reason.as_str(),
        topic = topic,
        next_step = status.recommended_next_step.as_str(),
        guides = status
            .generated_guides
            .iter()
            .map(|p| format!("- `{}`", p))
            .collect::<Vec<_>>()
            .join("\n")
    )
}

fn render_small_corpus_manager_guide(status: &WorkflowStatus, topic_title: Option<&str>) -> String {
    let topic = topic_title.or(status.topic.as_deref()).unwrap_or("<topic>");
    format!(
        r#"# Manager Guide — Karpathy-style LLM Wiki Workflow

## Scope

This guide applies because the corpus contains **{paper_count} papers**, which is below the RAG threshold of **{threshold}**.

The Manager may organize direct LLM-assisted Wiki compilation from the local files, paper stubs, topic scaffolds, and deterministic reports. Do not treat Rust PDF extraction as semantically complete.

## Topic

`{topic}`

## Manager Mission

1. Read `llm-wiki.toml`, `LLM/workflow.md`, `rules/`, and `LLM/tasks/index.md` first.
2. Assign Worker tasks in small batches.
3. Make Workers extract paper key information into `processing/paper_profiles/` and `wiki/papers/`.
4. Make Workers build a topic story narrative in `wiki/topics/`.
5. Require WikiLinks from topic claims to paper pages.
6. Send uncertain relation claims, anchor-paper choices, and conflicting interpretations to Human Review.
7. Never ask a Worker to overwrite `raw/`.

## Required Outputs

- Completed paper profiles for the topic-relevant papers.
- A concise topic narrative with links to paper pages.
- A list of unresolved or human-review-required claims.
- A brief session note under `LLM/memory/` after each accepted batch.

## Human Review Gates

Use Human Review before approving:

- anchor papers;
- topic storyline claims;
- relation edges promoted from candidate to confirmed;
- merges into reviewed Wiki pages;
- any change to `rules/`.
"#,
        paper_count = status.paper_count,
        threshold = status.rag_threshold,
        topic = topic,
    )
}

fn render_small_corpus_paper_worker_guide(
    status: &WorkflowStatus,
    topic_title: Option<&str>,
) -> String {
    let topic = topic_title.or(status.topic.as_deref()).unwrap_or("<topic>");
    format!(
        r#"# Worker Guide — Paper Key Information Extraction

## Scope

Corpus size is **{paper_count} papers** (`< {threshold}`), so a Worker may process paper stubs directly in small batches.

## Topic Context

`{topic}`

## Inputs

- `raw/papers/*.pdf`
- `processing/text/*.txt` when available
- `processing/sections/*/introduction.txt` when available
- `wiki/papers/*.md` paper stubs
- `processing/paper_profiles/` target directory

## Required Extraction Fields

For each assigned paper, extract or explicitly mark missing:

1. title
2. authors
3. journal / venue
4. year
5. DOI if visible
6. abstract
7. keywords
8. introduction summary or introduction text excerpt
9. references section status
10. topic relevance notes

## Output Rules

- Write durable structured notes to `processing/paper_profiles/<paper_id>.md`.
- Update the matching `wiki/papers/<paper_id>.md` only when the task authorizes it.
- Preserve evidence notes such as page/section/paragraph location.
- Do not invent missing metadata.
- Do not edit `raw/`.

## Minimum Paper Profile Shape

```markdown
# <Paper Title>

## Key Information

| Field | Value | Evidence | Status |
|---|---|---|---|
| Title | ... | page 1 title block | extracted |
| Authors | ... | page 1 title block | extracted |
| Journal | ... | page/header/DOI page | needs review |

## Abstract

...

## Keywords

...

## Introduction Notes

...

## Topic Links

- Related topic: [[topics/{topic}]]

## Review Notes

- Human review required: yes/no
```
"#,
        paper_count = status.paper_count,
        threshold = status.rag_threshold,
        topic = topic,
    )
}

fn render_small_corpus_topic_worker_guide(
    status: &WorkflowStatus,
    topic_title: Option<&str>,
) -> String {
    let topic = topic_title.or(status.topic.as_deref()).unwrap_or("<topic>");
    format!(
        r#"# Worker Guide — Topic Story Narrative

## Scope

Corpus size is **{paper_count} papers** (`< {threshold}`), so the topic narrative can be constructed from the paper profiles, topic workspace, and relation candidates without a full RAG layer.

## Topic

`{topic}`

## Inputs

- `wiki/papers/*.md`
- `processing/paper_profiles/*.md`
- `topics/{topic}/literature.md`
- `topics/{topic}/graph/topic_graph.md`
- `processing/refs/refs_index_*.md`
- `processing/refs/refs_graph_*.json`

## Required Narrative Sections

1. Background and motivation
2. Core scientific / technical question
3. Key papers and why they matter
4. Method or idea evolution
5. Relationship between papers
6. Controversies or unresolved gaps
7. Recommended next reading path
8. Human review checklist

## Linking Rules

- Every important claim should link to at least one paper page, e.g. `[[papers/<paper_id>]]`.
- Use `[[topics/{topic}]]` for topic-level backlinks.
- Mark weak or inferred links as `needs_human_review` instead of treating them as final.

## Output

Write or update:

- `wiki/topics/{topic}.md`
- optional draft notes under `topics/{topic}/drafts/`
- human-review notes under `topics/{topic}/review/`
"#,
        paper_count = status.paper_count,
        threshold = status.rag_threshold,
        topic = topic,
    )
}

fn render_small_corpus_human_review_guide(
    status: &WorkflowStatus,
    topic_title: Option<&str>,
) -> String {
    let topic = topic_title.or(status.topic.as_deref()).unwrap_or("<topic>");
    format!(
        r#"# Human Review Checklist — Karpathy-style Small Corpus

## Scope

Corpus size is **{paper_count} papers** (`< {threshold}`), so Human Review focuses on semantic correctness rather than retrieval coverage.

## Topic

`{topic}`

## Review Items

- [ ] Confirm anchor papers for the topic.
- [ ] Confirm whether extracted titles/authors/journals are correct for important papers.
- [ ] Confirm that topic narrative claims are supported by linked papers.
- [ ] Confirm which relation candidates should become confirmed edges.
- [ ] Mark weak links, ambiguous claims, and missing papers for follow-up.
- [ ] Approve or reject Worker edits before treating pages as reviewed.

## Human Decision Outputs

Write durable decisions to one of:

- `topics/{topic}/review/`
- `LLM/memory/completed_tasks.md`
- future `state/human_tasks/archive/` once the Human Inbox becomes interactive.

## Do Not Review Through Interface Artifacts Alone

`interfaces/html/` pages are convenient views. They are not the source of truth. After making a decision, ensure the result is written back to Markdown/JSON/TOML under the durable workspace.
"#,
        paper_count = status.paper_count,
        threshold = status.rag_threshold,
        topic = topic,
    )
}

fn render_small_corpus_shared_agent_protocol(
    status: &WorkflowStatus,
    topic_title: Option<&str>,
) -> String {
    let topic = topic_title.or(status.topic.as_deref()).unwrap_or("<topic>");
    format!(
        r#"# Shared Agent Protocol — Small Corpus LLM Wiki

This protocol is shared by OpenClaw, Claude Code, Codex, and future Manager/Worker agents.

## Mode

- Mode: `{mode}`
- Corpus size: `{paper_count}`
- Threshold: `{threshold}`
- Topic: `{topic}`

## Principle

For fewer than 200 papers, use a Karpathy-style LLM Wiki workflow:

1. keep source materials in `raw/`;
2. build reviewable Markdown knowledge pages under `wiki/`;
3. use `topics/` for topic-specific relationship and importance work;
4. use `LLM/tasks/` and `LLM/handoff/` to coordinate Manager/Worker work;
5. use `kb view` and `kb view --wiki` as human-facing review and reading entrances.

## Agent Separation

Agent-specific files may live under their own directories:

- `agents/openclaw/`
- `agents/claude-code/`
- `agents/codex/`

Shared protocol files may live under `agents/shared/`.

## Required Safety Boundary

Agents must not rewrite `raw/`. They may propose or edit `wiki/`, `processing/`, and topic workspaces only inside a bounded task.
"#,
        mode = status.mode.as_str(),
        paper_count = status.paper_count,
        threshold = status.rag_threshold,
        topic = topic,
    )
}

fn render_rag_required_placeholder(status: &WorkflowStatus, topic_title: Option<&str>) -> String {
    let topic = topic_title.or(status.topic.as_deref()).unwrap_or("<topic>");
    format!(
        r#"# RAG-assisted Workflow Required

The corpus contains **{paper_count} papers**, which is at or above the threshold of **{threshold}**.

Topic: `{topic}`

This version records the routing decision but does not yet implement the RAG workflow. Do not ask LLM Workers to read the complete corpus directly. The next implementation phase should add chunk manifests, retrieval index preparation, and topic evidence packs.
"#,
        paper_count = status.paper_count,
        threshold = status.rag_threshold,
        topic = topic,
    )
}

fn relative_path_string(kb_path: &Path, path: &Path) -> String {
    path.strip_prefix(kb_path)
        .unwrap_or(path)
        .display()
        .to_string()
        .replace('\\', "/")
}
