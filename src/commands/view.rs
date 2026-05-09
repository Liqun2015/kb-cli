use anyhow::{anyhow, Result};
use chrono::Utc;
use clap::Args;
use std::fs;
use std::path::{Path, PathBuf};
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
}

impl ViewArgs {
    fn is_dry_run(&self) -> bool {
        self.dry_run || self.preview
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
        println!("  no files written");
        return Ok(());
    }

    fs::create_dir_all(&output_root)?;
    fs::write(&output_path, html)?;

    println!("Static HTML viewer generated:");
    println!("  {}", output_path.display());
    println!();
    println!("Open this file in a browser. The sidebar command box is display-only:");
    println!(
        "  it can navigate tabs and search visible content, but it cannot execute kb commands."
    );

    Ok(())
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
    html.push_str("<title>LLM Wiki Viewer</title>\n");
    html.push_str("<style>\n");
    html.push_str(VIEWER_CSS);
    html.push_str("\n</style>\n</head>\n<body>\n<div class=\"container\">\n");
    html.push_str("<aside class=\"sidebar\" id=\"wikiSidebar\">\n");
    html.push_str("<div class=\"sidebar-head\">📚 LLM Wiki Navigator</div>\n");
    html.push_str("<div class=\"sidebar-body\">\n");
    html.push_str("<div class=\"meta-card\"><strong>Knowledge base</strong><br><span>");
    html.push_str(&kb_display);
    html.push_str("</span><br><strong>Generated</strong><br><span>");
    html.push_str(&generated_at);
    html.push_str("</span></div>\n");
    html.push_str("<div class=\"sidebar-links\">\n");
    html.push_str(&sidebar_links);
    html.push_str("</div>\n");
    html.push_str("<div class=\"terminal\">\n<div class=\"terminal-head\">kb-view&gt; display commands only</div>\n<div class=\"terminal-log\" id=\"commandLog\"><div class=\"terminal-msg\">Type <code>help</code>. This box cannot execute local kb commands.</div></div>\n<div class=\"terminal-input\"><input id=\"viewCommand\" type=\"text\" placeholder=\"help / open health / find DOI\"/><button id=\"runViewCommand\">Run</button></div>\n</div>\n");
    html.push_str(
        "</div>\n</aside>\n<button class=\"toggle-sidebar\" id=\"toggleBtn\">‹</button>\n",
    );
    html.push_str("<main class=\"main\">\n<header><h1>LLM Wiki Static Viewer</h1><p class=\"subtitle\">Markdown/JSON results rendered as a local, read-only review dashboard.</p></header>\n");
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
.sidebar-body { flex: 1; padding: 1rem; overflow-y: auto; display: flex; flex-direction: column; gap: 1rem; }
.meta-card { font-size: 0.85rem; background: #f7fafc; border: 1px solid #e2e8f0; border-radius: 8px; padding: 0.8rem; word-break: break-all; }
.sidebar-links { display: flex; flex-direction: column; gap: 0.4rem; }
.sidebar-link { text-align: left; border: 1px solid #e2e8f0; background: #f7fafc; color: #2b6cb0; padding: 0.55rem 0.7rem; border-radius: 6px; cursor: pointer; }
.sidebar-link:hover { background: #e8f4f8; }
.toggle-sidebar { position: absolute; top: 12px; left: 340px; z-index: 10; background: #2b6cb0; color: white; border: 0; width: 32px; height: 32px; border-radius: 4px; cursor: pointer; transition: left 0.2s; }
.toggle-sidebar.collapsed { left: 0; }
.terminal { border: 1px solid #cbd5e0; border-radius: 8px; overflow: hidden; background: #1a202c; color: #e2e8f0; }
.terminal-head { padding: 0.5rem 0.7rem; background: #2d3748; font-size: 0.85rem; }
.terminal-log { padding: 0.7rem; height: 190px; overflow-y: auto; font-size: 0.82rem; }
.terminal-msg { margin-bottom: 0.45rem; }
.terminal-input { display: flex; border-top: 1px solid #4a5568; }
.terminal-input input { flex: 1; padding: 0.55rem; border: 0; outline: 0; background: #edf2f7; color: #1a202c; }
.terminal-input button { padding: 0.55rem 0.8rem; border: 0; background: #4299e1; color: white; cursor: pointer; }
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
