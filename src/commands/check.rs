use anyhow::{anyhow, Result};
use chrono::Utc;
use clap::Args;
use serde::Serialize;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

use crate::commands::init::get_kb_path;

#[derive(Debug, Clone, Args)]
pub struct CheckArgs {
    #[arg(
        long = "output-dir",
        value_name = "DIR",
        help = "Output directory relative to the knowledge base. Defaults to interfaces/html/."
    )]
    pub output_dir: Option<PathBuf>,

    #[arg(long, help = "Preview the generated viewer path without writing HTML")]
    pub dry_run: bool,

    #[arg(long, help = "Alias for --dry-run")]
    pub preview: bool,

    #[arg(
        long = "no-open",
        help = "Generate the static viewer without opening the browser"
    )]
    pub no_open: bool,

    #[arg(
        long,
        help = "Generate the topic relationship overview or a single-topic graph instead of the regular dashboard"
    )]
    pub relations: bool,

    #[arg(long, hide = true, help = "Legacy option moved to `kb view`.")]
    pub wiki: bool,

    #[arg(
        long,
        value_name = "TOPIC",
        help = "Filter --relations mode to one concrete topic graph"
    )]
    pub topic: Option<String>,

    #[arg(
        long = "data-only",
        help = "In --relations mode, generate only relationship_data.json"
    )]
    pub data_only: bool,

    #[arg(long, hide = true, help = "Legacy no-op: kb check opens by default")]
    pub open: bool,
}

impl CheckArgs {
    fn is_dry_run(&self) -> bool {
        self.dry_run || self.preview
    }

    fn should_open(&self) -> bool {
        !self.no_open
    }
}

#[derive(Debug, Clone)]
struct CheckSection {
    id: String,
    title: String,
    subtitle: String,
    html: String,
}

#[derive(Debug, Clone)]
struct SourceCard {
    title: String,
    path: String,
    html: String,
}

pub fn execute(custom_kb: Option<&Path>, args: &CheckArgs) -> Result<()> {
    let kb_path = get_kb_path(custom_kb);
    if !kb_path.exists() {
        return Err(anyhow!(
            "knowledge base path does not exist: {}",
            kb_path.display()
        ));
    }

    if args.relations {
        return execute_relationship_check(&kb_path, args);
    }

    if args.wiki {
        return Err(anyhow!("`kb check --wiki` has moved to `kb view`. `kb check` is now reserved for system status and task-scene inspection."));
    }

    if args.topic.is_some() {
        eprintln!("Warning: --topic is only used with --relations; ignoring it for the regular dashboard.");
    }
    if args.data_only {
        eprintln!("Warning: --data-only is only used with --relations; ignoring it for the regular dashboard.");
    }

    let output_dir = args
        .output_dir
        .clone()
        .unwrap_or_else(|| PathBuf::from("interfaces/html"));
    let output_root = resolve_under_kb(&kb_path, &output_dir);
    let output_path = output_root.join("index.html");

    let sections = build_sections(&kb_path)?;
    let html = render_check_dashboard(&kb_path, &sections);

    if args.is_dry_run() {
        println!("kb check preview:");
        println!("  knowledge base : {}", kb_path.display());
        println!("  output         : {}", output_path.display());
        println!("  sections       : {}", sections.len());
        println!(
            "  relation page  : {}",
            output_root.join("relationship_viewer.html").display()
        );
        println!("  open browser   : {}", args.should_open());
        println!("  no files written");
        return Ok(());
    }

    fs::create_dir_all(&output_root)?;
    fs::write(&output_path, html)?;

    let relationship_data = build_relationship_data(&kb_path, args)?;
    let relationship_data_path = output_root.join("relationship_data.json");
    let relationship_html_path = output_root.join("relationship_viewer.html");
    fs::write(
        &relationship_data_path,
        serde_json::to_string_pretty(&relationship_data)?,
    )?;
    fs::write(
        &relationship_html_path,
        render_relationship_viewer(&kb_path, &relationship_data)?,
    )?;
    println!("Static HTML check dashboard generated:");
    println!("  {}", output_path.display());
    println!("Relationship graph viewer generated:");
    println!("  {}", relationship_html_path.display());
    println!("Relationship graph data generated:");
    println!("  {}", relationship_data_path.display());
    println!();
    if args.should_open() {
        open_in_default_browser(&output_path)?;
        println!("Opened in the system default browser.");
    } else {
        println!("HTML generated without opening a browser because --no-open was set.");
        println!("Open this file manually in a browser when needed.");
    }
    println!("The sidebar command box is display-only:");
    println!(
        "  it can navigate tabs and search visible content, but it cannot execute kb commands."
    );

    Ok(())
}

#[derive(Debug, Clone)]
struct WikiPageSource {
    id: String,
    title: String,
    rel_path: String,
    abs_path: PathBuf,
    folder: String,
    content: String,
    summary: String,
}

#[derive(Debug, Serialize)]
struct RelationshipData {
    meta: RelationshipMeta,
    overview: RelationshipOverview,
    nodes: Vec<RelationshipNode>,
    edges: Vec<RelationshipEdge>,
    topics: Vec<RelationshipTopic>,
    llm_manager_tasks: Vec<RelationshipTask>,
    llm_worker_tasks: Vec<RelationshipTask>,
}

#[derive(Debug, Serialize)]
struct RelationshipMeta {
    version: String,
    generated_by: String,
    generated_at: String,
    knowledge_base: String,
    default_topic: Option<String>,
    source_refs_graph: Option<String>,
    warnings: Vec<String>,
}

#[derive(Debug, Default, Serialize)]
struct RelationshipOverview {
    paper_count: usize,
    topic_count: usize,
    node_count: usize,
    edge_count: usize,
    confirmed_edge_count: usize,
    candidate_edge_count: usize,
    ambiguous_edge_count: usize,
    missing_edge_count: usize,
    llm_review_edge_count: usize,
}

#[derive(Debug, Clone, Serialize)]
struct RelationshipNode {
    id: String,
    label: String,
    kind: String,
    path: Option<String>,
    topic: Option<String>,
    status: String,
    evidence: Vec<String>,
    needs_llm_review: bool,
}

#[derive(Debug, Clone, Serialize)]
struct RelationshipEdge {
    id: String,
    source: String,
    target: String,
    kind: String,
    status: String,
    evidence: Vec<String>,
    needs_llm_review: bool,
    confidence: Option<f64>,
    topic: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct RelationshipTopic {
    slug: String,
    title: String,
    path: String,
    file_count: usize,
    relation_file_count: usize,
    importance_file_count: usize,
}

#[derive(Debug, Clone, Serialize)]
struct RelationshipTask {
    id: String,
    role: String,
    title: String,
    status: String,
    files: Vec<String>,
    evidence: Vec<String>,
}

fn execute_relationship_check(kb_path: &Path, args: &CheckArgs) -> Result<()> {
    let output_dir = args
        .output_dir
        .clone()
        .unwrap_or_else(|| PathBuf::from("interfaces/html"));
    let output_root = resolve_under_kb(kb_path, &output_dir);
    let data_path = output_root.join("relationship_data.json");
    let html_path = output_root.join("relationship_viewer.html");

    let data = build_relationship_data(kb_path, args)?;

    if args.is_dry_run() {
        println!("kb check --relations preview:");
        println!("  knowledge base : {}", kb_path.display());
        println!("  data output    : {}", data_path.display());
        if args.data_only {
            println!("  html output    : skipped (--data-only)");
        } else {
            println!("  html output    : {}", html_path.display());
        }
        println!(
            "  default topic  : {}",
            args.topic.as_deref().unwrap_or("<none>")
        );
        println!("  nodes          : {}", data.nodes.len());
        println!("  edges          : {}", data.edges.len());
        println!(
            "  open browser   : {}",
            args.should_open() && !args.data_only
        );
        println!("  no files written");
        return Ok(());
    }

    fs::create_dir_all(&output_root)?;
    fs::write(&data_path, serde_json::to_string_pretty(&data)?)?;

    println!("Relationship graph data generated:");
    println!("  {}", data_path.display());

    if args.data_only {
        println!("HTML skipped because --data-only was set.");
        return Ok(());
    }

    let html = render_relationship_viewer(kb_path, &data)?;
    fs::write(&html_path, html)?;
    println!("Relationship graph viewer generated:");
    println!("  {}", html_path.display());
    println!();

    if args.should_open() {
        open_in_default_browser(&html_path)?;
        println!("Opened in the system default browser.");
    } else {
        println!("HTML generated without opening a browser because --no-open was set.");
        println!("Open this file manually in a browser when needed.");
    }
    println!(
        "This static page is review-only. It does not call LLMs or write relationship decisions."
    );
    Ok(())
}

fn build_relationship_data(kb_path: &Path, args: &CheckArgs) -> Result<RelationshipData> {
    let mut warnings = Vec::new();
    let mut nodes_by_id: BTreeMap<String, RelationshipNode> = BTreeMap::new();
    let mut edges: Vec<RelationshipEdge> = Vec::new();
    let mut source_refs_graph = None;

    if args.topic.is_none() {
        if let Some(path) = latest_matching_path(kb_path, "processing/refs", "refs_graph_", "json")?
        {
            let rel = relative_path_string(kb_path, &path);
            source_refs_graph = Some(rel.clone());
            match fs::read_to_string(&path)
                .ok()
                .and_then(|content| serde_json::from_str::<Value>(&content).ok())
            {
                Some(value) => {
                    import_refs_graph_json(kb_path, &value, &mut nodes_by_id, &mut edges)
                }
                None => warnings.push(format!(
                    "Latest refs graph JSON could not be parsed: {}",
                    rel
                )),
            }
        }
    }

    let topics = add_topic_relationships(
        kb_path,
        args.topic.as_deref(),
        &mut nodes_by_id,
        &mut edges,
        &mut warnings,
    )?;

    edges.sort_by(|a, b| {
        a.status
            .cmp(&b.status)
            .then_with(|| a.source.cmp(&b.source))
            .then_with(|| a.target.cmp(&b.target))
            .then_with(|| a.kind.cmp(&b.kind))
    });
    dedupe_edges(&mut edges);

    let mut nodes = nodes_by_id.into_values().collect::<Vec<_>>();
    nodes.sort_by(|a, b| a.id.cmp(&b.id));

    let overview = build_relationship_overview(&nodes, &edges, topics.len());
    let llm_manager_tasks =
        build_relationship_manager_tasks(&overview, &source_refs_graph, args.topic.as_deref());
    let llm_worker_tasks =
        build_relationship_worker_tasks(&overview, &source_refs_graph, args.topic.as_deref());

    Ok(RelationshipData {
        meta: RelationshipMeta {
            version: "v0.7.40".to_string(),
            generated_by: "kb check --relations".to_string(),
            generated_at: Utc::now().to_rfc3339(),
            knowledge_base: kb_path.display().to_string(),
            default_topic: args.topic.clone(),
            source_refs_graph,
            warnings,
        },
        overview,
        nodes,
        edges,
        topics,
        llm_manager_tasks,
        llm_worker_tasks,
    })
}

fn import_refs_graph_json(
    kb_path: &Path,
    value: &Value,
    nodes_by_id: &mut BTreeMap<String, RelationshipNode>,
    edges: &mut Vec<RelationshipEdge>,
) {
    if let Some(nodes) = value.get("nodes").and_then(|v| v.as_array()) {
        for node in nodes {
            let Some(id) = json_string(node, "id") else {
                continue;
            };
            let label = json_string(node, "label").unwrap_or_else(|| id.clone());
            let mut kind = json_string(node, "kind")
                .or_else(|| json_string(node, "node_type"))
                .unwrap_or_else(|| "paper".to_string());
            if kind == "unresolved_reference" {
                kind = "missing_reference".to_string();
            }
            let path = json_string(node, "path").or_else(|| json_string(node, "source_path"));
            let status = json_string(node, "status").unwrap_or_else(|| "indexed".to_string());
            let evidence = json_string_array(node, "evidence");
            let needs_llm_review =
                json_bool(node, "needs_human_review").unwrap_or(status != "confirmed");
            let topic = json_string(node, "topic");
            insert_relationship_node(
                nodes_by_id,
                RelationshipNode {
                    id,
                    label,
                    kind,
                    path: path.map(|p| normalize_relative_path(kb_path, &p)),
                    topic,
                    status,
                    evidence,
                    needs_llm_review,
                },
            );
        }
    }

    if let Some(graph_edges) = value.get("edges").and_then(|v| v.as_array()) {
        for edge in graph_edges {
            let Some(source) = json_string(edge, "source") else {
                continue;
            };
            let Some(target) = json_string(edge, "target") else {
                continue;
            };
            let status = json_string(edge, "status").unwrap_or_else(|| "candidate".to_string());
            let id = json_string(edge, "id")
                .unwrap_or_else(|| format!("edge:{}->{}:{}", source, target, status));
            let kind = json_string(edge, "kind")
                .or_else(|| json_string(edge, "relation_type"))
                .unwrap_or_else(|| "bibliographic".to_string());
            let evidence = json_string_array(edge, "evidence");
            let needs_llm_review = json_bool(edge, "needs_human_review")
                .or_else(|| json_bool(edge, "human_final_guarantee_required"))
                .unwrap_or(!matches!(status.as_str(), "confirmed" | "accepted"));
            let confidence = json_f64(edge, "confidence");
            edges.push(RelationshipEdge {
                id,
                source,
                target,
                kind,
                status,
                evidence,
                needs_llm_review,
                confidence,
                topic: json_string(edge, "topic"),
            });
        }
    }
}

fn add_raw_paper_nodes(
    kb_path: &Path,
    nodes_by_id: &mut BTreeMap<String, RelationshipNode>,
) -> Result<()> {
    let raw_dir = kb_path.join("raw");
    if !raw_dir.exists() {
        return Ok(());
    }
    for entry in WalkDir::new(&raw_dir)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
    {
        let path = entry.path();
        let Some(ext) = path.extension().and_then(|s| s.to_str()) else {
            continue;
        };
        if !ext.eq_ignore_ascii_case("pdf") {
            continue;
        }
        let rel = relative_path_string(kb_path, path);
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("paper");
        let id = node_id_for_path("paper", &rel);
        insert_relationship_node(
            nodes_by_id,
            RelationshipNode {
                id,
                label: stem.to_string(),
                kind: "paper".to_string(),
                path: Some(rel.clone()),
                topic: None,
                status: "indexed".to_string(),
                evidence: vec![rel],
                needs_llm_review: false,
            },
        );
    }
    Ok(())
}

fn add_topic_relationships(
    kb_path: &Path,
    default_topic: Option<&str>,
    nodes_by_id: &mut BTreeMap<String, RelationshipNode>,
    edges: &mut Vec<RelationshipEdge>,
    warnings: &mut Vec<String>,
) -> Result<Vec<RelationshipTopic>> {
    let topics_dir = kb_path.join("topics");
    if !topics_dir.exists() {
        if default_topic.is_some() {
            warnings.push(
                "No topics/ directory found; the requested topic graph cannot be generated yet."
                    .to_string(),
            );
        } else {
            warnings.push("No topics/ directory found; run `kb topic init <topic>` or `kb topic build <topic>` first.".to_string());
        }
        return Ok(Vec::new());
    }

    let mut topic_paths = fs::read_dir(&topics_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .map(|entry| entry.path())
        .collect::<Vec<_>>();
    topic_paths.sort();

    let requested_slug = default_topic.map(slugify);
    if let Some(slug) = &requested_slug {
        if !topics_dir.join(slug).exists() {
            warnings.push(format!(
                "Requested topic `{}` was not found under topics/. Run `kb topic build {}` first.",
                slug, slug
            ));
            return Ok(Vec::new());
        }
        topic_paths.retain(|path| {
            path.file_name()
                .and_then(|s| s.to_str())
                .map(|name| name == slug)
                .unwrap_or(false)
        });
    }

    let mut topics = Vec::new();
    for topic_root in topic_paths {
        let slug = topic_root
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown-topic")
            .to_string();
        let title = read_topic_title(&topic_root).unwrap_or_else(|| slug.clone());
        let topic_rel_path = relative_path_string(kb_path, &topic_root);
        let relation_file_count = count_files(topic_root.join("relations"), "md");
        let importance_file_count = count_files(topic_root.join("importance"), "md");
        let file_count = count_files(topic_root.clone(), "md");
        topics.push(RelationshipTopic {
            slug: slug.clone(),
            title: title.clone(),
            path: topic_rel_path.clone(),
            file_count,
            relation_file_count,
            importance_file_count,
        });

        let topic_id = format!("topic:{}", slug);
        insert_relationship_node(
            nodes_by_id,
            RelationshipNode {
                id: topic_id.clone(),
                label: title,
                kind: "topic".to_string(),
                path: Some(topic_rel_path.clone()),
                topic: Some(slug.clone()),
                status: if requested_slug.is_some() {
                    "focused"
                } else {
                    "indexed"
                }
                .to_string(),
                evidence: vec![topic_rel_path],
                needs_llm_review: false,
            },
        );

        // `kb check --relations` without `--topic` is now a topic index page.
        // It intentionally stops at topic-level summary data and does not load
        // paper nodes or topic-local edges. Use `--topic <topic>` for the
        // concrete directed graph of one topic.
        if requested_slug.is_none() {
            continue;
        }

        add_topic_literature_edges(kb_path, &topic_root, &slug, &topic_id, nodes_by_id, edges)?;
        add_topic_importance_edges(kb_path, &topic_root, &slug, &topic_id, nodes_by_id, edges)?;
        add_topic_relation_file_edges(kb_path, &topic_root, &slug, nodes_by_id, edges)?;
    }

    Ok(topics)
}

fn add_topic_literature_edges(
    kb_path: &Path,
    topic_root: &Path,
    slug: &str,
    topic_id: &str,
    nodes_by_id: &mut BTreeMap<String, RelationshipNode>,
    edges: &mut Vec<RelationshipEdge>,
) -> Result<()> {
    let path = topic_root.join("literature.md");
    if !path.exists() {
        return Ok(());
    }
    let content = fs::read_to_string(&path).unwrap_or_default();
    let rel_path = relative_path_string(kb_path, &path);
    for (idx, line) in content.lines().enumerate() {
        let cells = markdown_table_cells(line);
        if cells.len() < 2 || is_markdown_header_row(&cells) {
            continue;
        }
        let paper = cells[0].trim();
        if paper.is_empty() || paper.eq_ignore_ascii_case("todo") {
            continue;
        }
        let role = cells
            .get(1)
            .cloned()
            .unwrap_or_else(|| "topic member".to_string());
        let status_cell = cells
            .get(2)
            .cloned()
            .unwrap_or_else(|| "candidate".to_string());
        let status = normalize_relation_status(&status_cell);
        let paper_id = node_id_for_path("paper", paper);
        insert_relationship_node(
            nodes_by_id,
            RelationshipNode {
                id: paper_id.clone(),
                label: label_from_pathish(paper),
                kind: "paper".to_string(),
                path: Some(paper.to_string()),
                topic: Some(slug.to_string()),
                status: status.clone(),
                evidence: vec![format!("{}:{}", rel_path, idx + 1)],
                needs_llm_review: !matches!(status.as_str(), "confirmed" | "accepted"),
            },
        );
        edges.push(RelationshipEdge {
            id: format!("edge:{}->{}:literature:{}", topic_id, paper_id, idx + 1),
            source: topic_id.to_string(),
            target: paper_id,
            kind: "topic_membership".to_string(),
            status: status.clone(),
            evidence: vec![format!("{}:{} | role={}", rel_path, idx + 1, role)],
            needs_llm_review: !matches!(status.as_str(), "confirmed" | "accepted"),
            confidence: None,
            topic: Some(slug.to_string()),
        });
    }
    Ok(())
}

fn add_topic_importance_edges(
    kb_path: &Path,
    topic_root: &Path,
    slug: &str,
    topic_id: &str,
    nodes_by_id: &mut BTreeMap<String, RelationshipNode>,
    edges: &mut Vec<RelationshipEdge>,
) -> Result<()> {
    let importance_dir = topic_root.join("importance");
    if !importance_dir.exists() {
        return Ok(());
    }
    let mut files = collect_markdown_files(&importance_dir);
    files.sort();
    for file in files {
        let name = file.file_name().and_then(|s| s.to_str()).unwrap_or("");
        let base_status = if name.contains("confirmed") {
            "confirmed"
        } else {
            "candidate"
        };
        let rel_path = relative_path_string(kb_path, &file);
        let content = fs::read_to_string(&file).unwrap_or_default();
        for (idx, line) in content.lines().enumerate() {
            let cells = markdown_table_cells(line);
            if cells.len() < 2 || is_markdown_header_row(&cells) {
                continue;
            }
            let paper = cells[0].trim();
            if paper.is_empty() || paper.eq_ignore_ascii_case("todo") {
                continue;
            }
            let importance = cells
                .get(1)
                .cloned()
                .unwrap_or_else(|| "unknown".to_string());
            let status = if cells.iter().any(|cell| {
                cell.eq_ignore_ascii_case("confirmed") || cell.eq_ignore_ascii_case("accepted")
            }) {
                "confirmed".to_string()
            } else {
                base_status.to_string()
            };
            let paper_id = node_id_for_path("paper", paper);
            insert_relationship_node(
                nodes_by_id,
                RelationshipNode {
                    id: paper_id.clone(),
                    label: label_from_pathish(paper),
                    kind: "paper".to_string(),
                    path: Some(paper.to_string()),
                    topic: Some(slug.to_string()),
                    status: status.clone(),
                    evidence: vec![format!("{}:{}", rel_path, idx + 1)],
                    needs_llm_review: status != "confirmed",
                },
            );
            edges.push(RelationshipEdge {
                id: format!(
                    "edge:{}->{}:importance:{}:{}",
                    topic_id,
                    paper_id,
                    slugify(&rel_path),
                    idx + 1
                ),
                source: topic_id.to_string(),
                target: paper_id,
                kind: "topic_importance".to_string(),
                status: status.clone(),
                evidence: vec![format!(
                    "{}:{} | importance={}",
                    rel_path,
                    idx + 1,
                    importance
                )],
                needs_llm_review: status != "confirmed",
                confidence: None,
                topic: Some(slug.to_string()),
            });
        }
    }
    Ok(())
}

fn add_topic_relation_file_edges(
    kb_path: &Path,
    topic_root: &Path,
    slug: &str,
    nodes_by_id: &mut BTreeMap<String, RelationshipNode>,
    edges: &mut Vec<RelationshipEdge>,
) -> Result<()> {
    let relations_dir = topic_root.join("relations");
    if !relations_dir.exists() {
        return Ok(());
    }
    let mut files = collect_markdown_files(&relations_dir);
    files.sort();
    for file in files {
        let rel_path = relative_path_string(kb_path, &file);
        let content = fs::read_to_string(&file).unwrap_or_default();
        for (idx, line) in content.lines().enumerate() {
            let cells = markdown_table_cells(line);
            if cells.len() < 3
                || is_markdown_header_row(&cells)
                || cells[0].eq_ignore_ascii_case("source")
            {
                continue;
            }
            let source = cells[0].trim();
            let relation_type = cells.get(1).map(|cell| cell.trim()).unwrap_or("related_to");
            let target = cells.get(2).map(|cell| cell.trim()).unwrap_or_default();
            if source.is_empty()
                || target.is_empty()
                || source.eq_ignore_ascii_case("todo")
                || target.eq_ignore_ascii_case("todo")
            {
                continue;
            }
            let status =
                normalize_relation_status(cells.get(3).map(String::as_str).unwrap_or("candidate"));
            let evidence = cells
                .get(4)
                .cloned()
                .unwrap_or_else(|| "topic relation file row".to_string());
            let needs_llm_review = cells
                .get(5)
                .map(|cell| parse_boolish(cell))
                .unwrap_or(!matches!(status.as_str(), "confirmed" | "accepted"));
            let source_id = node_id_for_path("paper", source);
            let target_id = node_id_for_path("paper", target);
            insert_relationship_node(
                nodes_by_id,
                RelationshipNode {
                    id: source_id.clone(),
                    label: label_from_pathish(source),
                    kind: "paper".to_string(),
                    path: Some(source.to_string()),
                    topic: Some(slug.to_string()),
                    status: "indexed".to_string(),
                    evidence: vec![format!("{}:{}", rel_path, idx + 1)],
                    needs_llm_review: false,
                },
            );
            insert_relationship_node(
                nodes_by_id,
                RelationshipNode {
                    id: target_id.clone(),
                    label: label_from_pathish(target),
                    kind: "paper".to_string(),
                    path: Some(target.to_string()),
                    topic: Some(slug.to_string()),
                    status: "indexed".to_string(),
                    evidence: vec![format!("{}:{}", rel_path, idx + 1)],
                    needs_llm_review: false,
                },
            );
            edges.push(RelationshipEdge {
                id: format!(
                    "edge:{}->{}:{}:{}",
                    source_id,
                    target_id,
                    slugify(relation_type),
                    idx + 1
                ),
                source: source_id,
                target: target_id,
                kind: relation_type.to_string(),
                status: status.clone(),
                evidence: vec![format!("{}:{} | {}", rel_path, idx + 1, evidence)],
                needs_llm_review,
                confidence: None,
                topic: Some(slug.to_string()),
            });
        }
    }
    Ok(())
}

fn parse_boolish(raw: &str) -> bool {
    let lower = raw.trim().to_ascii_lowercase();
    matches!(
        lower.as_str(),
        "true" | "yes" | "y" | "1" | "needs_human" | "needs_human_review"
    )
}

fn insert_relationship_node(
    nodes_by_id: &mut BTreeMap<String, RelationshipNode>,
    node: RelationshipNode,
) {
    nodes_by_id.entry(node.id.clone()).or_insert(node);
}

fn dedupe_edges(edges: &mut Vec<RelationshipEdge>) {
    let mut seen = BTreeMap::new();
    edges.retain(|edge| {
        let key = format!(
            "{}|{}|{}|{}",
            edge.source, edge.target, edge.kind, edge.status
        );
        if seen.contains_key(&key) {
            false
        } else {
            seen.insert(key, true);
            true
        }
    });
}

fn build_relationship_overview(
    nodes: &[RelationshipNode],
    edges: &[RelationshipEdge],
    topic_count: usize,
) -> RelationshipOverview {
    let mut overview = RelationshipOverview {
        paper_count: nodes.iter().filter(|n| n.kind == "paper").count(),
        topic_count,
        node_count: nodes.len(),
        edge_count: edges.len(),
        ..RelationshipOverview::default()
    };
    for edge in edges {
        match edge.status.as_str() {
            "confirmed" | "accepted" => overview.confirmed_edge_count += 1,
            "ambiguous" => overview.ambiguous_edge_count += 1,
            "missing" | "unresolved" => overview.missing_edge_count += 1,
            _ => overview.candidate_edge_count += 1,
        }
        if edge.needs_llm_review {
            overview.llm_review_edge_count += 1;
        }
    }
    overview
}

fn build_relationship_manager_tasks(
    overview: &RelationshipOverview,
    source_refs_graph: &Option<String>,
    topic: Option<&str>,
) -> Vec<RelationshipTask> {
    let mut files = vec!["interfaces/html/relationship_data.json".to_string()];
    if let Some(path) = source_refs_graph {
        files.push(path.clone());
    }
    if let Some(topic) = topic {
        files.push(format!("topics/{}/", slugify(topic)));
        vec![RelationshipTask {
            id: "manager:relationship-review-plan".to_string(),
            role: "Manager LLM".to_string(),
            title:
                "Plan single-topic relation review batches without making final scholarly claims"
                    .to_string(),
            status: "open".to_string(),
            files,
            evidence: vec![format!(
                "{} edges need review; {} missing/unresolved edges; {} ambiguous edges.",
                overview.llm_review_edge_count,
                overview.missing_edge_count,
                overview.ambiguous_edge_count
            )],
        }]
    } else {
        files.push("topics/".to_string());
        vec![RelationshipTask {
            id: "manager:choose-topic-relation-review".to_string(),
            role: "Manager LLM".to_string(),
            title: "Choose a topic workspace before assigning relation review work".to_string(),
            status: "open".to_string(),
            files,
            evidence: vec![format!(
                "{} topic workspaces indexed. Use `kb check --relations --topic <topic>` for a concrete topic graph.",
                overview.topic_count
            )],
        }]
    }
}

fn build_relationship_worker_tasks(
    overview: &RelationshipOverview,
    source_refs_graph: &Option<String>,
    topic: Option<&str>,
) -> Vec<RelationshipTask> {
    let mut files = vec!["interfaces/html/relationship_data.json".to_string()];
    if let Some(path) = source_refs_graph {
        files.push(path.clone());
    }
    if let Some(topic) = topic {
        files.push(format!("topics/{}/importance/", slugify(topic)));
        files.push(format!("topics/{}/relations/", slugify(topic)));
        vec![RelationshipTask {
            id: "worker:verify-candidate-edges".to_string(),
            role: "Worker LLM".to_string(),
            title: "Verify candidate, ambiguous, and missing relation evidence".to_string(),
            status: "open".to_string(),
            files,
            evidence: vec![format!(
                "candidate={}, ambiguous={}, missing={}, review_needed={}",
                overview.candidate_edge_count,
                overview.ambiguous_edge_count,
                overview.missing_edge_count,
                overview.llm_review_edge_count
            )],
        }]
    } else {
        files.push("topics/".to_string());
        vec![RelationshipTask {
            id: "worker:wait-for-topic-selection".to_string(),
            role: "Worker LLM".to_string(),
            title: "Wait for a concrete topic graph before verifying edges".to_string(),
            status: "blocked".to_string(),
            files,
            evidence: vec![
                "Overview mode lists topics only; it does not expose edge-level review work."
                    .to_string(),
            ],
        }]
    }
}

fn render_relationship_viewer(kb_path: &Path, data: &RelationshipData) -> Result<String> {
    let kb_display = html_escape(&kb_path.display().to_string());
    let kb_name_raw = kb_path
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.trim().is_empty())
        .unwrap_or("LLM Wiki");
    let kb_name = html_escape(kb_name_raw);
    let default_topic = html_escape(data.meta.default_topic.as_deref().unwrap_or("Global"));
    let json = script_json_escape(&serde_json::to_string_pretty(data)?);

    let mut html = String::new();
    html.push_str("<!DOCTYPE html>\n<html lang=\"zh-CN\">\n<head>\n<meta charset=\"UTF-8\" />\n<meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\" />\n");
    html.push_str(&format!("<title>{} - Topic Relations</title>\n", kb_name));
    html.push_str("<style>\n");
    html.push_str(RELATIONSHIP_VIEWER_CSS);
    html.push_str("\n</style>\n</head>\n<body>\n<div class=\"relation-shell\">\n");
    html.push_str("<aside class=\"relation-sidebar\">\n<div class=\"sidebar-head\">🕸️ Topic Relations</div>\n");
    html.push_str("<div class=\"meta-card\"><strong>Knowledge base</strong><br><span>");
    html.push_str(&kb_display);
    html.push_str("</span><br><strong>Focus</strong><br><span>");
    html.push_str(&default_topic);
    html.push_str("</span></div>\n<nav class=\"sidebar-links\">\n");
    html.push_str(
        "<button class=\"sidebar-link active\" data-target=\"overview\">Overview</button>\n",
    );
    html.push_str("<button class=\"sidebar-link\" data-target=\"graph\">Graph</button>\n");
    html.push_str("<button class=\"sidebar-link\" data-target=\"topics\">Topics</button>\n");
    html.push_str(
        "<button class=\"sidebar-link\" data-target=\"manager-tasks\">LLM Manager Tasks</button>\n",
    );
    html.push_str(
        "<button class=\"sidebar-link\" data-target=\"worker-tasks\">LLM Worker Tasks</button>\n",
    );
    html.push_str("<button class=\"sidebar-link\" data-target=\"nodes\">Nodes</button>\n");
    html.push_str("<button class=\"sidebar-link\" data-target=\"edges\">Edges</button>\n");
    html.push_str("<button class=\"sidebar-link\" data-target=\"raw-json\">Raw JSON</button>\n");
    html.push_str("<a class=\"sidebar-link sidebar-anchor\" href=\"index.html\">Back to Wiki Dashboard</a>\n</nav>\n");
    html.push_str("<div class=\"legend\"><div><span class=\"line solid\"></span> confirmed/accepted</div><div><span class=\"line dashed\"></span> candidate/needs review</div><div><span class=\"line dotted\"></span> missing/unresolved</div></div>\n");
    html.push_str("</aside>\n<main class=\"relation-main\">\n<header><h1>");
    html.push_str(&kb_name);
    html.push_str("</h1><p>Static topic relationship review page. Without --topic it lists topic workspaces; with --topic it shows one topic-local directed graph.</p><div class=\"artifact-notice\"><strong>Interface artifact:</strong> this HTML file is generated under <code>interfaces/</code> for human review and agent communication only. Do not treat it as the knowledge source; write accepted decisions back to Markdown/JSON/TOML outside <code>interfaces/</code>.</div></header>\n");
    html.push_str("<section class=\"panel active\" id=\"overview\"><h2>Overview</h2><div id=\"overviewGrid\" class=\"metric-grid\"></div><div id=\"warnings\"></div></section>\n");
    html.push_str("<section class=\"panel\" id=\"graph\"><h2>Graph</h2><p class=\"hint\">Solid edges are confirmed/accepted. Dashed edges are candidates or need LLM review. Dotted edges are missing or unresolved references.</p><div class=\"graph-wrap\"><svg id=\"graphSvg\" viewBox=\"0 0 1100 680\" role=\"img\" aria-label=\"Relationship graph\"></svg></div></section>\n");
    html.push_str("<section class=\"panel\" id=\"topics\"><h2>Topics</h2><div id=\"topicsTable\"></div></section>\n");
    html.push_str("<section class=\"panel\" id=\"manager-tasks\"><h2>LLM Manager Tasks</h2><div id=\"managerTasks\"></div></section>\n");
    html.push_str("<section class=\"panel\" id=\"worker-tasks\"><h2>LLM Worker Tasks</h2><div id=\"workerTasks\"></div></section>\n");
    html.push_str("<section class=\"panel\" id=\"nodes\"><h2>Nodes</h2><div id=\"nodesTable\"></div></section>\n");
    html.push_str("<section class=\"panel\" id=\"edges\"><h2>Edges</h2><div id=\"edgesTable\"></div></section>\n");
    html.push_str("<section class=\"panel\" id=\"raw-json\"><h2>Raw JSON</h2><pre id=\"rawJson\" class=\"json-window\"></pre></section>\n");
    html.push_str("</main>\n</div>\n<script id=\"relationship-data\" type=\"application/json\">\n");
    html.push_str(&json);
    html.push_str("\n</script>\n<script>\n");
    html.push_str(RELATIONSHIP_VIEWER_JS);
    html.push_str("\n</script>\n</body>\n</html>\n");
    Ok(html)
}

const RELATIONSHIP_VIEWER_CSS: &str = r#"
.artifact-notice { margin-top: 0.8rem; padding: 0.8rem 1rem; border: 1px solid #f6ad55; background: #fffaf0; color: #744210; border-radius: 12px; font-size: 0.92rem; }
.artifact-notice code { background: rgba(116,66,16,0.08); padding: 0.1rem 0.25rem; border-radius: 4px; }
* { box-sizing: border-box; margin: 0; padding: 0; font-family: "Segoe UI", Arial, sans-serif; }
body { background: #f5f7fa; color: #243042; line-height: 1.6; }
.relation-shell { display: flex; height: 100vh; min-height: 100vh; overflow: hidden; }
.relation-sidebar { width: 320px; height: 100vh; flex-shrink: 0; background: #fff; border-right: 1px solid #d9e2ec; padding: 1rem; display: flex; flex-direction: column; gap: 1rem; overflow-y: auto; }
.sidebar-head { background: #2b6cb0; color: #fff; padding: 0.85rem 1rem; border-radius: 8px; font-weight: 700; }
.meta-card { font-size: 0.86rem; background: #f7fafc; border: 1px solid #e2e8f0; border-radius: 8px; padding: 0.85rem; word-break: break-word; }
.sidebar-links { display: flex; flex-direction: column; gap: 0.45rem; }
.sidebar-link { display: block; width: 100%; text-align: left; border: 1px solid #d9e2ec; background: #f7fafc; color: #245a91; padding: 0.58rem 0.75rem; border-radius: 7px; cursor: pointer; text-decoration: none; font-weight: 600; }
.sidebar-link:hover { background: #e8f4f8; border-color: #4299e1; }
.sidebar-link.active { background: #4299e1; border-color: #4299e1; color: white; }
.legend { margin-top: auto; background: #f7fafc; border: 1px solid #e2e8f0; border-radius: 8px; padding: 0.8rem; font-size: 0.85rem; }
.line { display: inline-block; width: 42px; margin-right: 0.45rem; vertical-align: middle; border-top: 3px solid #2d3748; }
.line.dashed { border-top-style: dashed; }
.line.dotted { border-top-style: dotted; }
.relation-main { flex: 1; height: 100vh; padding: 2rem; max-width: 1260px; margin: 0 auto; overflow-y: auto; }
header { margin-bottom: 1.3rem; padding-bottom: 1rem; border-bottom: 2px solid #4299e1; }
h1 { color: #2b6cb0; font-size: 1.75rem; }
h2 { font-size: 1.25rem; color: #1f3f68; margin-bottom: 0.75rem; border-left: 4px solid #4299e1; padding-left: 0.6rem; }
.panel { display: none; background: #fff; border-radius: 10px; padding: 1.25rem; margin-bottom: 1rem; box-shadow: 0 1px 4px rgba(0,0,0,0.08); }
.panel.active { display: block; }
.metric-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(170px, 1fr)); gap: 0.85rem; }
.metric { border: 1px solid #e2e8f0; border-radius: 8px; padding: 0.9rem; background: #f8fafc; }
.metric strong { display: block; font-size: 1.45rem; color: #2b6cb0; }
.warning { border-left: 4px solid #d69e2e; background: #fffaf0; padding: 0.75rem; margin-top: 0.85rem; border-radius: 6px; }
.hint { color: #64748b; margin-bottom: 0.8rem; }
.graph-wrap { width: 100%; overflow: auto; border: 1px solid #d9e2ec; border-radius: 10px; background: #fbfdff; }
#graphSvg { width: 100%; min-width: 900px; height: 680px; }
.node circle { fill: #fff; stroke: #2b6cb0; stroke-width: 2; }
.node.topic circle { fill: #e8f4f8; }
.node.missing_reference circle { stroke-dasharray: 5 4; }
.node text { font-size: 12px; fill: #1f2937; pointer-events: none; }
.edge { stroke: #4a5568; stroke-width: 1.6; fill: none; opacity: 0.82; }
.edge.candidate, .edge.ambiguous { stroke-dasharray: 7 5; }
.edge.missing, .edge.unresolved { stroke-dasharray: 2 5; }
table { width: 100%; border-collapse: collapse; margin-top: 0.7rem; font-size: 0.9rem; }
th, td { border: 1px solid #e2e8f0; padding: 0.55rem; text-align: left; vertical-align: top; }
th { background: #f7fafc; }
.badge { display: inline-block; border-radius: 999px; padding: 0.1rem 0.5rem; background: #edf2f7; font-size: 0.78rem; font-weight: 700; }
.badge.confirmed, .badge.accepted { background: #e6fffa; }
.badge.candidate { background: #ebf8ff; }
.badge.ambiguous { background: #fffaf0; }
.badge.missing, .badge.unresolved { background: #fff5f5; }
.task-card { border: 1px solid #e2e8f0; border-radius: 8px; padding: 1rem; margin: 0.8rem 0; background: #f8fafc; }
pre { white-space: pre-wrap; overflow: auto; background: #1a202c; color: #edf2f7; padding: 1rem; border-radius: 8px; }
.json-window { max-height: 560px; min-height: 300px; overflow: auto; white-space: pre; border: 1px solid #2d3748; }
.empty { padding: 1rem; border: 1px dashed #cbd5e0; background: #f7fafc; color: #718096; border-radius: 8px; }
@media (max-width: 860px) { .relation-shell { flex-direction: column; height: auto; min-height: 100vh; overflow: visible; } .relation-sidebar { width: 100%; height: auto; max-height: none; } .relation-main { height: auto; padding: 1rem; overflow: visible; } }
    "#;

const RELATIONSHIP_VIEWER_JS: &str = r##"
const relationData = JSON.parse(document.getElementById('relationship-data').textContent);
const rawJson = JSON.stringify(relationData, null, 2);

document.querySelectorAll('[data-target]').forEach(btn => {
  btn.addEventListener('click', () => showPanel(btn.dataset.target));
});

function showPanel(id) {
  document.querySelectorAll('.panel').forEach(panel => panel.classList.remove('active'));
  document.querySelectorAll('[data-target]').forEach(btn => btn.classList.remove('active'));
  const panel = document.getElementById(id);
  if (panel) panel.classList.add('active');
  document.querySelectorAll(`[data-target="${id}"]`).forEach(btn => btn.classList.add('active'));
}

function esc(value) {
  return String(value ?? '').replace(/[&<>"']/g, c => ({'&':'&amp;', '<':'&lt;', '>':'&gt;', '"':'&quot;', "'":'&#39;'}[c]));
}

function badge(value) {
  const v = esc(value || 'unknown');
  return `<span class="badge ${v}">${v}</span>`;
}

function renderOverview() {
  const o = relationData.overview || {};
  const labels = [
    ['Nodes', o.node_count], ['Edges', o.edge_count], ['Papers', o.paper_count], ['Topics', o.topic_count],
    ['Confirmed', o.confirmed_edge_count], ['Candidate', o.candidate_edge_count], ['Ambiguous', o.ambiguous_edge_count], ['Missing', o.missing_edge_count],
    ['Needs LLM Review', o.llm_review_edge_count]
  ];
  document.getElementById('overviewGrid').innerHTML = labels.map(([label, value]) => `<div class="metric"><strong>${esc(value ?? 0)}</strong>${esc(label)}</div>`).join('');
  const warnings = relationData.meta?.warnings || [];
  document.getElementById('warnings').innerHTML = warnings.map(w => `<div class="warning">${esc(w)}</div>`).join('');
}

function renderTopics() {
  const topics = relationData.topics || [];
  if (!topics.length) {
    document.getElementById('topicsTable').innerHTML = '<div class="empty">No topic workspace found.</div>';
    return;
  }
  document.getElementById('topicsTable').innerHTML = table(['slug', 'title', 'path', 'files', 'relations', 'importance'], topics.map(t => [t.slug, t.title, t.path, t.file_count, t.relation_file_count, t.importance_file_count]));
}

function renderTasks(id, tasks) {
  const root = document.getElementById(id);
  if (!tasks || !tasks.length) {
    root.innerHTML = '<div class="empty">No task entry generated.</div>';
    return;
  }
  root.innerHTML = tasks.map(task => `<article class="task-card"><h3>${esc(task.title)}</h3><p>${badge(task.status)} ${esc(task.role)}</p><p><strong>Files:</strong> ${esc((task.files || []).join(', '))}</p><p><strong>Evidence:</strong> ${esc((task.evidence || []).join(' | '))}</p></article>`).join('');
}

function renderTables() {
  const nodes = relationData.nodes || [];
  const edges = relationData.edges || [];
  document.getElementById('nodesTable').innerHTML = nodes.length ? table(['id', 'label', 'kind', 'status', 'path'], nodes.map(n => [n.id, n.label, n.kind, badge(n.status), n.path || ''])) : '<div class="empty">No nodes found.</div>';
  document.getElementById('edgesTable').innerHTML = edges.length ? table(['source', 'target', 'kind', 'status', 'review', 'evidence'], edges.map(e => [e.source, e.target, e.kind, badge(e.status), e.needs_llm_review ? 'yes' : 'no', (e.evidence || []).join(' | ')])) : '<div class="empty">No edges found.</div>';
  document.getElementById('rawJson').textContent = rawJson;
}

function table(headers, rows) {
  return `<table><thead><tr>${headers.map(h => `<th>${esc(h)}</th>`).join('')}</tr></thead><tbody>${rows.map(row => `<tr>${row.map(cell => `<td>${String(cell).startsWith('<span class="badge') ? cell : esc(cell)}</td>`).join('')}</tr>`).join('')}</tbody></table>`;
}

function renderGraph() {
  const svg = document.getElementById('graphSvg');
  const allNodes = relationData.nodes || [];
  const allEdges = relationData.edges || [];
  svg.innerHTML = '';
  if (!allNodes.length) {
    svg.innerHTML = '<text x="40" y="60" font-size="18" fill="#718096">No relationship data found yet. Run preparation / indexing commands first.</text>';
    return;
  }
  const nodes = allNodes.slice(0, 90);
  const nodeIds = new Set(nodes.map(n => n.id));
  const edges = allEdges.filter(e => nodeIds.has(e.source) && nodeIds.has(e.target)).slice(0, 180);
  const cx = 550, cy = 330, radius = Math.min(270, 90 + nodes.length * 4);
  const pos = new Map();
  nodes.forEach((node, i) => {
    const angle = (Math.PI * 2 * i) / Math.max(nodes.length, 1) - Math.PI / 2;
    pos.set(node.id, { x: cx + radius * Math.cos(angle), y: cy + radius * Math.sin(angle) });
  });
  const edgeLayer = document.createElementNS('http://www.w3.org/2000/svg', 'g');
  const nodeLayer = document.createElementNS('http://www.w3.org/2000/svg', 'g');
  svg.appendChild(edgeLayer);
  svg.appendChild(nodeLayer);
  edges.forEach(edge => {
    const a = pos.get(edge.source), b = pos.get(edge.target);
    if (!a || !b) return;
    const line = document.createElementNS('http://www.w3.org/2000/svg', 'line');
    line.setAttribute('x1', a.x); line.setAttribute('y1', a.y); line.setAttribute('x2', b.x); line.setAttribute('y2', b.y);
    line.setAttribute('class', `edge ${edge.status || 'candidate'}`);
    line.appendChild(title(`${edge.kind}: ${edge.status}\n${edge.source} → ${edge.target}\n${(edge.evidence || []).join('\n')}`));
    edgeLayer.appendChild(line);
  });
  nodes.forEach(node => {
    const p = pos.get(node.id);
    const g = document.createElementNS('http://www.w3.org/2000/svg', 'g');
    g.setAttribute('class', `node ${node.kind || ''}`);
    const c = document.createElementNS('http://www.w3.org/2000/svg', 'circle');
    c.setAttribute('cx', p.x); c.setAttribute('cy', p.y); c.setAttribute('r', node.kind === 'topic' ? 18 : 13);
    const t = document.createElementNS('http://www.w3.org/2000/svg', 'text');
    t.setAttribute('x', p.x + 16); t.setAttribute('y', p.y + 4);
    const short = (node.label || node.id).length > 36 ? (node.label || node.id).slice(0, 34) + '…' : (node.label || node.id);
    t.textContent = short;
    g.appendChild(c); g.appendChild(t); g.appendChild(title(`${node.kind}: ${node.label}\n${node.path || ''}\nstatus: ${node.status}`));
    nodeLayer.appendChild(g);
  });
}

function title(text) {
  const el = document.createElementNS('http://www.w3.org/2000/svg', 'title');
  el.textContent = text;
  return el;
}

renderOverview();
renderTopics();
renderTasks('managerTasks', relationData.llm_manager_tasks);
renderTasks('workerTasks', relationData.llm_worker_tasks);
renderTables();
renderGraph();
    "##;

fn latest_matching_path(
    kb_path: &Path,
    rel_dir: &str,
    prefix: &str,
    extension: &str,
) -> Result<Option<PathBuf>> {
    let dir = kb_path.join(rel_dir);
    if !dir.exists() {
        return Ok(None);
    }
    let mut candidates = fs::read_dir(&dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().map(|t| t.is_file()).unwrap_or(false))
        .map(|entry| entry.path())
        .filter(|path| {
            let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
            let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
            name.starts_with(prefix) && ext.eq_ignore_ascii_case(extension)
        })
        .collect::<Vec<_>>();
    candidates.sort();
    Ok(candidates.pop())
}

fn read_topic_title(topic_root: &Path) -> Option<String> {
    for rel in ["README.md", "scope.md", "literature.md"] {
        let Ok(content) = fs::read_to_string(topic_root.join(rel)) else {
            continue;
        };
        for line in content.lines() {
            let trimmed = line.trim();
            if let Some(title) = trimmed.strip_prefix("# ") {
                let clean = title.trim();
                if !clean.is_empty() {
                    return Some(clean.to_string());
                }
            }
        }
    }
    None
}

fn markdown_table_cells(line: &str) -> Vec<String> {
    let trimmed = line.trim();
    if !trimmed.starts_with('|') || !trimmed.ends_with('|') || trimmed.contains("|---") {
        return Vec::new();
    }
    trimmed
        .trim_matches('|')
        .split('|')
        .map(|cell| cell.trim().trim_matches('`').to_string())
        .collect()
}

fn is_markdown_header_row(cells: &[String]) -> bool {
    cells.is_empty()
        || cells.iter().all(|cell| {
            cell.chars()
                .all(|ch| ch == '-' || ch == ':' || ch.is_whitespace())
        })
        || cells[0].eq_ignore_ascii_case("paper")
        || cells[0].eq_ignore_ascii_case("source_item")
        || cells[0].eq_ignore_ascii_case("todo")
}

fn normalize_relation_status(raw: &str) -> String {
    let lower = raw.trim().to_ascii_lowercase();
    if lower.contains("confirm") || lower.contains("accept") || lower == "done" {
        "confirmed".to_string()
    } else if lower.contains("ambiguous") || lower.contains("uncertain") {
        "ambiguous".to_string()
    } else if lower.contains("missing") || lower.contains("unresolved") {
        "missing".to_string()
    } else {
        "candidate".to_string()
    }
}

fn node_id_for_path(kind: &str, path: &str) -> String {
    format!("{}:{}", kind, slugify(path))
}

fn label_from_pathish(path: &str) -> String {
    Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .filter(|s| !s.is_empty())
        .unwrap_or(path)
        .to_string()
}

fn normalize_relative_path(kb_path: &Path, raw: &str) -> String {
    let path = Path::new(raw);
    if path.is_absolute() {
        relative_path_string(kb_path, path)
    } else {
        raw.replace('\\', "/")
    }
}

fn json_string(value: &Value, key: &str) -> Option<String> {
    value.get(key)?.as_str().map(|s| s.to_string())
}

fn json_bool(value: &Value, key: &str) -> Option<bool> {
    value.get(key)?.as_bool()
}

fn json_f64(value: &Value, key: &str) -> Option<f64> {
    value.get(key)?.as_f64()
}

fn json_string_array(value: &Value, key: &str) -> Vec<String> {
    value
        .get(key)
        .and_then(|v| v.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default()
}

fn script_json_escape(json: &str) -> String {
    json.replace("</", "<\\/")
}

fn open_in_default_browser(path: &Path) -> Result<()> {
    let status = if cfg!(target_os = "windows") {
        // Use Windows FileProtocolHandler instead of `cmd /C start`.
        // `start` can fail with "Access is denied" on some machines when
        // opening .html files through a broken or restricted file association.
        Command::new("rundll32.exe")
            .arg("url.dll,FileProtocolHandler")
            .arg(path)
            .status()?
    } else if cfg!(target_os = "macos") {
        Command::new("open").arg(path).status()?
    } else {
        Command::new("xdg-open").arg(path).status()?
    };

    if status.success() {
        Ok(())
    } else {
        Err(anyhow!(
            "failed to open viewer in the system default browser: {}",
            path.display()
        ))
    }
}

fn build_sections(kb_path: &Path) -> Result<Vec<CheckSection>> {
    let mut sections = Vec::new();

    sections.push(CheckSection {
        id: "overview".to_string(),
        title: "Overview".to_string(),
        subtitle: "Local LLM Wiki structure and latest generated artifacts".to_string(),
        html: render_overview(kb_path),
    });

    sections.push(CheckSection {
        id: "wiki".to_string(),
        title: "Wiki".to_string(),
        subtitle: "Wiki home or project README".to_string(),
        html: render_source_card_or_empty(
            read_first_existing(kb_path, &["wiki/Home.md", "wiki/index.md", "README.md"])?,
            "No wiki Home.md or README.md was found.",
        ),
    });

    sections.push(CheckSection {
        id: "refs-index".to_string(),
        title: "Refs Index".to_string(),
        subtitle: "Latest bibliographic index relation candidate report".to_string(),
        html: render_refs_index(kb_path)?,
    });

    sections.push(CheckSection {
        id: "refs-graph".to_string(),
        title: "Refs Graph".to_string(),
        subtitle: "Latest graph export files for third-party visualizers".to_string(),
        html: render_refs_graph(kb_path)?,
    });

    sections.push(CheckSection {
        id: "keywords".to_string(),
        title: "Keywords".to_string(),
        subtitle: "Latest keyword/topic relation candidate report".to_string(),
        html: render_source_card_or_empty(
            latest_matching_file(kb_path, "processing/keywords", "keywords_", "md")?,
            "No keyword report found. Run `kb keywords` first.",
        ),
    });

    sections.push(CheckSection {
        id: "health".to_string(),
        title: "Health".to_string(),
        subtitle: "Latest deterministic project health report".to_string(),
        html: render_source_card_or_empty(
            latest_matching_file(kb_path, "interfaces/reports", "health_", "md")?,
            "No health report found. Run `kb health` first.",
        ),
    });

    sections.push(CheckSection {
        id: "llm-launch".to_string(),
        title: "About launching LLM".to_string(),
        subtitle: "Copy safe launch commands and Manager prompts for external LLM agents"
            .to_string(),
        html: render_llm_launch(kb_path)?,
    });

    sections.push(CheckSection {
        id: "tasks".to_string(),
        title: "LLM Tasks".to_string(),
        subtitle: "Latest handoff task list and task progress for Manager/Worker LLM workflows"
            .to_string(),
        html: render_tasks_dashboard(kb_path)?,
    });

    sections.push(CheckSection {
        id: "memory".to_string(),
        title: "LLM Memory".to_string(),
        subtitle: "Completed task memory records".to_string(),
        html: render_source_card_or_empty(
            read_first_existing(kb_path, &["LLM/memory/completed_tasks.md"])?,
            "No completed task memory found. Run `kb memory --task-id ... --summary ...` after completing work.",
        ),
    });

    sections.push(CheckSection {
        id: "topics".to_string(),
        title: "Topics".to_string(),
        subtitle: "Topic-specific relationship overlays".to_string(),
        html: render_topics(kb_path)?,
    });

    Ok(sections)
}

fn render_tasks_dashboard(kb_path: &Path) -> Result<String> {
    let mut html = String::new();
    let records = crate::commands::task::collect_task_records(kb_path).unwrap_or_default();
    html.push_str("<div class=\"suggestion-box\"><strong>Task progress command:</strong> use <code>kb task list</code>, <code>kb task show &lt;id&gt;</code>, and <code>kb task status &lt;id&gt; --mark &lt;state&gt;</code> so the Manager can resume after restart.</div>\n");

    if records.is_empty() {
        html.push_str("<div class=\"empty\">No explicit task items or task state records were found. Run <code>kb tasks</code> or <code>kb batch paper-profile --topic &lt;topic&gt; --limit 5</code>.</div>\n");
    } else {
        let mut counts: BTreeMap<String, usize> = BTreeMap::new();
        for record in &records {
            *counts.entry(record.state.clone()).or_insert(0) += 1;
        }
        html.push_str("<div class=\"task-stats\">\n");
        for state in [
            "pending",
            "assigned",
            "in_progress",
            "needs_human",
            "blocked",
            "completed",
            "rejected",
        ] {
            let count = counts.get(state).copied().unwrap_or(0);
            html.push_str(&format!(
                "<div class=\"task-stat task-state-{}\"><span>{}</span><strong>{}</strong></div>\n",
                html_escape(state),
                html_escape(state),
                count
            ));
        }
        html.push_str("</div>\n");

        html.push_str("<h3>Task Queue</h3>\n<table><thead><tr><th>state</th><th>task_id</th><th>priority</th><th>category</th><th>assignee</th><th>updated</th><th>file</th></tr></thead><tbody>\n");
        for record in &records {
            html.push_str(&format!(
                "<tr><td><span class=\"badge {}\">{}</span></td><td><code>{}</code></td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>\n",
                html_escape(&record.state),
                html_escape(&record.state),
                html_escape(&record.task_id),
                html_escape(record.priority.as_deref().unwrap_or("")),
                html_escape(record.category.as_deref().unwrap_or("")),
                html_escape(record.assignee.as_deref().unwrap_or("")),
                html_escape(record.updated_at.as_deref().unwrap_or("")),
                record.task_file.as_deref().map(|path| format!("<code>{}</code>", html_escape(path))).unwrap_or_default(),
            ));
        }
        html.push_str("</tbody></table>\n");
    }

    html.push_str(
        "<details class=\"compact-toggle\"><summary>Latest task handoff snapshot</summary>\n",
    );
    html.push_str(&render_source_card_or_empty(
        latest_matching_file(kb_path, "LLM/tasks", "llm_tasks_", "md")?,
        "No LLM task handoff found. Run `kb tasks` first.",
    ));
    html.push_str("</details>\n");
    Ok(html)
}

fn render_check_dashboard(kb_path: &Path, sections: &[CheckSection]) -> String {
    let generated_at = html_escape(&Utc::now().to_rfc3339());
    let kb_display = html_escape(&kb_path.display().to_string());
    let kb_name_raw = kb_path
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.trim().is_empty())
        .unwrap_or("LLM Wiki");
    let kb_name = html_escape(kb_name_raw);

    let mut nav_buttons = String::new();
    let mut sidebar_links = String::new();
    let mut section_html = String::new();

    for (idx, section) in sections.iter().enumerate() {
        let active = if idx == 0 { " active" } else { "" };
        nav_buttons.push_str(&format!(
            "<button class=\"nav-tab{active}\" data-target=\"{}\">{}</button>\n",
            html_escape(&section.id),
            html_escape(&section.title)
        ));
        sidebar_links.push_str(&format!(
            "<button class=\"sidebar-link{active}\" data-target=\"{}\">{}</button>\n",
            html_escape(&section.id),
            html_escape(&section.title)
        ));
        section_html.push_str(&format!(
            "<section class=\"content-section{active}\" id=\"{}\">\n<h2>{}</h2>\n<p class=\"section-subtitle\">{}</p>\n{}\n</section>\n",
            html_escape(&section.id),
            html_escape(&section.title),
            html_escape(&section.subtitle),
            section.html
        ));
    }

    let mut html = String::new();
    html.push_str("<!DOCTYPE html>\n<html lang=\"zh-CN\">\n<head>\n<meta charset=\"UTF-8\" />\n<meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\" />\n");
    html.push_str(&format!(
        "<title>{} - LLM Wiki Check Dashboard</title>\n",
        kb_name
    ));
    html.push_str("<style>\n");
    html.push_str(VIEWER_CSS);
    html.push_str("\n</style>\n</head>\n<body>\n<div class=\"container\">\n");
    html.push_str("<aside class=\"sidebar\" id=\"wikiSidebar\">\n");
    html.push_str("<div class=\"sidebar-head\">📚 LLM Wiki Navigator</div>\n");
    html.push_str("<div class=\"sidebar-body\">\n");
    html.push_str("<div class=\"sidebar-scroll\">\n");
    html.push_str("<div class=\"meta-card\"><strong>Knowledge base</strong><br><span>");
    html.push_str(&kb_display);
    html.push_str("</span><br><strong>Generated</strong><br><span>");
    html.push_str(&generated_at);
    html.push_str("</span></div>\n");
    html.push_str("<div class=\"sidebar-links\">\n");
    html.push_str(&sidebar_links);
    html.push_str("<a class=\"sidebar-link sidebar-anchor\" href=\"relationship_viewer.html\">Topic Relations</a>\n");
    html.push_str("</div>\n</div>\n");
    html.push_str("<div class=\"terminal\">\n<div class=\"terminal-head\">kb-check&gt; display commands only</div>\n<div class=\"terminal-log\" id=\"commandLog\"><div class=\"terminal-msg\">Type <code>help</code>. This box cannot execute local kb commands.</div></div>\n<div class=\"terminal-input\"><input id=\"checkCommand\" type=\"text\" placeholder=\"help / open health / find DOI\"/><button id=\"runCheckCommand\">Run</button></div>\n</div>\n");
    html.push_str(
        "</div>\n</aside>\n<button class=\"toggle-sidebar\" id=\"toggleBtn\">‹</button>\n",
    );
    html.push_str("<main class=\"main\">\n<header><h1>");
    html.push_str(&kb_name);
    html.push_str("</h1><p class=\"subtitle\">LLM Wiki of your knowledge base for the purpose of swift and high quality research</p><div class=\"header-actions\"><a class=\"secondary-action\" href=\"browse.html\">用户视图</a><a class=\"secondary-action\" href=\"relationship_viewer.html\">查看关系图</a></div><div class=\"artifact-notice\"><strong>Interface artifact:</strong> this HTML file is generated under <code>interfaces/</code> for human review and agent communication only. Do not treat it as the knowledge source; write accepted decisions back to Markdown/JSON/TOML outside <code>interfaces/</code>.</div></header>\n");
    html.push_str("<nav class=\"nav-tabs\">\n");
    html.push_str(&nav_buttons);
    html.push_str("</nav>\n");
    html.push_str(&section_html);
    html.push_str("</main>\n</div>\n<script>\n");
    html.push_str(VIEWER_JS);
    html.push_str("\n</script>\n</body>\n</html>\n");
    html
}

const VIEWER_CSS: &str = r#"
* { margin: 0; padding: 0; box-sizing: border-box; font-family: "Segoe UI", Arial, sans-serif; }
body { background: #f5f7fa; color: #2d3748; line-height: 1.6; }
html, body { min-height: 100%; }
.artifact-notice { margin-top: 0.8rem; padding: 0.8rem 1rem; border: 1px solid #f6ad55; background: #fffaf0; color: #744210; border-radius: 12px; font-size: 0.92rem; }
.artifact-notice code { background: rgba(116,66,16,0.08); padding: 0.1rem 0.25rem; border-radius: 4px; }
.container { display: flex; height: 100vh; min-height: 100vh; position: relative; overflow: hidden; }
.sidebar { width: 340px; height: 100vh; background: #fff; border-right: 1px solid #e2e8f0; display: flex; flex-direction: column; transition: width 0.2s; overflow: hidden; flex-shrink: 0; }
.sidebar.hidden { width: 0; }
.sidebar-head { padding: 1rem 1.2rem; background: #2b6cb0; color: white; font-weight: 700; }
.sidebar-body { flex: 1; min-height: 0; padding: 1rem; overflow: hidden; display: flex; flex-direction: column; gap: 1rem; }
.sidebar-scroll { flex: 1 1 auto; min-height: 0; overflow-y: auto; display: flex; flex-direction: column; gap: 1rem; padding-right: 0.15rem; }
.meta-card { font-size: 0.85rem; background: #f7fafc; border: 1px solid #e2e8f0; border-radius: 8px; padding: 0.8rem; word-break: break-all; }
.sidebar-links { display: flex; flex-direction: column; gap: 0.4rem; }
.sidebar-link { text-align: left; border: 1px solid #e2e8f0; background: #f7fafc; color: #2b6cb0; padding: 0.55rem 0.7rem; border-radius: 6px; cursor: pointer; }
.sidebar-anchor { display: block; text-decoration: none; font-weight: 600; }
.sidebar-link:hover { background: #e8f4f8; }
.sidebar-link.active { background: #4299e1; border-color: #4299e1; color: white; font-weight: 700; }
.toggle-sidebar { position: absolute; top: 12px; left: 340px; z-index: 10; background: #2b6cb0; color: white; border: 0; width: 32px; height: 32px; border-radius: 4px; cursor: pointer; transition: left 0.2s; }
.toggle-sidebar.collapsed { left: 0; }
.terminal { margin-top: auto; flex: 0 0 340px; min-height: 320px; border: 1px solid #cbd5e0; border-radius: 8px; overflow: hidden; background: #1a202c; color: #e2e8f0; display: flex; flex-direction: column; }
.terminal-head { padding: 0.5rem 0.7rem; background: #2d3748; font-size: 0.85rem; }
.terminal-log { flex: 1; min-height: 230px; padding: 0.85rem; overflow-y: auto; font-size: 0.84rem; }
.terminal-msg { margin-bottom: 0.45rem; }
.terminal-input { display: flex; border-top: 1px solid #4a5568; }
.terminal-input input { flex: 1; min-height: 46px; padding: 0.8rem; border: 0; outline: 0; background: #edf2f7; color: #1a202c; font-size: 0.9rem; }
.terminal-input button { min-height: 46px; padding: 0.8rem 0.95rem; border: 0; background: #4299e1; color: white; cursor: pointer; font-weight: 600; }
.main { flex: 1; height: 100vh; padding: 2rem; max-width: 1180px; margin: 0 auto; overflow-x: hidden; overflow-y: auto; }
@media (max-width: 860px) { .container { flex-direction: column; height: auto; min-height: 100vh; overflow: visible; } .sidebar { width: 100%; height: auto; max-height: none; } .sidebar.hidden { width: 0; height: 0; } .main { height: auto; padding: 1rem; overflow: visible; } .toggle-sidebar { display: none; } }
header { text-align: center; margin-bottom: 1.5rem; padding-bottom: 1rem; border-bottom: 2px solid #4299e1; }
h1 { color: #2b6cb0; font-size: 1.65rem; }
.subtitle, .section-subtitle, .path { color: #718096; font-size: 0.92rem; }

.header-actions { display: flex; gap: 0.65rem; flex-wrap: wrap; margin-top: 1rem; }
.primary-action, .secondary-action, .copy-btn { border: 0; border-radius: 8px; padding: 0.62rem 0.9rem; font-weight: 700; cursor: pointer; text-decoration: none; display: inline-flex; align-items: center; gap: 0.35rem; }
.primary-action { background: #2b6cb0; color: white; }
.secondary-action { background: #e8f4f8; color: #2b6cb0; border: 1px solid #bee3f8; }
.copy-btn { background: #edf2f7; color: #2d3748; border: 1px solid #cbd5e0; font-size: 0.85rem; padding: 0.45rem 0.7rem; }
.copy-btn:hover, .primary-action:hover, .secondary-action:hover { filter: brightness(0.97); }
.llm-launch-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(280px, 1fr)); gap: 1rem; margin: 1rem 0; }
.llm-card { border: 1px solid #e2e8f0; border-radius: 10px; padding: 1rem; background: #f8fafc; }
.llm-card h3 { margin-top: 0; }
.launch-snippet { position: relative; }
.launch-snippet pre { margin-top: 0.45rem; }
.launch-note { background: #fffaf0; border: 1px solid #f6ad55; color: #744210; border-radius: 8px; padding: 0.8rem; margin: 0.8rem 0; }

.nav-tabs { display: flex; gap: 0.5rem; background: #fff; padding: 0.5rem; border-radius: 8px; box-shadow: 0 1px 3px rgba(0,0,0,0.1); margin-bottom: 1.5rem; flex-wrap: wrap; }
.nav-tab { padding: 0.55rem 1rem; border: 0; border-radius: 5px; background: #e8f4f8; color: #2b6cb0; cursor: pointer; }
.nav-tab.active { background: #4299e1; color: white; }
.content-section { display: none; background: white; padding: 1.5rem; border-radius: 8px; box-shadow: 0 1px 4px rgba(0,0,0,0.1); margin-bottom: 1rem; }
.content-section.active { display: block; }
h2 { font-size: 1.3rem; margin-bottom: 0.4rem; border-left: 4px solid #4299e1; padding-left: 0.6rem; }
h3 { margin: 1rem 0 0.4rem; color: #2b6cb0; }
h4, h5, h6 { margin: 0.8rem 0 0.3rem; }
p { margin: 0.6rem 0; }
ul { margin: 0.6rem 0 0.8rem 1.3rem; }
table { width: 100%; border-collapse: collapse; margin: 1rem 0; }
th, td { border: 1px solid #e2e8f0; padding: 0.55rem; text-align: left; }
th { background: #f7fafc; }
pre { white-space: pre-wrap; overflow-x: auto; background: #1a202c; color: #edf2f7; padding: 1rem; border-radius: 6px; margin: 0.8rem 0; }
.json-window { max-height: 520px; min-height: 260px; overflow: auto; white-space: pre; border: 1px solid #2d3748; }
code { background: #edf2f7; color: #2d3748; padding: 0.1rem 0.25rem; border-radius: 4px; }
pre code { background: transparent; color: inherit; padding: 0; }
.highlight, .suggestion-box { background: #e8f4f8; padding: 1rem; border-radius: 6px; margin: 1rem 0; border-left: 4px solid #4299e1; }
.source-card, .topic-card { border: 1px solid #e2e8f0; border-radius: 8px; padding: 1rem; margin: 1rem 0; }
.relation-candidate-list { display: grid; gap: 0.85rem; margin: 1rem 0; }
.relation-candidate-card { border: 1px solid #d9e2ec; border-radius: 10px; padding: 0.95rem; background: #fbfdff; }
.candidate-card-head { display: flex; flex-wrap: wrap; gap: 0.55rem; align-items: center; margin-bottom: 0.65rem; }
.badge { display: inline-block; border-radius: 999px; padding: 0.15rem 0.55rem; background: #edf2f7; font-size: 0.78rem; font-weight: 700; }
.badge.confirmed, .badge.accepted { background: #e6fffa; color: #234e52; }
.badge.candidate { background: #ebf8ff; color: #2b6cb0; }
.badge.ambiguous { background: #fffaf0; color: #744210; }
.badge.missing, .badge.unresolved { background: #fff5f5; color: #742a2a; }
dl { display: grid; grid-template-columns: minmax(90px, 130px) 1fr; gap: 0.45rem 0.75rem; }
dt { color: #4a5568; font-weight: 700; }
dd { min-width: 0; overflow-wrap: anywhere; }
.refs-graph-wrap { width: 100%; overflow: auto; border: 1px solid #d9e2ec; border-radius: 10px; background: #fbfdff; margin: 1rem 0; }
.refs-graph-svg { width: 100%; min-width: 900px; height: 680px; }
.refs-edge { stroke: #4a5568; stroke-width: 1.7; fill: none; opacity: 0.82; }
.refs-edge-candidate, .refs-edge-ambiguous, .refs-edge-needs-human { stroke-dasharray: 7 5; }
.refs-edge-missing, .refs-edge-unresolved { stroke-dasharray: 2 5; }
.refs-node circle { fill: #fff; stroke: #2b6cb0; stroke-width: 2; }
.refs-node-unresolved-reference circle, .refs-node-status-missing circle { fill: #fff; stroke-dasharray: 5 4; }
.refs-node text { font-size: 12px; fill: #1f2937; pointer-events: none; }
.refs-graph-summary { margin-top: 1rem; }
.empty { color: #718096; background: #f7fafc; border: 1px dashed #cbd5e0; padding: 1rem; border-radius: 6px; }
details { margin: 0.8rem 0; border: 1px solid #e2e8f0; border-radius: 6px; padding: 0.7rem; }
summary { cursor: pointer; color: #2b6cb0; font-weight: 600; }
.compact-toggle { margin: 0.8rem 0; border: 1px solid #d9e2ec; border-radius: 6px; padding: 0; background: white; overflow: hidden; }
.compact-toggle summary { cursor: pointer; color: #2b6cb0; font-weight: 700; padding: 0.75rem 0.9rem; background: white; list-style-position: inside; }
.compact-toggle summary::marker { color: #2b6cb0; font-size: 0.9rem; }
.compact-toggle summary:hover { background: #f7fafc; }
.compact-toggle[open] summary { border-bottom: 1px solid #e2e8f0; background: #f7fafc; }
.compact-toggle > :not(summary) { margin-left: 1rem; margin-right: 1rem; }
.compact-toggle > :last-child { margin-bottom: 1rem; }
.task-stats { display: grid; grid-template-columns: repeat(auto-fit, minmax(130px, 1fr)); gap: 0.65rem; margin: 1rem 0; }
.task-stat { border: 1px solid #e2e8f0; border-radius: 10px; padding: 0.75rem; background: #f8fafc; }
.task-stat span { display: block; font-size: 0.78rem; color: #718096; }
.task-stat strong { display: block; font-size: 1.45rem; color: #2b6cb0; }
mark { background: #fefcbf; padding: 0 0.15rem; }
"#;

const VIEWER_JS: &str = r#"
const sidebar = document.getElementById('wikiSidebar');
const toggleBtn = document.getElementById('toggleBtn');
const commandLog = document.getElementById('commandLog');
const commandInput = document.getElementById('checkCommand');
const commandButton = document.getElementById('runCheckCommand');

function switchTab(id) {
  const target = document.getElementById(id);
  if (!target) return false;
  document.querySelectorAll('.nav-tab, .sidebar-link, .primary-action').forEach(t => t.classList.remove('active'));
  document.querySelectorAll('.content-section').forEach(c => c.classList.remove('active'));
  document.querySelectorAll(`[data-target="${id}"]`).forEach(t => t.classList.add('active'));
  target.classList.add('active');
  target.scrollIntoView({behavior: 'smooth', block: 'start'});
  return true;
}

function log(msg) {
  const div = document.createElement('div');
  div.className = 'terminal-msg';
  div.innerHTML = msg;
  commandLog.appendChild(div);
  commandLog.scrollTop = commandLog.scrollHeight;
}

function runCheckCommand() {
  const raw = commandInput.value.trim();
  if (!raw) return;
  log(`<span style="color:#90cdf4">kb-check&gt;</span> ${escapeHtml(raw)}`);
  commandInput.value = '';
  const lower = raw.toLowerCase();
  if (lower === 'help') {
    log('Commands: <code>open overview</code>, <code>open llm-launch</code>, <code>open refs-index</code>, <code>open refs-graph</code>, <code>open keywords</code>, <code>open health</code>, <code>open tasks</code>, <code>open memory</code>, <code>open topics</code>, <code>find WORD</code>, <code>topic NAME</code>, <code>clear</code>.');
  } else if (lower === 'clear') {
    commandLog.innerHTML = '';
  } else if (lower.startsWith('open ')) {
    const id = lower.slice(5).trim().replace(/\s+/g, '-');
    if (switchTab(id)) log(`Opened <code>${escapeHtml(id)}</code>.`);
    else log('Unknown page. No action taken.');
  } else if (lower.startsWith('find ')) {
    const term = raw.slice(5).trim();
    if (term) {
      window.find(term);
      log(`Searched visible page for <code>${escapeHtml(term)}</code>.`);
    }
  } else if (lower.startsWith('topic ')) {
    switchTab('topics');
    const name = raw.slice(6).trim().toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '');
    const el = document.getElementById(`topic-${name}`);
    if (el) {
      el.scrollIntoView({behavior: 'smooth', block: 'start'});
      log(`Opened topic <code>${escapeHtml(name)}</code>.`);
    } else {
      log('Topic not found in this static viewer. No action taken.');
    }
  } else {
    log('Unknown command. No action taken.');
  }
}

function escapeHtml(s) {
  return s.replace(/[&<>"']/g, c => ({'&':'&amp;', '<':'&lt;', '>':'&gt;', '"':'&quot;', "'":'&#39;'}[c]));
}


async function copyTextFrom(id, button) {
  const target = document.getElementById(id);
  if (!target) return;
  const text = target.innerText;
  try {
    await navigator.clipboard.writeText(text);
    if (button) {
      const original = button.textContent;
      button.textContent = '已复制';
      setTimeout(() => button.textContent = original, 1200);
    }
    log(`Copied <code>${escapeHtml(id)}</code> to clipboard.`);
  } catch (err) {
    log('Clipboard copy failed. Select and copy the block manually.');
  }
}

document.querySelectorAll('[data-copy-target]').forEach(btn => {
  btn.addEventListener('click', () => copyTextFrom(btn.dataset.copyTarget, btn));
});

toggleBtn.addEventListener('click', () => {
  sidebar.classList.toggle('hidden');
  toggleBtn.classList.toggle('collapsed');
  toggleBtn.textContent = sidebar.classList.contains('hidden') ? '›' : '‹';
});

document.querySelectorAll('.nav-tab, .sidebar-link, .primary-action').forEach(tab => {
  tab.addEventListener('click', () => switchTab(tab.dataset.target));
});

commandButton.addEventListener('click', runCheckCommand);
commandInput.addEventListener('keydown', e => {
  if (e.key === 'Enter') runCheckCommand();
});
"#;

pub fn render_user_knowledge_view(kb_path: &Path, output_root: &Path) -> Result<String> {
    let pages = collect_wiki_page_sources(kb_path)?;
    render_wiki_reader_from_pages(kb_path, output_root, &pages)
}

pub fn count_user_knowledge_pages(kb_path: &Path) -> Result<usize> {
    Ok(collect_wiki_page_sources(kb_path)?.len())
}

fn collect_wiki_page_sources(kb_path: &Path) -> Result<Vec<WikiPageSource>> {
    let wiki_root = kb_path.join("wiki");
    if !wiki_root.exists() {
        return Ok(Vec::new());
    }

    let mut paths = collect_markdown_files(&wiki_root);
    paths.sort();

    let mut pages = Vec::new();
    let mut used_ids = BTreeSet::new();
    for (idx, path) in paths.into_iter().enumerate() {
        let content = fs::read_to_string(&path).unwrap_or_default();
        let rel_path = relative_path_string(kb_path, &path);
        let rel_without_ext = rel_path
            .strip_suffix(".md")
            .unwrap_or(&rel_path)
            .to_string();
        let mut id = slugify(&rel_without_ext);
        if id.is_empty() {
            id = format!("wiki-page-{}", idx + 1);
        }
        let base_id = id.clone();
        let mut suffix = 2;
        while used_ids.contains(&id) {
            id = format!("{}-{}", base_id, suffix);
            suffix += 1;
        }
        used_ids.insert(id.clone());

        let title = wiki_page_title(&content, &path);
        let summary = wiki_page_summary(&content);
        let folder = path
            .parent()
            .and_then(|parent| parent.strip_prefix(&wiki_root).ok())
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "Home".to_string());

        pages.push(WikiPageSource {
            id,
            title,
            rel_path,
            abs_path: path,
            folder,
            content,
            summary,
        });
    }
    Ok(pages)
}

fn render_wiki_reader_from_pages(
    kb_path: &Path,
    output_root: &Path,
    pages: &[WikiPageSource],
) -> Result<String> {
    let generated_at = html_escape(&Utc::now().to_rfc3339());
    let kb_display = html_escape(&kb_path.display().to_string());
    let kb_name_raw = kb_path
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.trim().is_empty())
        .unwrap_or("LLM Wiki");
    let kb_name = html_escape(kb_name_raw);
    let link_index = build_wiki_link_index(pages);

    let mut folder_map: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for (idx, page) in pages.iter().enumerate() {
        folder_map.entry(page.folder.clone()).or_default().push(idx);
    }

    let mut sidebar = String::new();
    if pages.is_empty() {
        sidebar.push_str("<p class=\"wiki-empty-small\">No wiki pages yet.</p>");
    } else {
        for (folder, indexes) in &folder_map {
            sidebar.push_str(&format!(
                "<details class=\"wiki-folder\" open><summary>{}</summary>",
                html_escape(folder)
            ));
            for idx in indexes {
                let page = &pages[*idx];
                sidebar.push_str(&format!(
                    "<a class=\"wiki-nav-link\" href=\"#{}\" data-page=\"{}\"><span>{}</span><small>{}</small></a>",
                    html_escape(&page.id),
                    html_escape(&page.id),
                    html_escape(&page.title),
                    html_escape(&page.rel_path)
                ));
            }
            sidebar.push_str("</details>");
        }
    }

    let mut cards = String::new();
    if pages.is_empty() {
        cards.push_str("<article class=\"wiki-empty\"><h2>还没有可阅读的 Wiki 页面</h2><p>请先运行 <code>kb build-wiki</code> 或由 Worker LLM 在 <code>wiki/</code> 下生成 Markdown 页面。这里将显示围绕主题组织的知识页、超链接和图片。</p></article>");
    } else {
        for page in pages {
            let body = markdown_to_wiki_html(
                &page.content,
                kb_path,
                output_root,
                &page.abs_path,
                &link_index,
            );
            cards.push_str(&format!(
                "<article class=\"wiki-page\" id=\"{}\" data-title=\"{}\" data-path=\"{}\"><div class=\"wiki-page-head\"><p class=\"wiki-page-path\">{}</p><h2>{}</h2><p class=\"wiki-page-summary\">{}</p></div><div class=\"wiki-page-body\">{}</div></article>",
                html_escape(&page.id),
                html_escape(&page.title.to_lowercase()),
                html_escape(&page.rel_path.to_lowercase()),
                html_escape(&page.rel_path),
                html_escape(&page.title),
                html_escape(&page.summary),
                body
            ));
        }
    }

    let mut html = String::new();
    html.push_str("<!DOCTYPE html><html lang=\"zh-CN\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">");
    html.push_str(&format!("<title>{} Knowledge Portal</title>", kb_name));
    html.push_str("<style>");
    html.push_str(WIKI_READER_CSS);
    html.push_str("</style></head><body>");
    html.push_str("<div class=\"wiki-shell\">");
    html.push_str("<aside class=\"wiki-reader-sidebar\"><div class=\"wiki-brand\"><strong>");
    html.push_str(&kb_name);
    html.push_str("</strong><span>Knowledge Portal</span></div><input id=\"wikiSearch\" class=\"wiki-search\" placeholder=\"搜索页面、标题、路径…\"><nav class=\"wiki-nav\">");
    html.push_str(&sidebar);
    html.push_str("</nav><div class=\"wiki-side-note\">This page renders <code>wiki/</code> as a reader-facing knowledge portal. It hides low-level task status, JSON, and agent handoff details.</div></aside>");
    html.push_str("<main class=\"wiki-reader-main\"><header class=\"wiki-reader-header\"><div><p class=\"eyebrow\">LLM Wiki knowledge portal</p><h1>");
    html.push_str(&kb_name);
    html.push_str("</h1><p>围绕主题阅读知识页、WikiLinks、图片与文献页面；底层任务、JSON 和审查面板仍在 <a href=\"index.html\">kb check dashboard</a> 中。</p></div><div class=\"wiki-meta\"><span>");
    html.push_str(&format!("{} pages", pages.len()));
    html.push_str("</span><span>");
    html.push_str(&generated_at);
    html.push_str("</span><span>");
    html.push_str(&kb_display);
    html.push_str("</span></div></header><section class=\"wiki-page-list\">");
    html.push_str(&cards);
    html.push_str("</section></main></div>");
    html.push_str("<script>");
    html.push_str(WIKI_READER_JS);
    html.push_str("</script></body></html>");
    Ok(html)
}

fn build_wiki_link_index(pages: &[WikiPageSource]) -> BTreeMap<String, String> {
    let mut index = BTreeMap::new();
    for page in pages {
        for key in wiki_link_keys(page) {
            index.entry(key).or_insert_with(|| page.id.clone());
        }
    }
    index
}

fn wiki_link_keys(page: &WikiPageSource) -> Vec<String> {
    let rel_no_ext = page.rel_path.strip_suffix(".md").unwrap_or(&page.rel_path);
    let stem = Path::new(&page.rel_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(rel_no_ext);
    vec![
        normalize_wiki_link_key(&page.title),
        normalize_wiki_link_key(stem),
        normalize_wiki_link_key(rel_no_ext),
        normalize_wiki_link_key(&page.rel_path),
        normalize_wiki_link_key(&rel_no_ext.replace('/', " ")),
    ]
    .into_iter()
    .filter(|s| !s.is_empty())
    .collect()
}

fn normalize_wiki_link_key(raw: &str) -> String {
    raw.trim()
        .trim_matches('[')
        .trim_matches(']')
        .trim_end_matches(".md")
        .replace('\\', "/")
        .to_ascii_lowercase()
}

fn wiki_page_title(content: &str, path: &Path) -> String {
    for line in strip_yaml_front_matter(content).lines() {
        if let Some((_, title)) = markdown_heading(line.trim()) {
            let clean = title.trim();
            if !clean.is_empty() {
                return clean.to_string();
            }
        }
    }
    path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Untitled")
        .replace('-', " ")
        .replace('_', " ")
}

fn wiki_page_summary(content: &str) -> String {
    for line in strip_yaml_front_matter(content).lines() {
        let t = line.trim();
        if t.is_empty()
            || t.starts_with('#')
            || t.starts_with('|')
            || t.starts_with('-')
            || t.starts_with('*')
            || t.starts_with("```")
        {
            continue;
        }
        let plain = t.replace("[[", "").replace("]]", "").replace('`', "");
        return shorten_label(&plain, 180);
    }
    "No summary yet.".to_string()
}

fn strip_yaml_front_matter(content: &str) -> &str {
    let trimmed = content.strip_prefix('\u{feff}').unwrap_or(content);
    if !trimmed.starts_with("---") {
        return trimmed;
    }
    let mut lines = trimmed.lines();
    if lines.next() != Some("---") {
        return trimmed;
    }
    let mut offset = 4usize;
    for line in lines {
        if line.trim() == "---" {
            offset += line.len() + 1;
            return trimmed.get(offset..).unwrap_or("");
        }
        offset += line.len() + 1;
    }
    trimmed
}

fn markdown_to_wiki_html(
    markdown: &str,
    kb_path: &Path,
    output_root: &Path,
    page_path: &Path,
    link_index: &BTreeMap<String, String>,
) -> String {
    let markdown = strip_yaml_front_matter(markdown);
    let lines: Vec<&str> = markdown.lines().collect();
    let mut out = String::new();
    let mut in_ul = false;
    let mut in_code = false;
    let mut paragraph = String::new();
    let mut i = 0usize;

    while i < lines.len() {
        let trimmed = lines[i].trim_end();
        let t = trimmed.trim();
        if t.starts_with("```") {
            flush_wiki_paragraph(
                &mut out,
                &mut paragraph,
                kb_path,
                output_root,
                page_path,
                link_index,
            );
            if in_ul {
                out.push_str("</ul>");
                in_ul = false;
            }
            if in_code {
                out.push_str("</code></pre>");
                in_code = false;
            } else {
                out.push_str("<pre><code>");
                in_code = true;
            }
            i += 1;
            continue;
        }
        if in_code {
            out.push_str(&html_escape(trimmed));
            out.push('\n');
            i += 1;
            continue;
        }
        if t.is_empty() {
            flush_wiki_paragraph(
                &mut out,
                &mut paragraph,
                kb_path,
                output_root,
                page_path,
                link_index,
            );
            if in_ul {
                out.push_str("</ul>");
                in_ul = false;
            }
            i += 1;
            continue;
        }
        if looks_like_markdown_table(&lines, i) {
            flush_wiki_paragraph(
                &mut out,
                &mut paragraph,
                kb_path,
                output_root,
                page_path,
                link_index,
            );
            if in_ul {
                out.push_str("</ul>");
                in_ul = false;
            }
            let (table_html, next_i) =
                render_wiki_table(&lines, i, kb_path, output_root, page_path, link_index);
            out.push_str(&table_html);
            i = next_i;
            continue;
        }
        if let Some((level, text)) = markdown_heading(t) {
            flush_wiki_paragraph(
                &mut out,
                &mut paragraph,
                kb_path,
                output_root,
                page_path,
                link_index,
            );
            if in_ul {
                out.push_str("</ul>");
                in_ul = false;
            }
            out.push_str(&format!(
                "<h{level}>{}</h{level}>",
                inline_wiki_markdown(text, kb_path, output_root, page_path, link_index)
            ));
        } else if let Some(item) = t.strip_prefix("- ").or_else(|| t.strip_prefix("* ")) {
            flush_wiki_paragraph(
                &mut out,
                &mut paragraph,
                kb_path,
                output_root,
                page_path,
                link_index,
            );
            if !in_ul {
                out.push_str("<ul>");
                in_ul = true;
            }
            out.push_str(&format!(
                "<li>{}</li>",
                inline_wiki_markdown(item, kb_path, output_root, page_path, link_index)
            ));
        } else if let Some(quote) = t.strip_prefix("> ") {
            flush_wiki_paragraph(
                &mut out,
                &mut paragraph,
                kb_path,
                output_root,
                page_path,
                link_index,
            );
            if in_ul {
                out.push_str("</ul>");
                in_ul = false;
            }
            out.push_str(&format!(
                "<blockquote>{}</blockquote>",
                inline_wiki_markdown(quote, kb_path, output_root, page_path, link_index)
            ));
        } else {
            if !paragraph.is_empty() {
                paragraph.push(' ');
            }
            paragraph.push_str(t);
        }
        i += 1;
    }

    flush_wiki_paragraph(
        &mut out,
        &mut paragraph,
        kb_path,
        output_root,
        page_path,
        link_index,
    );
    if in_ul {
        out.push_str("</ul>");
    }
    if in_code {
        out.push_str("</code></pre>");
    }
    out
}

fn flush_wiki_paragraph(
    out: &mut String,
    paragraph: &mut String,
    kb_path: &Path,
    output_root: &Path,
    page_path: &Path,
    link_index: &BTreeMap<String, String>,
) {
    if !paragraph.trim().is_empty() {
        out.push_str(&format!(
            "<p>{}</p>",
            inline_wiki_markdown(
                paragraph.trim(),
                kb_path,
                output_root,
                page_path,
                link_index
            )
        ));
    }
    paragraph.clear();
}

fn looks_like_markdown_table(lines: &[&str], index: usize) -> bool {
    if index + 1 >= lines.len() {
        return false;
    }
    let first = lines[index].trim();
    let second = lines[index + 1].trim();
    first.starts_with('|')
        && first.ends_with('|')
        && second.starts_with('|')
        && second.contains("---")
}

fn render_wiki_table(
    lines: &[&str],
    start: usize,
    kb_path: &Path,
    output_root: &Path,
    page_path: &Path,
    link_index: &BTreeMap<String, String>,
) -> (String, usize) {
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut i = start;
    while i < lines.len() {
        let t = lines[i].trim();
        if !(t.starts_with('|') && t.ends_with('|')) {
            break;
        }
        rows.push(split_markdown_table_row(t));
        i += 1;
    }
    let mut out = String::new();
    out.push_str("<div class=\"wiki-table-wrap\"><table>");
    if !rows.is_empty() {
        let mut body_start = 1usize;
        out.push_str("<thead><tr>");
        for cell in &rows[0] {
            out.push_str(&format!(
                "<th>{}</th>",
                inline_wiki_markdown(cell, kb_path, output_root, page_path, link_index)
            ));
        }
        out.push_str("</tr></thead>");
        if rows.len() > 1 && is_table_separator_row(&rows[1]) {
            body_start = 2;
        }
        out.push_str("<tbody>");
        for row in rows.iter().skip(body_start) {
            out.push_str("<tr>");
            for cell in row {
                out.push_str(&format!(
                    "<td>{}</td>",
                    inline_wiki_markdown(cell, kb_path, output_root, page_path, link_index)
                ));
            }
            out.push_str("</tr>");
        }
        out.push_str("</tbody>");
    }
    out.push_str("</table></div>");
    (out, i)
}

fn split_markdown_table_row(line: &str) -> Vec<String> {
    line.trim()
        .trim_matches('|')
        .split('|')
        .map(|cell| cell.trim().to_string())
        .collect()
}

fn is_table_separator_row(row: &[String]) -> bool {
    row.iter().all(|cell| {
        !cell.is_empty()
            && cell
                .chars()
                .all(|ch| ch == '-' || ch == ':' || ch.is_whitespace())
    })
}

fn inline_wiki_markdown(
    text: &str,
    kb_path: &Path,
    output_root: &Path,
    page_path: &Path,
    link_index: &BTreeMap<String, String>,
) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::new();
    let mut i = 0usize;
    while i < chars.len() {
        if i + 1 < chars.len() && chars[i] == '!' && chars[i + 1] == '[' {
            if let Some(close) = find_char(&chars, i + 2, ']') {
                if close + 1 < chars.len() && chars[close + 1] == '(' {
                    if let Some(end) = find_char(&chars, close + 2, ')') {
                        let alt = chars_to_string(&chars[i + 2..close]);
                        let target = chars_to_string(&chars[close + 2..end]);
                        out.push_str(&render_wiki_image(
                            kb_path,
                            output_root,
                            page_path,
                            &alt,
                            &target,
                        ));
                        i = end + 1;
                        continue;
                    }
                }
            }
        }
        if i + 1 < chars.len() && chars[i] == '[' && chars[i + 1] == '[' {
            if let Some(end) = find_double_close(&chars, i + 2) {
                let raw = chars_to_string(&chars[i + 2..end]);
                out.push_str(&render_wiki_xref(&raw, link_index));
                i = end + 2;
                continue;
            }
        }
        if chars[i] == '[' {
            if let Some(close) = find_char(&chars, i + 1, ']') {
                if close + 1 < chars.len() && chars[close + 1] == '(' {
                    if let Some(end) = find_char(&chars, close + 2, ')') {
                        let label = chars_to_string(&chars[i + 1..close]);
                        let target = chars_to_string(&chars[close + 2..end]);
                        out.push_str(&render_wiki_markdown_link(
                            kb_path,
                            output_root,
                            page_path,
                            &label,
                            &target,
                            link_index,
                        ));
                        i = end + 1;
                        continue;
                    }
                }
            }
        }
        if chars[i] == '`' {
            if let Some(end) = find_char(&chars, i + 1, '`') {
                let code = chars_to_string(&chars[i + 1..end]);
                out.push_str(&format!("<code>{}</code>", html_escape(&code)));
                i = end + 1;
                continue;
            }
        }
        if i + 1 < chars.len() && chars[i] == '*' && chars[i + 1] == '*' {
            if let Some(end) = find_double_star(&chars, i + 2) {
                let bold = chars_to_string(&chars[i + 2..end]);
                out.push_str(&format!("<strong>{}</strong>", html_escape(&bold)));
                i = end + 2;
                continue;
            }
        }
        out.push_str(&html_escape(&chars[i].to_string()));
        i += 1;
    }
    out
}

fn render_wiki_xref(raw: &str, link_index: &BTreeMap<String, String>) -> String {
    let mut parts = raw.splitn(2, '|');
    let target = parts.next().unwrap_or("").trim();
    let label = parts.next().unwrap_or(target).trim();
    let key = normalize_wiki_link_key(target.split('#').next().unwrap_or(target));
    if let Some(id) = link_index.get(&key) {
        format!(
            "<a class=\"wiki-xref\" href=\"#{}\">{}</a>",
            html_escape(id),
            html_escape(label)
        )
    } else {
        format!(
            "<span class=\"wiki-xref wiki-xref-missing\" title=\"unresolved WikiLink: {}\">{}</span>",
            html_escape(target),
            html_escape(label)
        )
    }
}

fn render_wiki_markdown_link(
    kb_path: &Path,
    output_root: &Path,
    page_path: &Path,
    label: &str,
    target: &str,
    link_index: &BTreeMap<String, String>,
) -> String {
    let trimmed = target.trim();
    let label_html = html_escape(label);
    if is_external_or_anchor(trimmed) {
        return format!("<a href=\"{}\">{}</a>", html_escape(trimmed), label_html);
    }
    if trimmed.to_ascii_lowercase().ends_with(".md") {
        let key = normalize_wiki_link_key(trimmed);
        if let Some(id) = link_index.get(&key) {
            return format!(
                "<a class=\"wiki-xref\" href=\"#{}\">{}</a>",
                html_escape(id),
                label_html
            );
        }
    }
    let href = resolve_local_asset_href(kb_path, output_root, page_path, trimmed);
    format!("<a href=\"{}\">{}</a>", html_escape(&href), label_html)
}

fn render_wiki_image(
    kb_path: &Path,
    output_root: &Path,
    page_path: &Path,
    alt: &str,
    target: &str,
) -> String {
    let src = if is_external_or_anchor(target.trim()) {
        target.trim().to_string()
    } else {
        resolve_local_asset_href(kb_path, output_root, page_path, target.trim())
    };
    let caption = if alt.trim().is_empty() {
        "".to_string()
    } else {
        format!("<figcaption>{}</figcaption>", html_escape(alt.trim()))
    };
    format!(
        "<figure class=\"wiki-figure\"><img src=\"{}\" alt=\"{}\">{}</figure>",
        html_escape(&src),
        html_escape(alt.trim()),
        caption
    )
}

fn is_external_or_anchor(target: &str) -> bool {
    let lower = target.to_ascii_lowercase();
    lower.starts_with("http://")
        || lower.starts_with("https://")
        || lower.starts_with("mailto:")
        || lower.starts_with("data:")
        || target.starts_with('#')
}

fn resolve_local_asset_href(
    kb_path: &Path,
    output_root: &Path,
    page_path: &Path,
    target: &str,
) -> String {
    let raw = target.split('#').next().unwrap_or(target);
    let anchor = target
        .find('#')
        .map(|idx| target[idx..].to_string())
        .unwrap_or_default();
    let target_path = Path::new(raw);
    let abs = if target_path.is_absolute() {
        target_path.to_path_buf()
    } else {
        page_path.parent().unwrap_or(kb_path).join(target_path)
    };
    format!("{}{}", relative_url_between(output_root, &abs), anchor)
}

fn relative_url_between(from_dir: &Path, to_path: &Path) -> String {
    let from_parts = path_component_strings(from_dir);
    let to_parts = path_component_strings(to_path);
    let mut common = 0usize;
    while common < from_parts.len()
        && common < to_parts.len()
        && from_parts[common].eq_ignore_ascii_case(&to_parts[common])
    {
        common += 1;
    }
    if common == 0 && to_path.is_absolute() {
        return format!("file:///{}", to_path.to_string_lossy().replace('\\', "/"));
    }
    let mut rel: Vec<String> = Vec::new();
    for _ in common..from_parts.len() {
        rel.push("..".to_string());
    }
    for part in to_parts.iter().skip(common) {
        rel.push(part.clone());
    }
    if rel.is_empty() {
        ".".to_string()
    } else {
        rel.join("/")
    }
}

fn path_component_strings(path: &Path) -> Vec<String> {
    path.components()
        .filter_map(|component| match component {
            std::path::Component::Prefix(prefix) => {
                Some(prefix.as_os_str().to_string_lossy().to_string())
            }
            std::path::Component::RootDir => Some("".to_string()),
            std::path::Component::Normal(value) => Some(value.to_string_lossy().to_string()),
            _ => None,
        })
        .collect()
}

fn find_char(chars: &[char], start: usize, target: char) -> Option<usize> {
    (start..chars.len()).find(|idx| chars[*idx] == target)
}

fn find_double_close(chars: &[char], start: usize) -> Option<usize> {
    if chars.len() < 2 {
        return None;
    }
    (start..chars.len().saturating_sub(1)).find(|idx| chars[*idx] == ']' && chars[*idx + 1] == ']')
}

fn find_double_star(chars: &[char], start: usize) -> Option<usize> {
    if chars.len() < 2 {
        return None;
    }
    (start..chars.len().saturating_sub(1)).find(|idx| chars[*idx] == '*' && chars[*idx + 1] == '*')
}

fn chars_to_string(chars: &[char]) -> String {
    chars.iter().collect()
}

const WIKI_READER_CSS: &str = r#"
* { box-sizing: border-box; }
body { margin: 0; font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif; background: #f5f8fb; color: #1a202c; }
a { color: #2b6cb0; text-decoration: none; }
a:hover { text-decoration: underline; }
.wiki-shell { display: flex; height: 100vh; min-height: 100vh; overflow: hidden; }
.wiki-reader-sidebar { width: 330px; height: 100vh; overflow-y: auto; padding: 1rem; border-right: 1px solid #d9e2ec; background: #ffffff; }
.wiki-brand { display: flex; flex-direction: column; gap: 0.15rem; color: #2b6cb0; margin-bottom: 1rem; }
.wiki-brand strong { font-size: 1.15rem; }
.wiki-brand span { color: #718096; font-size: 0.85rem; }
.wiki-search { width: 100%; border: 1px solid #cbd5e0; border-radius: 9px; padding: 0.7rem 0.8rem; margin-bottom: 1rem; outline: none; }
.wiki-search:focus { border-color: #4299e1; box-shadow: 0 0 0 3px rgba(66,153,225,0.16); }
.wiki-folder { border: 1px solid #e2e8f0; border-radius: 8px; margin: 0.65rem 0; background: white; padding: 0.2rem 0; }
.wiki-folder summary { cursor: pointer; color: #2b6cb0; font-weight: 700; padding: 0.65rem 0.75rem; }
.wiki-nav-link { display: block; border-top: 1px solid #edf2f7; padding: 0.65rem 0.8rem; color: #2b6cb0; }
.wiki-nav-link span { display: block; font-weight: 650; overflow-wrap: anywhere; }
.wiki-nav-link small { display: block; color: #718096; margin-top: 0.2rem; overflow-wrap: anywhere; }
.wiki-nav-link.active { background: #ebf8ff; border-left: 4px solid #4299e1; padding-left: calc(0.8rem - 4px); }
.wiki-side-note, .wiki-empty-small { color: #718096; font-size: 0.82rem; line-height: 1.45; margin-top: 1rem; }
.wiki-reader-main { flex: 1; height: 100vh; overflow-y: auto; padding: 2rem; }
.wiki-reader-header { max-width: 1040px; margin: 0 auto 1.2rem; background: white; border: 1px solid #e2e8f0; border-radius: 14px; padding: 1.3rem 1.5rem; box-shadow: 0 1px 4px rgba(0,0,0,0.05); }
.eyebrow { margin: 0 0 0.35rem; text-transform: uppercase; letter-spacing: 0.08em; color: #718096; font-size: 0.75rem; font-weight: 800; }
.wiki-reader-header h1 { margin: 0; color: #2b6cb0; font-size: 1.8rem; }
.wiki-reader-header p { line-height: 1.55; }
.wiki-meta { display: flex; flex-wrap: wrap; gap: 0.5rem; margin-top: 0.85rem; }
.wiki-meta span { background: #edf2f7; color: #4a5568; border-radius: 999px; padding: 0.22rem 0.6rem; font-size: 0.78rem; }
.wiki-page-list { max-width: 1040px; margin: 0 auto; }
.wiki-page { background: white; border: 1px solid #e2e8f0; border-radius: 14px; margin: 1rem 0; padding: 1.4rem 1.55rem; box-shadow: 0 1px 4px rgba(0,0,0,0.05); scroll-margin-top: 1rem; }
.wiki-page-head { border-bottom: 1px solid #edf2f7; margin-bottom: 1rem; padding-bottom: 0.85rem; }
.wiki-page-path { color: #718096; font-size: 0.83rem; margin: 0 0 0.25rem; }
.wiki-page h2 { margin: 0.15rem 0; color: #1a365d; font-size: 1.45rem; border-left: 4px solid #4299e1; padding-left: 0.6rem; }
.wiki-page-summary { color: #4a5568; margin-bottom: 0; }
.wiki-page-body { line-height: 1.72; font-size: 1rem; }
.wiki-page-body h1, .wiki-page-body h2, .wiki-page-body h3 { color: #2b6cb0; margin-top: 1.25rem; }
.wiki-page-body h1 { font-size: 1.5rem; }
.wiki-page-body h2 { font-size: 1.28rem; }
.wiki-page-body h3 { font-size: 1.1rem; }
.wiki-page-body ul { padding-left: 1.35rem; }
.wiki-page-body blockquote { margin: 1rem 0; border-left: 4px solid #bee3f8; background: #f7fafc; padding: 0.7rem 1rem; color: #4a5568; }
.wiki-page-body pre { overflow: auto; background: #1a202c; color: #edf2f7; padding: 1rem; border-radius: 8px; }
.wiki-page-body code { background: #edf2f7; border-radius: 4px; padding: 0.08rem 0.24rem; }
.wiki-page-body pre code { background: transparent; padding: 0; }
.wiki-xref { background: #ebf8ff; border: 1px solid #bee3f8; border-radius: 999px; padding: 0.08rem 0.42rem; font-weight: 650; }
.wiki-xref-missing { background: #fffaf0; border-color: #f6ad55; color: #744210; }
.wiki-figure { margin: 1rem auto; border: 1px solid #e2e8f0; border-radius: 12px; padding: 0.7rem; background: #fbfdff; text-align: center; }
.wiki-figure img { max-width: 100%; height: auto; border-radius: 8px; }
.wiki-figure figcaption { color: #718096; font-size: 0.86rem; margin-top: 0.5rem; }
.wiki-table-wrap { overflow: auto; margin: 1rem 0; border: 1px solid #e2e8f0; border-radius: 10px; }
.wiki-table-wrap table { width: 100%; border-collapse: collapse; }
.wiki-table-wrap th, .wiki-table-wrap td { border-bottom: 1px solid #edf2f7; padding: 0.6rem 0.7rem; text-align: left; vertical-align: top; }
.wiki-table-wrap th { background: #f7fafc; color: #2d3748; }
.wiki-empty { background: white; border: 1px dashed #cbd5e0; border-radius: 14px; padding: 2rem; }
.hidden-by-search { display: none; }
@media (max-width: 900px) { .wiki-shell { flex-direction: column; height: auto; overflow: visible; } .wiki-reader-sidebar { width: 100%; height: auto; max-height: none; } .wiki-reader-main { height: auto; padding: 1rem; overflow: visible; } }
"#;

const WIKI_READER_JS: &str = r#"
const links = Array.from(document.querySelectorAll('.wiki-nav-link'));
const pages = Array.from(document.querySelectorAll('.wiki-page'));
const search = document.getElementById('wikiSearch');
function setActive(id) {
  links.forEach(link => link.classList.toggle('active', link.dataset.page === id));
}
links.forEach(link => {
  link.addEventListener('click', () => setActive(link.dataset.page));
});
if (links.length) setActive(links[0].dataset.page);
search?.addEventListener('input', () => {
  const q = search.value.trim().toLowerCase();
  pages.forEach(page => {
    const hay = `${page.dataset.title || ''} ${page.dataset.path || ''} ${page.textContent || ''}`.toLowerCase();
    const hit = !q || hay.includes(q);
    page.classList.toggle('hidden-by-search', !hit);
  });
  links.forEach(link => {
    const page = document.getElementById(link.dataset.page);
    const hit = page && !page.classList.contains('hidden-by-search');
    link.classList.toggle('hidden-by-search', !hit);
  });
});
const observer = new IntersectionObserver(entries => {
  const visible = entries.filter(e => e.isIntersecting).sort((a,b) => b.intersectionRatio - a.intersectionRatio)[0];
  if (visible) setActive(visible.target.id);
}, {root: document.querySelector('.wiki-reader-main'), threshold: [0.15, 0.35, 0.6]});
pages.forEach(page => observer.observe(page));
"#;

fn render_llm_launch(kb_path: &Path) -> Result<String> {
    let kb_display = kb_path.display().to_string();
    let mut topics = Vec::new();
    let topics_dir = kb_path.join("topics");
    if topics_dir.exists() {
        for entry in fs::read_dir(&topics_dir)? {
            let entry = entry?;
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                let file_name = entry.file_name();
                if let Some(name) = file_name.to_str() {
                    if !name.trim().is_empty() {
                        topics.push(name.to_string());
                    }
                }
            }
        }
    }
    topics.sort();

    let default_topic = topics
        .first()
        .cloned()
        .unwrap_or_else(|| "<topic>".to_string());
    let topic_tasks = if default_topic == "<topic>" {
        "topics/<topic>/tasks/index.md".to_string()
    } else {
        format!("topics/{}/tasks/index.md", default_topic)
    };
    let topic_index = if default_topic == "<topic>" {
        "topics/<topic>/index.md".to_string()
    } else {
        format!("topics/{}/index.md", default_topic)
    };

    let topic_options = if topics.is_empty() {
        "<p class=\"empty\">No topic directory found yet. Run <code>kb build &lt;topic&gt;</code> before launching an LLM Worker.</p>".to_string()
    } else {
        let mut out = String::from("<ul>");
        for topic in &topics {
            out.push_str(&format!(
                "<li><code>{}</code> — <code>topics/{}/tasks/index.md</code></li>",
                html_escape(topic),
                html_escape(topic)
            ));
        }
        out.push_str("</ul>");
        out
    };

    let claude_command = format!("cd \"{}\"\nclaude", kb_display);
    let codex_command = format!("cd \"{}\"\ncodex", kb_display);
    let manager_prompt = format!(
        "请先不要修改任何文件。\n\n你现在进入的是一个 LLM Wiki 知识库。请先阅读：\n1. AGENTS.md\n2. CLAUDE.md（如果你是 Claude Code）\n3. LLM/handoff/current.md\n4. {}\n5. {}\n6. topics/{}/handoff/AGENTS.md（如果存在）\n\n然后返回：\n1. 当前 topic 的研究目标；\n2. 当前已有材料；\n3. 当前任务列表；\n4. 哪些任务适合由你处理；\n5. 哪些任务必须人工确认；\n6. 你建议优先处理的前三个任务。\n\n在我确认前，不要修改文件。",
        topic_index,
        topic_tasks,
        default_topic
    );
    let worker_prompt = format!(
        "请作为 Worker Agent 处理 {} 中的一个具体任务。\n\n规则：\n1. 一次只处理一个 task；\n2. 只依据 raw/、processing/、wiki/、topics/、LLM/ 中已有材料；\n3. 不要把 interfaces/ 当作知识源；\n4. 不要删除 raw/；\n5. 不要把 candidate relation 自动改为 confirmed；\n6. 证据不足时标记 needs_human_review；\n7. 修改完成后在任务文件末尾添加 Work Log；\n8. 最后列出 changed files、evidence used、remaining uncertainty。",
        topic_tasks
    );

    let mut out = String::new();
    out.push_str("<div class=\"launch-note\"><strong>安全边界：</strong>这个 HTML 是静态界面，不能、也不应该直接执行本机命令。\n它只提供启动外部 LLM Agent 的命令和提示词；请复制到终端或对应 Agent 界面后执行。</div>");
    out.push_str("<div class=\"llm-launch-grid\">");
    out.push_str("<article class=\"llm-card\"><h3>Claude Code</h3><p>适合语义型知识工作：paper card、relation evidence、concept page、topic synthesis。</p>");
    out.push_str(
        "<button class=\"copy-btn\" data-copy-target=\"claude-launch-command\">复制命令</button>",
    );
    out.push_str(&format!("<div class=\"launch-snippet\"><pre id=\"claude-launch-command\"><code>{}</code></pre></div></article>", html_escape(&claude_command)));
    out.push_str("<article class=\"llm-card\"><h3>Codex</h3><p>适合工程型工作：检查目录结构、修 Markdown、维护 kb-cli、跑命令、更新文档。</p>");
    out.push_str(
        "<button class=\"copy-btn\" data-copy-target=\"codex-launch-command\">复制命令</button>",
    );
    out.push_str(&format!("<div class=\"launch-snippet\"><pre id=\"codex-launch-command\"><code>{}</code></pre></div></article>", html_escape(&codex_command)));
    out.push_str("</div>");

    out.push_str("<h3>当前 Topic 任务入口</h3>");
    out.push_str(&topic_options);

    out.push_str("<h3>Manager 启动提示词</h3>");
    out.push_str("<button class=\"copy-btn\" data-copy-target=\"manager-launch-prompt\">复制 Manager Prompt</button>");
    out.push_str(&format!(
        "<pre id=\"manager-launch-prompt\"><code>{}</code></pre>",
        html_escape(&manager_prompt)
    ));

    out.push_str("<h3>Worker 领取任务提示词</h3>");
    out.push_str("<button class=\"copy-btn\" data-copy-target=\"worker-launch-prompt\">复制 Worker Prompt</button>");
    out.push_str(&format!(
        "<pre id=\"worker-launch-prompt\"><code>{}</code></pre>",
        html_escape(&worker_prompt)
    ));

    out.push_str("<div class=\"suggestion-box\"><strong>推荐流程：</strong><br><code>kb build &lt;topic&gt;</code> → <code>kb check</code> → 打开 <strong>About launching LLM</strong> → 复制命令与 Manager Prompt → Agent 读取任务 → 人工审查回写。</div>");
    Ok(out)
}

fn render_overview(kb_path: &Path) -> String {
    let rows = [
        ("Wiki pages", count_files(kb_path.join("wiki"), "md")),
        ("Raw papers", count_files(kb_path.join("raw/papers"), "pdf")),
        (
            "Extracted text files",
            count_files(kb_path.join("processing/text"), "txt"),
        ),
        (
            "Refs reports",
            count_prefixed(kb_path.join("processing/refs"), "refs_index_"),
        ),
        (
            "Refs graph exports",
            count_prefixed(kb_path.join("processing/refs"), "refs_graph_"),
        ),
        (
            "Keyword reports",
            count_prefixed(kb_path.join("processing/keywords"), "keywords_"),
        ),
        (
            "LLM task files",
            count_files(kb_path.join("LLM/tasks"), "md"),
        ),
        (
            "LLM memory files",
            count_files(kb_path.join("LLM/memory"), "md"),
        ),
        ("Topic directories", count_dirs(kb_path.join("topics"))),
    ];

    let mut out = String::new();
    out.push_str("<div class=\"highlight\"><strong>Purpose:</strong> This viewer displays generated Markdown/JSON results. It does not execute kb commands, call LLMs, or modify files.</div>");
    out.push_str("<table><thead><tr><th>Area</th><th>Count</th></tr></thead><tbody>");
    for (label, count) in rows {
        out.push_str(&format!(
            "<tr><td>{}</td><td>{}</td></tr>",
            html_escape(label),
            count
        ));
    }
    out.push_str("</tbody></table>");
    out.push_str("<div class=\"suggestion-box\"><strong>Suggested refresh flow:</strong><br><code>kb health</code> → <code>kb refs-index</code> → <code>kb refs-graph</code> → <code>kb keywords</code> → <code>kb tasks</code> → <code>kb check</code></div>");
    out
}

fn render_workflow_status_block(kb_path: &Path) -> String {
    let status_path = kb_path.join("processing/workflow_status.json");
    if let Ok(content) = fs::read_to_string(&status_path) {
        if let Ok(value) = serde_json::from_str::<Value>(&content) {
            let paper_count = value
                .get("paper_count")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let threshold = value
                .get("rag_threshold")
                .and_then(|v| v.as_u64())
                .unwrap_or(200);
            let mode_label = value
                .get("mode_label")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown workflow mode");
            let reason = value
                .get("mode_reason")
                .and_then(|v| v.as_str())
                .unwrap_or("workflow status generated by kb-cli");
            let next = value
                .get("recommended_next_step")
                .and_then(|v| v.as_str())
                .unwrap_or("Review LLM workflow guides.");
            return format!(
                "<div class=\"suggestion-box\"><strong>Workflow mode:</strong> {}<br><strong>Corpus size:</strong> {} papers / threshold {}<br><strong>Reason:</strong> <code>{}</code><br><strong>Next:</strong> {}</div>",
                html_escape(mode_label),
                paper_count,
                threshold,
                html_escape(reason),
                html_escape(next)
            );
        }
    }

    let paper_count = crate::commands::workflow::count_raw_papers(kb_path);
    let threshold = crate::commands::workflow::RAG_THRESHOLD;
    let mode_label = if paper_count < threshold {
        "Karpathy-style LLM Wiki workflow"
    } else {
        "RAG-assisted LLM Wiki workflow required"
    };
    let next = if paper_count < threshold {
        "Run kb create or kb build to generate Manager/Worker/Human Review guides for direct LLM Wiki compilation."
    } else {
        "RAG workflow routing is required before topic narrative work."
    };
    format!(
        "<div class=\"suggestion-box\"><strong>Workflow mode:</strong> {}<br><strong>Corpus size:</strong> {} papers / threshold {}<br><strong>Next:</strong> {}</div>",
        html_escape(mode_label),
        paper_count,
        threshold,
        html_escape(next)
    )
}

fn render_refs_index(kb_path: &Path) -> Result<String> {
    let Some(path) = latest_matching_path(kb_path, "processing/refs", "refs_index_", "md")? else {
        return Ok("<p class=\"empty\">No refs-index report found. Run <code>kb refs-index</code> first.</p>".to_string());
    };

    let rel = relative_path_string(kb_path, &path);
    let title = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("refs_index.md")
        .to_string();
    let content = fs::read_to_string(&path).unwrap_or_default();
    let (before, candidate_rows, after) = split_refs_index_candidate_section(&content);

    let mut out = String::new();
    out.push_str(&format!(
        "<article class=\"source-card\"><h3>{}</h3><p class=\"path\">{}</p>",
        html_escape(&title),
        html_escape(&rel)
    ));
    out.push_str("<div class=\"highlight\"><strong>Readable review view:</strong> Relation candidates are rendered as review cards instead of a wide raw Markdown table.</div>");
    out.push_str(&markdown_to_html(&before));
    out.push_str(&format!(
        "<details class=\"compact-toggle\"><summary>Relation candidates / relation index <span class=\"badge\">{}</span></summary>",
        candidate_rows.len()
    ));
    out.push_str(&render_relation_candidate_cards(&candidate_rows));
    out.push_str("</details>");
    if !after.trim().is_empty() {
        out.push_str("<details class=\"compact-toggle\"><summary>Deferred human / LLM task handoff</summary>");
        out.push_str(&markdown_to_html(&after));
        out.push_str("</details>");
    }
    out.push_str("</article>");
    Ok(out)
}

fn split_refs_index_candidate_section(markdown: &str) -> (String, Vec<Vec<String>>, String) {
    enum Mode {
        Before,
        Candidates,
        After,
    }

    let mut mode = Mode::Before;
    let mut before = String::new();
    let mut after = String::new();
    let mut rows = Vec::new();

    for line in markdown.lines() {
        let trimmed = line.trim();
        if trimmed == "## Relation candidates" {
            mode = Mode::Candidates;
            continue;
        }
        if matches!(mode, Mode::Candidates) && trimmed.starts_with("## ") {
            mode = Mode::After;
        }

        match mode {
            Mode::Before => {
                before.push_str(line);
                before.push('\n');
            }
            Mode::Candidates => {
                let cells = markdown_table_cells(line);
                if cells.len() >= 6
                    && !is_markdown_header_row(&cells)
                    && !cells[0].eq_ignore_ascii_case("status")
                {
                    rows.push(cells);
                } else if !trimmed.is_empty() && !trimmed.starts_with('|') {
                    after.push_str(line);
                    after.push('\n');
                }
            }
            Mode::After => {
                after.push_str(line);
                after.push('\n');
            }
        }
    }

    (before, rows, after)
}

fn render_relation_candidate_cards(rows: &[Vec<String>]) -> String {
    if rows.is_empty() {
        return "<p class=\"empty\">No relation candidates returned.</p>".to_string();
    }

    let mut out = String::new();
    out.push_str("<div class=\"relation-candidate-list\">");
    for (idx, row) in rows.iter().enumerate() {
        let status = row.get(0).cloned().unwrap_or_else(|| "unknown".to_string());
        let source = row.get(1).cloned().unwrap_or_default();
        let target = row.get(2).cloned().unwrap_or_default();
        let score = row.get(3).cloned().unwrap_or_default();
        let review = row.get(4).cloned().unwrap_or_default();
        let evidence = row.get(5).cloned().unwrap_or_default();
        out.push_str(&format!(
            "<article class=\"relation-candidate-card\"><div class=\"candidate-card-head\"><span class=\"badge {}\">{}</span><strong>Candidate #{}</strong><span>score {}</span><span>review {}</span></div><dl><dt>Source</dt><dd><code>{}</code></dd><dt>Target</dt><dd><code>{}</code></dd><dt>Evidence</dt><dd>{}</dd></dl></article>",
            html_class_token(&status),
            html_escape(&status),
            idx + 1,
            html_escape(&score),
            html_escape(&review),
            html_escape(&source),
            html_escape(&target),
            html_escape(&evidence)
        ));
    }
    out.push_str("</div>");
    out
}

fn render_refs_graph(kb_path: &Path) -> Result<String> {
    let Some(json_path) = latest_matching_path(kb_path, "processing/refs", "refs_graph_", "json")?
    else {
        return Ok("<p class=\"empty\">No refs graph JSON export found. Run <code>kb refs-graph</code> first.</p>".to_string());
    };

    let rel = relative_path_string(kb_path, &json_path);
    let title = json_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("refs_graph.json")
        .to_string();
    let content = fs::read_to_string(&json_path).unwrap_or_default();
    let parsed = serde_json::from_str::<Value>(&content).ok();

    let mut out = String::new();
    out.push_str(&format!(
        "<article class=\"source-card\"><h3>{}</h3><p class=\"path\">{}</p>",
        html_escape(&title),
        html_escape(&rel)
    ));
    out.push_str("<div class=\"highlight\"><strong>Visual protocol:</strong> circles = literature/reference nodes; solid lines = confirmed/accepted relations; dashed lines = candidate/ambiguous relations; dotted lines = missing/unresolved references.</div>");
    out.push_str("<p><a class=\"secondary-action\" href=\"relationship_viewer.html\">打开完整关系图页面</a></p>");

    if let Some(value) = parsed.as_ref() {
        out.push_str(&render_refs_graph_svg(value));
        out.push_str(&render_refs_graph_summary(value));
    } else {
        out.push_str("<p class=\"empty\">The latest refs graph JSON could not be parsed. The raw file is still available below for debugging.</p>");
    }

    out.push_str("<details><summary>Raw graph JSON</summary><pre class=\"json-window\">");
    out.push_str(&html_escape(&content));
    out.push_str("</pre></details>");

    let mut extras = Vec::new();
    for ext in ["mmd", "dot"] {
        if let Some(card) = latest_matching_file(kb_path, "processing/refs", "refs_graph_", ext)? {
            extras.push(card);
        }
    }
    if !extras.is_empty() {
        out.push_str("<details><summary>Mermaid / DOT exports</summary>");
        for card in extras {
            out.push_str(&format!(
                "<article class=\"source-card\"><h4>{}</h4><p class=\"path\">{}</p>{}</article>",
                html_escape(&card.title),
                html_escape(&card.path),
                card.html
            ));
        }
        out.push_str("</details>");
    }

    out.push_str("</article>");
    Ok(out)
}

fn render_refs_graph_svg(value: &Value) -> String {
    let mut nodes = value
        .get("nodes")
        .and_then(|v| v.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|node| {
                    let id = json_string(node, "id")?;
                    let label = json_string(node, "label").unwrap_or_else(|| id.clone());
                    let node_type = json_string(node, "node_type")
                        .or_else(|| json_string(node, "kind"))
                        .unwrap_or_else(|| "paper".to_string());
                    let status =
                        json_string(node, "status").unwrap_or_else(|| "indexed".to_string());
                    Some((id, label, node_type, status))
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let all_edges = value
        .get("edges")
        .and_then(|v| v.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|edge| {
                    let source = json_string(edge, "source")?;
                    let target = json_string(edge, "target")?;
                    let status =
                        json_string(edge, "status").unwrap_or_else(|| "candidate".to_string());
                    let relation = json_string(edge, "relation_label")
                        .or_else(|| json_string(edge, "relation_type"))
                        .or_else(|| json_string(edge, "kind"))
                        .unwrap_or_else(|| "relation".to_string());
                    let evidence = json_string_array(edge, "evidence").join("\n");
                    Some((source, target, status, relation, evidence))
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    if nodes.is_empty() && !all_edges.is_empty() {
        let mut ids = BTreeSet::new();
        for (source, target, _, _, _) in &all_edges {
            ids.insert(source.clone());
            ids.insert(target.clone());
        }
        nodes = ids
            .into_iter()
            .map(|id| {
                (
                    id.clone(),
                    id,
                    "reference".to_string(),
                    "indexed".to_string(),
                )
            })
            .collect();
    }

    if nodes.is_empty() {
        return "<p class=\"empty\">No graph nodes found in the latest refs graph JSON.</p>"
            .to_string();
    }

    let node_limit = 90usize;
    let edge_limit = 180usize;
    let visible_nodes = nodes.into_iter().take(node_limit).collect::<Vec<_>>();
    let visible_ids = visible_nodes
        .iter()
        .map(|(id, _, _, _)| id.clone())
        .collect::<BTreeSet<_>>();
    let visible_edges = all_edges
        .into_iter()
        .filter(|(source, target, _, _, _)| {
            visible_ids.contains(source) && visible_ids.contains(target)
        })
        .take(edge_limit)
        .collect::<Vec<_>>();

    let cx = 550.0f64;
    let cy = 330.0f64;
    let radius = (110.0 + visible_nodes.len() as f64 * 4.0).min(270.0);
    let mut positions = BTreeMap::new();
    for (idx, (id, _, _, _)) in visible_nodes.iter().enumerate() {
        let angle = std::f64::consts::PI * 2.0 * idx as f64 / visible_nodes.len().max(1) as f64
            - std::f64::consts::PI / 2.0;
        positions.insert(
            id.clone(),
            (cx + radius * angle.cos(), cy + radius * angle.sin()),
        );
    }

    let mut svg = String::new();
    svg.push_str("<div class=\"refs-graph-wrap\"><svg class=\"refs-graph-svg\" viewBox=\"0 0 1100 680\" role=\"img\" aria-label=\"Refs relationship graph\">");
    svg.push_str("<g class=\"refs-edge-layer\">");
    for (source, target, status, relation, evidence) in &visible_edges {
        let Some((x1, y1)) = positions.get(source) else {
            continue;
        };
        let Some((x2, y2)) = positions.get(target) else {
            continue;
        };
        svg.push_str(&format!(
            "<line class=\"refs-edge refs-edge-{}\" x1=\"{:.1}\" y1=\"{:.1}\" x2=\"{:.1}\" y2=\"{:.1}\"><title>{}</title></line>",
            html_class_token(status),
            x1,
            y1,
            x2,
            y2,
            html_escape(&format!("{}: {}\n{} → {}\n{}", relation, status, source, target, evidence))
        ));
    }
    svg.push_str("</g><g class=\"refs-node-layer\">");
    for (id, label, node_type, status) in &visible_nodes {
        let Some((x, y)) = positions.get(id) else {
            continue;
        };
        let short = shorten_label(label, 34);
        let r = if node_type == "unresolved_reference" || status == "missing" {
            15
        } else {
            13
        };
        svg.push_str(&format!(
            "<g class=\"refs-node refs-node-{} refs-node-status-{}\"><circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"{}\"></circle><text x=\"{:.1}\" y=\"{:.1}\">{}</text><title>{}\n{}\nstatus: {}</title></g>",
            html_class_token(node_type),
            html_class_token(status),
            x,
            y,
            r,
            x + 16.0,
            y + 4.0,
            html_escape(&short),
            html_escape(node_type),
            html_escape(label),
            html_escape(status)
        ));
    }
    svg.push_str("</g></svg></div>");

    let hidden_nodes = value
        .get("node_count")
        .and_then(|v| v.as_u64())
        .map(|count| count.saturating_sub(visible_nodes.len() as u64))
        .unwrap_or(0);
    let hidden_edges = value
        .get("edge_count")
        .and_then(|v| v.as_u64())
        .map(|count| count.saturating_sub(visible_edges.len() as u64))
        .unwrap_or(0);
    if hidden_nodes > 0 || hidden_edges > 0 {
        svg.push_str(&format!(
            "<p class=\"hint\">Showing a readable preview of {} nodes and {} edges. Hidden for readability: {} nodes, {} edges. Open the full relation page or raw JSON for complete data.</p>",
            visible_nodes.len(),
            visible_edges.len(),
            hidden_nodes,
            hidden_edges
        ));
    }
    svg
}

fn render_refs_graph_summary(value: &Value) -> String {
    let node_count = value
        .get("node_count")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let edge_count = value
        .get("edge_count")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let graph_kind = value
        .get("graph_kind")
        .and_then(|v| v.as_str())
        .unwrap_or("refs_graph");
    let layer = value
        .get("relation_layer")
        .and_then(|v| v.as_str())
        .unwrap_or("bibliographic_index");
    let generated_at = value
        .get("generated_at")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    format!(
        "<div class=\"metric-grid refs-graph-summary\"><div class=\"metric\"><strong>{}</strong>Nodes</div><div class=\"metric\"><strong>{}</strong>Edges</div><div class=\"metric\"><strong>{}</strong>Graph kind</div><div class=\"metric\"><strong>{}</strong>Layer</div></div><p class=\"path\">Generated at: {}</p>",
        node_count,
        edge_count,
        html_escape(graph_kind),
        html_escape(layer),
        html_escape(generated_at)
    )
}

fn shorten_label(label: &str, max_chars: usize) -> String {
    let mut out = String::new();
    for (idx, ch) in label.chars().enumerate() {
        if idx >= max_chars {
            out.push('…');
            return out;
        }
        out.push(ch);
    }
    out
}

fn html_class_token(value: &str) -> String {
    let slug = slugify(value);
    if slug.is_empty() {
        "unknown".to_string()
    } else {
        slug
    }
}

fn render_topics(kb_path: &Path) -> Result<String> {
    let topics_dir = kb_path.join("topics");
    if !topics_dir.exists() {
        return Ok("<p class=\"empty\">No topics/ directory found. Run <code>kb init</code> with a recent version, or create topics/&lt;topic&gt;/ manually.</p>".to_string());
    }

    let mut topics = fs::read_dir(&topics_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .map(|entry| entry.path())
        .collect::<Vec<_>>();
    topics.sort();

    if topics.is_empty() {
        return Ok("<p class=\"empty\">No topic directories yet. Topic-specific causal, method, evidence, idea, and importance relations belong under <code>topics/&lt;topic&gt;/</code>.</p>".to_string());
    }

    let mut out = String::new();
    for topic in topics {
        let topic_name = topic
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown-topic");
        out.push_str(&format!(
            "<article class=\"topic-card\" id=\"topic-{}\"><h3>{}</h3>",
            slugify(topic_name),
            html_escape(topic_name)
        ));
        for rel in ["README.md", "scope.md", "literature.md", "importance.md"] {
            let path = topic.join(rel);
            if path.exists() {
                let content = fs::read_to_string(&path).unwrap_or_default();
                out.push_str(&format!(
                    "<details><summary>{}</summary>{}</details>",
                    html_escape(rel),
                    markdown_to_html(&content)
                ));
            }
        }
        let relations_dir = topic.join("relations");
        if relations_dir.exists() {
            let mut relation_files = collect_markdown_files(&relations_dir);
            relation_files.sort();
            for path in relation_files {
                let name = path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("relation.md");
                let content = fs::read_to_string(&path).unwrap_or_default();
                out.push_str(&format!(
                    "<details><summary>relations/{}</summary>{}</details>",
                    html_escape(name),
                    markdown_to_html(&content)
                ));
            }
        }
        out.push_str("</article>");
    }

    Ok(out)
}

fn render_source_card_or_empty(card: Option<SourceCard>, empty: &str) -> String {
    match card {
        Some(card) => format!(
            "<article class=\"source-card\"><h3>{}</h3><p class=\"path\">{}</p>{}</article>",
            html_escape(&card.title),
            html_escape(&card.path),
            card.html
        ),
        None => format!("<p class=\"empty\">{}</p>", html_escape(empty)),
    }
}

fn read_first_existing(kb_path: &Path, rel_paths: &[&str]) -> Result<Option<SourceCard>> {
    for rel in rel_paths {
        let path = kb_path.join(rel);
        if path.exists() && path.is_file() {
            return card_from_file(kb_path, &path).map(Some);
        }
    }
    Ok(None)
}

fn latest_matching_file(
    kb_path: &Path,
    rel_dir: &str,
    prefix: &str,
    extension: &str,
) -> Result<Option<SourceCard>> {
    let dir = kb_path.join(rel_dir);
    if !dir.exists() {
        return Ok(None);
    }
    let mut candidates = fs::read_dir(&dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().map(|t| t.is_file()).unwrap_or(false))
        .map(|entry| entry.path())
        .filter(|path| {
            let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
            let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
            name.starts_with(prefix) && ext.eq_ignore_ascii_case(extension)
        })
        .collect::<Vec<_>>();
    candidates.sort();
    candidates
        .last()
        .map(|path| card_from_file(kb_path, path))
        .transpose()
}

fn card_from_file(kb_path: &Path, path: &Path) -> Result<SourceCard> {
    let rel = relative_path_string(kb_path, path);
    let content = fs::read_to_string(path).unwrap_or_default();
    let title = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("file")
        .to_string();
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
    let html = if ext.eq_ignore_ascii_case("md") {
        markdown_to_html(&content)
    } else if ext.eq_ignore_ascii_case("json") {
        format!("<pre class=\"json-window\">{}</pre>", html_escape(&content))
    } else {
        format!("<pre>{}</pre>", html_escape(&content))
    };
    Ok(SourceCard {
        title,
        path: rel,
        html,
    })
}

fn markdown_to_html(markdown: &str) -> String {
    let mut out = String::new();
    let mut in_ul = false;
    let mut in_code = false;
    let mut paragraph = String::new();

    for line in markdown.lines() {
        let trimmed = line.trim_end();
        if trimmed.starts_with("```") {
            flush_paragraph(&mut out, &mut paragraph);
            if in_ul {
                out.push_str("</ul>");
                in_ul = false;
            }
            if in_code {
                out.push_str("</code></pre>");
                in_code = false;
            } else {
                out.push_str("<pre><code>");
                in_code = true;
            }
            continue;
        }
        if in_code {
            out.push_str(&html_escape(trimmed));
            out.push('\n');
            continue;
        }
        let t = trimmed.trim();
        if t.is_empty() {
            flush_paragraph(&mut out, &mut paragraph);
            if in_ul {
                out.push_str("</ul>");
                in_ul = false;
            }
            continue;
        }
        if let Some((level, text)) = markdown_heading(t) {
            flush_paragraph(&mut out, &mut paragraph);
            if in_ul {
                out.push_str("</ul>");
                in_ul = false;
            }
            out.push_str(&format!("<h{level}>{}</h{level}>", inline_markdown(text)));
        } else if let Some(item) = t.strip_prefix("- ").or_else(|| t.strip_prefix("* ")) {
            flush_paragraph(&mut out, &mut paragraph);
            if !in_ul {
                out.push_str("<ul>");
                in_ul = true;
            }
            out.push_str(&format!("<li>{}</li>", inline_markdown(item)));
        } else {
            if !paragraph.is_empty() {
                paragraph.push(' ');
            }
            paragraph.push_str(t);
        }
    }

    flush_paragraph(&mut out, &mut paragraph);
    if in_ul {
        out.push_str("</ul>");
    }
    if in_code {
        out.push_str("</code></pre>");
    }
    out
}

fn markdown_heading(line: &str) -> Option<(usize, &str)> {
    let hashes = line.chars().take_while(|ch| *ch == '#').count();
    if (1..=6).contains(&hashes) && line.chars().nth(hashes) == Some(' ') {
        Some((hashes, line[hashes + 1..].trim()))
    } else {
        None
    }
}

fn flush_paragraph(out: &mut String, paragraph: &mut String) {
    if !paragraph.trim().is_empty() {
        out.push_str(&format!("<p>{}</p>", inline_markdown(paragraph.trim())));
    }
    paragraph.clear();
}

fn inline_markdown(text: &str) -> String {
    html_escape(text).replace('`', "")
}

fn collect_markdown_files(root: &Path) -> Vec<PathBuf> {
    if !root.exists() {
        return Vec::new();
    }
    WalkDir::new(root)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| {
            entry
                .path()
                .extension()
                .and_then(|s| s.to_str())
                .map(|s| s.eq_ignore_ascii_case("md"))
                .unwrap_or(false)
        })
        .map(|entry| entry.path().to_path_buf())
        .collect()
}

fn count_files(root: PathBuf, extension: &str) -> usize {
    if !root.exists() {
        return 0;
    }
    WalkDir::new(root)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| {
            entry
                .path()
                .extension()
                .and_then(|s| s.to_str())
                .map(|s| s.eq_ignore_ascii_case(extension))
                .unwrap_or(false)
        })
        .count()
}

fn count_prefixed(root: PathBuf, prefix: &str) -> usize {
    if !root.exists() {
        return 0;
    }
    fs::read_dir(root)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().map(|t| t.is_file()).unwrap_or(false))
        .filter(|entry| {
            entry
                .file_name()
                .to_str()
                .map(|name| name.starts_with(prefix))
                .unwrap_or(false)
        })
        .count()
}

fn count_dirs(root: PathBuf) -> usize {
    if !root.exists() {
        return 0;
    }
    fs::read_dir(root)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .count()
}

fn resolve_under_kb(kb_path: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        kb_path.join(path)
    }
}

fn relative_path_string(kb_path: &Path, path: &Path) -> String {
    path.strip_prefix(kb_path)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn html_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn slugify(input: &str) -> String {
    let mut out = String::new();
    let mut last_dash = false;
    for ch in input.chars() {
        if ch.is_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            last_dash = false;
        } else if !last_dash && !out.is_empty() {
            out.push('-');
            last_dash = true;
        }
    }
    out.trim_matches('-').to_string()
}
