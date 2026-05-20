use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use clap::Args;

use crate::commands;

#[derive(Debug, Clone, Args)]
pub struct CreateArgs {
    #[arg(
        long = "from",
        value_name = "B",
        help = "Existing literature/source-material folder to convert into the LLM Wiki."
    )]
    pub from: Option<PathBuf>,

    #[arg(
        long = "about",
        alias = "topic",
        value_name = "TOPIC",
        help = "Topic workspace to create and build. Alias: --topic."
    )]
    pub about: Option<String>,

    #[arg(
        value_name = "LITERATURE_FOLDER",
        hide = true,
        help = "Deprecated compatibility form. Use --from instead."
    )]
    pub legacy_source: Option<PathBuf>,

    #[arg(
        long = "name",
        hide = true,
        help = "Deprecated compatibility option from v0.7.21. Use --wiki instead."
    )]
    pub legacy_name: Option<String>,

    #[arg(
        long = "move",
        help = "Move source files into raw/ instead of copying them during setup. Default is copy."
    )]
    pub move_files: bool,

    #[arg(
        long,
        help = "Overwrite generated files where the setup/build stages support it."
    )]
    pub force: bool,

    #[arg(long, help = "Preview the create pipeline without writing files.")]
    pub dry_run: bool,

    #[arg(long, help = "Alias for --dry-run.")]
    pub preview: bool,

    #[arg(
        long,
        default_value_t = 50,
        help = "Maximum number of topic-local importance candidates during build. Use 0 for no limit."
    )]
    pub limit: usize,
}

pub fn execute(
    custom_kb: Option<&Path>,
    custom_source: Option<&Path>,
    args: &CreateArgs,
) -> Result<()> {
    let kb_path = resolve_create_lib(
        custom_kb,
        args.legacy_name.as_deref(),
        args.from.as_deref(),
        custom_source,
        args.legacy_source.as_deref(),
    )?;
    let source_path = resolve_create_from(
        args.from.as_deref(),
        custom_source,
        args.legacy_source.as_deref(),
    )?;
    if !source_path.is_dir() {
        return Err(anyhow!(
            "literature/source folder is not a directory: {}",
            source_path.display()
        ));
    }

    let topic = resolve_create_about(args.about.as_deref())?;
    let topic_slug = commands::topic::slugify_topic_name(&topic);
    if topic_slug.is_empty() {
        return Err(anyhow!(
            "topic name must contain at least one letter or digit"
        ));
    }

    let dry_run = args.dry_run || args.preview;

    println!("LLM Wiki create pipeline");
    println!("  wiki: {}", kb_path.display());
    println!("  from: {}", source_path.display());
    println!("  about: {}", topic);
    println!("  recursive: true");
    println!(
        "  ingest mode: {}",
        if args.move_files { "move" } else { "copy" }
    );
    println!("  dry_run: {}", dry_run);
    println!();
    println!("Equivalent expanded commands:");
    println!(
        "  kb --wiki \"{}\" --source \"{}\" setup --recursive --topic \"{}\"{}{}",
        kb_path.display(),
        source_path.display(),
        topic,
        if args.move_files { " --move" } else { "" },
        if args.force {
            " --force-init --force-metadata"
        } else {
            ""
        },
    );
    println!(
        "  kb --wiki \"{}\" build \"{}\"{}{}",
        kb_path.display(),
        topic_slug,
        if args.force { " --force" } else { "" },
        if dry_run { " --dry-run" } else { "" },
    );

    if dry_run {
        println!("\nDry run: no setup/build stages were executed.");
        return Ok(());
    }

    println!("\n=== setup --recursive ===");
    let setup_args = commands::ingest::SetupArgs {
        ingest: commands::ingest::IngestArgs {
            copy: !args.move_files,
            move_files: args.move_files,
            recursive: true,
            dry_run: false,
            preview: false,
        },
        name: "knowledgebase".to_string(),
        topic: Some(topic.clone()),
        force_init: args.force,
        force_metadata: args.force,
        no_ingest: false,
        skip_metadata: false,
        skip_build: false,
    };
    commands::ingest::execute_setup(Some(&kb_path), Some(&source_path), &setup_args)?;

    println!("\n=== build ===");
    let build_args = commands::build::BuildArgs {
        topic: topic_slug.clone(),
        title: Some(topic.clone()),
        limit: args.limit,
        force: args.force,
        dry_run: false,
        preview: false,
        skip_extract: false,
        skip_wiki: false,
        skip_refs: false,
        skip_handoff: false,
        skip_review: false,
        skip_tasks: false,
        no_init: false,
    };
    commands::build::execute(Some(&kb_path), &build_args)?;

    println!("\n=== workflow guides ===");
    let workflow_status = commands::workflow::refresh_workflow_guides(
        &kb_path,
        Some(&topic_slug),
        Some(&topic),
        args.force,
    )?;
    println!(
        "Workflow mode: {} ({} papers; threshold {})",
        workflow_status.mode_label, workflow_status.paper_count, workflow_status.rag_threshold
    );
    for guide in &workflow_status.generated_guides {
        println!("  guide: {}", guide);
    }

    println!("\nCreate pipeline complete.");
    println!("LLM Wiki workspace: {}", kb_path.display());
    println!(
        "Obsidian human entry: {}",
        kb_path.join("obsidian_main.md").display()
    );
    println!("Agent entry: {}", kb_path.join("AGENTS.md").display());
    println!(
        "Topic workbench: {}",
        kb_path
            .join("topics")
            .join(topic_slug)
            .join("index.md")
            .display()
    );
    Ok(())
}

fn resolve_create_lib(
    global_kb: Option<&Path>,
    legacy_name: Option<&str>,
    arg_from: Option<&Path>,
    global_source: Option<&Path>,
    legacy_source: Option<&Path>,
) -> Result<PathBuf> {
    if let Some(global_kb) = global_kb {
        return Ok(commands::init::resolve_user_path(global_kb));
    }

    // v0.7.21 compatibility: `kb create <literature-folder> --topic <topic>`
    // created `<literature-folder>/knowledgebase` unless a custom workspace path was supplied.
    let source_for_legacy = arg_from.or(global_source).or(legacy_source);
    if let Some(source) = source_for_legacy {
        let source = commands::init::resolve_user_path(source);
        let name = legacy_name
            .unwrap_or(commands::init::DEFAULT_WORKSPACE_DIR)
            .trim();
        if name.is_empty() {
            return Err(anyhow!("legacy workspace name cannot be empty"));
        }
        return Ok(source.join(name));
    }

    Err(anyhow!(
        "kb create requires a target LLM Wiki workspace. Use `kb create --wiki <A> --from <B> --about <topic>`."
    ))
}

fn resolve_create_from(
    arg_from: Option<&Path>,
    global_source: Option<&Path>,
    legacy_source: Option<&Path>,
) -> Result<PathBuf> {
    let mut selected: Option<PathBuf> = None;
    for (label, value) in [
        ("--from", arg_from),
        ("global --source", global_source),
        ("legacy positional source", legacy_source),
    ] {
        if let Some(path) = value {
            let path = commands::init::resolve_user_path(path);
            if let Some(existing) = &selected {
                if existing != &path {
                    return Err(anyhow!(
                        "conflicting create source paths: {}={} but earlier source={}",
                        label,
                        path.display(),
                        existing.display()
                    ));
                }
            } else {
                selected = Some(path);
            }
        }
    }

    selected.ok_or_else(|| {
        anyhow!("kb create requires a source literature folder. Use `kb create --wiki <A> --from <B> --about <topic>`.")
    })
}

fn resolve_create_about(explicit_about: Option<&str>) -> Result<String> {
    if let Some(topic) = explicit_about {
        let topic = topic.trim();
        if !topic.is_empty() {
            return Ok(topic.to_string());
        }
    }

    Err(anyhow!(
        "kb create requires a topic. Use `kb create --wiki <A> --from <B> --about <topic>`."
    ))
}
