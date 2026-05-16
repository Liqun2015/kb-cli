# LLM Wiki Root Marker

Current version: `v0.7.35`

`llm-wiki.toml` is the root marker file for a local LLM Wiki workspace. It plays the same practical role that `Cargo.toml` plays for a Rust project or `.git/` plays for a Git repository: it lets humans, deterministic commands, and future LLM agents recognize the managed workspace boundary.

## Location

For the recommended layout:

```bash
kb --source /path/to/literature-folder setup --recursive
```

the marker is created at:

```text
/path/to/literature-folder/knowledgebase/llm-wiki.toml
```

The original literature folder remains outside the managed workspace.

## Example

```toml
[project]
type = "llm-wiki"
name = "knowledgebase"
schema_version = "0.1"
created_by = "kb-cli"
created_with = "kb-cli v0.7.21"

[layout]
raw_dir = "raw"
wiki_dir = "wiki"
rules_dir = "rules"
processing_dir = "processing"
interfaces_dir = "interfaces"
artifacts_dir = "interfaces"
llm_tasks_dir = "LLM/tasks"
llm_memory_dir = "LLM/memory"
topics_dir = "topics"
references_dir = "references"
logs_dir = "interfaces/logs"

[source]
mode = "external"
source_dir = ".."

[policy]
llm_edit_scope = "inside_knowledgebase_only"
allow_edit_raw = false
allow_edit_wiki = true
allow_edit_rules = false
allow_edit_processing = true
allow_edit_interfaces = false
interfaces_are_disposable = true
interfaces_are_source_of_truth = false
```

## Agent boundary rule

Future LLM agents should treat the folder containing `llm-wiki.toml` as the knowledge-base root. When `source.mode = "external"`, files outside this root are source materials and should not be modified unless the human explicitly asks.


## Interface Boundary

`interfaces/` is the centralized directory for interface artifacts. Generated HTML, reports, answers, figures, slides, and logs may be inspected for review context, but they are not the source of truth. Agents must write durable decisions back to Markdown/JSON/TOML files outside `interfaces/`.
