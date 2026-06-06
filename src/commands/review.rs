use anyhow::Result;
use clap::Args;
use std::path::Path;

use crate::commands::init::get_kb_path;

#[derive(Debug, Clone, Args)]
pub struct ReviewArgs {
    #[arg(
        long,
        help = "Preview the future review-writing workflow entry without writing files"
    )]
    pub dry_run: bool,

    #[arg(long, help = "Alias for --dry-run")]
    pub preview: bool,
}

pub fn execute(custom_kb: Option<&Path>, _args: &ReviewArgs) -> Result<()> {
    let kb_path = get_kb_path(custom_kb);
    println!("kb review is reserved for the future literature-review writing workflow.");
    println!("Current LLM Wiki workspace: {}", kb_path.display());
    println!("No files were written. Use `kb view` to browse knowledge and `kb check` to inspect system status for now.");
    Ok(())
}
