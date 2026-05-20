use anyhow::{anyhow, Result};
use clap::{Args, ValueEnum};
use serde::Serialize;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum AgentTarget {
    Generic,
    ClaudeCode,
    Opencode,
    Openclaw,
    Codex,
}

impl fmt::Display for AgentTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.slug())
    }
}

impl AgentTarget {
    fn slug(self) -> &'static str {
        match self {
            AgentTarget::Generic => "generic",
            AgentTarget::ClaudeCode => "claude-code",
            AgentTarget::Opencode => "opencode",
            AgentTarget::Openclaw => "openclaw",
            AgentTarget::Codex => "codex",
        }
    }

    fn is_adapter(self) -> bool {
        self != AgentTarget::Generic
    }
}

#[derive(Debug, Clone, Args)]
pub struct HandoffArgs {
    #[arg(
        long,
        value_name = "TOPIC",
        help = "Generate topic-level handoff files for one topic"
    )]
    pub topic: Option<String>,

    #[arg(
        long = "all-topics",
        help = "Generate topic-level handoff files for every existing topic"
    )]
    pub all_topics: bool,

    #[arg(
        long,
        value_enum,
        default_value_t = AgentTarget::Generic,
        help = "Agent adapter to generate: generic, claude-code, opencode, openclaw, or codex"
    )]
    pub agent: AgentTarget,

    #[arg(
        long = "all-agents",
        help = "Generate the generic handoff plus all supported agent adapters"
    )]
    pub all_agents: bool,

    #[arg(long, help = "Overwrite existing generated handoff files")]
    pub force: bool,

    #[arg(long, help = "Preview handoff generation without writing files")]
    pub dry_run: bool,

    #[arg(long, help = "Alias for --dry-run")]
    pub preview: bool,

    #[arg(long, help = "Print a machine-readable JSON handoff report")]
    pub json: bool,
}

#[derive(Debug, Serialize)]
pub struct HandoffReport {
    schema_version: String,
    generated_by: String,
    generated_at: String,
    kb_path: String,
    dry_run: bool,
    force: bool,
    agents: Vec<String>,
    project_files: Vec<HandoffFileReport>,
    topic_files: Vec<HandoffFileReport>,
    topics: Vec<String>,
    warnings: Vec<String>,
    next_steps: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HandoffFileReport {
    pub path: String,
    pub level: String,
    pub status: String,
}

pub fn execute(custom_kb: Option<&Path>, args: &HandoffArgs) -> Result<()> {
    let kb_path = crate::commands::init::get_kb_path(custom_kb);
    if !kb_path.exists() {
        return Err(anyhow!(
            "knowledge base path does not exist: {}. Run `kb --wiki <database-name> init` or `kb --source <literature-dir> setup` first.",
            kb_path.display()
        ));
    }

    let dry_run = args.dry_run || args.preview;
    let agents = selected_agents(args.agent, args.all_agents);
    let mut report = build_project_report(&kb_path, args.force, dry_run, &agents)?;

    let requested_topics =
        resolve_requested_topics(&kb_path, args.topic.as_deref(), args.all_topics)?;
    for topic in requested_topics {
        let topic_report = build_topic_report(&kb_path, &topic, args.force, dry_run, &agents)?;
        report.topics.push(topic);
        report.topic_files.extend(topic_report);
    }

    if args.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print_handoff_report(&report);
    }
    Ok(())
}

pub fn ensure_default_handoffs(
    kb_path: &Path,
    topic: Option<&str>,
    force: bool,
) -> Result<HandoffReport> {
    let agents = vec![AgentTarget::Generic];
    let mut report = build_project_report(kb_path, force, false, &agents)?;
    if let Some(topic) = topic {
        let topic_slug = topic_slug(topic);
        if !topic_slug.is_empty() {
            let topic_report = build_topic_report(kb_path, &topic_slug, force, false, &agents)?;
            report.topics.push(topic_slug);
            report.topic_files.extend(topic_report);
        }
    }
    Ok(report)
}

pub fn execute_topic_handoff(
    custom_kb: Option<&Path>,
    topic: &str,
    force: bool,
    dry_run: bool,
    json: bool,
    agent: AgentTarget,
    all_agents: bool,
) -> Result<()> {
    let kb_path = crate::commands::init::get_kb_path(custom_kb);
    if !kb_path.exists() {
        return Err(anyhow!(
            "knowledge base path does not exist: {}. Run `kb --wiki <database-name> init` or `kb --source <literature-dir> setup` first.",
            kb_path.display()
        ));
    }
    let slug = topic_slug(topic);
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

    let agents = selected_agents(agent, all_agents);
    let mut report = HandoffReport {
        schema_version: "handoff.v0.7.19".to_string(),
        generated_by: "kb-cli topic handoff".to_string(),
        generated_at: chrono::Utc::now().to_rfc3339(),
        kb_path: kb_path.display().to_string(),
        dry_run,
        force,
        agents: agent_names(&agents),
        project_files: Vec::new(),
        topic_files: build_topic_report(&kb_path, &slug, force, dry_run, &agents)?,
        topics: vec![slug],
        warnings: Vec::new(),
        next_steps: next_steps_for_agents(&agents, true),
    };

    if report.topic_files.is_empty() {
        report
            .warnings
            .push("no topic handoff files were generated".to_string());
    }

    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print_handoff_report(&report);
    }
    Ok(())
}

fn build_project_report(
    kb_path: &Path,
    force: bool,
    dry_run: bool,
    agents: &[AgentTarget],
) -> Result<HandoffReport> {
    let mut files = Vec::new();

    files.extend(generic_project_specs(kb_path, agents));
    for agent in agents.iter().copied().filter(|agent| agent.is_adapter()) {
        files.extend(adapter_project_specs(kb_path, agent));
    }

    let mut project_files = Vec::new();
    for spec in files {
        project_files.push(write_handoff_spec(kb_path, spec, force, dry_run)?);
    }

    Ok(HandoffReport {
        schema_version: "handoff.v0.7.19".to_string(),
        generated_by: "kb-cli handoff".to_string(),
        generated_at: chrono::Utc::now().to_rfc3339(),
        kb_path: kb_path.display().to_string(),
        dry_run,
        force,
        agents: agent_names(agents),
        project_files,
        topic_files: Vec::new(),
        topics: Vec::new(),
        warnings: Vec::new(),
        next_steps: next_steps_for_agents(agents, false),
    })
}

fn build_topic_report(
    kb_path: &Path,
    topic: &str,
    force: bool,
    dry_run: bool,
    agents: &[AgentTarget],
) -> Result<Vec<HandoffFileReport>> {
    let slug = topic_slug(topic);
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

    let title = read_topic_title(&topic_root).unwrap_or_else(|| topic_title(&slug));
    let mut files = generic_topic_specs(kb_path, &topic_root, &slug, &title, agents);
    for agent in agents.iter().copied().filter(|agent| agent.is_adapter()) {
        files.extend(adapter_topic_specs(&topic_root, &slug, &title, agent));
    }

    let mut reports = Vec::new();
    for spec in files {
        reports.push(write_handoff_spec(kb_path, spec, force, dry_run)?);
    }
    Ok(reports)
}

fn selected_agents(agent: AgentTarget, all_agents: bool) -> Vec<AgentTarget> {
    if all_agents {
        return vec![
            AgentTarget::Generic,
            AgentTarget::ClaudeCode,
            AgentTarget::Opencode,
            AgentTarget::Openclaw,
            AgentTarget::Codex,
        ];
    }
    if agent == AgentTarget::Generic {
        vec![AgentTarget::Generic]
    } else {
        vec![AgentTarget::Generic, agent]
    }
}

fn agent_names(agents: &[AgentTarget]) -> Vec<String> {
    agents
        .iter()
        .map(|agent| agent.slug().to_string())
        .collect()
}

fn next_steps_for_agents(agents: &[AgentTarget], topic_only: bool) -> Vec<String> {
    let mut out = vec![
        "Open AGENTS.md before starting any external agent work.".to_string(),
        "Run `kb tasks` for global deterministic task handoff.".to_string(),
        "Run `kb topic tasks <topic>` and `kb topic handoff <topic>` for topic-local work."
            .to_string(),
    ];
    if topic_only {
        out = vec![
            "Open topics/<topic>/handoff/AGENTS.md before assigning topic work.".to_string(),
            "Assign one task item at a time from topics/<topic>/tasks/index.md.".to_string(),
            "Review all changes with Git diff or equivalent diff tooling before accepting them."
                .to_string(),
        ];
    }
    if agents.contains(&AgentTarget::ClaudeCode) {
        out.push(
            "For Claude Code, read CLAUDE.md and the claude-code adapter after AGENTS.md."
                .to_string(),
        );
    }
    if agents.contains(&AgentTarget::Opencode) {
        out.push("For OpenCode, read LLM/handoff/adapters/opencode.md or the topic-local OpenCode adapter.".to_string());
    }
    if agents.contains(&AgentTarget::Openclaw) {
        out.push("For OpenClaw, use the adapter as a controlled task brief and avoid granting broad service access by default.".to_string());
    }
    if agents.contains(&AgentTarget::Codex) {
        out.push("For Codex, read LLM/handoff/adapters/codex.md or the topic-local Codex adapter after the generic handoff.".to_string());
    }
    out
}

fn generic_project_specs(kb_path: &Path, agents: &[AgentTarget]) -> Vec<HandoffSpec> {
    vec![
        HandoffSpec {
            path: kb_path.join("AGENTS.md"),
            level: "project-generic".to_string(),
            content: render_project_agents_md(kb_path),
        },
        HandoffSpec {
            path: kb_path.join("obsidian_main.md"),
            level: "obsidian-main".to_string(),
            content: render_obsidian_main(kb_path),
        },
        HandoffSpec {
            path: kb_path.join("interfaces/README.md"),
            level: "interface-boundary".to_string(),
            content: render_interfaces_readme(),
        },
        HandoffSpec {
            path: kb_path.join("LLM/handoff/current.md"),
            level: "global-current".to_string(),
            content: render_current_handoff(kb_path, agents),
        },
        HandoffSpec {
            path: kb_path.join("LLM/handoff/index.md"),
            level: "global-index".to_string(),
            content: render_global_handoff_index(kb_path, agents),
        },
        HandoffSpec {
            path: kb_path.join("LLM/handoff/protocol.md"),
            level: "global-protocol".to_string(),
            content: render_global_protocol(),
        },
        HandoffSpec {
            path: kb_path.join("LLM/handoff/manager.md"),
            level: "global-manager".to_string(),
            content: render_global_manager_handoff(),
        },
        HandoffSpec {
            path: kb_path.join("LLM/handoff/worker.md"),
            level: "global-worker".to_string(),
            content: render_global_worker_handoff(),
        },
        HandoffSpec {
            path: kb_path.join("LLM/handoff/task_schema.md"),
            level: "global-task-schema".to_string(),
            content: render_task_schema(),
        },
        HandoffSpec {
            path: kb_path.join("LLM/handoff/safety.md"),
            level: "global-safety".to_string(),
            content: render_safety_policy(),
        },
        HandoffSpec {
            path: kb_path.join("LLM/handoff/handoff.json"),
            level: "global-json".to_string(),
            content: render_global_handoff_json(kb_path, agents),
        },
    ]
}

fn adapter_project_specs(kb_path: &Path, agent: AgentTarget) -> Vec<HandoffSpec> {
    match agent {
        AgentTarget::ClaudeCode => vec![
            HandoffSpec {
                path: kb_path.join("CLAUDE.md"),
                level: "adapter-claude-code-root".to_string(),
                content: render_project_claude_md(kb_path),
            },
            HandoffSpec {
                path: kb_path.join("LLM/handoff/adapters/claude-code.md"),
                level: "adapter-claude-code".to_string(),
                content: render_adapter_handoff(agent),
            },
        ],
        AgentTarget::Opencode => vec![HandoffSpec {
            path: kb_path.join("LLM/handoff/adapters/opencode.md"),
            level: "adapter-opencode".to_string(),
            content: render_adapter_handoff(agent),
        }],
        AgentTarget::Openclaw => vec![HandoffSpec {
            path: kb_path.join("LLM/handoff/adapters/openclaw.md"),
            level: "adapter-openclaw".to_string(),
            content: render_adapter_handoff(agent),
        }],
        AgentTarget::Codex => vec![HandoffSpec {
            path: kb_path.join("LLM/handoff/adapters/codex.md"),
            level: "adapter-codex".to_string(),
            content: render_adapter_handoff(agent),
        }],
        AgentTarget::Generic => Vec::new(),
    }
}

fn generic_topic_specs(
    kb_path: &Path,
    topic_root: &Path,
    slug: &str,
    title: &str,
    agents: &[AgentTarget],
) -> Vec<HandoffSpec> {
    vec![
        HandoffSpec {
            path: topic_root.join("index.md"),
            level: "topic-index".to_string(),
            content: render_topic_index(slug, title),
        },
        HandoffSpec {
            path: topic_root.join("handoff/AGENTS.md"),
            level: "topic-generic".to_string(),
            content: render_topic_agents_md(slug, title),
        },
        HandoffSpec {
            path: topic_root.join("handoff/protocol.md"),
            level: "topic-protocol".to_string(),
            content: render_topic_protocol(slug, title),
        },
        HandoffSpec {
            path: topic_root.join("handoff/manager.md"),
            level: "topic-manager".to_string(),
            content: render_topic_manager_handoff(slug, title),
        },
        HandoffSpec {
            path: topic_root.join("handoff/worker_task_template.md"),
            level: "topic-worker".to_string(),
            content: render_topic_worker_template(slug, title),
        },
        HandoffSpec {
            path: topic_root.join("handoff/start_prompt.md"),
            level: "topic-start-prompt".to_string(),
            content: render_topic_start_prompt(slug, title),
        },
        HandoffSpec {
            path: topic_root.join("handoff/handoff.json"),
            level: "topic-json".to_string(),
            content: render_topic_handoff_json(kb_path, slug, title, agents),
        },
    ]
}

fn adapter_topic_specs(
    topic_root: &Path,
    slug: &str,
    title: &str,
    agent: AgentTarget,
) -> Vec<HandoffSpec> {
    match agent {
        AgentTarget::ClaudeCode => vec![
            HandoffSpec {
                path: topic_root.join("handoff/CLAUDE.md"),
                level: "topic-adapter-claude-code-root".to_string(),
                content: render_topic_claude_md(slug, title),
            },
            HandoffSpec {
                path: topic_root.join("handoff/adapters/claude-code.md"),
                level: "topic-adapter-claude-code".to_string(),
                content: render_topic_adapter_handoff(slug, title, agent),
            },
        ],
        AgentTarget::Opencode => vec![HandoffSpec {
            path: topic_root.join("handoff/adapters/opencode.md"),
            level: "topic-adapter-opencode".to_string(),
            content: render_topic_adapter_handoff(slug, title, agent),
        }],
        AgentTarget::Openclaw => vec![HandoffSpec {
            path: topic_root.join("handoff/adapters/openclaw.md"),
            level: "topic-adapter-openclaw".to_string(),
            content: render_topic_adapter_handoff(slug, title, agent),
        }],
        AgentTarget::Codex => vec![HandoffSpec {
            path: topic_root.join("handoff/adapters/codex.md"),
            level: "topic-adapter-codex".to_string(),
            content: render_topic_adapter_handoff(slug, title, agent),
        }],
        AgentTarget::Generic => Vec::new(),
    }
}

struct HandoffSpec {
    path: PathBuf,
    level: String,
    content: String,
}

fn write_handoff_spec(
    kb_path: &Path,
    spec: HandoffSpec,
    force: bool,
    dry_run: bool,
) -> Result<HandoffFileReport> {
    let relative = relative_path_string(kb_path, &spec.path);
    let status = if dry_run {
        if spec.path.exists() && !force {
            "would-skip-existing"
        } else if spec.path.exists() && force {
            "would-overwrite"
        } else {
            "would-create"
        }
    } else if spec.path.exists() && !force {
        "skipped-existing"
    } else {
        if let Some(parent) = spec.path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&spec.path, spec.content)?;
        "written"
    };

    Ok(HandoffFileReport {
        path: relative,
        level: spec.level,
        status: status.to_string(),
    })
}

fn resolve_requested_topics(
    kb_path: &Path,
    topic: Option<&str>,
    all_topics: bool,
) -> Result<Vec<String>> {
    if let Some(topic) = topic {
        let slug = topic_slug(topic);
        if slug.is_empty() {
            return Err(anyhow!(
                "topic name must contain at least one letter or digit"
            ));
        }
        return Ok(vec![slug]);
    }
    if all_topics {
        return collect_topic_slugs(kb_path);
    }
    Ok(Vec::new())
}

fn collect_topic_slugs(kb_path: &Path) -> Result<Vec<String>> {
    let topics_dir = kb_path.join("topics");
    let mut out = Vec::new();
    if !topics_dir.exists() {
        return Ok(out);
    }
    for entry in fs::read_dir(topics_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        out.push(name.to_string());
    }
    out.sort();
    Ok(out)
}

fn render_interfaces_readme() -> String {
    r#"# interfaces/ — Interface Artifacts

This directory contains generated, regenerable files for human-machine review, machine-machine handoff, and operational diagnostics.

It is **not** a durable knowledge layer.

## What belongs here

- `html/` — browser review pages generated by `kb view`
- `reports/` — health, lint, and audit reports
- `logs/` — generated metadata and operational logs
- `answers/` — generated answers or temporary outputs
- `figures/` and `slides/` — generated visual/presentation artifacts

## Source of truth

The durable LLM Wiki source of truth is stored outside this directory, mainly in:

- `raw/`
- `wiki/`
- `topics/`
- `processing/`
- `LLM/tasks/`
- `LLM/memory/`
- `rules/`
- `llm-wiki.toml`
- `AGENTS.md`, `CLAUDE.md`, and handoff files

## Agent rule

LLM agents may inspect files under `interfaces/` for context, diagnostics, and review convenience, but must not treat them as authoritative knowledge. If a review decision is made from an HTML page or report, the result must be written back to the appropriate Markdown/JSON/TOML source file outside `interfaces/`.

Generated HTML files are interface review artifacts. Delete and regenerate them when needed.
"#.to_string()
}

fn render_obsidian_main(kb_path: &Path) -> String {
    let name = kb_path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("knowledgebase");
    let topics = collect_topic_slugs(kb_path).unwrap_or_default();
    let mut out = String::new();
    out.push_str(&format!("# obsidian_main.md — READ THIS FIRST\n\n"));
    out.push_str(&format!(
        "This is the human-facing Obsidian homepage for the LLM Wiki `{name}`.\n\n"
    ));
    out.push_str("Open the whole knowledge-base directory as an Obsidian vault, then open this file first. Do not start from `raw/`, `processing/`, or a random paper page.\n\n");
    out.push_str("## First Reading Order\n\n");
    out.push_str("1. `obsidian_main.md` — this page.\n");
    out.push_str("2. `LLM/handoff/current.md` — current Manager entry and topic/task routing.\n");
    out.push_str("3. `AGENTS.md` — shared rules for all external agents.\n");
    out.push_str("4. `topics/<topic>/index.md` — the topic workspace homepage.\n");
    out.push_str("5. `topics/<topic>/tasks/index.md` — the topic task queue.\n");
    out.push_str("6. `topics/<topic>/relations/` and `topics/<topic>/review/` — relation and review materials.\n\n");
    out.push_str("## Agent Entry Files\n\n");
    out.push_str("| role/tool | first file | then read |\n");
    out.push_str("|---|---|---|\n");
    out.push_str(
        "| Human in Obsidian | `obsidian_main.md` | `LLM/handoff/current.md`, then topic index |\n",
    );
    out.push_str("| Generic Manager | `AGENTS.md` | `LLM/handoff/current.md` |\n");
    out.push_str("| Claude Code | `CLAUDE.md` when generated | `AGENTS.md`, `LLM/handoff/current.md`, Claude adapter |\n");
    out.push_str("| Codex | `AGENTS.md` | `LLM/handoff/current.md`, Codex adapter |\n");
    out.push_str(
        "| OpenCode/OpenClaw | `AGENTS.md` | `LLM/handoff/current.md`, matching adapter |\n\n",
    );
    out.push_str("## Topics\n\n");
    if topics.is_empty() {
        out.push_str("No topic workspace exists yet. Create one with `kb topic init <topic>` or run `kb --source <literature-dir> setup --topic <topic>`.\n\n");
    } else {
        for topic in topics {
            out.push_str(&format!("- `topics/{topic}/index.md` → topic homepage\n"));
            out.push_str(&format!(
                "  - `topics/{topic}/tasks/index.md` → task queue\n"
            ));
            out.push_str(&format!(
                "  - `topics/{topic}/handoff/AGENTS.md` → topic agent entry\n"
            ));
        }
        out.push('\n');
    }
    out.push_str("## Human Review Rule\n\n");
    out.push_str("Obsidian is for reading, linking, and review. It does not decide tasks automatically. Use this page and `LLM/handoff/current.md` to choose a task, then let Claude Code/Codex/OpenCode process one bounded task if needed.\n\n");
    out.push_str("Never treat `candidate` relations as confirmed just because they appear in a graph or task file.\n");
    out
}

fn render_current_handoff(kb_path: &Path, agents: &[AgentTarget]) -> String {
    let topics = collect_topic_slugs(kb_path).unwrap_or_default();
    let mut out = String::new();
    out.push_str("# Current LLM Wiki Entry\n\n");
    out.push_str("This is the shared routing file for Manager agents. Read it after the tool-specific root entry file.\n\n");
    out.push_str("## Entry Resolution\n\n");
    out.push_str("1. Human/Obsidian: open `obsidian_main.md`, then this file.\n");
    out.push_str("2. Generic agent: read `AGENTS.md`, then this file.\n");
    out.push_str("3. Claude Code: read `CLAUDE.md`, then `AGENTS.md`, then this file, then `LLM/handoff/adapters/claude-code.md`.\n");
    out.push_str("4. Codex: read `AGENTS.md`, then this file, then `LLM/handoff/adapters/codex.md` if generated.\n");
    out.push_str("5. OpenCode/OpenClaw: read `AGENTS.md`, then this file, then the matching adapter if generated.\n\n");
    out.push_str("## Manager Task Discovery Loop\n\n");
    out.push_str("```text\n");
    out.push_str("read root entry file\n");
    out.push_str("→ read LLM/handoff/current.md\n");
    out.push_str("→ choose one topic from the list below\n");
    out.push_str("→ read topics/<topic>/index.md\n");
    out.push_str("→ read topics/<topic>/handoff/AGENTS.md\n");
    out.push_str("→ read topics/<topic>/tasks/index.md\n");
    out.push_str("→ claim exactly one task item\n");
    out.push_str("→ process as Worker or assign to Worker\n");
    out.push_str("→ leave Work Log and review notes\n");
    out.push_str("```\n\n");
    out.push_str("## Available Topics\n\n");
    if topics.is_empty() {
        out.push_str("No topic workspace found. Create one with `kb topic init <topic>` or convert a literature directory with `kb --source <literature-dir> setup --topic <topic>`.\n\n");
    } else if topics.len() == 1 {
        let topic = &topics[0];
        out.push_str(&format!("Default topic: `{topic}`\n\n"));
        out.push_str(&format!("- Topic homepage: `topics/{topic}/index.md`\n"));
        out.push_str(&format!(
            "- Topic handoff: `topics/{topic}/handoff/AGENTS.md`\n"
        ));
        out.push_str(&format!("- Topic tasks: `topics/{topic}/tasks/index.md`\n"));
        out.push_str(&format!("- Topic review: `topics/{topic}/review/`\n\n"));
    } else {
        out.push_str(
            "Multiple topics exist. A Manager must choose one topic before claiming a task.\n\n",
        );
        for topic in &topics {
            out.push_str(&format!(
                "- `{topic}` → `topics/{topic}/index.md`, `topics/{topic}/tasks/index.md`\n"
            ));
        }
        out.push('\n');
    }
    out.push_str("## Generated Agent Adapters\n\n");
    let adapter_names: Vec<String> = agents
        .iter()
        .copied()
        .filter(|agent| agent.is_adapter())
        .map(|agent| agent.slug().to_string())
        .collect();
    if adapter_names.is_empty() {
        out.push_str("No adapter files were requested. Generate one with `kb handoff --agent claude-code`, `kb handoff --agent codex`, or `kb handoff --all-agents`.\n\n");
    } else {
        for adapter in adapter_names {
            out.push_str(&format!(
                "- `{adapter}` → `LLM/handoff/adapters/{adapter}.md`\n"
            ));
        }
        out.push('\n');
    }
    out.push_str("## Claiming Rule\n\n");
    out.push_str("A Manager may recommend a task, but should not silently complete broad semantic work. A Worker claims exactly one task item under `topics/<topic>/tasks/items/` and writes a Work Log when done.\n");
    out
}

fn render_topic_index(slug: &str, title: &str) -> String {
    format!(
        r#"# Topic Workspace: {title}

Topic slug: `{slug}`

This is the topic homepage. Manager agents and humans should read this file before opening individual tasks.

## Read Order

1. `../../LLM/handoff/current.md` — global routing and current task discovery.
2. `handoff/AGENTS.md` — topic-level agent rules.
3. `scope.md` — topic boundary.
4. `literature.md` — included/candidate literature.
5. `tasks/index.md` — topic task queue.
6. One task item under `tasks/items/`.

## Topic Workbench

- Scope: `scope.md`
- Literature: `literature.md`
- Tasks: `tasks/index.md`
- Relations: `relations/`
- Review queue: `review/`
- Importance candidates: `importance/`
- Graph artifacts: `graph/`
- Agent handoff: `handoff/AGENTS.md`
- Interface artifacts: `../../interfaces/` (review-only; not source of truth)

## Manager Rule

Choose exactly one task from `tasks/index.md` before editing. If there is no task queue yet, run:

```bash
kb topic tasks {slug}
```

## Review Rule

Keep high-value academic judgments reviewable. Do not treat `candidate` or `needs_human_review` as `confirmed` unless a human explicitly approves it.
"#
    )
}

fn render_project_agents_md(kb_path: &Path) -> String {
    let name = kb_path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("knowledgebase");
    format!(
        r#"# AGENTS.md

This repository is an LLM Wiki knowledge base named `{name}`.

This file is the generic entry contract for any external LLM work agent, including Claude Code, OpenCode, OpenClaw, Codex-like agents, or human-operated agent shells.

## Core Principle

Do not treat this repository as a normal note dump or an automatic RAG cache. The purpose is to build a traceable local Markdown Wiki from source materials under `raw/`, then organize papers, concepts, citations, topic-specific relations, and synthesis drafts into reviewable knowledge work.

## Division of Labor

`kb-cli` performs deterministic preparation:

- initialize the knowledge base
- ingest source files
- build manifest and metadata
- extract text and sections
- build citation/index candidates
- build topic-local candidate relation views
- generate reviewable task handoff files

External agents perform bounded semantic work:

- read one task item at a time
- inspect listed evidence
- propose relation evidence
- draft paper cards, concept pages, or topic synthesis notes
- update Markdown only when the task permits it
- leave work ready for human review

Humans confirm high-value academic judgments.

## Read First

Before modifying files, read:

1. `LLM/handoff/current.md` — the routing file that tells Managers where to go next.
2. `llm-wiki.toml`
3. `rules/`
4. `interfaces/README.md` — artifact boundary notice.
5. `LLM/handoff/index.md`
6. `LLM/handoff/protocol.md`
7. `LLM/tasks/index.md` when it exists
8. the relevant `topics/<topic>/index.md`
9. the relevant `topics/<topic>/handoff/AGENTS.md`
10. the specific task item under `topics/<topic>/tasks/items/`

## Artifact Boundary

`interfaces/` is the single directory for interface artifacts: generated HTML review pages, reports, answers, figures, slides, and logs. Agents may inspect it for context, but must not treat it as the source of truth. Durable knowledge belongs in Markdown/JSON/TOML files outside `interfaces/`.

## Manager Entry Resolution

A Manager must not guess the active task from the file tree. Use this path:

```text
AGENTS.md
→ LLM/handoff/current.md
→ topics/<topic>/index.md
→ topics/<topic>/tasks/index.md
→ topics/<topic>/tasks/items/<task-id>.md
```

Claim exactly one task before editing.

## Hard Rules

- Do not modify, rename, or delete `raw/` files.
- Do not invent references, DOIs, quotations, numbers, or claims.
- Do not silently promote `candidate` or `needs_human_review` relations to `confirmed`.
- Do not rewrite the whole wiki without a task.
- Do not change `rules/` unless explicitly instructed.
- Do not change CLI behavior without updating README, docs, and CHANGELOG.
- Do not treat citation edges as semantic relations unless evidence is supplied.
- Do not treat generated HTML/reports/logs under `interfaces/` as durable knowledge.

## Standard Work Loop

```text
read AGENTS.md
read handoff files
read task index
select one bounded task
return a short plan
edit only permitted files
run deterministic checks when useful
show changed files
leave final confirmation to human review
```

## Recommended Commands

```bash
kb health
kb tasks
kb topic list
kb topic tasks <topic>
kb topic handoff <topic>
kb view --relations --topic <topic> --no-open
```

## Review Status Vocabulary

Use these statuses consistently:

- `candidate`
- `needs_human_review`
- `confirmed`
- `rejected`
- `ambiguous`

Default new semantic relations to `candidate` or `needs_human_review`.
"#
    )
}

fn render_project_claude_md(kb_path: &Path) -> String {
    let name = kb_path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("knowledgebase");
    format!(
        r#"# CLAUDE.md

This repository is an LLM Wiki knowledge base named `{name}`.

Claude Code is only one external-agent adapter. The generic project contract is `AGENTS.md`; read that first.

## Claude Code Entry Order

1. Read `AGENTS.md`.
2. Read `LLM/handoff/current.md` to find the active topic/task route.
3. Read `llm-wiki.toml`.
4. Read `rules/`.
5. Read `interfaces/README.md` to understand the artifact boundary.
6. Read `LLM/handoff/index.md`.
7. Read `LLM/handoff/adapters/claude-code.md`.
8. For topic work, read `topics/<topic>/index.md`, `topics/<topic>/handoff/AGENTS.md`, and `topics/<topic>/handoff/adapters/claude-code.md`.
9. Read exactly one assigned task item.

## Claude Code Specific Rules

- Use plan mode or a short written plan before editing.
- Prefer small diffs over broad rewrites.
- Run deterministic `kb` commands when useful, but do not hide semantic judgment inside command output.
- Report changed files and unresolved review points at the end of each task.
- Never modify `raw/`.
- Treat `interfaces/` as interface review artifacts, not knowledge source files.

The generic rules in `AGENTS.md` override this adapter when there is a conflict.
"#
    )
}

fn render_global_handoff_index(kb_path: &Path, agents: &[AgentTarget]) -> String {
    let topics = collect_topic_slugs(kb_path).unwrap_or_default();
    let adapter_names: Vec<String> = agents
        .iter()
        .copied()
        .filter(|agent| agent.is_adapter())
        .map(|agent| agent.slug().to_string())
        .collect();
    let mut out = String::new();
    out.push_str("# LLM Wiki Handoff Index\n\n");
    out.push_str("This directory is the global handoff layer for external LLM agents. It explains how an agent should enter the knowledge base without hiding semantic work inside `kb-cli`.\n\n");
    out.push_str("## Handoff Levels\n\n");
    out.push_str("| level | path | purpose |\n");
    out.push_str("|---|---|---|\n");
    out.push_str(
        "| project generic | `AGENTS.md` | Repository-wide contract for all external agents. |\n",
    );
    out.push_str("| Obsidian first page | `obsidian_main.md` | Human-facing first page when the folder is opened as an Obsidian vault. |\n");
    out.push_str("| current route | `LLM/handoff/current.md` | Shared Manager routing file for topic/task discovery. |\n");
    out.push_str("| global protocol | `LLM/handoff/protocol.md` | Shared Manager/Worker workflow and status rules. |\n");
    out.push_str("| global manager | `LLM/handoff/manager.md` | How a Manager agent should inspect the whole Wiki. |\n");
    out.push_str("| global worker | `LLM/handoff/worker.md` | Rules for bounded Worker tasks. |\n");
    out.push_str("| task schema | `LLM/handoff/task_schema.md` | Expected shape of task files and work logs. |\n");
    out.push_str("| safety | `LLM/handoff/safety.md` | Agent safety and evidence boundaries. |\n");
    out.push_str("| machine handoff | `LLM/handoff/handoff.json` | Machine-readable project handoff summary. |\n");
    out.push_str("| global tasks | `LLM/tasks/index.md` | Deterministic task queue generated by `kb tasks`. |\n");
    out.push_str("| topic generic | `topics/<topic>/handoff/AGENTS.md` | Topic-local entry contract for all agents. |\n");
    out.push_str(
        "| topic tasks | `topics/<topic>/tasks/index.md` | Topic-local Manager dashboard. |\n\n",
    );

    out.push_str("## Generated Adapters\n\n");
    if adapter_names.is_empty() {
        out.push_str("No agent-specific adapters were requested. Run `kb handoff --agent claude-code`, `kb handoff --agent opencode`, `kb handoff --agent openclaw`, `kb handoff --agent codex`, or `kb handoff --all-agents` if needed.\n\n");
    } else {
        for adapter in adapter_names {
            out.push_str(&format!(
                "- `{adapter}` → `LLM/handoff/adapters/{adapter}.md`\n"
            ));
        }
        out.push('\n');
    }

    out.push_str("## Existing Topics\n\n");
    if topics.is_empty() {
        out.push_str(
            "No topic workspaces detected yet. Create one with `kb topic init <topic>`.\n\n",
        );
    } else {
        for topic in topics {
            out.push_str(&format!(
                "- `{topic}` → `topics/{topic}/handoff/AGENTS.md`\n"
            ));
        }
        out.push('\n');
    }

    out.push_str("## Safe External-Agent Entry\n\n");
    out.push_str("```bash\n");
    out.push_str("kb health\n");
    out.push_str("kb tasks\n");
    out.push_str("kb topic list\n");
    out.push_str("kb topic tasks <topic>\n");
    out.push_str("kb topic handoff <topic>\n");
    out.push_str("```\n\n");
    out.push_str("Every agent should start by reading the generic handoff files. Adapter files only explain tool-specific entry habits.\n");
    out
}

fn render_global_protocol() -> String {
    r#"# Generic Agent Handoff Protocol

This protocol applies to any external agent working on the LLM Wiki.

## Roles

- **Manager agent**: reads the global and topic state, chooses a bounded task, prevents scope drift, and requests human review for high-value claims.
- **Worker agent**: handles exactly one assigned task item and leaves a traceable work log.
- **Human reviewer**: confirms scholarly judgments, high-impact relation status changes, and synthesis conclusions.

## Default Flow

```text
kb-cli deterministic preparation
→ handoff files
→ Manager selects one task
→ Worker edits bounded files
→ deterministic checks / diff review
→ human confirmation
→ memory record
```

## Status Discipline

Use `candidate`, `needs_human_review`, `confirmed`, `rejected`, and `ambiguous` explicitly. New semantic relations should default to `candidate` or `needs_human_review`.

## Evidence Discipline

A claim is not accepted just because it is plausible. Relation evidence should identify source file paths, section extracts, citation rows, or reviewed notes already present in the Wiki.

## Artifact Discipline

Files under `interfaces/` are interface review artifacts. They may summarize or visualize knowledge, but they are not durable knowledge. If a human or agent accepts a decision from an HTML report, the accepted decision must be written back to the appropriate Markdown/JSON/TOML source file outside `interfaces/`.
"#
    .to_string()
}

fn render_global_manager_handoff() -> String {
    r#"# Global Manager Agent Handoff

The Manager agent coordinates work across the LLM Wiki. It should not directly perform every semantic task.

## Entry Checklist

1. Read `AGENTS.md`.
2. Read `LLM/handoff/current.md`.
3. Read `llm-wiki.toml`.
4. Read `rules/`.
5. Read `interfaces/README.md`.
6. Read `LLM/handoff/index.md` and `LLM/handoff/protocol.md`.
7. Run or inspect `kb health`.
8. Inspect `LLM/tasks/index.md` when it exists.
9. Inspect topic workspaces with `kb topic list`.
10. For a specific topic, switch to `topics/<topic>/index.md`, then `topics/<topic>/handoff/AGENTS.md`.

## Manager Responsibilities

- choose a topic or global task
- assign one bounded Worker task at a time
- prevent scope drift
- require evidence for relation claims
- keep candidate/confirmed/rejected status explicit
- ask for human review on high-impact academic claims

## Manager Must Not

- rewrite the whole Wiki in one pass
- turn missing evidence into confident prose
- accept topic relation candidates automatically
- hide uncertainty in polished summaries
- modify `raw/`

## Suggested First Prompt

```text
Please read AGENTS.md, LLM/handoff/current.md, llm-wiki.toml, rules/, LLM/handoff/index.md, LLM/handoff/protocol.md, and LLM/tasks/index.md if it exists. Do not modify files yet. Return: current Wiki state, available topics, highest-priority task queues, and the safest next three actions.
```
"#
    .to_string()
}

fn render_global_worker_handoff() -> String {
    r#"# Global Worker Agent Handoff

A Worker agent performs one bounded task. It is not the project owner.

## Worker Rules

1. Read the assigned task file completely.
2. Work only on listed files unless Manager/human expands scope.
3. Preserve uncertainty.
4. Cite source paths and evidence snippets already present in the knowledge base.
5. Do not modify `raw/`.
6. Do not use `interfaces/` as the source of truth; it is review-only and regenerable.
7. Do not mark relations as `confirmed` unless explicitly allowed.
8. End with changed files, unresolved issues, and review needs.

## Worker Output Template

```markdown
## Work Summary

## Files Changed

## Evidence Used

## Remaining Uncertainty

## Recommended Review
```
"#
    .to_string()
}

fn render_task_schema() -> String {
    r#"# LLM Wiki Task Schema

Task files are reviewable contracts. They should be small enough for one Worker agent to complete without roaming through the whole Wiki.

## Recommended Fields

```markdown
# TASK-XXX: <short title>

## Goal

## Scope

## Allowed Files

## Evidence Inputs

## Forbidden Actions

## Output Requirements

## Review Requirement

## Work Log
```

## Required Behavior

- If evidence is insufficient, mark the result as `needs_human_review`.
- If the task asks for relation work, preserve source, target, relation type, evidence, and status.
- If the task asks for synthesis, separate confirmed claims from candidate claims.
"#
    .to_string()
}

fn render_safety_policy() -> String {
    r#"# Agent Safety and Evidence Boundaries

## Files and Directories

Agents must not modify, rename, or delete `raw/`. Extracted files under `processing/` should be treated as evidence, not as polished prose.

## Claims

Agents must not invent references, DOIs, author names, quotes, numerical results, or relation evidence. Missing information should remain missing and be logged as a task or review issue.

## Academic Judgment

High-value academic judgments require human review before becoming confirmed, including:

- core literature importance
- contradicts / improves / causal relations
- topic synthesis conclusions
- deletion or rejection of literature from a topic

## Tool Access

Agent-specific adapters may describe tool habits, but the generic evidence and review rules remain binding.
"#
    .to_string()
}

fn render_global_handoff_json(kb_path: &Path, agents: &[AgentTarget]) -> String {
    let topics = collect_topic_slugs(kb_path).unwrap_or_default();
    let value = serde_json::json!({
        "schema_version": "handoff.v0.7.19",
        "level": "project",
        "kb_path": kb_path.display().to_string(),
        "agents": agent_names(agents),
        "generic_entry": "AGENTS.md",
        "human_entry": "obsidian_main.md",
        "manager_route": "LLM/handoff/current.md",
        "handoff_files": {
            "current": "LLM/handoff/current.md",
            "obsidian_main": "obsidian_main.md",
            "index": "LLM/handoff/index.md",
            "protocol": "LLM/handoff/protocol.md",
            "manager": "LLM/handoff/manager.md",
            "worker": "LLM/handoff/worker.md",
            "task_schema": "LLM/handoff/task_schema.md",
            "safety": "LLM/handoff/safety.md"
        },
        "adapter_dir": "LLM/handoff/adapters",
        "topics": topics,
        "rules": [
            "Do not modify raw/.",
            "Do not invent references or evidence.",
            "Default new semantic relations to candidate or needs_human_review.",
            "Human review confirms high-value academic judgments.",
            "Treat interfaces/ as interface review artifacts, not as source-of-truth knowledge."
        ]
    });
    serde_json::to_string_pretty(&value).unwrap_or_else(|_| "{}".to_string())
}

fn render_adapter_handoff(agent: AgentTarget) -> String {
    match agent {
        AgentTarget::ClaudeCode => r#"# Claude Code Adapter

Claude Code should work as an external LLM Wiki agent, not as an invisible automatic knowledge generator. `interfaces/` is a interface artifact directory, not a source-of-truth layer.

## Entry

1. Read `AGENTS.md`.
2. Read `LLM/handoff/current.md` to discover the topic/task route.
3. Read `LLM/handoff/index.md` and `LLM/handoff/protocol.md`.
4. Read this adapter.
5. For topic work, read `topics/<topic>/index.md`, `topics/<topic>/handoff/AGENTS.md`, and `topics/<topic>/handoff/adapters/claude-code.md`.
6. Read one assigned task item.

## Claude Code Habits

- Start with a short plan before editing.
- Prefer small, reviewable diffs.
- Use shell commands only to inspect or run deterministic checks.
- Show changed files and unresolved issues at the end.
- Ask for human confirmation before accepting high-value academic judgments.
"#
        .to_string(),
        AgentTarget::Opencode => r#"# OpenCode Adapter

OpenCode should read the generic LLM Wiki handoff first and then operate on one bounded task. `interfaces/` is a interface artifact directory, not a source-of-truth layer.

## Entry

1. Read `AGENTS.md`.
2. Read `LLM/handoff/current.md` to discover the topic/task route.
3. Read `LLM/handoff/index.md` and `LLM/handoff/protocol.md`.
4. Read `LLM/handoff/manager.md` or `LLM/handoff/worker.md`, depending on role.
5. For topic work, read `topics/<topic>/index.md` and `topics/<topic>/handoff/AGENTS.md`.
6. Read exactly one task item before editing.

## OpenCode Habits

- Treat the repository as a knowledge base, not just a codebase.
- Keep diffs small and file-scoped.
- Do not run broad formatters over Markdown unless assigned.
- Use deterministic `kb` commands for checks, not for hidden semantic judgment.
"#
        .to_string(),
        AgentTarget::Openclaw => r#"# OpenClaw Adapter

OpenClaw-like personal assistant agents may have broader service access than a code-only agent. For LLM Wiki work, keep the scope narrow. `interfaces/` is a interface artifact directory, not a source-of-truth layer.

## Entry

1. Read `AGENTS.md`.
2. Read `LLM/handoff/current.md` to discover the topic/task route.
3. Read `LLM/handoff/index.md` and `LLM/handoff/protocol.md`.
4. Read one task item or one human-provided task brief.
5. Do not access email, calendar, browser, or external services unless the task explicitly asks for it.

## OpenClaw Boundaries

- Use this repository's files as the source of truth.
- Do not import outside claims into the Wiki without marking them as external and unreviewed.
- Do not automate communications, scheduling, or external account actions as part of Wiki maintenance.
- Return a work note instead of silently editing high-value academic judgments.
"#
        .to_string(),
        AgentTarget::Codex => r#"# Codex Adapter

Codex should work as an external LLM Wiki coding/knowledge agent. Use it for reviewable repository work, not invisible automatic knowledge generation. `interfaces/` is a interface artifact directory, not a source-of-truth layer.

## Entry

1. Read `AGENTS.md`.
2. Read `LLM/handoff/current.md` to discover the topic/task route.
3. Read `LLM/handoff/index.md` and `LLM/handoff/protocol.md`.
4. Read `LLM/handoff/manager.md` or `LLM/handoff/worker.md`, depending on role.
5. For topic work, read `topics/<topic>/index.md`, `topics/<topic>/handoff/AGENTS.md`, and `topics/<topic>/handoff/adapters/codex.md`.
6. Read exactly one task item before editing.

## Codex Habits

- Treat the repository as a knowledge base first and a codebase second.
- Prefer small, reviewable diffs and explicit work logs.
- Use shell commands only for inspection and deterministic checks.
- Do not run broad rewrites or formatters unless the task explicitly asks for them.
- Leave high-value academic judgments as `candidate` or `needs_human_review` unless explicitly authorized.

## Suggested First Prompt

```text
Please read AGENTS.md, LLM/handoff/current.md, LLM/handoff/protocol.md, and this Codex adapter. Do not modify files yet. Return the safest one-task work plan, the files you need to inspect, and any review risks.
```
"#
        .to_string(),
        AgentTarget::Generic => String::new(),
    }
}

fn render_topic_agents_md(slug: &str, title: &str) -> String {
    format!(
        r#"# Topic Agent Handoff: {title}

Topic slug: `{slug}`

This file is the topic-local entry point for any external LLM Wiki work agent.

## Read First

1. `AGENTS.md`
2. `LLM/handoff/current.md`
3. `llm-wiki.toml`
4. `rules/`
5. `topics/{slug}/index.md`
6. `topics/{slug}/scope.md`
7. `topics/{slug}/literature.md`
8. `topics/{slug}/tasks/index.md`
9. One task item under `topics/{slug}/tasks/items/`

## Artifact Boundary

`../../interfaces/` contains interface artifacts. Use it for context only; durable topic knowledge belongs in Markdown/JSON/TOML under `topics/`, `wiki/`, `processing/`, `LLM/`, `rules/`, and the root marker.

## Topic Boundary

The topic boundary is controlled by `scope.md`. Do not expand the topic merely because a paper is interesting.

## Allowed Work

- repair or complete topic task files
- draft evidence notes for candidate relations
- draft paper cards and concept pages when assigned
- prepare synthesis outlines from confirmed or clearly labeled candidate evidence
- update topic memory after reviewed work is accepted

## Forbidden Work

- do not modify `raw/`
- do not silently convert candidate relations to confirmed
- do not add literature outside `scope.md` without marking it candidate
- do not erase uncertainty
- do not overwrite review decisions

## Recommended Local Commands

```bash
kb topic status {slug}
kb topic tasks {slug}
kb topic build {slug}
kb topic review {slug}
kb topic relations {slug}
kb view --relations --topic {slug} --no-open
```

## Handoff Files

- `handoff/protocol.md` — topic-local handoff protocol
- `handoff/manager.md` — topic Manager instructions
- `handoff/worker_task_template.md` — Worker task template
- `handoff/start_prompt.md` — generic start prompt
- `handoff/handoff.json` — machine-readable topic handoff
- `handoff/adapters/` — optional agent-specific adapters
"#
    )
}

fn render_topic_protocol(slug: &str, title: &str) -> String {
    format!(
        r#"# Topic Handoff Protocol: {title}

Topic slug: `{slug}`

## Goal

Use this topic workspace to summarize and refine ideas from multiple papers without losing evidence traceability.

## Flow

```text
read topic scope
→ inspect literature list
→ inspect topic tasks
→ process one task
→ leave evidence and uncertainty
→ request human review for high-value claims
```

## Evidence Sources

Prefer topic-local and deterministic evidence:

- `topics/{slug}/scope.md`
- `topics/{slug}/literature.md`
- `topics/{slug}/importance/`
- `topics/{slug}/relations/`
- `topics/{slug}/review/`
- `processing/sections/`
- `processing/refs/`

## Output Boundary

Do not turn a topic synthesis into a polished final literature review unless a task explicitly asks for that. The default output is a reviewable draft with status labels.

`interfaces/` files are review artifacts only. Do not store topic conclusions in HTML/reports/logs as the final knowledge source.
"#
    )
}

fn render_topic_manager_handoff(slug: &str, title: &str) -> String {
    format!(
        r#"# Topic Manager Handoff: {title}

Topic slug: `{slug}`

## Manager Goal

Coordinate topic-local work so the Wiki can summarize and refine ideas from multiple papers without losing evidence traceability.

## Entry Steps

1. Read `index.md` to understand the topic workbench.
2. Read `scope.md` and decide whether the topic boundary is usable.
3. Read `literature.md` and check whether candidate papers are inside scope.
4. Read `tasks/index.md` and choose one task.
5. Assign only one Worker task at a time.
6. Review Git diff or equivalent file diff before accepting changes.

## What Counts as Progress

- topic scope is explicit
- literature table is bounded
- importance candidates have evidence
- relation candidates have source/target/evidence/status
- confirmed claims are separated from candidate claims
- synthesis drafts state their evidential basis

## Human Review Required

Require human review before accepting:

- `core` importance labels
- `contradicts`, `improves`, or causal claims
- topic synthesis conclusions
- deletion or rejection of literature from the topic
"#
    )
}

fn render_topic_worker_template(slug: &str, title: &str) -> String {
    format!(
        r#"# Topic Worker Task Template: {title}

Topic slug: `{slug}`

Use this template when assigning an external agent a bounded topic-local task.

```text
Please process exactly one task:

Task file:
- topics/{slug}/tasks/items/<task-id>.md

Rules:
1. Read AGENTS.md, LLM/handoff/current.md, topics/{slug}/index.md, and topics/{slug}/handoff/AGENTS.md first.
2. Work only on files listed in the task unless I explicitly expand scope.
3. Do not modify raw/.
4. Do not treat interfaces/ as durable knowledge; it is disposable review context.
5. Do not invent references or evidence.
6. Keep candidate relations as candidate unless the task explicitly allows confirmation.
7. At the end, list changed files and unresolved review points.
```
"#
    )
}

fn render_topic_start_prompt(slug: &str, title: &str) -> String {
    format!(
        r#"# Generic Agent Start Prompt: {title}

Paste this when starting any external agent inside the knowledge base root.

```text
You are entering an LLM Wiki knowledge base as an external working agent.

Please do not modify files yet.

First read:
1. AGENTS.md
2. LLM/handoff/current.md
3. llm-wiki.toml
4. rules/
5. interfaces/README.md
6. LLM/handoff/index.md
7. LLM/handoff/protocol.md
8. topics/{slug}/index.md
9. topics/{slug}/handoff/AGENTS.md
10. topics/{slug}/scope.md
11. topics/{slug}/literature.md
12. topics/{slug}/tasks/index.md, if it exists

Then return:
1. the topic goal and boundary;
2. what deterministic materials already exist;
3. which task items are safe for you to process;
4. which items require human review;
5. the safest first three actions.

Do not rewrite wiki pages until I assign one specific task item.
```
"#
    )
}

fn render_topic_handoff_json(
    kb_path: &Path,
    slug: &str,
    title: &str,
    agents: &[AgentTarget],
) -> String {
    let value = serde_json::json!({
        "schema_version": "handoff.v0.7.19",
        "level": "topic",
        "kb_path": kb_path.display().to_string(),
        "topic_slug": slug,
        "topic_title": title,
        "agents": agent_names(agents),
        "generic_entry": format!("topics/{}/handoff/AGENTS.md", slug),
        "topic_homepage": format!("topics/{}/index.md", slug),
        "handoff_files": {
            "protocol": format!("topics/{}/handoff/protocol.md", slug),
            "manager": format!("topics/{}/handoff/manager.md", slug),
            "worker_template": format!("topics/{}/handoff/worker_task_template.md", slug),
            "start_prompt": format!("topics/{}/handoff/start_prompt.md", slug)
        },
        "adapter_dir": format!("topics/{}/handoff/adapters", slug),
        "task_index": format!("topics/{}/tasks/index.md", slug),
        "rules": [
            "Stay within scope.md unless asked to propose expansion.",
            "Do not modify raw/.",
            "Do not confirm semantic relations without explicit permission.",
            "Leave evidence and uncertainty visible.",
            "Treat interfaces/ as interface review artifacts, not as source-of-truth knowledge."
        ]
    });
    serde_json::to_string_pretty(&value).unwrap_or_else(|_| "{}".to_string())
}

fn render_topic_adapter_handoff(slug: &str, title: &str, agent: AgentTarget) -> String {
    match agent {
        AgentTarget::ClaudeCode => format!(
            r#"# Claude Code Topic Adapter: {title}

Topic slug: `{slug}`

Read `topics/{slug}/handoff/AGENTS.md` first. This file only adds Claude Code entry habits.

## Recommended Claude Code Prompt

```text
Please read AGENTS.md, LLM/handoff/current.md, LLM/handoff/protocol.md, topics/{slug}/index.md, topics/{slug}/handoff/AGENTS.md, topics/{slug}/scope.md, topics/{slug}/literature.md, and topics/{slug}/tasks/index.md. Do not modify files yet. Return the topic boundary, available task items, review risks, and the safest next three actions.
```

## Claude Code Rules

- Start with a short plan before editing.
- Use small diffs.
- Do not edit `raw/`.
- Do not promote relation status without explicit task permission.
- End by listing changed files and unresolved review points.
"#
        ),
        AgentTarget::Opencode => format!(
            r#"# OpenCode Topic Adapter: {title}

Topic slug: `{slug}`

OpenCode should use the generic topic handoff as its contract and process only one topic task at a time.

## Recommended Entry

1. Read `AGENTS.md`.
2. Read `LLM/handoff/current.md`.
3. Read `topics/{slug}/index.md`.
4. Read `topics/{slug}/handoff/AGENTS.md`.
5. Read `topics/{slug}/tasks/index.md`.
6. Select or accept exactly one task item.
7. Keep the diff small and reviewable.

## Boundary

Do not treat topic relation candidates as confirmed facts. Preserve evidence, uncertainty, and review status.
"#
        ),
        AgentTarget::Openclaw => format!(
            r#"# OpenClaw Topic Adapter: {title}

Topic slug: `{slug}`

OpenClaw-like agents should act only on the supplied LLM Wiki task brief. Do not use broad personal assistant capabilities unless explicitly asked.

## Recommended Entry

1. Read `AGENTS.md`.
2. Read `LLM/handoff/current.md`.
3. Read `topics/{slug}/index.md`.
4. Read `topics/{slug}/handoff/AGENTS.md`.
5. Read the assigned task item.
6. Return a work note or small Markdown change set.
7. Do not access outside services as part of topic maintenance.

## Boundary

Use local Wiki evidence first. Outside information, if explicitly requested, must be marked as external and unreviewed.
"#
        ),
        AgentTarget::Codex => format!(
            r#"# Codex Topic Adapter: {title}

Topic slug: `{slug}`

Codex should use the generic topic handoff as its contract and process only one topic task at a time.

## Recommended Codex Prompt

```text
Please read AGENTS.md, LLM/handoff/current.md, LLM/handoff/protocol.md, topics/{slug}/index.md, topics/{slug}/handoff/AGENTS.md, topics/{slug}/handoff/adapters/codex.md, topics/{slug}/scope.md, topics/{slug}/literature.md, and topics/{slug}/tasks/index.md. Do not modify files yet. Return a one-task plan, the files to inspect, and the review risks.
```

## Codex Rules

- Keep diffs small and reviewable.
- Do not edit `raw/`.
- Do not run broad Markdown rewrites unless assigned.
- Do not promote relation status without explicit task permission.
- End by listing changed files, commands run, and unresolved review points.
"#
        ),
        AgentTarget::Generic => String::new(),
    }
}

fn render_topic_claude_md(slug: &str, title: &str) -> String {
    format!(
        r#"# Topic Claude Handoff: {title}

Topic slug: `{slug}`

This file is a Claude Code compatibility entry point. The generic topic contract is `topics/{slug}/handoff/AGENTS.md`; read that first.

## Read First

1. `AGENTS.md`
2. `CLAUDE.md`
3. `LLM/handoff/current.md`
4. `llm-wiki.toml`
5. `rules/`
6. `topics/{slug}/index.md`
7. `topics/{slug}/handoff/AGENTS.md`
8. `topics/{slug}/handoff/adapters/claude-code.md`
9. `topics/{slug}/scope.md`
10. `topics/{slug}/literature.md`
11. `topics/{slug}/tasks/index.md`
12. One task item under `topics/{slug}/tasks/items/`

## Claude Code Boundary

Do not modify `raw/`. Do not silently convert candidate relations to confirmed. Use a short plan and small diffs.
"#
    )
}

fn print_handoff_report(report: &HandoffReport) {
    println!("Agent handoff:");
    println!("  wiki path  : {}", report.kb_path);
    println!("  dry run  : {}", report.dry_run);
    println!("  force    : {}", report.force);
    if !report.agents.is_empty() {
        println!("  agents   : {}", report.agents.join(", "));
    }
    if !report.topics.is_empty() {
        println!("  topics   : {}", report.topics.join(", "));
    }
    println!();
    println!("Project/global files:");
    if report.project_files.is_empty() {
        println!("  <none>");
    } else {
        for file in &report.project_files {
            println!("  - [{}] {} ({})", file.level, file.path, file.status);
        }
    }
    println!();
    println!("Topic files:");
    if report.topic_files.is_empty() {
        println!("  <none>");
    } else {
        for file in &report.topic_files {
            println!("  - [{}] {} ({})", file.level, file.path, file.status);
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
