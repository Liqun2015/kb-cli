use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use clap::Args;
use serde::Serialize;
use walkdir::WalkDir;

#[derive(Debug, Clone, Args)]
pub struct RefsGraphArgs {
    #[arg(
        long,
        value_name = "PATH",
        help = "refs-index Markdown report file or directory to read. Defaults to processing/refs/."
    )]
    pub path: Option<PathBuf>,

    #[arg(
        long = "output-dir",
        value_name = "DIR",
        help = "Output directory relative to the knowledge base. Defaults to processing/refs/."
    )]
    pub output_dir: Option<PathBuf>,

    #[arg(
        long,
        default_value_t = 0,
        help = "Maximum number of graph edges to include. Use 0 for no limit."
    )]
    pub limit: usize,

    #[arg(long, help = "Preview graph export without writing files")]
    pub dry_run: bool,

    #[arg(long, help = "Alias for --dry-run")]
    pub preview: bool,

    #[arg(long, help = "Print JSON graph data to stdout")]
    pub json: bool,

    #[arg(long, help = "Print Mermaid graph text to stdout")]
    pub mermaid: bool,

    #[arg(long, help = "Print Graphviz DOT graph text to stdout")]
    pub dot: bool,
}

#[derive(Debug, Clone, Serialize)]
struct VisualStyle {
    edge_style: String,
    source_node_style: String,
    target_node_style: String,
}

#[derive(Debug, Clone, Serialize)]
struct GraphNode {
    id: String,
    label: String,
    node_type: String,
    status: String,
    source_path: Option<String>,
    visual: BTreeMap<String, String>,
    needs_human_review: bool,
    evidence: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct GraphEdge {
    source: String,
    target: String,
    relation_type: String,
    status: String,
    confidence: f64,
    evidence: Vec<String>,
    visual: VisualStyle,
    needs_human_review: bool,
}

#[derive(Debug, Clone)]
struct ParsedRelation {
    status: String,
    source_text_file: String,
    source_line: usize,
    target_field: String,
    score: f64,
    review_required: bool,
    evidence: String,
}

#[derive(Debug, Clone, Serialize)]
struct DeferredGraphTask {
    target_agent: String,
    goal: String,
    requirements: Vec<String>,
    files: Vec<String>,
    evidence: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct RefsGraphReport {
    schema_version: String,
    generated_by: String,
    generated_at: String,
    input_path: String,
    output_dir: String,
    node_count: usize,
    edge_count: usize,
    limit: usize,
    dry_run: bool,
    json_path: Option<String>,
    mermaid_path: Option<String>,
    dot_path: Option<String>,
    counts: BTreeMap<String, usize>,
    nodes: Vec<GraphNode>,
    edges: Vec<GraphEdge>,
    deferred_tasks: Vec<DeferredGraphTask>,
}

pub fn execute(custom_kb: Option<&Path>, args: &RefsGraphArgs) -> Result<()> {
    let kb_path = crate::commands::init::get_kb_path(custom_kb);
    if !kb_path.exists() {
        return Err(anyhow!(
            "knowledge base path does not exist: {}. Run `kb init` first.",
            kb_path.display()
        ));
    }

    let mut report = run_refs_graph(&kb_path, args)?;
    let dry_run = args.dry_run || args.preview;
    let mermaid = render_mermaid(&report);
    let dot = render_dot(&report);

    if !dry_run {
        write_graph_outputs(&kb_path, args, &mut report, &mermaid, &dot)?;
    }

    if args.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else if args.mermaid {
        println!("{mermaid}");
    } else if args.dot {
        println!("{dot}");
    } else {
        print_report(&report);
    }

    Ok(())
}

fn run_refs_graph(kb_path: &Path, args: &RefsGraphArgs) -> Result<RefsGraphReport> {
    let input_path = resolve_input_path(kb_path, args)?;
    let output_dir = match &args.output_dir {
        Some(path) if path.is_absolute() => path.clone(),
        Some(path) => kb_path.join(path),
        None => kb_path.join("processing/refs"),
    };

    let mut relations = parse_refs_index_reports(kb_path, &input_path)?;
    relations.sort_by(|a, b| {
        a.status
            .cmp(&b.status)
            .then_with(|| a.source_text_file.cmp(&b.source_text_file))
            .then_with(|| a.source_line.cmp(&b.source_line))
    });

    if args.limit > 0 && relations.len() > args.limit {
        relations.truncate(args.limit);
    }

    let mut nodes_by_id = BTreeMap::new();
    let mut edges = Vec::new();
    let mut counts = BTreeMap::new();

    for relation in &relations {
        *counts.entry(relation.status.clone()).or_insert(0) += 1;
        let source_id = node_id_from_path(&relation.source_text_file);
        ensure_source_node(&mut nodes_by_id, &source_id, relation);

        let targets = relation_targets(relation);
        for target in targets {
            let (target_id, target_label, target_path, node_type, node_status, target_style) = target;
            ensure_target_node(
                &mut nodes_by_id,
                &target_id,
                &target_label,
                target_path.as_deref(),
                &node_type,
                &node_status,
                &target_style,
                relation.review_required,
                relation,
            );

            edges.push(GraphEdge {
                source: source_id.clone(),
                target: target_id,
                relation_type: "bibliographic_index".to_string(),
                status: relation.status.clone(),
                confidence: relation.score,
                evidence: evidence_for_relation(relation),
                visual: visual_for_status(&relation.status),
                needs_human_review: relation.review_required || relation.status != "confirmed",
            });
        }
    }

    let mut nodes = nodes_by_id.into_values().collect::<Vec<_>>();
    nodes.sort_by(|a, b| a.id.cmp(&b.id));
    edges.sort_by(|a, b| {
        a.status
            .cmp(&b.status)
            .then_with(|| a.source.cmp(&b.source))
            .then_with(|| a.target.cmp(&b.target))
    });

    let deferred_tasks = build_deferred_tasks(&edges);

    Ok(RefsGraphReport {
        schema_version: "0.5.11".to_string(),
        generated_by: "kb-cli refs-graph".to_string(),
        generated_at: chrono::Utc::now().to_rfc3339(),
        input_path: relative_path_string(kb_path, &input_path),
        output_dir: relative_path_string(kb_path, &output_dir),
        node_count: nodes.len(),
        edge_count: edges.len(),
        limit: args.limit,
        dry_run: args.dry_run || args.preview,
        json_path: None,
        mermaid_path: None,
        dot_path: None,
        counts,
        nodes,
        edges,
        deferred_tasks,
    })
}

fn resolve_input_path(kb_path: &Path, args: &RefsGraphArgs) -> Result<PathBuf> {
    let path = match &args.path {
        Some(path) if path.is_absolute() => path.clone(),
        Some(path) => kb_path.join(path),
        None => kb_path.join("processing/refs"),
    };

    if path.is_file() {
        return Ok(path);
    }
    if !path.exists() {
        return Err(anyhow!(
            "refs-graph input path does not exist: {}. Run `kb refs-index` first or pass --path.",
            path.display()
        ));
    }

    let mut reports = collect_refs_index_reports(&path)?;
    reports.sort();
    reports.pop().ok_or_else(|| {
        anyhow!(
            "no refs_index_*.md reports found under {}. Run `kb refs-index` first.",
            path.display()
        )
    })
}

fn collect_refs_index_reports(root: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in WalkDir::new(root)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
    {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        if name.starts_with("refs_index_") && name.ends_with(".md") {
            files.push(path.to_path_buf());
        }
    }
    Ok(files)
}

fn parse_refs_index_reports(kb_path: &Path, path: &Path) -> Result<Vec<ParsedRelation>> {
    let mut reports = Vec::new();
    if path.is_file() {
        reports.push(path.to_path_buf());
    } else {
        reports = collect_refs_index_reports(path)?;
    }

    let mut relations = Vec::new();
    for report_path in reports {
        let content = fs::read_to_string(&report_path)?;
        for line in content.lines() {
            if !line.starts_with('|') {
                continue;
            }
            if line.contains("| Status |") || line.contains("|---") {
                continue;
            }
            let columns = split_markdown_row(line);
            if columns.len() < 6 {
                continue;
            }
            let status = clean_cell(&columns[0]);
            if !matches!(
                status.as_str(),
                "confirmed" | "candidate" | "ambiguous" | "missing" | "needs_human"
            ) {
                continue;
            }
            let source = clean_cell(&columns[1]);
            let (source_text_file, source_line) = parse_source_ref(&source);
            let target_field = clean_cell(&columns[2]);
            let score = clean_cell(&columns[3]).parse::<f64>().unwrap_or(0.0);
            let review_required = clean_cell(&columns[4]) == "true";
            let evidence = clean_cell(&columns[5]);

            relations.push(ParsedRelation {
                status,
                source_text_file: normalize_relative(&relative_or_raw(kb_path, &source_text_file)),
                source_line,
                target_field,
                score,
                review_required,
                evidence,
            });
        }
    }

    Ok(relations)
}

fn split_markdown_row(line: &str) -> Vec<String> {
    let trimmed = line.trim().trim_matches('|');
    let mut cells = Vec::new();
    let mut current = String::new();
    let mut escaped = false;

    for ch in trimmed.chars() {
        if escaped {
            current.push(ch);
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            current.push(ch);
            continue;
        }
        if ch == '|' {
            cells.push(current.trim().to_string());
            current.clear();
        } else {
            current.push(ch);
        }
    }
    cells.push(current.trim().to_string());
    cells
}

fn clean_cell(cell: &str) -> String {
    let mut value = cell.trim().to_string();
    if value.starts_with('`') && value.ends_with('`') && value.len() >= 2 {
        value = value[1..value.len() - 1].to_string();
    }
    value.replace("\\|", "|").trim().to_string()
}

fn parse_source_ref(source: &str) -> (String, usize) {
    if let Some((path, line)) = source.rsplit_once(':') {
        if let Ok(line_number) = line.parse::<usize>() {
            return (path.to_string(), line_number);
        }
    }
    (source.to_string(), 0)
}

fn relation_targets(
    relation: &ParsedRelation,
) -> Vec<(String, String, Option<String>, String, String, String)> {
    if relation.status == "missing" || relation.target_field.trim().is_empty() {
        let id = format!(
            "unresolved::{}::{}",
            node_id_from_path(&relation.source_text_file),
            relation.source_line
        );
        let label = format!("Unresolved reference at line {}", relation.source_line);
        return vec![
            (
                id,
                label,
                None,
                "unresolved_reference".to_string(),
                "missing".to_string(),
                "hollow".to_string(),
            ),
        ];
    }

    let mut targets = Vec::new();
    for raw in relation.target_field.split(';') {
        let path = raw.trim();
        if path.is_empty() {
            continue;
        }
        let id = node_id_from_path(path);
        targets.push((
            id,
            label_from_path(path),
            Some(path.to_string()),
            "paper".to_string(),
            if relation.status == "confirmed" {
                "confirmed".to_string()
            } else {
                "candidate".to_string()
            },
            "filled".to_string(),
        ));
    }

    if targets.is_empty() {
        relation_targets(&ParsedRelation {
            target_field: String::new(),
            ..relation.clone()
        })
    } else {
        targets
    }
}

fn ensure_source_node(nodes: &mut BTreeMap<String, GraphNode>, id: &str, relation: &ParsedRelation) {
    nodes.entry(id.to_string()).or_insert_with(|| {
        let mut visual = BTreeMap::new();
        visual.insert("node_style".to_string(), "filled".to_string());
        GraphNode {
            id: id.to_string(),
            label: label_from_path(&relation.source_text_file),
            node_type: "source_text".to_string(),
            status: "local".to_string(),
            source_path: Some(relation.source_text_file.clone()),
            visual,
            needs_human_review: false,
            evidence: vec![format!("source text file: {}", relation.source_text_file)],
        }
    });
}

fn ensure_target_node(
    nodes: &mut BTreeMap<String, GraphNode>,
    id: &str,
    label: &str,
    source_path: Option<&str>,
    node_type: &str,
    status: &str,
    node_style: &str,
    needs_human_review: bool,
    relation: &ParsedRelation,
) {
    nodes.entry(id.to_string()).or_insert_with(|| {
        let mut visual = BTreeMap::new();
        visual.insert("node_style".to_string(), node_style.to_string());
        GraphNode {
            id: id.to_string(),
            label: label.to_string(),
            node_type: node_type.to_string(),
            status: status.to_string(),
            source_path: source_path.map(|s| s.to_string()),
            visual,
            needs_human_review,
            evidence: evidence_for_relation(relation),
        }
    });
}

fn visual_for_status(status: &str) -> VisualStyle {
    match status {
        "confirmed" => VisualStyle {
            edge_style: "solid".to_string(),
            source_node_style: "filled".to_string(),
            target_node_style: "filled".to_string(),
        },
        "missing" | "needs_human" => VisualStyle {
            edge_style: "dashed".to_string(),
            source_node_style: "filled".to_string(),
            target_node_style: "hollow".to_string(),
        },
        _ => VisualStyle {
            edge_style: "dashed".to_string(),
            source_node_style: "filled".to_string(),
            target_node_style: "filled".to_string(),
        },
    }
}

fn evidence_for_relation(relation: &ParsedRelation) -> Vec<String> {
    let mut evidence = Vec::new();
    evidence.push(format!(
        "{}:{}",
        relation.source_text_file, relation.source_line
    ));
    if !relation.evidence.is_empty() {
        evidence.push(relation.evidence.clone());
    }
    evidence
}

fn build_deferred_tasks(edges: &[GraphEdge]) -> Vec<DeferredGraphTask> {
    let uncertain = edges
        .iter()
        .filter(|edge| edge.needs_human_review || edge.status != "confirmed")
        .collect::<Vec<_>>();
    if uncertain.is_empty() {
        return Vec::new();
    }

    let mut files = BTreeSet::new();
    let mut evidence = Vec::new();
    for edge in uncertain.iter().take(40) {
        files.insert(edge.source.clone());
        files.insert(edge.target.clone());
        evidence.push(format!(
            "{} -> {} [{} confidence={:.2}]",
            edge.source, edge.target, edge.status, edge.confidence
        ));
    }

    vec![DeferredGraphTask {
        target_agent: "Human Reference Graph Reviewer".to_string(),
        goal: "Review uncertain bibliographic graph edges before treating them as confirmed literature relations.".to_string(),
        requirements: vec![
            "Confirm, reject, or mark missing each candidate / ambiguous / missing graph relation.".to_string(),
            "Preserve evidence lines and source reference entries for every decision.".to_string(),
            "Do not let an LLM serve as the final guarantee for bibliographic identity.".to_string(),
            "After review, record accepted decisions with kb memory.".to_string(),
        ],
        files: files.into_iter().collect(),
        evidence,
    }]
}

fn write_graph_outputs(
    kb_path: &Path,
    args: &RefsGraphArgs,
    report: &mut RefsGraphReport,
    mermaid: &str,
    dot: &str,
) -> Result<()> {
    let output_dir = match &args.output_dir {
        Some(path) if path.is_absolute() => path.clone(),
        Some(path) => kb_path.join(path),
        None => kb_path.join("processing/refs"),
    };
    fs::create_dir_all(&output_dir)?;
    let stamp = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();

    let json_path = output_dir.join(format!("refs_graph_{stamp}.json"));
    fs::write(&json_path, serde_json::to_string_pretty(report)?)?;
    report.json_path = Some(relative_path_string(kb_path, &json_path));

    if args.mermaid {
        let path = output_dir.join(format!("refs_graph_{stamp}.mmd"));
        fs::write(&path, mermaid)?;
        report.mermaid_path = Some(relative_path_string(kb_path, &path));
    }

    if args.dot {
        let path = output_dir.join(format!("refs_graph_{stamp}.dot"));
        fs::write(&path, dot)?;
        report.dot_path = Some(relative_path_string(kb_path, &path));
    }

    Ok(())
}

fn render_mermaid(report: &RefsGraphReport) -> String {
    let mut out = String::new();
    out.push_str("graph LR\n");
    for node in &report.nodes {
        let label = escape_mermaid_label(&node.label);
        if node.visual.get("node_style").map(|s| s.as_str()) == Some("hollow") {
            out.push_str(&format!("    {}((\"{}\"))\n", mermaid_id(&node.id), label));
        } else {
            out.push_str(&format!("    {}[\"{}\"]\n", mermaid_id(&node.id), label));
        }
    }
    for edge in &report.edges {
        let connector = if edge.visual.edge_style == "solid" {
            "-->"
        } else {
            "-.->"
        };
        out.push_str(&format!(
            "    {} {} {}\n",
            mermaid_id(&edge.source),
            connector,
            mermaid_id(&edge.target)
        ));
    }
    out
}

fn render_dot(report: &RefsGraphReport) -> String {
    let mut out = String::new();
    out.push_str("digraph refs_graph {\n");
    out.push_str("  rankdir=LR;\n");
    for node in &report.nodes {
        let shape = if node.visual.get("node_style").map(|s| s.as_str()) == Some("hollow") {
            "circle"
        } else {
            "box"
        };
        let style = if shape == "circle" { "" } else { "filled" };
        out.push_str(&format!(
            "  \"{}\" [label=\"{}\", shape={}, style=\"{}\"];\n",
            escape_dot(&node.id),
            escape_dot(&node.label),
            shape,
            style
        ));
    }
    for edge in &report.edges {
        let style = if edge.visual.edge_style == "solid" {
            "solid"
        } else {
            "dashed"
        };
        out.push_str(&format!(
            "  \"{}\" -> \"{}\" [style={}, label=\"{} {:.2}\"];\n",
            escape_dot(&edge.source),
            escape_dot(&edge.target),
            style,
            edge.status,
            edge.confidence
        ));
    }
    out.push_str("}\n");
    out
}

fn print_report(report: &RefsGraphReport) {
    println!("kb refs-graph");
    println!("Input: {}", report.input_path);
    println!("Nodes: {}", report.node_count);
    println!("Edges: {}", report.edge_count);
    println!("Dry run: {}", report.dry_run);
    if let Some(path) = &report.json_path {
        println!("JSON graph: {path}");
    }
    if let Some(path) = &report.mermaid_path {
        println!("Mermaid graph: {path}");
    }
    if let Some(path) = &report.dot_path {
        println!("DOT graph: {path}");
    }
    println!();

    if !report.counts.is_empty() {
        println!("Edges by status:");
        for (status, count) in &report.counts {
            println!("  {status}: {count}");
        }
        println!();
    }

    if !report.deferred_tasks.is_empty() {
        println!("Deferred review tasks:");
        for task in &report.deferred_tasks {
            println!("  - {}: {}", task.target_agent, task.goal);
        }
        println!();
    }

    println!("Visual protocol:");
    println!("  confirmed relation        -> solid edge");
    println!("  candidate / ambiguous     -> dashed edge");
    println!("  missing / unresolved node -> hollow node");
}

fn node_id_from_path(path: &str) -> String {
    path.chars()
        .map(|ch| if ch.is_alphanumeric() { ch.to_ascii_lowercase() } else { '_' })
        .collect::<String>()
        .split('_')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("_")
}

fn label_from_path(path: &str) -> String {
    Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(path)
        .replace("__", "/")
        .replace('_', " ")
}

fn mermaid_id(id: &str) -> String {
    let normalized = node_id_from_path(id);
    if normalized.is_empty() {
        "node".to_string()
    } else {
        normalized
    }
}

fn escape_mermaid_label(value: &str) -> String {
    value.replace('"', "'").replace('\n', " ")
}

fn escape_dot(value: &str) -> String {
    value.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', " ")
}

fn relative_or_raw(kb_path: &Path, path: &str) -> String {
    let candidate = PathBuf::from(path);
    if candidate.is_absolute() {
        relative_path_string(kb_path, &candidate)
    } else {
        path.to_string()
    }
}

fn normalize_relative(path: &str) -> String {
    path.replace('\\', "/")
}

fn relative_path_string(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}
