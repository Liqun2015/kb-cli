use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod cli;
mod commands;
mod intent;

#[derive(Parser)]
#[command(name = "kb")]
#[command(about = "A minimal CLI for local knowledge base management", long_about = None)]
struct Cli {
    #[arg(short = 'k', long = "kb-path", value_name = "PATH", global = true, help = "Knowledge base directory name or path (default: KnowledgeBase)")]
    kb_path: Option<PathBuf>,
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Initialize or ensure KnowledgeBase directory structure")]
    Init {
        #[arg(long, help = "Re-create missing directories and generated bootstrap files when safe")]
        force: bool,
    },
    #[command(about = "Organize source files into raw/ subfolders")]
    Ingest(commands::ingest::IngestArgs),
    #[command(about = "Initialize, ingest, extract metadata, and build wiki")]
    Bootstrap(commands::ingest::BootstrapArgs),
    #[command(about = "Build Wiki from raw materials")]
    BuildWiki,
    #[command(name = "prepare", alias = "compile", about = "Prepare LLM/wiki handoff artifacts from raw files without calling an LLM")]
    Prepare(commands::prepare::PrepareArgs),
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
    #[command(about = "Enter REPL mode for interactive knowledge base management")]
    Repl,
    #[command(about = "List all configured models")]
    ListModels,
    #[command(about = "Show current model configuration")]
    ShowModel,
    #[command(about = "Add a new model configuration")]
    AddModel(commands::model::ModelArgs),
    #[command(about = "Switch to a different model")]
    SwitchModel {
        id: String,
    },
    #[command(about = "Delete a model configuration")]
    DeleteModel {
        id: String,
    },
    #[command(about = "Validate model configuration")]
    ValidateModel {
        id: Option<String>,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Init { force }) => commands::init::execute(cli.kb_path.as_deref(), *force),
        Some(Commands::Ingest(args)) => commands::ingest::execute(cli.kb_path.as_deref(), args),
        Some(Commands::Bootstrap(args)) => commands::ingest::execute_bootstrap(cli.kb_path.as_deref(), args),
        Some(Commands::BuildWiki) => commands::build_wiki::execute(cli.kb_path.as_deref()),
        Some(Commands::Prepare(args)) => commands::prepare::execute(cli.kb_path.as_deref(), args),
        Some(Commands::SyncWiki(args)) => commands::sync_wiki::execute(cli.kb_path.as_deref(), args),
        Some(Commands::LintStatic(args)) => commands::lint_static::execute(cli.kb_path.as_deref(), args),
        Some(Commands::Status(args)) => commands::manifest::execute(cli.kb_path.as_deref(), args),
        Some(Commands::ExtractMetadata { force }) => commands::extract_metadata::execute(cli.kb_path.as_deref(), *force),
        Some(Commands::Repl) => commands::repl::execute(),
        Some(Commands::ListModels) => commands::model::execute_list_models(),
        Some(Commands::ShowModel) => commands::model::execute_show_model(),
        Some(Commands::AddModel(args)) => commands::model::execute_add_model(args.clone()),
        Some(Commands::SwitchModel { id }) => commands::model::execute_switch_model(id),
        Some(Commands::DeleteModel { id }) => commands::model::execute_delete_model(id),
        Some(Commands::ValidateModel { id }) => commands::model::execute_validate_model(id.as_deref()),
        None => {
            println!("Available commands:");
            println!("  init [--force]              Initialize/ensure knowledge base");
            println!("  ingest [--copy|--move]      Organize files into raw/ subfolders");
            println!("  bootstrap [--copy|--move]   Initialize, ingest, extract metadata, build wiki");
            println!("  extract-metadata [--force]  Extract PDF metadata");
            println!("  build-wiki                  Build wiki pages");
            println!("  prepare [--new|--file PATH] [--dry-run|--preview]  Prepare LLM/wiki handoff artifacts");
            println!("  status [--dry-run|--preview] [--json] [--unprocessed]  Refresh manifest and show raw-file status");
            println!("  sync-wiki [--dry-run|--preview] [--json]  Link wiki front matter back to manifest");
            println!("  lint-static [--dry-run|--preview|--no-report] [--json] [--strict]  Check broken links, orphans, and source front matter");
            println!("  list-models                 List all configured models");
            println!("  show-model                  Show current model");
            println!("  add-model                   Add a new model");
            println!("  switch-model <id>           Switch to a different model");
            println!("  delete-model <id>           Delete a model");
            println!("  validate-model [id]         Validate model configuration");
            println!("  repl                        Enter interactive REPL mode");
            Ok(())
        }
    }
}
