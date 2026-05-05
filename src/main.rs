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
    #[command(about = "Initialize KnowledgeBase directory structure")]
    Init {
        #[arg(long, help = "Re-create missing directories and overwrite generated bootstrap files")]
        force: bool,
    },
    #[command(about = "Build Wiki from raw materials")]
    BuildWiki,
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
        Some(Commands::BuildWiki) => commands::build_wiki::execute(cli.kb_path.as_deref()),
        Some(Commands::ExtractMetadata { force }) => commands::extract_metadata::execute(cli.kb_path.as_deref(), *force),
        Some(Commands::Repl) => commands::repl::execute(),
        Some(Commands::ListModels) => commands::model::execute_list_models(),
        Some(Commands::ShowModel) => commands::model::execute_show_model(),
        Some(Commands::AddModel(args)) => commands::model::execute_add_model(args.clone()),
        Some(Commands::SwitchModel { id }) => commands::model::execute_switch_model(&id),
        Some(Commands::DeleteModel { id }) => commands::model::execute_delete_model(&id),
        Some(Commands::ValidateModel { id }) => commands::model::execute_validate_model(id.as_deref()),
        None => {
            println!("Available commands:");
            println!("  init [--force]          Initialize knowledge base");
            println!("  extract-metadata [--force] Extract PDF metadata");
            println!("  build-wiki             Build wiki pages");
            println!("  list-models            List all configured models");
            println!("  show-model              Show current model");
            println!("  add-model               Add a new model");
            println!("  switch-model <id>       Switch to a different model");
            println!("  delete-model <id>       Delete a model");
            println!("  validate-model [id]      Validate model configuration");
            println!("  repl                    Enter interactive REPL mode");
            Ok(())
        }
    }
}
