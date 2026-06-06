use anyhow::{anyhow, Result};
use clap::Args;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::commands::init::get_kb_path;

#[derive(Debug, Clone, Args)]
pub struct ViewArgs {
    #[arg(
        long = "output-dir",
        value_name = "DIR",
        help = "Output directory relative to the knowledge base. Defaults to interfaces/html/."
    )]
    pub output_dir: Option<PathBuf>,

    #[arg(
        long,
        help = "Preview the generated user-facing knowledge portal path without writing HTML"
    )]
    pub dry_run: bool,

    #[arg(long, help = "Alias for --dry-run")]
    pub preview: bool,

    #[arg(
        long = "no-open",
        help = "Generate the knowledge portal without opening the browser"
    )]
    pub no_open: bool,

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
        .unwrap_or_else(|| PathBuf::from("interfaces/html"));
    let output_root = resolve_under_kb(&kb_path, &output_dir);
    let output_path = output_root.join("browse.html");
    let page_count = crate::commands::check::count_user_knowledge_pages(&kb_path)?;

    if args.is_dry_run() {
        println!("kb view preview:");
        println!("  knowledge base : {}", kb_path.display());
        println!("  output         : {}", output_path.display());
        println!("  wiki pages     : {}", page_count);
        println!("  open browser   : {}", args.should_open());
        println!("  no files written");
        return Ok(());
    }

    fs::create_dir_all(&output_root)?;
    let html = crate::commands::check::render_user_knowledge_view(&kb_path, &output_root)?;
    fs::write(&output_path, html)?;

    println!("User-facing LLM Wiki knowledge portal generated:");
    println!("  {}", output_path.display());
    println!("Pages rendered from wiki/: {}", page_count);
    println!("This page hides low-level task status, JSON, and agent handoff details.");

    if args.should_open() {
        open_in_default_browser(&output_path)?;
        println!("Opened in the system default browser.");
    } else {
        println!("HTML generated without opening a browser because --no-open was set.");
        println!("Open this file manually in a browser when needed.");
    }

    Ok(())
}

fn resolve_under_kb(kb_path: &Path, output_dir: &Path) -> PathBuf {
    if output_dir.is_absolute() {
        output_dir.to_path_buf()
    } else {
        kb_path.join(output_dir)
    }
}

fn open_in_default_browser(path: &Path) -> Result<()> {
    let target = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let target_str = target.to_string_lossy().to_string();

    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(["/C", "start", "", &target_str])
            .status()
            .map_err(|err| anyhow!("failed to open browser: {err}"))?;
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(&target_str)
            .status()
            .map_err(|err| anyhow!("failed to open browser: {err}"))?;
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        Command::new("xdg-open")
            .arg(&target_str)
            .status()
            .map_err(|err| anyhow!("failed to open browser: {err}"))?;
    }

    Ok(())
}
