use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod cli;
mod commands;
mod intent;

#[derive(Parser)]
#[command(name = "kb")]
#[command(about = "A minimal CLI for local knowledge base management", long_about = None)]
struct Cli {
    #[arg(
        short = 'k',
        long = "kb-path",
        value_name = "PATH",
        global = true,
        help = "Knowledge base directory name or path (default: KnowledgeBase)"
    )]
    kb_path: Option<PathBuf>,
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Initialize or ensure KnowledgeBase directory structure")]
    Init {
        #[arg(
            long,
            help = "Re-create missing directories and generated bootstrap files when safe"
        )]
        force: bool,
    },
    #[command(about = "Organize source files into raw/ subfolders")]
    Ingest(commands::ingest::IngestArgs),
    #[command(about = "Initialize, ingest, extract metadata, and build wiki")]
    Bootstrap(commands::ingest::BootstrapArgs),
    #[command(about = "Build Wiki from raw materials")]
    BuildWiki,
    #[command(
        name = "prepare",
        alias = "compile",
        about = "Prepare LLM/wiki handoff artifacts from raw files without calling an LLM"
    )]
    Prepare(commands::prepare::PrepareArgs),
    #[command(about = "Search wiki Markdown pages with local keyword matching")]
    Query(commands::query::QueryArgs),
    #[command(
        about = "Scan wiki Markdown pages for WikiLinks and deterministic link-resolution hints"
    )]
    Links(commands::links::LinksArgs),
    #[command(about = "Detect keyword/topic co-occurrence candidates among extracted text files")]
    Keywords(commands::keywords::KeywordsArgs),
    #[command(
        about = "Scan extracted text for reference headings, DOI values, and citation-pattern hints"
    )]
    Refs(commands::refs::RefsArgs),
    #[command(
        about = "Build bibliographic index relation candidates between reference entries and local papers"
    )]
    RefsIndex(commands::refs_index::RefsIndexArgs),
    #[command(
        about = "Export bibliographic relation candidates as graph data for third-party visualization"
    )]
    RefsGraph(commands::refs_graph::RefsGraphArgs),
    #[command(about = "Generate deferred LLM/agent task handoff lists from deterministic checks")]
    Tasks(commands::tasks::TasksArgs),
    #[command(about = "Append a completed task memory record under LLM/memory/")]
    Memory(commands::memory::MemoryArgs),
    #[command(about = "Search text files with Rust-native grep-like matching")]
    Grep(commands::grep::GrepArgs),
    #[command(about = "Summarize deterministic LLM Wiki and literature-relation health checks")]
    Health(commands::health::HealthArgs),
    #[command(about = "Audit whether a material pile has become a reviewable LLM Wiki")]
    AuditWiki(commands::audit_wiki::AuditWikiArgs),
    #[command(about = "Manage topic-specific literature relationship workspaces")]
    Topic(commands::topic::TopicArgs),
    #[command(about = "Generate a static local HTML viewer for LLM Wiki results")]
    View(commands::view::ViewArgs),
    #[command(about = "Sync wiki page source front matter back into processing/manifest.json")]
    SyncWiki(commands::sync_wiki::SyncWikiArgs),
    #[command(about = "Run static wiki health checks for links, orphans, and source front matter")]
    LintStatic(commands::lint_static::LintStaticArgs),
    #[command(about = "Refresh manifest and show raw-file status")]
    Status(commands::manifest::StatusArgs),
    #[command(about = "Extract metadata from papers")]
    ExtractMetadata {
        #[arg(long, help = "Overwrite existing metadata file")]
        force: bool,
    },
    #[command(
        about = "Extract plain text from supported source files with deterministic best-effort methods"
    )]
    ExtractText(commands::extract_text::ExtractTextArgs),
    #[command(
        name = "shell",
        alias = "repl",
        about = "Enter deterministic interactive shell mode"
    )]
    Shell,
    #[command(about = "List all configured models")]
    ListModels,
    #[command(about = "Show current model configuration")]
    ShowModel,
    #[command(about = "Add a new model configuration")]
    AddModel(commands::model::ModelArgs),
    #[command(about = "Switch to a different model")]
    SwitchModel { id: String },
    #[command(about = "Delete a model configuration")]
    DeleteModel { id: String },
    #[command(about = "Validate model configuration")]
    ValidateModel { id: Option<String> },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Init { force }) => commands::init::execute(cli.kb_path.as_deref(), *force),
        Some(Commands::Ingest(args)) => commands::ingest::execute(cli.kb_path.as_deref(), args),
        Some(Commands::Bootstrap(args)) => {
            commands::ingest::execute_bootstrap(cli.kb_path.as_deref(), args)
        }
        Some(Commands::BuildWiki) => commands::build_wiki::execute(cli.kb_path.as_deref()),
        Some(Commands::Prepare(args)) => commands::prepare::execute(cli.kb_path.as_deref(), args),
        Some(Commands::Query(args)) => commands::query::execute(cli.kb_path.as_deref(), args),
        Some(Commands::Links(args)) => commands::links::execute(cli.kb_path.as_deref(), args),
        Some(Commands::Keywords(args)) => commands::keywords::execute(cli.kb_path.as_deref(), args),
        Some(Commands::Refs(args)) => commands::refs::execute(cli.kb_path.as_deref(), args),
        Some(Commands::RefsIndex(args)) => {
            commands::refs_index::execute(cli.kb_path.as_deref(), args)
        }
        Some(Commands::RefsGraph(args)) => {
            commands::refs_graph::execute(cli.kb_path.as_deref(), args)
        }
        Some(Commands::Tasks(args)) => commands::tasks::execute(cli.kb_path.as_deref(), args),
        Some(Commands::Memory(args)) => commands::memory::execute(cli.kb_path.as_deref(), args),
        Some(Commands::Grep(args)) => commands::grep::execute(cli.kb_path.as_deref(), args),
        Some(Commands::Health(args)) => commands::health::execute(cli.kb_path.as_deref(), args),
        Some(Commands::AuditWiki(args)) => {
            commands::audit_wiki::execute(cli.kb_path.as_deref(), args)
        }
        Some(Commands::Topic(args)) => commands::topic::execute(cli.kb_path.as_deref(), args),
        Some(Commands::View(args)) => commands::view::execute(cli.kb_path.as_deref(), args),
        Some(Commands::SyncWiki(args)) => {
            commands::sync_wiki::execute(cli.kb_path.as_deref(), args)
        }
        Some(Commands::LintStatic(args)) => {
            commands::lint_static::execute(cli.kb_path.as_deref(), args)
        }
        Some(Commands::Status(args)) => commands::manifest::execute(cli.kb_path.as_deref(), args),
        Some(Commands::ExtractMetadata { force }) => {
            commands::extract_metadata::execute(cli.kb_path.as_deref(), *force)
        }
        Some(Commands::ExtractText(args)) => {
            commands::extract_text::execute(cli.kb_path.as_deref(), args)
        }
        Some(Commands::Shell) => commands::repl::execute(cli.kb_path.as_deref()),
        Some(Commands::ListModels) => commands::model::execute_list_models(),
        Some(Commands::ShowModel) => commands::model::execute_show_model(),
        Some(Commands::AddModel(args)) => commands::model::execute_add_model(args.clone()),
        Some(Commands::SwitchModel { id }) => commands::model::execute_switch_model(id),
        Some(Commands::DeleteModel { id }) => commands::model::execute_delete_model(id),
        Some(Commands::ValidateModel { id }) => {
            commands::model::execute_validate_model(id.as_deref())
        }
        None => {
            println!("Available commands:");
            println!("  init [--force]              Initialize/ensure knowledge base");
            println!("  ingest [--copy|--move]      Organize files into raw/ subfolders");
            println!(
                "  bootstrap [--copy|--move]   Initialize, ingest, extract metadata, build wiki"
            );
            println!("  extract-metadata [--force]  Extract PDF metadata");
            println!("  extract-text [--dry-run|--preview] [--force] [--json]  Extract text into processing/text");
            println!("  build-wiki                  Build wiki pages");
            println!("  prepare [--new|--file PATH] [--dry-run|--preview]  Prepare LLM/wiki handoff artifacts");
            println!("  query <terms...> [--limit N] [--snippets N] [--json] [--title-only]  Search wiki Markdown pages");
            println!("  links [--unresolved|--ambiguous|--resolved] [--json]  Scan wiki WikiLinks and resolution hints");
            println!("  keywords [terms...] [--path PATH] [--json] [--dry-run]  Detect keyword/topic relation candidates");
            println!("  refs [--path PATH] [--citations] [--json]  Scan text for reference and DOI hints");
            println!("  refs-index [--path PATH] [--dry-run|--preview] [--json]  Build bibliographic index relation candidates");
            println!("  refs-graph [--json|--mermaid|--dot] [--dry-run]  Export refs-index relations as graph data");
            println!("  tasks [--dry-run|--preview] [--json] [--index-only|--no-index]  Generate LLM task handoff lists and LLM/tasks/index.md");
            println!("  memory --task-id ID --summary TEXT [--file PATH]  Record completed task memory under LLM/memory");
            println!("  grep <pattern> [--path PATH] [--regex] [--json]  Search text files with line numbers");
            println!("  health [--dry-run|--preview] [--json] [--strict]  Summarize LLM Wiki relationship health");
            println!("  audit-wiki [--json] [--strict]  Prove/check whether the folder is a reviewable LLM Wiki");
            println!("  topic init <topic> [--title TITLE] [--force]  Initialize a topic relationship workspace");
            println!("  topic list [--json]          List topic relationship workspaces");
            println!(
                "  topic status <topic> [--json|--strict]  Check topic workspace completeness"
            );
            println!("  topic rank <topic> [--json] [--dry-run]  Generate topic-local importance candidates");
            println!(
                "  topic review <topic> [--json] [--dry-run] [--force]  Build a topic review queue"
            );
            println!("  view [--no-open] [--dry-run|--preview] [--output-dir DIR]  Generate/open static HTML viewer");
            println!(
                "  shell                         Enter deterministic interactive command shell"
            );
            println!("  status [--dry-run|--preview] [--json] [--unprocessed]  Refresh manifest and show raw-file status");
            println!("  sync-wiki [--dry-run|--preview] [--json]  Link wiki front matter back to manifest");
            println!("  lint-static [--dry-run|--preview|--no-report] [--json] [--strict]  Check broken links, orphans, and source front matter");
            println!("  list-models                 List all configured models");
            println!("  show-model                  Show current model");
            println!("  add-model                   Add a new model");
            println!("  switch-model <id>           Switch to a different model");
            println!("  delete-model <id>           Delete a model");
            println!("  validate-model [id]         Validate model configuration");
            Ok(())
        }
    }
}
