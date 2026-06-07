use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod cli;
mod commands;
mod intent;

#[derive(Parser)]
#[command(name = "kb")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "A minimal CLI for local knowledge base management", long_about = None)]
struct Cli {
    #[arg(
        short = 'w',
        long = "wiki",
        value_name = "PATH",
        global = true,
        help = "LLM Wiki workspace root path. For setup with --source, this overrides the default <source>/knowledgebase workspace."
    )]
    wiki: Option<PathBuf>,

    #[arg(
        short = 's',
        long = "source",
        value_name = "PATH",
        global = true,
        help = "Existing source-material directory for setup/ingest. Setup defaults to creating <source>/knowledgebase."
    )]
    source: Option<PathBuf>,
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Initialize or ensure knowledge base directory structure")]
    Init(commands::init::InitArgs),
    #[command(about = "Organize source files into raw/ subfolders")]
    Ingest(commands::ingest::IngestArgs),
    #[command(
        name = "setup",
        visible_alias = "bootstrap",
        about = "Set up a knowledge base: init, ingest, extract metadata, and build wiki"
    )]
    Setup(commands::ingest::SetupArgs),
    #[command(
        name = "create",
        about = "Create a full LLM Wiki workspace from source materials: setup --recursive, then build <topic>"
    )]
    Create(commands::create::CreateArgs),
    #[command(
        name = "build",
        visible_alias = "assemble",
        about = "Run the one-command deterministic pipeline that prepares a topic for LLM Wiki agents"
    )]
    Build(commands::build::BuildArgs),
    #[command(about = "Build Wiki from raw materials")]
    BuildWiki,
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
    #[command(
        name = "task",
        about = "List, show, and mark Worker/Human task progress"
    )]
    Task(commands::task::TaskArgs),
    #[command(
        name = "prompt",
        about = "Print standard Worker/Human prompt templates for common LLM Wiki tasks"
    )]
    Prompt(commands::prompt::PromptArgs),
    #[command(name = "batch", about = "Create small bounded Worker task batches")]
    Batch(commands::batch::BatchArgs),
    #[command(about = "Generate generic/agent-adapter handoff files for external LLM Wiki agents")]
    Handoff(commands::handoff::HandoffArgs),
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
    #[command(
        name = "check",
        about = "Generate static local HTML check dashboards for LLM Wiki system status"
    )]
    Check(commands::check::CheckArgs),
    #[command(
        name = "view",
        about = "Generate a reader-facing research issue browser"
    )]
    View(commands::view::ViewArgs),
    #[command(
        name = "review",
        about = "Reserved entry for the future literature-review writing workflow"
    )]
    Review(commands::review::ReviewArgs),
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
    #[command(about = "Slice Introduction and References sections from extracted paper text")]
    ExtractSections(commands::extract_sections::ExtractSectionsArgs),
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
        Some(Commands::Init(args)) => commands::init::execute(cli.wiki.as_deref(), args),
        Some(Commands::Ingest(args)) => {
            commands::ingest::execute(cli.wiki.as_deref(), cli.source.as_deref(), args)
        }
        Some(Commands::Setup(args)) => {
            commands::ingest::execute_setup(cli.wiki.as_deref(), cli.source.as_deref(), args)
        }
        Some(Commands::Create(args)) => {
            commands::create::execute(cli.wiki.as_deref(), cli.source.as_deref(), args)
        }
        Some(Commands::Build(args)) => commands::build::execute(cli.wiki.as_deref(), args),
        Some(Commands::BuildWiki) => commands::build_wiki::execute(cli.wiki.as_deref()),
        Some(Commands::Query(args)) => commands::query::execute(cli.wiki.as_deref(), args),
        Some(Commands::Links(args)) => commands::links::execute(cli.wiki.as_deref(), args),
        Some(Commands::Keywords(args)) => commands::keywords::execute(cli.wiki.as_deref(), args),
        Some(Commands::Refs(args)) => commands::refs::execute(cli.wiki.as_deref(), args),
        Some(Commands::RefsIndex(args)) => commands::refs_index::execute(cli.wiki.as_deref(), args),
        Some(Commands::RefsGraph(args)) => commands::refs_graph::execute(cli.wiki.as_deref(), args),
        Some(Commands::Tasks(args)) => commands::tasks::execute(cli.wiki.as_deref(), args),
        Some(Commands::Task(args)) => commands::task::execute(cli.wiki.as_deref(), args),
        Some(Commands::Prompt(args)) => commands::prompt::execute(cli.wiki.as_deref(), args),
        Some(Commands::Batch(args)) => commands::batch::execute(cli.wiki.as_deref(), args),
        Some(Commands::Handoff(args)) => commands::handoff::execute(cli.wiki.as_deref(), args),
        Some(Commands::Memory(args)) => commands::memory::execute(cli.wiki.as_deref(), args),
        Some(Commands::Grep(args)) => commands::grep::execute(cli.wiki.as_deref(), args),
        Some(Commands::Health(args)) => commands::health::execute(cli.wiki.as_deref(), args),
        Some(Commands::AuditWiki(args)) => commands::audit_wiki::execute(cli.wiki.as_deref(), args),
        Some(Commands::Topic(args)) => commands::topic::execute(cli.wiki.as_deref(), args),
        Some(Commands::Check(args)) => commands::check::execute(cli.wiki.as_deref(), args),
        Some(Commands::View(args)) => commands::view::execute(cli.wiki.as_deref(), args),
        Some(Commands::Review(args)) => commands::review::execute(cli.wiki.as_deref(), args),
        Some(Commands::SyncWiki(args)) => commands::sync_wiki::execute(cli.wiki.as_deref(), args),
        Some(Commands::LintStatic(args)) => {
            commands::lint_static::execute(cli.wiki.as_deref(), args)
        }
        Some(Commands::Status(args)) => commands::manifest::execute(cli.wiki.as_deref(), args),
        Some(Commands::ExtractMetadata { force }) => {
            commands::extract_metadata::execute(cli.wiki.as_deref(), *force)
        }
        Some(Commands::ExtractText(args)) => {
            commands::extract_text::execute(cli.wiki.as_deref(), args)
        }
        Some(Commands::ExtractSections(args)) => {
            commands::extract_sections::execute(cli.wiki.as_deref(), args)
        }
        Some(Commands::Shell) => commands::repl::execute(cli.wiki.as_deref()),
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
            println!("  init [--topic TOPIC] [--force]  Initialize/ensure a named knowledge base and default topic");
            println!("  ingest [--copy|--move]      Organize files into raw/ subfolders");
            println!(
                "  setup [--copy|--move]       Set up KB: init, marker, ingest, metadata, wiki"
            );
            println!(
                "       use --source PATH to create PATH/knowledgebase as the default workspace"
            );
            println!("  bootstrap                   Compatibility alias for setup");
            println!("  create --wiki <A> --from <B> --about <topic>  Create an LLM Wiki workspace from a literature folder, then issue LLM/Human task guides");
            println!("  extract-metadata [--force]  Extract PDF metadata");
            println!("  extract-text [--dry-run|--preview] [--force] [--json]  Extract text into processing/text");
            println!("  extract-sections [--dry-run|--preview] [--force] [--json]  Slice Introduction/References into processing/sections");
            println!("  build <topic> [--force] [--dry-run]  Run extract/ref/topic/task/handoff pipeline for one topic");
            println!("  assemble <topic>            Compatibility alias for build");
            println!("  build-wiki                  Build wiki pages");
            println!("  query <terms...> [--limit N] [--snippets N] [--json] [--title-only]  Search wiki Markdown pages");
            println!("  links [--unresolved|--ambiguous|--resolved] [--json]  Scan wiki WikiLinks and resolution hints");
            println!("  keywords [terms...] [--path PATH] [--json] [--dry-run]  Detect keyword/topic relation candidates");
            println!("  refs [--path PATH] [--citations] [--json]  Scan text for reference and DOI hints");
            println!("  refs-index [--path PATH] [--dry-run|--preview] [--json]  Build bibliographic index relation candidates");
            println!("  refs-graph [--json|--mermaid|--dot] [--dry-run]  Export refs-index relations as graph data");
            println!("  tasks [--dry-run|--preview] [--json] [--index-only|--no-index]  Generate LLM task handoff lists and LLM/tasks/index.md");
            println!(
                "  task list|show|status        List, inspect, and mark Worker/Human task progress"
            );
            println!("  prompt paper-profile|topic-narrative|relation-review|wiki-link-repair|human-review  Print standard Worker/Human prompt templates");
            println!("  batch paper-profile --topic TOPIC --limit 5 [--assignee manager]  Create a small bounded paper-profile Worker batch");
            println!("  handoff [--topic TOPIC|--all-topics] [--agent AGENT|--all-agents] [--force]  Generate generic/agent-adapter handoff files");
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
            println!("  topic relations <topic> [--json] [--dry-run]  Compile directed topic relations into graph artifacts");
            println!("  topic build <topic> [--limit N] [--json] [--dry-run]  Run rank + relations for a topic");
            println!(
                "  topic review <topic> [--json] [--dry-run] [--force]  Build a topic review queue"
            );
            println!(
                "  topic tasks <topic> [--json] [--dry-run] [--limit N]  Generate topic-local LLM handoff tasks"
            );
            println!(
                "  topic handoff <topic> [--agent AGENT|--all-agents] [--json] [--dry-run] [--force]  Generate topic-local generic/agent-adapter handoff files"
            );
            println!("  check [--relations] [--topic TOPIC] [--data-only] [--no-open]  Generate system check dashboard or relationship HTML viewers");
            println!("  view [--no-open] [--dry-run]  Generate research issue browser at interfaces/html/browse.html");
            println!(
                "  review [--dry-run]  Reserved for future literature-review writing workflow"
            );
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
