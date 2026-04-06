# Agent API Documentation

This document describes how AI agents can interact with `kb-cli` for knowledge base management.

## Overview

`kb-cli` follows a consistent command pattern:
```
kb [OPTIONS] <COMMAND> [ARGS]
```

### Important Options

| Option | Description | Example |
|---------|-------------|---------|
| `--kb-path <path>` | Specify knowledge base directory (required for agents) | `kb --kb-path /user/kb init` |
| `--format json` | Output in JSON format (for agent parsing) | `kb --format json search "query"` |
| `--model <id>` | Use specific LLM model | `kb --model deepseek search "query"` |

### Commands

#### Initialization

```bash
kb --kb-path /path/to/kb init
```

**Output (JSON format)**:
```json
{
  "status": "success",
  "message": "Knowledge base initialized at /path/to/kb",
  "created_dirs": ["raw", "wiki", "logs", "outputs"]
}
```

#### Paper Operations

**Add a paper**:
```bash
kb --kb-path /path/to/kb add-paper /path/to/paper.pdf
```

**Extract metadata**:
```bash
kb --kb-path /path/to/kb extract-metadata
```

**Output (JSON format)**:
```json
{
  "status": "success",
  "papers_processed": 15,
  "papers": [
    {"id": "paper001", "title": "...", "authors": [...]}
  ]
}
```

#### Search Operations

```bash
kb --kb-path /path/to/kb search "droplet microfluidics"
```

**Output (JSON format)**:
```json
{
  "status": "success",
  "results": [
    {"type": "paper", "id": "paper001", "title": "...", "relevance": 0.95},
    {"type": "note", "id": "note001", "title": "...", "relevance": 0.88}
  ],
  "total": 25
}
```

#### List Operations

```bash
kb --kb-path /path/to/kb list-papers
kb --kb-path /path/to/kb list-notes
```

**Output (JSON format)**:
```json
{
  "status": "success",
  "items": [
    {"id": "paper001", "title": "...", "created_at": "..."},
    {"id": "paper002", "title": "...", "created_at": "..."}
  ]
}
```

### Output Formats

All commands with `--format json` output structured JSON:

```json
{
  "status": "success" | "error",
  "data": {...},
  "message": "...",
  "error": {...}
}
```

This format is designed for:
- Easy parsing by agent scripts
- Machine-readable metadata
- Consistent structure across all commands

### Error Handling

All errors follow this format:

```json
{
  "status": "error",
  "error": {
    "code": "KB_NOT_FOUND",
    "message": "Knowledge base not found at specified path",
    "hint": "Run 'kb init' to create a new knowledge base"
  }
}
```

### Error Codes

| Code | Description | Recovery |
|-------|-------------|------------|
| `KB_NOT_FOUND` | Knowledge base directory not found | Run `kb init` |
| `INVALID_PATH` | Path does not exist | Check path and try again |
| `PAPER_NOT_FOUND` | Paper ID not found | Search again or check ID |
| `MODEL_ERROR` | LLM model error | Check model configuration |

### LLM Integration

Agents can leverage LLM capabilities:

```bash
# Search with LLM analysis
kb --kb-path /path/to/kb --use-llm search "droplet formation mechanisms"

# Output includes LLM-generated summary
{
  "status": "success",
  "results": [...],
  "llm_summary": "Based on 5 papers, droplet formation involves...",
  "llm_confidence": 0.87
}
```

### Memory Pattern

Agents should cache command results:

```python
class KnowledgeAgent:
    def __init__(self):
        self.cache = {}  # Cache paper IDs, metadata
        self.last_sync = None

    def get_papers(self, force_refresh=False):
        if not force_refresh and "papers" in self.cache:
            return self.cache["papers"]

        # Fetch fresh data
        result = subprocess.run(["kb", "--format", "json", "list-papers"])
        self.cache["papers"] = json.loads(result.stdout)
        return self.cache["papers"]
```
