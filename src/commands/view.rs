use anyhow::{anyhow, Result};
use chrono::Utc;
use clap::Args;
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

use crate::commands::init::get_kb_path;

#[derive(Debug, Clone, Args)]
pub struct ViewArgs {
    #[arg(
        long = "output-dir",
        value_name = "DIR",
        help = "Output directory relative to the knowledge base. Defaults to outputs/html/."
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
        help = "Generate the relationship graph review page instead of the regular dashboard"
    )]
    pub relations: bool,

    #[arg(
        long,
        value_name = "TOPIC",
        help = "Default topic focus for --relations mode"
    )]
    pub topic: Option<String>,

    #[arg(
        long = "data-only",
        help = "In --relations mode, generate only relationship_data.json"
    )]
    pub data_only: bool,

    #[arg(long, hide = true, help = "Legacy no-op: kb view opens by default")]
    pub open: bool,
}

impl ViewArgs {
    fn is_dry_run(&self) -> bool {
        self.dry_run || self.preview
    }

    fn should_open(&self) -> bool {
        !self.no_open
    }
}

#[derive(Debug, Clone)]
struct ViewerSection {
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

pub fn execute(custom_kb: Option<&Path>, args: &ViewArgs) -> Result<()> {
    let kb_path = get_kb_path(custom_kb);
    if !kb_path.exists() {
        return Err(anyhow!(
            "knowledge base path does not exist: {}",
            kb_path.display()
        ));
    }

    if args.relations {
        return execute_relationship_view(&kb_path, args);
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
        .unwrap_or_else(|| PathBuf::from("outputs/html"));
    let output_root = resolve_under_kb(&kb_path, &output_dir);
    let output_path = output_root.join("index.html");

    let sections = build_sections(&kb_path)?;
    let html = render_viewer(&kb_path, &sections);

    if args.is_dry_run() {
        println!("kb view preview:");
        println!("  knowledge base : {}", kb_path.display());
        println!("  output         : {}", output_path.display());
        println!("  sections       : {}", sections.len());
        println!("  open browser   : {}", args.should_open());
        println!("  no files written");
        return Ok(());
    }

    fs::create_dir_all(&output_root)?;
    fs::write(&output_path, html)?;

    println!("Static HTML viewer generated:");
    println!("  {}", output_path.display());
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

fn execute_relationship_view(kb_path: &Path, args: &ViewArgs) -> Result<()> {
    let output_dir = args
        .output_dir
        .clone()
        .unwrap_or_else(|| PathBuf::from("outputs/html"));
    let output_root = resolve_under_kb(kb_path, &output_dir);
    let data_path = output_root.join("relationship_data.json");
    let html_path = output_root.join("relationship_viewer.html");

    let data = build_relationship_data(kb_path, args)?;

    if args.is_dry_run() {
        println!("kb view --relations preview:");
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

fn build_relationship_data(kb_path: &Path, args: &ViewArgs) -> Result<RelationshipData> {
    let mut warnings = Vec::new();
    let mut nodes_by_id: BTreeMap<String, RelationshipNode> = BTreeMap::new();
    let mut edges: Vec<RelationshipEdge> = Vec::new();
    let mut source_refs_graph = None;

    if let Some(refs_graph_path) =
        latest_matching_path(kb_path, "processing/refs", "refs_graph_", "json")?
    {
        source_refs_graph = Some(relative_path_string(kb_path, &refs_graph_path));
        match fs::read_to_string(&refs_graph_path)
            .ok()
            .and_then(|content| serde_json::from_str::<Value>(&content).ok())
        {
            Some(value) => import_refs_graph_json(kb_path, &value, &mut nodes_by_id, &mut edges),
            None => warnings.push(format!(
                "Could not parse latest refs graph JSON: {}",
                relative_path_string(kb_path, &refs_graph_path)
            )),
        }
    } else {
        warnings.push("No processing/refs/refs_graph_*.json file found; run `kb refs-graph --json` for richer bibliographic edges.".to_string());
    }

    add_raw_paper_nodes(kb_path, &mut nodes_by_id)?;
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
            version: "v0.7.9".to_string(),
            generated_by: "kb view --relations".to_string(),
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
            warnings.push("No topics/ directory found; --topic cannot be focused yet.".to_string());
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
                "Requested topic `{}` was not found under topics/. The viewer will still show global relations.",
                slug
            ));
        }
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
                status: if Some(&slug) == requested_slug.as_ref() {
                    "focused"
                } else {
                    "indexed"
                }
                .to_string(),
                evidence: vec![topic_rel_path],
                needs_llm_review: false,
            },
        );

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
    let mut files = vec!["outputs/html/relationship_data.json".to_string()];
    if let Some(path) = source_refs_graph {
        files.push(path.clone());
    }
    if let Some(topic) = topic {
        files.push(format!("topics/{}/", slugify(topic)));
    }
    vec![RelationshipTask {
        id: "manager:relationship-review-plan".to_string(),
        role: "Manager LLM".to_string(),
        title: "Plan relation review batches without making final scholarly claims".to_string(),
        status: "open".to_string(),
        files,
        evidence: vec![format!(
            "{} edges need review; {} missing/unresolved edges; {} ambiguous edges.",
            overview.llm_review_edge_count,
            overview.missing_edge_count,
            overview.ambiguous_edge_count
        )],
    }]
}

fn build_relationship_worker_tasks(
    overview: &RelationshipOverview,
    source_refs_graph: &Option<String>,
    topic: Option<&str>,
) -> Vec<RelationshipTask> {
    let mut files = vec!["outputs/html/relationship_data.json".to_string()];
    if let Some(path) = source_refs_graph {
        files.push(path.clone());
    }
    if let Some(topic) = topic {
        files.push(format!("topics/{}/importance/", slugify(topic)));
        files.push(format!("topics/{}/relations/", slugify(topic)));
    }
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
    html.push_str(&format!(
        "<title>{} - Relationship Graph</title>\n",
        kb_name
    ));
    html.push_str("<style>\n");
    html.push_str(RELATIONSHIP_VIEWER_CSS);
    html.push_str("\n</style>\n</head>\n<body>\n<div class=\"relation-shell\">\n");
    html.push_str("<aside class=\"relation-sidebar\">\n<div class=\"sidebar-head\">🕸️ Relationship Graph</div>\n");
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
    html.push_str("</h1><p>Static relationship graph review page for paper-level index relations and topic-local academic viewpoint candidates.</p></header>\n");
    html.push_str("<section class=\"panel active\" id=\"overview\"><h2>Overview</h2><div id=\"overviewGrid\" class=\"metric-grid\"></div><div id=\"warnings\"></div></section>\n");
    html.push_str("<section class=\"panel\" id=\"graph\"><h2>Graph</h2><p class=\"hint\">Solid edges are confirmed/accepted. Dashed edges are candidates or need LLM review. Dotted edges are missing or unresolved references.</p><div class=\"graph-wrap\"><svg id=\"graphSvg\" viewBox=\"0 0 1100 680\" role=\"img\" aria-label=\"Relationship graph\"></svg></div></section>\n");
    html.push_str("<section class=\"panel\" id=\"topics\"><h2>Topics</h2><div id=\"topicsTable\"></div></section>\n");
    html.push_str("<section class=\"panel\" id=\"manager-tasks\"><h2>LLM Manager Tasks</h2><div id=\"managerTasks\"></div></section>\n");
    html.push_str("<section class=\"panel\" id=\"worker-tasks\"><h2>LLM Worker Tasks</h2><div id=\"workerTasks\"></div></section>\n");
    html.push_str("<section class=\"panel\" id=\"nodes\"><h2>Nodes</h2><div id=\"nodesTable\"></div></section>\n");
    html.push_str("<section class=\"panel\" id=\"edges\"><h2>Edges</h2><div id=\"edgesTable\"></div></section>\n");
    html.push_str("<section class=\"panel\" id=\"raw-json\"><h2>Raw JSON</h2><pre id=\"rawJson\"></pre></section>\n");
    html.push_str("</main>\n</div>\n<script id=\"relationship-data\" type=\"application/json\">\n");
    html.push_str(&json);
    html.push_str("\n</script>\n<script>\n");
    html.push_str(RELATIONSHIP_VIEWER_JS);
    html.push_str("\n</script>\n</body>\n</html>\n");
    Ok(html)
}

const RELATIONSHIP_VIEWER_CSS: &str = r#"
* { box-sizing: border-box; margin: 0; padding: 0; font-family: "Segoe UI", Arial, sans-serif; }
body { background: #f5f7fa; color: #243042; line-height: 1.6; }
.relation-shell { display: flex; min-height: 100vh; }
.relation-sidebar { width: 320px; flex-shrink: 0; background: #fff; border-right: 1px solid #d9e2ec; padding: 1rem; display: flex; flex-direction: column; gap: 1rem; }
.sidebar-head { background: #2b6cb0; color: #fff; padding: 0.85rem 1rem; border-radius: 8px; font-weight: 700; }
.meta-card { font-size: 0.86rem; background: #f7fafc; border: 1px solid #e2e8f0; border-radius: 8px; padding: 0.85rem; word-break: break-word; }
.sidebar-links { display: flex; flex-direction: column; gap: 0.45rem; }
.sidebar-link { display: block; width: 100%; text-align: left; border: 1px solid #d9e2ec; background: #f7fafc; color: #245a91; padding: 0.58rem 0.75rem; border-radius: 7px; cursor: pointer; text-decoration: none; font-weight: 600; }
.sidebar-link:hover, .sidebar-link.active { background: #e8f4f8; border-color: #4299e1; }
.legend { margin-top: auto; background: #f7fafc; border: 1px solid #e2e8f0; border-radius: 8px; padding: 0.8rem; font-size: 0.85rem; }
.line { display: inline-block; width: 42px; margin-right: 0.45rem; vertical-align: middle; border-top: 3px solid #2d3748; }
.line.dashed { border-top-style: dashed; }
.line.dotted { border-top-style: dotted; }
.relation-main { flex: 1; padding: 2rem; max-width: 1260px; margin: 0 auto; }
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
.empty { padding: 1rem; border: 1px dashed #cbd5e0; background: #f7fafc; color: #718096; border-radius: 8px; }
@media (max-width: 860px) { .relation-shell { flex-direction: column; } .relation-sidebar { width: 100%; } .relation-main { padding: 1rem; } }
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

fn build_sections(kb_path: &Path) -> Result<Vec<ViewerSection>> {
    let mut sections = Vec::new();

    sections.push(ViewerSection {
        id: "overview".to_string(),
        title: "Overview".to_string(),
        subtitle: "Local LLM Wiki structure and latest generated artifacts".to_string(),
        html: render_overview(kb_path),
    });

    sections.push(ViewerSection {
        id: "wiki".to_string(),
        title: "Wiki".to_string(),
        subtitle: "Wiki home or project README".to_string(),
        html: render_source_card_or_empty(
            read_first_existing(kb_path, &["wiki/Home.md", "wiki/index.md", "README.md"])?,
            "No wiki Home.md or README.md was found.",
        ),
    });

    sections.push(ViewerSection {
        id: "refs-index".to_string(),
        title: "Refs Index".to_string(),
        subtitle: "Latest bibliographic index relation candidate report".to_string(),
        html: render_source_card_or_empty(
            latest_matching_file(kb_path, "processing/refs", "refs_index_", "md")?,
            "No refs-index report found. Run `kb refs-index` first.",
        ),
    });

    sections.push(ViewerSection {
        id: "refs-graph".to_string(),
        title: "Refs Graph".to_string(),
        subtitle: "Latest graph export files for third-party visualizers".to_string(),
        html: render_refs_graph(kb_path)?,
    });

    sections.push(ViewerSection {
        id: "keywords".to_string(),
        title: "Keywords".to_string(),
        subtitle: "Latest keyword/topic relation candidate report".to_string(),
        html: render_source_card_or_empty(
            latest_matching_file(kb_path, "processing/keywords", "keywords_", "md")?,
            "No keyword report found. Run `kb keywords` first.",
        ),
    });

    sections.push(ViewerSection {
        id: "health".to_string(),
        title: "Health".to_string(),
        subtitle: "Latest deterministic project health report".to_string(),
        html: render_source_card_or_empty(
            latest_matching_file(kb_path, "outputs/reports", "health_", "md")?,
            "No health report found. Run `kb health` first.",
        ),
    });

    sections.push(ViewerSection {
        id: "tasks".to_string(),
        title: "LLM Tasks".to_string(),
        subtitle: "Latest handoff task list for Manager/Worker LLM workflows".to_string(),
        html: render_source_card_or_empty(
            latest_matching_file(kb_path, "LLM/tasks", "llm_tasks_", "md")?,
            "No LLM task handoff found. Run `kb tasks` first.",
        ),
    });

    sections.push(ViewerSection {
        id: "memory".to_string(),
        title: "LLM Memory".to_string(),
        subtitle: "Completed task memory records".to_string(),
        html: render_source_card_or_empty(
            read_first_existing(kb_path, &["LLM/memory/completed_tasks.md"])?,
            "No completed task memory found. Run `kb memory --task-id ... --summary ...` after completing work.",
        ),
    });

    sections.push(ViewerSection {
        id: "topics".to_string(),
        title: "Topics".to_string(),
        subtitle: "Topic-specific relationship overlays".to_string(),
        html: render_topics(kb_path)?,
    });

    Ok(sections)
}

fn render_viewer(kb_path: &Path, sections: &[ViewerSection]) -> String {
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
            "<button class=\"sidebar-link\" data-target=\"{}\">{}</button>\n",
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
    html.push_str(&format!("<title>{} - LLM Wiki Viewer</title>\n", kb_name));
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
    html.push_str("<a class=\"sidebar-link sidebar-anchor\" href=\"relationship_viewer.html\">Relationship Graph</a>\n");
    html.push_str("</div>\n</div>\n");
    html.push_str("<div class=\"terminal\">\n<div class=\"terminal-head\">kb-view&gt; display commands only</div>\n<div class=\"terminal-log\" id=\"commandLog\"><div class=\"terminal-msg\">Type <code>help</code>. This box cannot execute local kb commands.</div></div>\n<div class=\"terminal-input\"><input id=\"viewCommand\" type=\"text\" placeholder=\"help / open health / find DOI\"/><button id=\"runViewCommand\">Run</button></div>\n</div>\n");
    html.push_str(
        "</div>\n</aside>\n<button class=\"toggle-sidebar\" id=\"toggleBtn\">‹</button>\n",
    );
    html.push_str("<main class=\"main\">\n<header><h1>");
    html.push_str(&kb_name);
    html.push_str("</h1><p class=\"subtitle\">LLM Wiki of your knowledge base for the purpose of swift and high quality research</p></header>\n");
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
.container { display: flex; min-height: 100vh; position: relative; }
.sidebar { width: 340px; background: #fff; border-right: 1px solid #e2e8f0; display: flex; flex-direction: column; transition: width 0.2s; overflow: hidden; flex-shrink: 0; }
.sidebar.hidden { width: 0; }
.sidebar-head { padding: 1rem 1.2rem; background: #2b6cb0; color: white; font-weight: 700; }
.sidebar-body { flex: 1; min-height: 0; padding: 1rem; overflow: hidden; display: flex; flex-direction: column; gap: 1rem; }
.sidebar-scroll { flex: 1 1 auto; min-height: 0; overflow-y: auto; display: flex; flex-direction: column; gap: 1rem; padding-right: 0.15rem; }
.meta-card { font-size: 0.85rem; background: #f7fafc; border: 1px solid #e2e8f0; border-radius: 8px; padding: 0.8rem; word-break: break-all; }
.sidebar-links { display: flex; flex-direction: column; gap: 0.4rem; }
.sidebar-link { text-align: left; border: 1px solid #e2e8f0; background: #f7fafc; color: #2b6cb0; padding: 0.55rem 0.7rem; border-radius: 6px; cursor: pointer; }
.sidebar-anchor { display: block; text-decoration: none; font-weight: 600; }
.sidebar-link:hover { background: #e8f4f8; }
.toggle-sidebar { position: absolute; top: 12px; left: 340px; z-index: 10; background: #2b6cb0; color: white; border: 0; width: 32px; height: 32px; border-radius: 4px; cursor: pointer; transition: left 0.2s; }
.toggle-sidebar.collapsed { left: 0; }
.terminal { margin-top: auto; flex: 0 0 340px; min-height: 320px; border: 1px solid #cbd5e0; border-radius: 8px; overflow: hidden; background: #1a202c; color: #e2e8f0; display: flex; flex-direction: column; }
.terminal-head { padding: 0.5rem 0.7rem; background: #2d3748; font-size: 0.85rem; }
.terminal-log { flex: 1; min-height: 230px; padding: 0.85rem; overflow-y: auto; font-size: 0.84rem; }
.terminal-msg { margin-bottom: 0.45rem; }
.terminal-input { display: flex; border-top: 1px solid #4a5568; }
.terminal-input input { flex: 1; min-height: 46px; padding: 0.8rem; border: 0; outline: 0; background: #edf2f7; color: #1a202c; font-size: 0.9rem; }
.terminal-input button { min-height: 46px; padding: 0.8rem 0.95rem; border: 0; background: #4299e1; color: white; cursor: pointer; font-weight: 600; }
.main { flex: 1; padding: 2rem; max-width: 1180px; margin: 0 auto; overflow-x: hidden; }
header { text-align: center; margin-bottom: 1.5rem; padding-bottom: 1rem; border-bottom: 2px solid #4299e1; }
h1 { color: #2b6cb0; font-size: 1.65rem; }
.subtitle, .section-subtitle, .path { color: #718096; font-size: 0.92rem; }
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
code { background: #edf2f7; color: #2d3748; padding: 0.1rem 0.25rem; border-radius: 4px; }
pre code { background: transparent; color: inherit; padding: 0; }
.highlight, .suggestion-box { background: #e8f4f8; padding: 1rem; border-radius: 6px; margin: 1rem 0; border-left: 4px solid #4299e1; }
.source-card, .topic-card { border: 1px solid #e2e8f0; border-radius: 8px; padding: 1rem; margin: 1rem 0; }
.empty { color: #718096; background: #f7fafc; border: 1px dashed #cbd5e0; padding: 1rem; border-radius: 6px; }
details { margin: 0.8rem 0; border: 1px solid #e2e8f0; border-radius: 6px; padding: 0.7rem; }
summary { cursor: pointer; color: #2b6cb0; font-weight: 600; }
mark { background: #fefcbf; padding: 0 0.15rem; }
"#;

const VIEWER_JS: &str = r#"
const sidebar = document.getElementById('wikiSidebar');
const toggleBtn = document.getElementById('toggleBtn');
const commandLog = document.getElementById('commandLog');
const commandInput = document.getElementById('viewCommand');
const commandButton = document.getElementById('runViewCommand');

function switchTab(id) {
  const target = document.getElementById(id);
  if (!target) return false;
  document.querySelectorAll('.nav-tab').forEach(t => t.classList.remove('active'));
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

function runViewCommand() {
  const raw = commandInput.value.trim();
  if (!raw) return;
  log(`<span style="color:#90cdf4">kb-view&gt;</span> ${escapeHtml(raw)}`);
  commandInput.value = '';
  const lower = raw.toLowerCase();
  if (lower === 'help') {
    log('Commands: <code>open overview</code>, <code>open refs-index</code>, <code>open refs-graph</code>, <code>open keywords</code>, <code>open health</code>, <code>open tasks</code>, <code>open memory</code>, <code>open topics</code>, <code>find WORD</code>, <code>topic NAME</code>, <code>clear</code>.');
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

toggleBtn.addEventListener('click', () => {
  sidebar.classList.toggle('hidden');
  toggleBtn.classList.toggle('collapsed');
  toggleBtn.textContent = sidebar.classList.contains('hidden') ? '›' : '‹';
});

document.querySelectorAll('.nav-tab, .sidebar-link').forEach(tab => {
  tab.addEventListener('click', () => switchTab(tab.dataset.target));
});

commandButton.addEventListener('click', runViewCommand);
commandInput.addEventListener('keydown', e => {
  if (e.key === 'Enter') runViewCommand();
});
"#;

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
    out.push_str("<div class=\"suggestion-box\"><strong>Suggested refresh flow:</strong><br><code>kb health</code> → <code>kb refs-index</code> → <code>kb refs-graph</code> → <code>kb keywords</code> → <code>kb tasks</code> → <code>kb view</code></div>");
    out
}

fn render_refs_graph(kb_path: &Path) -> Result<String> {
    let mut cards = Vec::new();
    for ext in ["json", "mmd", "dot"] {
        if let Some(card) = latest_matching_file(kb_path, "processing/refs", "refs_graph_", ext)? {
            cards.push(card);
        }
    }

    if cards.is_empty() {
        return Ok("<p class=\"empty\">No refs graph export found. Run <code>kb refs-graph --json</code>, <code>--mermaid</code>, or <code>--dot</code> first.</p>".to_string());
    }

    let mut out = String::new();
    out.push_str("<div class=\"highlight\"><strong>Visual protocol:</strong> solid arrows = confirmed relations; dashed arrows = candidate/ambiguous relations; hollow nodes = missing/unresolved references; node size = literature importance.</div>");
    for card in cards {
        out.push_str(&format!(
            "<article class=\"source-card\"><h3>{}</h3><p class=\"path\">{}</p>{}</article>",
            html_escape(&card.title),
            html_escape(&card.path),
            card.html
        ));
    }
    Ok(out)
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
