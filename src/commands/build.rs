use std::path::Path;

use anyhow::{anyhow, Result};
use clap::Args;

use crate::commands;

#[derive(Debug, Clone, Args)]
pub struct BuildArgs {
    #[arg(
        value_name = "TOPIC",
        help = "Topic name or slug to build into an agent-ready LLM Wiki workspace"
    )]
    pub topic: String,

    #[arg(
        long,
        value_name = "TITLE",
        help = "Human-readable topic title when a missing topic workspace is initialized"
    )]
    pub title: Option<String>,

    #[arg(
        long,
        default_value_t = 50,
        help = "Maximum number of topic-local importance candidates to include. Use 0 for no limit."
    )]
    pub limit: usize,

    #[arg(
        long,
        help = "Overwrite generated extraction, review, and handoff files where supported"
    )]
    pub force: bool,

    #[arg(
        long,
        help = "Preview the build pipeline without writing files where supported"
    )]
    pub dry_run: bool,

    #[arg(long, help = "Alias for --dry-run")]
    pub preview: bool,

    #[arg(long, help = "Skip extract-text and extract-sections stages")]
    pub skip_extract: bool,

    #[arg(long, help = "Skip refs, refs-index, and refs-graph stages")]
    pub skip_refs: bool,

    #[arg(long, help = "Skip project/topic handoff generation")]
    pub skip_handoff: bool,

    #[arg(long, help = "Skip topic review queue generation")]
    pub skip_review: bool,

    #[arg(long, help = "Skip topic task generation")]
    pub skip_tasks: bool,

    #[arg(
        long,
        help = "Do not initialize a missing topic workspace before running topic build"
    )]
    pub no_init: bool,
}

#[derive(Debug, Clone)]
struct Stage {
    name: &'static str,
    command: String,
}

pub fn execute(custom_kb: Option<&Path>, args: &BuildArgs) -> Result<()> {
    let kb_path = commands::init::get_kb_path(custom_kb);
    if !kb_path.exists() {
        return Err(anyhow!(
            "knowledge base path does not exist: {}. Run `kb --kb-path <database-name> init` or `kb --source <literature-dir> setup` first.",
            kb_path.display()
        ));
    }

    let topic = commands::topic::slugify_topic_name(&args.topic);
    if topic.is_empty() {
        return Err(anyhow!(
            "topic name must contain at least one letter or digit"
        ));
    }

    let dry_run = args.dry_run || args.preview;
    let stages = planned_stages(args, &topic, dry_run);

    println!("LLM Wiki build pipeline");
    println!("  kb_path: {}", kb_path.display());
    println!("  topic: {}", topic);
    println!("  dry_run: {}", dry_run);
    println!("  stages:");
    for (idx, stage) in stages.iter().enumerate() {
        println!("    {}. {}  ({})", idx + 1, stage.name, stage.command);
    }
    println!();

    if !args.skip_extract {
        run_stage("extract-text", || {
            let stage_args = commands::extract_text::ExtractTextArgs {
                path: None,
                output_dir: None,
                dry_run,
                preview: false,
                force: args.force,
                limit: 0,
                json: false,
            };
            commands::extract_text::execute(custom_kb, &stage_args)
        })?;

        run_stage("extract-sections", || {
            let stage_args = commands::extract_sections::ExtractSectionsArgs {
                path: None,
                output_dir: None,
                dry_run,
                preview: false,
                force: args.force,
                limit: 0,
                no_llm_task: false,
                json: false,
            };
            commands::extract_sections::execute(custom_kb, &stage_args)
        })?;
    }

    if !args.skip_refs {
        run_stage("refs", || {
            let stage_args = commands::refs::RefsArgs {
                path: None,
                limit: 200,
                citations: false,
                json: false,
            };
            commands::refs::execute(custom_kb, &stage_args)
        })?;

        run_stage("refs-index", || {
            let stage_args = commands::refs_index::RefsIndexArgs {
                path: None,
                limit: 100,
                dry_run,
                preview: false,
                json: false,
            };
            commands::refs_index::execute(custom_kb, &stage_args)
        })?;

        run_stage("refs-graph", || {
            let stage_args = commands::refs_graph::RefsGraphArgs {
                path: None,
                output_dir: None,
                limit: 0,
                dry_run,
                preview: false,
                json: false,
                mermaid: false,
                dot: false,
            };
            commands::refs_graph::execute(custom_kb, &stage_args)
        })?;
    }

    run_stage("topic build", || {
        let stage_args = commands::topic::TopicArgs {
            command: commands::topic::TopicCommand::Build(commands::topic::TopicBuildArgs {
                topic: topic.clone(),
                title: args.title.clone(),
                limit: args.limit,
                no_init: args.no_init,
                dry_run,
                preview: false,
                json: false,
            }),
        };
        commands::topic::execute(custom_kb, &stage_args)
    })?;

    if !args.skip_review {
        run_stage("topic review", || {
            let stage_args = commands::topic::TopicArgs {
                command: commands::topic::TopicCommand::Review(commands::topic::TopicReviewArgs {
                    topic: topic.clone(),
                    dry_run,
                    preview: false,
                    json: false,
                    force: args.force,
                }),
            };
            commands::topic::execute(custom_kb, &stage_args)
        })?;
    }

    if !args.skip_tasks {
        run_stage("topic tasks", || {
            let stage_args = commands::topic::TopicArgs {
                command: commands::topic::TopicCommand::Tasks(commands::topic::TopicTasksArgs {
                    topic: topic.clone(),
                    dry_run,
                    preview: false,
                    limit: 0,
                    json: false,
                }),
            };
            commands::topic::execute(custom_kb, &stage_args)
        })?;
    }

    if !args.skip_handoff {
        run_stage("handoff --all-agents", || {
            let stage_args = commands::handoff::HandoffArgs {
                topic: None,
                all_topics: false,
                agent: commands::handoff::AgentTarget::Generic,
                all_agents: true,
                force: args.force,
                dry_run,
                preview: false,
                json: false,
            };
            commands::handoff::execute(custom_kb, &stage_args)
        })?;

        run_stage("topic handoff --all-agents", || {
            commands::handoff::execute_topic_handoff(
                custom_kb,
                &topic,
                args.force,
                dry_run,
                false,
                commands::handoff::AgentTarget::Generic,
                true,
            )
        })?;
    }

    println!("\nBuild pipeline complete.");
    println!("Next human entry: obsidian_main.md");
    println!("Next agent entry: AGENTS.md / CLAUDE.md -> LLM/handoff/current.md");
    println!("Topic workbench: topics/{}/index.md", topic);
    Ok(())
}

fn run_stage<F>(name: &str, mut f: F) -> Result<()>
where
    F: FnMut() -> Result<()>,
{
    println!("\n=== {} ===", name);
    f()
}

fn planned_stages(args: &BuildArgs, topic: &str, dry_run: bool) -> Vec<Stage> {
    let mut stages = Vec::new();
    let flag = if dry_run { " --dry-run" } else { "" };
    let force = if args.force { " --force" } else { "" };

    if !args.skip_extract {
        stages.push(Stage {
            name: "extract text",
            command: format!("kb extract-text{}{}", flag, force),
        });
        stages.push(Stage {
            name: "extract sections",
            command: format!("kb extract-sections{}{}", flag, force),
        });
    }

    if !args.skip_refs {
        stages.push(Stage {
            name: "reference scan",
            command: "kb refs".to_string(),
        });
        stages.push(Stage {
            name: "reference index",
            command: format!("kb refs-index{}", flag),
        });
        stages.push(Stage {
            name: "reference graph",
            command: format!("kb refs-graph{}", flag),
        });
    }

    stages.push(Stage {
        name: "topic build",
        command: format!("kb topic build {}{}", topic, flag),
    });

    if !args.skip_review {
        stages.push(Stage {
            name: "topic review",
            command: format!("kb topic review {}{}{}", topic, flag, force),
        });
    }

    if !args.skip_tasks {
        stages.push(Stage {
            name: "topic tasks",
            command: format!("kb topic tasks {}{}", topic, flag),
        });
    }

    if !args.skip_handoff {
        stages.push(Stage {
            name: "project handoff",
            command: format!("kb handoff --all-agents{}{}", flag, force),
        });
        stages.push(Stage {
            name: "topic handoff",
            command: format!("kb topic handoff {} --all-agents{}{}", topic, flag, force),
        });
    }

    stages
}
