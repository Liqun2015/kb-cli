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
        schema_version: "workflow.v0.7.42".to_string(),
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
            (
                "agents/openclaw/README.md",
                render_openclaw_readme(&status, topic_title),
            ),
            (
                "agents/openclaw/manager.md",
                render_openclaw_manager_guide(&status, topic_title),
            ),
            (
                "agents/openclaw/create-topic-library.md",
                render_openclaw_create_topic_library(&status, topic_title),
            ),
            (
                "agents/openclaw/daily-maintenance.md",
                render_openclaw_daily_maintenance(&status, topic_title),
            ),
            (
                "agents/openclaw/worker-dispatch.md",
                render_openclaw_worker_dispatch(&status, topic_title),
            ),
            (
                "agents/openclaw/start-prompt.md",
                render_openclaw_start_prompt(&status, topic_title),
            ),
            (
                "LLM/tasks/openclaw_manager_create_and_maintain.md",
                render_openclaw_manager_task(&status, topic_title),
            ),
            (
                "agents/claude-code/README.md",
                render_claude_code_readme(&status, topic_title),
            ),
            (
                "agents/claude-code/manager.md",
                render_claude_code_manager_guide(&status, topic_title),
            ),
            (
                "agents/claude-code/create-topic-library.md",
                render_claude_code_create_topic_library(&status, topic_title),
            ),
            (
                "agents/claude-code/daily-maintenance.md",
                render_claude_code_daily_maintenance(&status, topic_title),
            ),
            (
                "agents/claude-code/worker-dispatch.md",
                render_claude_code_worker_dispatch(&status, topic_title),
            ),
            (
                "agents/claude-code/start-prompt.md",
                render_claude_code_start_prompt(&status, topic_title),
            ),
            (
                "LLM/tasks/claude_code_manager_create_and_maintain.md",
                render_claude_code_manager_task(&status, topic_title),
            ),
            (
                "CLAUDE.md",
                render_claude_code_root_entry(&status, topic_title),
            ),
            (
                "agents/codex/README.md",
                render_codex_readme(&status, topic_title),
            ),
            (
                "agents/codex/manager.md",
                render_codex_manager_guide(&status, topic_title),
            ),
            (
                "agents/codex/create-topic-library.md",
                render_codex_create_topic_library(&status, topic_title),
            ),
            (
                "agents/codex/daily-maintenance.md",
                render_codex_daily_maintenance(&status, topic_title),
            ),
            (
                "agents/codex/worker-dispatch.md",
                render_codex_worker_dispatch(&status, topic_title),
            ),
            (
                "agents/codex/start-prompt.md",
                render_codex_start_prompt(&status, topic_title),
            ),
            (
                "agents/codex/skills/llm-wiki-manager/SKILL.md",
                render_codex_llm_wiki_manager_skill(&status, topic_title),
            ),
            (
                "LLM/tasks/codex_manager_create_and_maintain.md",
                render_codex_manager_task(&status, topic_title),
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
5. use `kb check` and `kb view` as human-facing review and reading entrances.

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

fn render_openclaw_readme(status: &WorkflowStatus, topic_title: Option<&str>) -> String {
    let topic = topic_title.or(status.topic.as_deref()).unwrap_or("<topic>");
    format!(
        r#"# OpenClaw Entry — LLM Wiki Manager

This directory is the OpenClaw-specific entry point. It is intentionally separate from `agents/claude-code/` and `agents/codex/` so that each agent keeps its own habits, prompts, and operational boundaries.

## Current Workspace Mode

- Mode: `{mode}`
- Corpus size: `{paper_count}` papers
- RAG threshold: `{threshold}` papers
- Current topic: `{topic}`

Because this version focuses on the `< 200 papers` case, OpenClaw should use the Karpathy-style LLM Wiki workflow: inspect the deterministic workspace, assign bounded Worker tasks, collect outputs, and route high-value judgments to Human Review.

## Read Order

1. `agents/openclaw/README.md` — this file.
2. `agents/openclaw/manager.md` — Manager role and boundaries.
3. `agents/openclaw/create-topic-library.md` — how to establish a topic library.
4. `agents/openclaw/daily-maintenance.md` — routine maintenance/update checklist.
5. `agents/openclaw/worker-dispatch.md` — how to call or brief Worker LLMs.
6. `LLM/workflow.md`.
7. `LLM/tasks/openclaw_manager_create_and_maintain.md`.
8. `AGENTS.md` and `LLM/handoff/current.md` for shared contracts.

## Default Manager Rule

OpenClaw is the Manager. It should not behave like an unrestricted autonomous editor. It should first call deterministic `kb` commands, then assign one bounded task at a time to a Worker LLM, and finally request Human Review for key academic decisions.

## Safe First Command

```bash
kb --wiki <workspace> check
```

Then inspect:

- `interfaces/html/index.html`
- `interfaces/html/browse.html`
- `LLM/tasks/index.md`
- `agents/openclaw/daily-maintenance.md`
"#,
        mode = status.mode.as_str(),
        paper_count = status.paper_count,
        threshold = status.rag_threshold,
        topic = topic,
    )
}

fn render_openclaw_manager_guide(status: &WorkflowStatus, topic_title: Option<&str>) -> String {
    let topic = topic_title.or(status.topic.as_deref()).unwrap_or("<topic>");
    format!(
        r#"# OpenClaw Manager Guide

## Identity

You are the OpenClaw Manager for this LLM Wiki workspace. Your job is not to write the whole Wiki yourself. Your job is to coordinate `kb-cli`, Worker LLMs, and Human Review.

## Current Mode

- Corpus size: **{paper_count} papers**
- Threshold: **{threshold} papers**
- Workflow mode: **{mode_label}**
- Topic: `{topic}`

## Manager Duties

1. Use `kb create --wiki <workspace> --from <literature-folder> --about <topic>` to establish or refresh a topic library.
2. Use deterministic commands first: `kb check`, `kb view`, `kb tasks`, `kb topic list`, `kb topic status <topic>`.
3. Read task lists before assigning work.
4. Assign Worker LLM tasks in small batches.
5. Keep Worker output inside permitted files.
6. Ask Human Review to approve anchor papers, topic narratives, confirmed relation edges, and merges into reviewed pages.
7. Record accepted work under `LLM/memory/` or topic review notes.

## Manager Must Not

- Do not modify `raw/`.
- Do not let Worker LLMs invent missing title, author, DOI, abstract, or references.
- Do not convert relation candidates to confirmed relations without evidence and review.
- Do not replace `kb-cli` deterministic checks with free-form guesses.
- Do not mix OpenClaw prompts with Claude Code or Codex prompts.

## Standard Loop

```text
observe workspace
→ run deterministic kb command
→ inspect generated task/interface files
→ assign one Worker task
→ review Worker output
→ route high-value judgments to Human Review
→ record accepted result
→ refresh kb check / kb view
```
"#,
        paper_count = status.paper_count,
        threshold = status.rag_threshold,
        mode_label = status.mode_label.as_str(),
        topic = topic,
    )
}

fn render_openclaw_create_topic_library(
    status: &WorkflowStatus,
    topic_title: Option<&str>,
) -> String {
    let topic = topic_title.or(status.topic.as_deref()).unwrap_or("<topic>");
    format!(
        r#"# OpenClaw Procedure — Create a Topic Library

This procedure tells OpenClaw how to establish a topic library from a literature folder.

## Canonical Command

```bash
kb create --wiki <workspace> --from <literature-folder> --about "{topic}"
```

## What This Command Means

`kb create` creates the LLM Wiki worksite. It does not semantically finish the Wiki. It creates:

- directory skeletons;
- raw literature registry;
- paper stubs;
- topic workspace;
- deterministic indexes;
- LLM Manager/Worker task guides;
- Human Review checklists;
- `kb check` and `kb view` HTML entrances.

## OpenClaw After-Create Checklist

1. Run or verify `kb create`.
2. Open `processing/workflow_status.json` and confirm `paper_count < 200` for this workflow.
3. Open `LLM/workflow.md`.
4. Open `LLM/tasks/index.md`.
5. Open `LLM/tasks/openclaw_manager_create_and_maintain.md`.
6. Open `interfaces/html/index.html` for the control console.
7. Open `interfaces/html/browse.html` for the research issue browser.
8. Assign the first Worker batch for paper key-info extraction.
9. Ask Human Review to choose or approve anchor papers before finalizing the topic story.

## First Worker Batch

Start with 3 to 10 topic-relevant papers. Each Worker task should extract:

- title;
- authors;
- journal / venue;
- year;
- DOI when visible;
- abstract;
- keywords;
- introduction notes;
- topic relevance notes;
- links to other paper pages.

Output should go to:

- `processing/paper_profiles/<paper_id>.md`
- `wiki/papers/<paper_id>.md` only when explicitly allowed.
"#,
        topic = topic,
    )
}

fn render_openclaw_daily_maintenance(status: &WorkflowStatus, topic_title: Option<&str>) -> String {
    let topic = topic_title.or(status.topic.as_deref()).unwrap_or("<topic>");
    format!(
        r#"# OpenClaw Daily Maintenance Protocol

This file is the daily maintenance checklist for OpenClaw Manager.

## Current Scope

- Topic: `{topic}`
- Corpus size: `{paper_count}` papers
- Workflow mode: `{mode_label}`

## Daily Start

1. Read `agents/openclaw/README.md`.
2. Read `LLM/workflow.md`.
3. Read `LLM/tasks/index.md` if present.
4. Run or request:

```bash
kb --wiki <workspace> check
kb --wiki <workspace> view
kb --wiki <workspace> tasks
kb --wiki <workspace> task list
kb --wiki <workspace> topic status {topic}
```

## Maintenance Queue

Handle only one queue item at a time:

1. New PDFs added to `raw/papers/` or source folder.
2. Paper stubs needing LLM extraction.
   - Create a small batch with `kb --wiki <workspace> batch paper-profile --topic {topic} --limit 5 --assignee openclaw-manager`.
   - Print the Worker prompt with `kb --wiki <workspace> prompt paper-profile --task <batch-id> --topic {topic}`.
   - Mark progress with `kb --wiki <workspace> task status <batch-id> --mark assigned|in_progress|completed|blocked`.
3. Paper profiles needing Human Review.
4. Topic narrative sections needing linked evidence.
5. Relation candidates needing evidence checks.
6. Broken WikiLinks or missing backlinks.
7. Outdated task files or handoff files.

## Update Rules

- If new literature appears, rerun `kb create --wiki <workspace> --from <literature-folder> --about "{topic}"` or the equivalent deterministic refresh command.
- If the corpus count reaches `{threshold}` or more, stop the direct small-corpus workflow and request RAG-assisted workflow preparation.
- After Worker edits, mark task progress with `kb task status <id> --mark <state>`.
- After accepted Worker outputs, refresh `kb check` and `kb view`.
- Record accepted maintenance under `LLM/memory/` after the task is marked completed.

## End-of-Day Note Template

```markdown
# OpenClaw Maintenance Note — <date>

## Commands Run

## Tasks Assigned

## Worker Outputs Accepted

## Human Review Needed

## Files Changed

## Next Maintenance Step
```
"#,
        topic = topic,
        paper_count = status.paper_count,
        mode_label = status.mode_label.as_str(),
        threshold = status.rag_threshold,
    )
}

fn render_openclaw_worker_dispatch(status: &WorkflowStatus, topic_title: Option<&str>) -> String {
    let topic = topic_title.or(status.topic.as_deref()).unwrap_or("<topic>");
    format!(
        r#"# OpenClaw Worker Dispatch Template

OpenClaw Manager may call a Worker LLM, but every Worker task must be bounded.

## Preferred Dispatch Commands

```bash
kb --wiki <workspace> batch paper-profile --topic {topic} --limit 5 --assignee openclaw-manager
kb --wiki <workspace> prompt paper-profile --task <batch-id> --topic {topic}
kb --wiki <workspace> task status <batch-id> --mark assigned --assignee <worker>
```

## Worker Brief Template

```text
You are a Worker LLM for an LLM Wiki.

Topic: {topic}
Corpus mode: {mode_label}

Read only the files listed below.
Work only on the allowed output files.
Do not modify raw/.
Do not invent missing metadata.
Mark uncertain claims as needs_human_review.

Task:
<one concrete task>

Input files:
- <file 1>
- <file 2>

Allowed output files:
- <output 1>

Return:
1. work summary;
2. files changed;
3. evidence used;
4. unresolved uncertainty;
5. human review needs.
```

## Good Worker Tasks

- Extract key information for one paper or a small batch.
- Draft one section of `wiki/topics/{topic}.md` with links to paper pages.
- Check one relation candidate and return evidence.
- Fix one small group of broken WikiLinks.

## Bad Worker Tasks

- Read the whole corpus and write the whole Wiki.
- Rewrite all topic pages.
- Confirm all relation candidates.
- Delete or reorganize `raw/`.
"#,
        topic = topic,
        mode_label = status.mode_label.as_str(),
    )
}

fn render_openclaw_start_prompt(status: &WorkflowStatus, topic_title: Option<&str>) -> String {
    let topic = topic_title.or(status.topic.as_deref()).unwrap_or("<topic>");
    format!(
        r#"You are OpenClaw acting as the Manager LLM for an LLM Wiki workspace.

First, read these files:

1. agents/openclaw/README.md
2. agents/openclaw/manager.md
3. agents/openclaw/create-topic-library.md
4. agents/openclaw/daily-maintenance.md
5. LLM/workflow.md
6. LLM/tasks/openclaw_manager_create_and_maintain.md
7. AGENTS.md
8. LLM/handoff/current.md

Current topic: {topic}
Current mode: {mode_label}
Corpus size: {paper_count} papers

Do not modify files yet. Return:

1. current workspace state;
2. whether this is still a <200-paper Karpathy-style workflow;
3. the top task queue;
4. the safest next three actions;
5. which decisions require Human Review.
"#,
        topic = topic,
        mode_label = status.mode_label.as_str(),
        paper_count = status.paper_count,
    )
}

fn render_openclaw_manager_task(status: &WorkflowStatus, topic_title: Option<&str>) -> String {
    let topic = topic_title.or(status.topic.as_deref()).unwrap_or("<topic>");
    format!(
        r#"# OpenClaw Manager Task — Create and Maintain Topic Library

## Goal

Use OpenClaw as the Manager LLM to establish and maintain this topic library.

## Topic

`{topic}`

## Mode

- `{mode_label}`
- Paper count: `{paper_count}`
- RAG threshold: `{threshold}`

## Required Manager Actions

1. Confirm the workspace was created by `kb create --wiki <workspace> --from <literature-folder> --about "{topic}"`.
2. Confirm `processing/workflow_status.json` reports `< {threshold}` papers.
3. Use `kb check` as the control console and `kb view` as the reader-facing research issue browser.
4. Assign Worker LLM tasks for paper key-info extraction.
5. Assign Worker LLM tasks for topic narrative drafting after enough paper profiles exist.
6. Route anchor-paper choices, confirmed relation edges, and final topic narrative claims to Human Review.
7. Record accepted work under `LLM/memory/`.
8. Keep Manager-specific notes inside that Manager adapter directory (`agents/openclaw/`, `agents/claude-code/`, or `agents/codex/`) or assigned task outputs.

## Forbidden Actions

- Do not treat Rust PDF text extraction as complete paper understanding.
- Do not ask Workers to process the whole corpus in one prompt.
- Do not modify `raw/`.
- Do not mix prompts across `agents/openclaw/`, `agents/claude-code/`, and `agents/codex/`.

## Done When

- Paper profiles exist for the topic-relevant papers.
- `wiki/topics/{topic}.md` contains a concise linked topic story.
- Important claims link to paper pages.
- Human Review has approved anchor papers and high-value judgments.
- `kb view` shows a readable topic-centered Wiki.
"#,
        topic = topic,
        mode_label = status.mode_label.as_str(),
        paper_count = status.paper_count,
        threshold = status.rag_threshold,
    )
}

fn render_claude_code_readme(status: &WorkflowStatus, topic_title: Option<&str>) -> String {
    let topic = topic_title.or(status.topic.as_deref()).unwrap_or("<topic>");
    format!(
        r#"# Claude Code Entry — LLM Wiki Manager

This directory is the Claude Code-specific entry point. It is intentionally separate from `agents/openclaw/` and `agents/codex/` so that each Manager agent keeps its own prompt style, habits, and operational boundary.

Claude Code may serve as the **LLM Manager** for this workspace. In that role, it should coordinate `kb-cli`, bounded Worker prompts, and Human Review. It should not behave as an unconstrained all-files editor.

## Current Workspace Mode

- Mode: `{mode}`
- Corpus size: `{paper_count}` papers
- RAG threshold: `{threshold}` papers
- Current topic: `{topic}`

This version focuses on the `< 200 papers` case. Claude Code should use the Karpathy-style LLM Wiki workflow: inspect deterministic reports, create small Worker batches, generate reusable prompts, review outputs, and route high-value decisions to Human Review.

## Claude Code Read Order

1. `CLAUDE.md` — root Claude Code compatibility entry.
2. `AGENTS.md` — generic external-agent contract.
3. `agents/claude-code/README.md` — this file.
4. `agents/claude-code/manager.md` — Manager identity and boundaries.
5. `agents/claude-code/create-topic-library.md` — how to establish a topic library.
6. `agents/claude-code/daily-maintenance.md` — routine maintenance/update checklist.
7. `agents/claude-code/worker-dispatch.md` — how to create Worker prompts and small batches.
8. `LLM/workflow.md`.
9. `LLM/tasks/claude_code_manager_create_and_maintain.md`.

## Safe First Command

```bash
kb --wiki <workspace> check
```

Then inspect:

- `interfaces/html/index.html`
- `interfaces/html/browse.html`
- `LLM/tasks/index.md`
- `agents/claude-code/daily-maintenance.md`
"#,
        mode = status.mode.as_str(),
        paper_count = status.paper_count,
        threshold = status.rag_threshold,
        topic = topic,
    )
}

fn render_claude_code_manager_guide(status: &WorkflowStatus, topic_title: Option<&str>) -> String {
    let topic = topic_title.or(status.topic.as_deref()).unwrap_or("<topic>");
    format!(
        r#"# Claude Code Manager Guide

## Identity

You are Claude Code acting as the Manager LLM for this LLM Wiki workspace.

Your job is not to complete the whole Wiki in one pass. Your job is to coordinate deterministic `kb-cli` commands, bounded Worker tasks, and Human Review so that paper understanding, topic narratives, and Wiki links are completed safely.

## Current Mode

- Corpus size: **{paper_count} papers**
- Threshold: **{threshold} papers**
- Workflow mode: **{mode_label}**
- Topic: `{topic}`

## Manager Duties

1. Read `CLAUDE.md`, `AGENTS.md`, `LLM/workflow.md`, `LLM/tasks/index.md`, and `agents/claude-code/` before editing.
2. Use deterministic commands first: `kb check`, `kb view`, `kb tasks`, `kb task list`, `kb topic status <topic>`.
3. Create Worker batches using `kb batch paper-profile --topic <topic> --limit 5 --assignee claude-code-manager`.
4. Generate standard Worker prompts using `kb prompt paper-profile --task <batch-id> --topic <topic>`.
5. Track progress with `kb task status <id> --mark <state>`.
6. Keep Worker output inside explicitly allowed files.
7. Ask Human Review to approve anchor papers, topic narratives, confirmed relation edges, and merges into reviewed pages.
8. Record accepted work under `LLM/memory/` or topic review notes.

## Claude Code Must Not

- Do not modify `raw/`.
- Do not invent missing title, author, DOI, abstract, references, or scientific claims.
- Do not convert relation candidates to confirmed relations without evidence and review.
- Do not let a Worker process the whole corpus in one prompt.
- Do not mix Claude Code instructions with `agents/openclaw/` or `agents/codex/` prompts.
- Do not treat `interfaces/html/` as a durable source of truth.

## Standard Loop

```text
observe workspace
→ run deterministic kb command
→ inspect generated task/interface files
→ create one small Worker batch or one bounded task
→ generate a reusable Worker prompt
→ review Worker output
→ route high-value judgments to Human Review
→ mark task status
→ record accepted result
→ refresh kb check / kb view
```
"#,
        paper_count = status.paper_count,
        threshold = status.rag_threshold,
        mode_label = status.mode_label.as_str(),
        topic = topic,
    )
}

fn render_claude_code_create_topic_library(
    status: &WorkflowStatus,
    topic_title: Option<&str>,
) -> String {
    let topic = topic_title.or(status.topic.as_deref()).unwrap_or("<topic>");
    format!(
        r#"# Claude Code Procedure — Create a Topic Library

This procedure tells Claude Code how to establish or refresh a topic library as the Manager LLM.

## Canonical Command

```bash
kb create --wiki <workspace> --from <literature-folder> --about "{topic}"
```

## What This Command Means

`kb create` creates the LLM Wiki worksite. It does not semantically finish the Wiki. It creates:

- directory skeletons;
- raw literature registry;
- paper stubs;
- topic workspace;
- deterministic indexes;
- Manager/Worker task guides;
- Human Review checklists;
- `kb check` and `kb view` HTML entrances.

## Claude Code After-Create Checklist

1. Verify `kb create` completed.
2. Open `processing/workflow_status.json` and confirm the workflow mode.
3. Open `LLM/workflow.md`.
4. Open `LLM/tasks/index.md`.
5. Open `LLM/tasks/claude_code_manager_create_and_maintain.md`.
6. Open `interfaces/html/index.html` for the control console.
7. Open `interfaces/html/browse.html` for the research issue browser.
8. Create the first paper-profile Worker batch.
9. Ask Human Review to choose or approve anchor papers before finalizing the topic story.

## First Worker Batch

```bash
kb --wiki <workspace> batch paper-profile --topic "{topic}" --limit 5 --assignee claude-code-manager
kb --wiki <workspace> prompt paper-profile --task <batch-id> --topic "{topic}"
```

Output should go to:

- `processing/paper_profiles/<paper_id>.md`
- `wiki/papers/<paper_id>.md` only when explicitly allowed.
"#,
        topic = topic,
    )
}

fn render_claude_code_daily_maintenance(
    status: &WorkflowStatus,
    topic_title: Option<&str>,
) -> String {
    let topic = topic_title.or(status.topic.as_deref()).unwrap_or("<topic>");
    format!(
        r#"# Claude Code Daily Maintenance Protocol

This file is the daily maintenance checklist for Claude Code Manager.

## Current Scope

- Topic: `{topic}`
- Corpus size: `{paper_count}` papers
- Workflow mode: `{mode_label}`

## Daily Start

1. Read `CLAUDE.md`.
2. Read `AGENTS.md`.
3. Read `agents/claude-code/README.md`.
4. Read `LLM/workflow.md`.
5. Read `LLM/tasks/index.md` if present.
6. Run or request:

```bash
kb --wiki <workspace> check
kb --wiki <workspace> view
kb --wiki <workspace> tasks
kb --wiki <workspace> task list
kb --wiki <workspace> topic status {topic}
```

## Maintenance Queue

Handle only one queue item at a time:

1. New PDFs added to `raw/papers/` or source folder.
2. Paper stubs needing LLM extraction.
   - Create a small batch with `kb --wiki <workspace> batch paper-profile --topic {topic} --limit 5 --assignee claude-code-manager`.
   - Print the Worker prompt with `kb --wiki <workspace> prompt paper-profile --task <batch-id> --topic {topic}`.
   - Mark progress with `kb --wiki <workspace> task status <batch-id> --mark assigned|in_progress|completed|blocked`.
3. Paper profiles needing Human Review.
4. Topic narrative sections needing linked evidence.
5. Relation candidates needing evidence checks.
6. Broken WikiLinks or missing backlinks.
7. Outdated task files or handoff files.

## Update Rules

- If new literature appears, rerun `kb create --wiki <workspace> --from <literature-folder> --about "{topic}"` or the equivalent deterministic refresh command.
- If the corpus count reaches `{threshold}` or more, stop the direct small-corpus workflow and request RAG-assisted workflow preparation.
- After Worker edits, mark task progress with `kb task status <id> --mark <state>`.
- After accepted Worker outputs, refresh `kb check` and `kb view`.
- Record accepted maintenance under `LLM/memory/` after the task is marked completed.

## End-of-Day Note Template

```markdown
# Claude Code Maintenance Note — <date>

## Commands Run

## Tasks Assigned

## Worker Outputs Accepted

## Human Review Needed

## Files Changed

## Next Maintenance Step
```
"#,
        topic = topic,
        paper_count = status.paper_count,
        mode_label = status.mode_label.as_str(),
        threshold = status.rag_threshold,
    )
}

fn render_claude_code_worker_dispatch(
    status: &WorkflowStatus,
    topic_title: Option<&str>,
) -> String {
    let topic = topic_title.or(status.topic.as_deref()).unwrap_or("<topic>");
    format!(
        r#"# Claude Code Worker Dispatch Template

Claude Code Manager may dispatch work to another LLM session or a bounded subtask, but every Worker task must be limited and auditable.

## Preferred Dispatch Commands

```bash
kb --wiki <workspace> batch paper-profile --topic {topic} --limit 5 --assignee claude-code-manager
kb --wiki <workspace> prompt paper-profile --task <batch-id> --topic {topic}
kb --wiki <workspace> task status <batch-id> --mark assigned --assignee claude-code-manager
```

## Worker Brief Template

```text
You are a Worker LLM for an LLM Wiki.

Topic: {topic}
Corpus mode: {mode_label}
Manager: Claude Code

Read only the files listed below.
Work only on the allowed output files.
Do not modify raw/.
Do not invent missing metadata.
Mark uncertain claims as needs_human_review.

Task:
<one concrete task>

Input files:
- <file 1>
- <file 2>

Allowed output files:
- <output 1>

Return:
1. work summary;
2. files changed;
3. evidence used;
4. unresolved uncertainty;
5. human review needs.
```

## Good Worker Tasks

- Extract key information for one paper or a small batch.
- Draft one section of `wiki/topics/{topic}.md` with links to paper pages.
- Check one relation candidate and return evidence.
- Fix one small group of broken WikiLinks.

## Bad Worker Tasks

- Read the whole corpus and write the whole Wiki.
- Rewrite all topic pages.
- Confirm all relation candidates.
- Delete or reorganize `raw/`.
"#,
        topic = topic,
        mode_label = status.mode_label.as_str(),
    )
}

fn render_claude_code_start_prompt(status: &WorkflowStatus, topic_title: Option<&str>) -> String {
    let topic = topic_title.or(status.topic.as_deref()).unwrap_or("<topic>");
    format!(
        r#"You are Claude Code acting as the Manager LLM for an LLM Wiki workspace.

First, read these files:

1. CLAUDE.md
2. AGENTS.md
3. agents/claude-code/README.md
4. agents/claude-code/manager.md
5. agents/claude-code/create-topic-library.md
6. agents/claude-code/daily-maintenance.md
7. LLM/workflow.md
8. LLM/tasks/claude_code_manager_create_and_maintain.md
9. LLM/handoff/current.md

Current topic: {topic}
Current mode: {mode_label}
Corpus size: {paper_count} papers

Do not modify files yet. Return:

1. current workspace state;
2. whether this is still a <200-paper Karpathy-style workflow;
3. the top task queue;
4. the safest next three actions;
5. which decisions require Human Review;
6. which `kb task status` updates are needed.
"#,
        topic = topic,
        mode_label = status.mode_label.as_str(),
        paper_count = status.paper_count,
    )
}

fn render_claude_code_manager_task(status: &WorkflowStatus, topic_title: Option<&str>) -> String {
    let topic = topic_title.or(status.topic.as_deref()).unwrap_or("<topic>");
    format!(
        r#"# Claude Code Manager Task — Create and Maintain Topic Library

## Goal

Use Claude Code as the Manager LLM to establish and maintain this topic library.

## Topic

`{topic}`

## Mode

- `{mode_label}`
- Paper count: `{paper_count}`
- RAG threshold: `{threshold}`

## Required Manager Actions

1. Confirm the workspace was created by `kb create --wiki <workspace> --from <literature-folder> --about "{topic}"`.
2. Confirm `processing/workflow_status.json` reports `< {threshold}` papers for direct small-corpus work.
3. Use `kb check` as the control console and `kb view` as the reader-facing research issue browser.
4. Use `kb task list` to recover task state after restart.
5. Use `kb batch paper-profile --topic "{topic}" --limit 5 --assignee claude-code-manager` to create bounded paper-profile Worker batches.
6. Use `kb prompt paper-profile --task <batch-id> --topic "{topic}"` to produce standard Worker prompts.
7. Route anchor-paper choices, confirmed relation edges, and final topic narrative claims to Human Review.
8. Record accepted work under `LLM/memory/`.
9. Keep Claude Code-specific notes inside `agents/claude-code/` or assigned task outputs.

## Forbidden Actions

- Do not treat Rust PDF text extraction as complete paper understanding.
- Do not ask Workers to process the whole corpus in one prompt.
- Do not modify `raw/`.
- Do not mix Claude Code prompts with `agents/openclaw/` or `agents/codex/`.

## Done When

- Paper profiles exist for the topic-relevant papers.
- `wiki/topics/{topic}.md` contains a concise linked topic story.
- Important claims link to paper pages.
- Human Review has approved anchor papers and high-value judgments.
- `kb view` shows a readable topic-centered Wiki.
"#,
        topic = topic,
        mode_label = status.mode_label.as_str(),
        paper_count = status.paper_count,
        threshold = status.rag_threshold,
    )
}

fn render_claude_code_root_entry(status: &WorkflowStatus, topic_title: Option<&str>) -> String {
    let topic = topic_title.or(status.topic.as_deref()).unwrap_or("<topic>");
    format!(
        r#"# CLAUDE.md — Claude Code Entry for LLM Wiki

Claude Code should use this file as the root entry for this LLM Wiki workspace.

## Current Role

Claude Code may act as the **LLM Manager** for this workspace.

It should coordinate deterministic `kb-cli` commands, bounded Worker prompts, and Human Review. It should not directly rewrite the whole Wiki or treat PDF text extraction as complete scholarly understanding.

## Current Workspace

- Topic: `{topic}`
- Workflow mode: `{mode_label}`
- Paper count: `{paper_count}`
- RAG threshold: `{threshold}`

## Read First

1. `AGENTS.md`
2. `LLM/workflow.md`
3. `LLM/tasks/index.md`
4. `agents/shared/karpathy-small-corpus-workflow.md`
5. `agents/claude-code/README.md`
6. `agents/claude-code/manager.md`
7. `agents/claude-code/daily-maintenance.md`
8. `LLM/tasks/claude_code_manager_create_and_maintain.md`

## Default Safe Commands

```bash
kb --wiki <workspace> check
kb --wiki <workspace> view
kb --wiki <workspace> tasks
kb --wiki <workspace> task list
```

## Core Loop

1. Observe the current workspace with deterministic `kb` commands.
2. Create one bounded Worker batch or one bounded task.
3. Generate a reusable Worker prompt with `kb prompt ...`.
4. Review Worker output before accepting it.
5. Route high-value judgments to Human Review.
6. Mark task status with `kb task status <id> --mark <state>`.
7. Record accepted results under `LLM/memory/`.

## Hard Boundaries

- Do not modify `raw/`.
- Do not invent missing metadata, citations, or paper claims.
- Do not confirm relation candidates without evidence and review.
- Do not treat files under `interfaces/html/` as durable source-of-truth files.
- Keep Claude Code-specific notes under `agents/claude-code/` or assigned task output files.
"#,
        topic = topic,
        mode_label = status.mode_label.as_str(),
        paper_count = status.paper_count,
        threshold = status.rag_threshold,
    )
}

fn render_codex_readme(status: &WorkflowStatus, topic_title: Option<&str>) -> String {
    let topic = topic_title.or(status.topic.as_deref()).unwrap_or("<topic>");
    format!(
        r#"# Codex Entry — LLM Wiki Manager

This directory is the Codex-specific entry point. It is intentionally separate from `agents/openclaw/` and `agents/claude-code/` so that each Manager agent keeps its own prompt style, operational habits, and boundary rules.

Codex may serve as the **LLM Manager** for this workspace. In that role, Codex should coordinate deterministic `kb-cli` commands, bounded Worker prompts, and Human Review. It should not behave as an unconstrained all-files editor.

## Current Workspace Mode

- Mode: `{mode}`
- Corpus size: `{paper_count}` papers
- RAG threshold: `{threshold}` papers
- Current topic: `{topic}`

This version focuses on the `< 200 papers` case. Codex should use the Karpathy-style LLM Wiki workflow: inspect deterministic reports, create small Worker batches, generate reusable prompts, review outputs, and route high-value decisions to Human Review.

## Codex Read Order

1. `AGENTS.md` — Codex project entry and shared external-agent contract.
2. `LLM/workflow.md` — current workflow mode and routing decision.
3. `LLM/tasks/index.md` — task dashboard.
4. `agents/shared/karpathy-small-corpus-workflow.md` — shared small-corpus protocol.
5. `agents/codex/README.md` — this file.
6. `agents/codex/manager.md` — Manager identity and boundaries.
7. `agents/codex/create-topic-library.md` — how to establish a topic library.
8. `agents/codex/daily-maintenance.md` — routine maintenance/update checklist.
9. `agents/codex/worker-dispatch.md` — how to create Worker prompts and small batches.
10. `agents/codex/skills/llm-wiki-manager/SKILL.md` — optional Codex Skill-style reusable workflow descriptor.
11. `LLM/tasks/codex_manager_create_and_maintain.md`.

## Safe First Command

```bash
kb --wiki <workspace> check
```

Then inspect:

- `interfaces/html/index.html`
- `interfaces/html/browse.html`
- `LLM/tasks/index.md`
- `agents/codex/daily-maintenance.md`
"#,
        mode = status.mode.as_str(),
        paper_count = status.paper_count,
        threshold = status.rag_threshold,
        topic = topic,
    )
}

fn render_codex_manager_guide(status: &WorkflowStatus, topic_title: Option<&str>) -> String {
    let topic = topic_title.or(status.topic.as_deref()).unwrap_or("<topic>");
    format!(
        r#"# Codex Manager Guide

## Identity

You are Codex acting as the Manager LLM for this LLM Wiki workspace.

Your job is not to complete the whole Wiki in one pass. Your job is to coordinate deterministic `kb-cli` commands, bounded Worker tasks, and Human Review so that paper understanding, topic narratives, and Wiki links are completed safely.

## Current Mode

- Corpus size: **{paper_count} papers**
- Threshold: **{threshold} papers**
- Workflow mode: **{mode_label}**
- Topic: `{topic}`

## Manager Duties

1. Read `AGENTS.md`, `LLM/workflow.md`, `LLM/tasks/index.md`, and `agents/codex/` before editing.
2. Use deterministic commands first: `kb check`, `kb view`, `kb tasks`, `kb task list`, `kb topic status <topic>`.
3. Create Worker batches using `kb batch paper-profile --topic <topic> --limit 5 --assignee codex-manager`.
4. Generate standard Worker prompts using `kb prompt paper-profile --task <batch-id> --topic <topic>`.
5. Track progress with `kb task status <id> --mark <state>`.
6. Keep Worker output inside explicitly allowed files.
7. Ask Human Review to approve anchor papers, topic narratives, confirmed relation edges, and merges into reviewed pages.
8. Record accepted work under `LLM/memory/` or topic review notes.

## Codex Must Not

- Do not modify `raw/`.
- Do not invent missing title, author, DOI, abstract, references, or scientific claims.
- Do not convert relation candidates to confirmed relations without evidence and review.
- Do not let a Worker process the whole corpus in one prompt.
- Do not mix Codex instructions with `agents/openclaw/` or `agents/claude-code/` prompts.
- Do not treat `interfaces/html/` as a durable source of truth.

## Standard Loop

```text
observe workspace
→ run deterministic kb command
→ inspect generated task/interface files
→ create one small Worker batch or one bounded task
→ generate a reusable Worker prompt
→ review Worker output
→ route high-value judgments to Human Review
→ mark task status
→ record accepted result
→ refresh kb check / kb view
```
"#,
        paper_count = status.paper_count,
        threshold = status.rag_threshold,
        mode_label = status.mode_label.as_str(),
        topic = topic,
    )
}

fn render_codex_create_topic_library(status: &WorkflowStatus, topic_title: Option<&str>) -> String {
    let topic = topic_title.or(status.topic.as_deref()).unwrap_or("<topic>");
    format!(
        r#"# Codex Procedure — Create a Topic Library

This procedure tells Codex how to establish or refresh a topic library as the Manager LLM.

## Canonical Command

```bash
kb create --wiki <workspace> --from <literature-folder> --about "{topic}"
```

## What This Command Means

`kb create` creates the LLM Wiki worksite. It does not semantically finish the Wiki. It creates:

- directory skeletons;
- raw literature registry;
- paper stubs;
- topic workspace;
- deterministic indexes;
- Manager/Worker task guides;
- Human Review checklists;
- `kb check` and `kb view` HTML entrances.

## Codex After-Create Checklist

1. Verify `kb create` completed.
2. Open `processing/workflow_status.json` and confirm the workflow mode.
3. Open `LLM/workflow.md`.
4. Open `LLM/tasks/index.md`.
5. Open `LLM/tasks/codex_manager_create_and_maintain.md`.
6. Open `interfaces/html/index.html` for the control console.
7. Open `interfaces/html/browse.html` for the research issue browser.
8. Create the first paper-profile Worker batch.
9. Ask Human Review to choose or approve anchor papers before finalizing the topic story.

## First Worker Batch

```bash
kb --wiki <workspace> batch paper-profile --topic "{topic}" --limit 5 --assignee codex-manager
kb --wiki <workspace> prompt paper-profile --task <batch-id> --topic "{topic}"
```

Output should go to:

- `processing/paper_profiles/<paper_id>.md`
- `wiki/papers/<paper_id>.md` only when explicitly allowed.
"#,
        topic = topic,
    )
}

fn render_codex_daily_maintenance(status: &WorkflowStatus, topic_title: Option<&str>) -> String {
    let topic = topic_title.or(status.topic.as_deref()).unwrap_or("<topic>");
    format!(
        r#"# Codex Daily Maintenance Protocol

This file is the daily maintenance checklist for Codex Manager.

## Current Scope

- Topic: `{topic}`
- Corpus size: `{paper_count}` papers
- Workflow mode: `{mode_label}`

## Daily Start

1. Read `AGENTS.md`.
2. Read `agents/codex/README.md`.
3. Read `agents/codex/manager.md`.
4. Read `LLM/workflow.md`.
5. Read `LLM/tasks/index.md` if present.
6. Run or request:

```bash
kb --wiki <workspace> check
kb --wiki <workspace> view
kb --wiki <workspace> tasks
kb --wiki <workspace> task list
kb --wiki <workspace> topic status {topic}
```

## Maintenance Queue

Handle only one queue item at a time:

1. New PDFs added to `raw/papers/` or source folder.
2. Paper stubs needing LLM extraction.
   - Create a small batch with `kb --wiki <workspace> batch paper-profile --topic {topic} --limit 5 --assignee codex-manager`.
   - Print the Worker prompt with `kb --wiki <workspace> prompt paper-profile --task <batch-id> --topic {topic}`.
   - Mark progress with `kb --wiki <workspace> task status <batch-id> --mark assigned|in_progress|completed|blocked`.
3. Paper profiles needing Human Review.
4. Topic narrative sections needing linked evidence.
5. Relation candidates needing evidence checks.
6. Broken WikiLinks or missing backlinks.
7. Outdated task files or handoff files.

## Update Rules

- If new literature appears, rerun `kb create --wiki <workspace> --from <literature-folder> --about "{topic}"` or the equivalent deterministic refresh command.
- If the corpus count reaches `{threshold}` or more, stop the direct small-corpus workflow and request RAG-assisted workflow preparation.
- After Worker edits, mark task progress with `kb task status <id> --mark <state>`.
- After accepted Worker outputs, refresh `kb check` and `kb view`.
- Record accepted maintenance under `LLM/memory/` after the task is marked completed.

## Maintenance Note Template

```markdown
# Codex Maintenance Note — <date>

## Commands Run

## Tasks Assigned

## Worker Outputs Accepted

## Human Review Needed

## Files Changed

## Next Maintenance Step
```
"#,
        topic = topic,
        paper_count = status.paper_count,
        mode_label = status.mode_label.as_str(),
        threshold = status.rag_threshold,
    )
}

fn render_codex_worker_dispatch(status: &WorkflowStatus, topic_title: Option<&str>) -> String {
    let topic = topic_title.or(status.topic.as_deref()).unwrap_or("<topic>");
    format!(
        r#"# Codex Worker Dispatch Template

Codex Manager may dispatch work to another LLM session or a bounded subtask, but every Worker task must be limited and auditable.

## Preferred Dispatch Commands

```bash
kb --wiki <workspace> batch paper-profile --topic {topic} --limit 5 --assignee codex-manager
kb --wiki <workspace> prompt paper-profile --task <batch-id> --topic {topic}
kb --wiki <workspace> task status <batch-id> --mark assigned --assignee codex-manager
```

## Worker Brief Template

```text
You are a Worker LLM for an LLM Wiki.

Topic: {topic}
Corpus mode: {mode_label}
Manager: Codex

Read only the files listed below.
Work only on the allowed output files.
Do not modify raw/.
Do not invent missing metadata.
Mark uncertain claims as needs_human_review.

Task:
<one concrete task>

Input files:
- <file 1>
- <file 2>

Allowed output files:
- <output 1>

Return:
1. work summary;
2. files changed;
3. evidence used;
4. unresolved uncertainty;
5. human review needs.
```

## Good Worker Tasks

- Extract key information for one paper or a small batch.
- Draft one section of `wiki/topics/{topic}.md` with links to paper pages.
- Check one relation candidate and return evidence.
- Fix one small group of broken WikiLinks.

## Bad Worker Tasks

- Read the whole corpus and write the whole Wiki.
- Rewrite all topic pages.
- Confirm all relation candidates.
- Delete or reorganize `raw/`.
"#,
        topic = topic,
        mode_label = status.mode_label.as_str(),
    )
}

fn render_codex_start_prompt(status: &WorkflowStatus, topic_title: Option<&str>) -> String {
    let topic = topic_title.or(status.topic.as_deref()).unwrap_or("<topic>");
    format!(
        r#"You are Codex acting as the Manager LLM for an LLM Wiki workspace.

First, read these files:

1. AGENTS.md
2. LLM/workflow.md
3. agents/codex/README.md
4. agents/codex/manager.md
5. agents/codex/create-topic-library.md
6. agents/codex/daily-maintenance.md
7. agents/codex/skills/llm-wiki-manager/SKILL.md
8. LLM/tasks/codex_manager_create_and_maintain.md
9. LLM/handoff/current.md

Current topic: {topic}
Current mode: {mode_label}
Corpus size: {paper_count} papers

Do not modify files yet. Return:

1. current workspace state;
2. whether this is still a <200-paper Karpathy-style workflow;
3. the top task queue;
4. the safest next three actions;
5. which decisions require Human Review;
6. which `kb task status` updates are needed.
"#,
        topic = topic,
        mode_label = status.mode_label.as_str(),
        paper_count = status.paper_count,
    )
}

fn render_codex_llm_wiki_manager_skill(
    status: &WorkflowStatus,
    topic_title: Option<&str>,
) -> String {
    let topic = topic_title.or(status.topic.as_deref()).unwrap_or("<topic>");
    format!(
        r#"---
name: llm-wiki-manager
description: Manage a small-corpus LLM Wiki workspace with kb-cli, bounded Worker prompts, and Human Review.
---

# LLM Wiki Manager Skill

Use this skill when Codex is asked to establish, maintain, or recover work in an LLM Wiki workspace.

## Current Workspace

- Topic: `{topic}`
- Workflow mode: `{mode_label}`
- Paper count: `{paper_count}`
- RAG threshold: `{threshold}`

## Default Procedure

1. Read `AGENTS.md` and `LLM/workflow.md`.
2. Read `agents/codex/README.md` and `agents/codex/manager.md`.
3. Run deterministic inspection commands before editing:

```bash
kb --wiki <workspace> check
kb --wiki <workspace> view
kb --wiki <workspace> task list
```

4. Create only small bounded Worker batches:

```bash
kb --wiki <workspace> batch paper-profile --topic {topic} --limit 5 --assignee codex-manager
kb --wiki <workspace> prompt paper-profile --task <batch-id> --topic {topic}
```

5. Mark task progress:

```bash
kb --wiki <workspace> task status <task-id> --mark in_progress --assignee codex-manager
```

6. Route anchor-paper decisions, relation confirmations, and final topic narratives to Human Review.

## Boundaries

- Never modify `raw/`.
- Never invent paper metadata or scientific claims.
- Never process the whole corpus in one prompt.
- Never mix Codex notes with OpenClaw or Claude Code adapter directories.
"#,
        topic = topic,
        mode_label = status.mode_label.as_str(),
        paper_count = status.paper_count,
        threshold = status.rag_threshold,
    )
}

fn render_codex_manager_task(status: &WorkflowStatus, topic_title: Option<&str>) -> String {
    let topic = topic_title.or(status.topic.as_deref()).unwrap_or("<topic>");
    format!(
        r#"# Codex Manager Task — Create and Maintain Topic Library

## Goal

Use Codex as the Manager LLM to establish and maintain this topic library.

## Topic

`{topic}`

## Mode

- `{mode_label}`
- Paper count: `{paper_count}`
- RAG threshold: `{threshold}`

## Required Manager Actions

1. Confirm the workspace was created by `kb create --wiki <workspace> --from <literature-folder> --about "{topic}"`.
2. Confirm `processing/workflow_status.json` reports `< {threshold}` papers for direct small-corpus work.
3. Use `kb check` as the control console and `kb view` as the reader-facing research issue browser.
4. Use `kb task list` to recover task state after restart.
5. Use `kb batch paper-profile --topic "{topic}" --limit 5 --assignee codex-manager` to create bounded paper-profile Worker batches.
6. Use `kb prompt paper-profile --task <batch-id> --topic "{topic}"` to produce standard Worker prompts.
7. Route anchor-paper choices, confirmed relation edges, and final topic narrative claims to Human Review.
8. Record accepted work under `LLM/memory/`.
9. Keep Codex-specific notes inside `agents/codex/` or assigned task outputs.

## Forbidden Actions

- Do not treat Rust PDF text extraction as complete paper understanding.
- Do not ask Workers to process the whole corpus in one prompt.
- Do not modify `raw/`.
- Do not mix Codex prompts with `agents/openclaw/` or `agents/claude-code/`.

## Done When

- Paper profiles exist for the topic-relevant papers.
- `wiki/topics/{topic}.md` contains a concise linked topic story.
- Important claims link to paper pages.
- Human Review has approved anchor papers and high-value judgments.
- `kb view` shows a readable topic-centered Wiki.
"#,
        topic = topic,
        mode_label = status.mode_label.as_str(),
        paper_count = status.paper_count,
        threshold = status.rag_threshold,
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
