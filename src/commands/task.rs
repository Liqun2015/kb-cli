use anyhow::{anyhow, Result};
use chrono::Utc;
use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Args)]
pub struct TaskArgs {
    #[command(subcommand)]
    pub command: TaskCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum TaskCommand {
    #[command(about = "List Worker/Human task items and their recorded states")]
    List(TaskListArgs),
    #[command(about = "Show one task item and its recorded state")]
    Show(TaskShowArgs),
    #[command(
        about = "Mark a task as assigned, in progress, completed, blocked, or needing human review"
    )]
    Status(TaskStatusArgs),
}

#[derive(Debug, Clone, Args)]
pub struct TaskListArgs {
    #[arg(
        long,
        help = "Filter by state, e.g. pending, assigned, in_progress, completed, blocked"
    )]
    pub state: Option<String>,
    #[arg(long, help = "Print JSON")]
    pub json: bool,
}

#[derive(Debug, Clone, Args)]
pub struct TaskShowArgs {
    pub id: String,
    #[arg(long, help = "Print JSON metadata instead of Markdown body")]
    pub json: bool,
}

#[derive(Debug, Clone, Args)]
pub struct TaskStatusArgs {
    pub id: String,
    #[arg(
        long,
        value_name = "STATE",
        help = "New state: pending, assigned, in_progress, needs_human, blocked, completed, rejected"
    )]
    pub mark: String,
    #[arg(long, help = "Optional note recorded with this state transition")]
    pub note: Option<String>,
    #[arg(
        long,
        help = "Optional assignee, e.g. openclaw, claude-code, codex, human"
    )]
    pub assignee: Option<String>,
    #[arg(long, help = "Print JSON")]
    pub json: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStateRecord {
    pub schema_version: String,
    pub task_id: String,
    pub state: String,
    pub assignee: Option<String>,
    pub note: Option<String>,
    pub updated_at: String,
    pub updated_by: String,
    pub task_file: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TaskListRecord {
    pub task_id: String,
    pub state: String,
    pub priority: Option<String>,
    pub category: Option<String>,
    pub target_agent: Option<String>,
    pub assignee: Option<String>,
    pub updated_at: Option<String>,
    pub task_file: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, Serialize)]
struct TaskListReport {
    schema_version: String,
    generated_by: String,
    generated_at: String,
    count: usize,
    tasks: Vec<TaskListRecord>,
}

pub fn execute(custom_kb: Option<&Path>, args: &TaskArgs) -> Result<()> {
    let kb_path = crate::commands::init::get_kb_path(custom_kb);
    if !kb_path.exists() {
        return Err(anyhow!(
            "knowledge base path does not exist: {}. Run `kb create --wiki <A> --from <B> --about <topic>` first.",
            kb_path.display()
        ));
    }

    match &args.command {
        TaskCommand::List(args) => execute_list(&kb_path, args),
        TaskCommand::Show(args) => execute_show(&kb_path, args),
        TaskCommand::Status(args) => execute_status(&kb_path, args),
    }
}

fn execute_list(kb_path: &Path, args: &TaskListArgs) -> Result<()> {
    let mut tasks = collect_task_records(kb_path)?;
    if let Some(filter) = &args.state {
        let filter = normalize_state(filter)?;
        tasks.retain(|task| task.state == filter);
    }

    let report = TaskListReport {
        schema_version: "0.7.40".to_string(),
        generated_by: "kb task list".to_string(),
        generated_at: Utc::now().to_rfc3339(),
        count: tasks.len(),
        tasks,
    };

    if args.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        println!("kb task list");
        println!("Generated at: {}", report.generated_at);
        println!("Tasks: {}", report.count);
        println!();
        if report.tasks.is_empty() {
            println!("No tasks found. Run `kb tasks` or `kb batch paper-profile --topic <topic> --limit 5`.");
        } else {
            println!("| state | task_id | priority | category | assignee | updated |");
            println!("|---|---|---|---|---|---|");
            for task in &report.tasks {
                println!(
                    "| `{}` | `{}` | `{}` | `{}` | `{}` | `{}` |",
                    task.state,
                    task.task_id,
                    task.priority.as_deref().unwrap_or(""),
                    task.category.as_deref().unwrap_or(""),
                    task.assignee.as_deref().unwrap_or(""),
                    task.updated_at.as_deref().unwrap_or("")
                );
            }
        }
    }
    Ok(())
}

fn execute_show(kb_path: &Path, args: &TaskShowArgs) -> Result<()> {
    let task_id = sanitize_task_id(&args.id);
    let state = load_task_state(kb_path, &task_id)?;
    let task_file = find_task_file(kb_path, &task_id);

    if args.json {
        let mut value = serde_json::Map::new();
        value.insert(
            "task_id".to_string(),
            serde_json::Value::String(task_id.clone()),
        );
        value.insert(
            "task_file".to_string(),
            task_file
                .as_ref()
                .map(|p| serde_json::Value::String(relative_path_string(kb_path, p)))
                .unwrap_or(serde_json::Value::Null),
        );
        value.insert(
            "state".to_string(),
            state
                .map(serde_json::to_value)
                .transpose()?
                .unwrap_or(serde_json::Value::Null),
        );
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::Value::Object(value))?
        );
        return Ok(());
    }

    println!("# Task `{}`", task_id);
    println!();
    if let Some(state) = state {
        println!("- State: `{}`", state.state);
        println!("- Assignee: `{}`", state.assignee.as_deref().unwrap_or(""));
        println!("- Updated at: `{}`", state.updated_at);
        println!("- Note: {}", state.note.as_deref().unwrap_or(""));
    } else {
        println!("- State: `pending` (implicit; no explicit state record yet)");
    }
    println!();

    if let Some(path) = task_file {
        println!("- Task file: `{}`", relative_path_string(kb_path, &path));
        println!();
        println!("---");
        println!();
        println!("{}", fs::read_to_string(path)?);
    } else {
        println!("No task item file found for `{}`.", task_id);
    }

    Ok(())
}

fn execute_status(kb_path: &Path, args: &TaskStatusArgs) -> Result<()> {
    let state = normalize_state(&args.mark)?;
    let record = mark_task_state(
        kb_path,
        &args.id,
        &state,
        args.assignee.as_deref(),
        args.note.as_deref(),
    )?;
    if args.json {
        println!("{}", serde_json::to_string_pretty(&record)?);
    } else {
        println!("Task status updated:");
        println!("  id       : {}", record.task_id);
        println!("  state    : {}", record.state);
        println!("  assignee : {}", record.assignee.as_deref().unwrap_or(""));
        println!("  updated  : {}", record.updated_at);
        if let Some(note) = &record.note {
            println!("  note     : {}", note);
        }
        if let Some(file) = &record.task_file {
            println!("  file     : {}", file);
        }
    }
    Ok(())
}

pub fn mark_task_state(
    kb_path: &Path,
    task_id: &str,
    state: &str,
    assignee: Option<&str>,
    note: Option<&str>,
) -> Result<TaskStateRecord> {
    let task_id = sanitize_task_id(task_id);
    let state = normalize_state(state)?;
    let task_file = find_task_file(kb_path, &task_id).map(|p| relative_path_string(kb_path, &p));
    let existing = load_task_state(kb_path, &task_id)?;
    let record = TaskStateRecord {
        schema_version: "0.7.40".to_string(),
        task_id: task_id.clone(),
        state,
        assignee: assignee
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty())
            .or_else(|| existing.as_ref().and_then(|v| v.assignee.clone())),
        note: note
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty())
            .or_else(|| existing.as_ref().and_then(|v| v.note.clone())),
        updated_at: Utc::now().to_rfc3339(),
        updated_by: "kb task status".to_string(),
        task_file,
    };

    let state_dir = kb_path.join("LLM/tasks/state");
    fs::create_dir_all(&state_dir)?;
    fs::write(
        state_dir.join(format!("{}.json", task_id)),
        serde_json::to_string_pretty(&record)?,
    )?;

    let event_dir = kb_path.join("LLM/tasks");
    fs::create_dir_all(&event_dir)?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(event_dir.join("status_events.jsonl"))?;
    writeln!(file, "{}", serde_json::to_string(&record)?)?;

    update_task_item_status(kb_path, &task_id, &record)?;

    Ok(record)
}

pub fn collect_task_records(kb_path: &Path) -> Result<Vec<TaskListRecord>> {
    let mut records: BTreeMap<String, TaskListRecord> = BTreeMap::new();

    for task_file in collect_task_item_files(kb_path)? {
        let Some(stem) = task_file.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        let task_id = stem.to_string();
        let metadata = parse_task_item_metadata(&task_file).unwrap_or_default();
        records.insert(
            task_id.clone(),
            TaskListRecord {
                task_id,
                state: metadata
                    .get("status")
                    .cloned()
                    .unwrap_or_else(|| "pending".to_string()),
                priority: metadata.get("priority").cloned(),
                category: metadata.get("category").cloned(),
                target_agent: metadata.get("target agent").cloned(),
                assignee: None,
                updated_at: None,
                task_file: Some(relative_path_string(kb_path, &task_file)),
                note: None,
            },
        );
    }

    let state_dir = kb_path.join("LLM/tasks/state");
    if state_dir.exists() {
        for entry in fs::read_dir(state_dir)? {
            let entry = entry?;
            if !entry.file_type()?.is_file() {
                continue;
            }
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let Ok(content) = fs::read_to_string(&path) else {
                continue;
            };
            let Ok(state) = serde_json::from_str::<TaskStateRecord>(&content) else {
                continue;
            };
            records
                .entry(state.task_id.clone())
                .and_modify(|record| {
                    record.state = state.state.clone();
                    record.assignee = state.assignee.clone();
                    record.updated_at = Some(state.updated_at.clone());
                    record.note = state.note.clone();
                    if record.task_file.is_none() {
                        record.task_file = state.task_file.clone();
                    }
                })
                .or_insert(TaskListRecord {
                    task_id: state.task_id.clone(),
                    state: state.state.clone(),
                    priority: None,
                    category: None,
                    target_agent: None,
                    assignee: state.assignee.clone(),
                    updated_at: Some(state.updated_at.clone()),
                    task_file: state.task_file.clone(),
                    note: state.note.clone(),
                });
        }
    }

    let mut list = records.into_values().collect::<Vec<_>>();
    list.sort_by(|a, b| {
        state_rank(&a.state)
            .cmp(&state_rank(&b.state))
            .then(a.task_id.cmp(&b.task_id))
    });
    Ok(list)
}

fn collect_task_item_files(kb_path: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for dir in [
        kb_path.join("LLM/tasks/items"),
        kb_path.join("LLM/tasks/batches"),
    ] {
        if !dir.exists() {
            continue;
        }
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("md") {
                    files.push(path);
                }
            }
        }
    }
    files.sort();
    Ok(files)
}

fn load_task_state(kb_path: &Path, task_id: &str) -> Result<Option<TaskStateRecord>> {
    let path = kb_path
        .join("LLM/tasks/state")
        .join(format!("{}.json", sanitize_task_id(task_id)));
    if !path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(path)?;
    Ok(Some(serde_json::from_str(&content)?))
}

fn find_task_file(kb_path: &Path, task_id: &str) -> Option<PathBuf> {
    let task_id = sanitize_task_id(task_id);
    [
        kb_path
            .join("LLM/tasks/items")
            .join(format!("{}.md", task_id)),
        kb_path
            .join("LLM/tasks/batches")
            .join(format!("{}.md", task_id)),
    ]
    .into_iter()
    .find(|path| path.exists())
}

fn update_task_item_status(kb_path: &Path, task_id: &str, record: &TaskStateRecord) -> Result<()> {
    let Some(path) = find_task_file(kb_path, task_id) else {
        return Ok(());
    };
    let content = fs::read_to_string(&path)?;
    let mut out = String::new();
    let mut replaced = false;
    for line in content.lines() {
        if line.trim_start().starts_with("- Status:") {
            out.push_str(&format!("- Status: `{}`\n", record.state));
            replaced = true;
        } else {
            out.push_str(line);
            out.push('\n');
        }
    }
    if !replaced {
        out.push_str(&format!("\n- Status: `{}`\n", record.state));
    }
    out.push_str("\n<!-- task_state -->\n");
    out.push_str(&format!("Updated: `{}`\n\n", record.updated_at));
    if let Some(assignee) = &record.assignee {
        out.push_str(&format!("Assignee: `{}`\n\n", assignee));
    }
    if let Some(note) = &record.note {
        out.push_str(&format!("Note: {}\n", note));
    }
    fs::write(path, out)?;
    Ok(())
}

fn parse_task_item_metadata(path: &Path) -> Result<BTreeMap<String, String>> {
    let content = fs::read_to_string(path)?;
    let mut map = BTreeMap::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("-") {
            continue;
        }
        let item = trimmed.trim_start_matches('-').trim();
        let Some((key, value)) = item.split_once(':') else {
            continue;
        };
        let key = key.trim().to_ascii_lowercase();
        let value = value.trim().trim_matches('`').to_string();
        match key.as_str() {
            "task id" | "status" | "priority" | "category" | "target agent" | "source command" => {
                map.insert(key, value);
            }
            _ => {}
        }
    }
    Ok(map)
}

fn normalize_state(value: &str) -> Result<String> {
    let state = value.trim().to_ascii_lowercase().replace('-', "_");
    match state.as_str() {
        "pending" | "assigned" | "in_progress" | "needs_human" | "blocked" | "completed" | "rejected" => Ok(state),
        _ => Err(anyhow!(
            "invalid task state `{}`. Use pending, assigned, in_progress, needs_human, blocked, completed, or rejected.",
            value
        )),
    }
}

fn state_rank(value: &str) -> usize {
    match value {
        "blocked" => 0,
        "needs_human" => 1,
        "in_progress" => 2,
        "assigned" => 3,
        "pending" => 4,
        "rejected" => 5,
        "completed" => 6,
        _ => 7,
    }
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
