# kb-cli

> A command-line interface between AI agents and the kind of knowledge base as 
> proposed by Andrej Karpathy for LLMs to maintain inside Obsidian.

## For AI Agents

`kb-cli` provides a standardized CLI interface for AI agents to access and manage Karpathy's bases(kb) of knowledge .

### Quick Start

```bash
# Initialize a Karpathy knowledge base
kb --kb-path /path/to/kb init

# Add a paper
kb --kb-path /path/to/kb add-paper /path/to/paper.pdf

# Search content
kb --kb-path /path/to/kb search "droplet"
```

### Command Reference

See [Agent API Documentation](docs/agent-api.md) for detailed usage.

### Integration Examples

```python
import subprocess

class KnowledgeAgent:
    def __init__(self, kb_path):
        self.kb_path = kb_path

    def search_papers(self, query):
        result = subprocess.run(
            ["kb", "--kb-path", self.kb_path, "search", query],
            capture_output=True
        )
        return self._parse_json_output(result.stdout)
```

---

## For Researchers

`kb-cli` is also a powerful tool for personal knowledge management.

### Features

- 📄 **Paper Management**: Auto-extract PDF metadata, build literature index
- 📝 **Wiki Generation**: Markdown format, Obsidian compatible
- 💬 **Interactive REPL**: Conversational interface for quick tasks
- 🤖 **LLM Integration**: Multi-model support for intelligent understanding

### Workflow

```bash
# Initialize
kb init

# Extract metadata from PDFs
kb extract-metadata

# Build wiki pages
kb build-wiki

# Interactive mode
kb repl
```

### Obsidian Integration

Wiki pages are generated in Markdown format with:
- [[WikiLinks]] for internal references
- Frontmatter for metadata
- Folder structure: papers/, notes/, concepts/, indexes/

Open your knowledge base folder in Obsidian to start organizing.

---

## Installation

```bash
# Clone
git clone https://github.com/liqun2015/kb-cli.git
cd kb-cli

# Build
cargo build --release

# Install
cargo install --path .
```

---

## Documentation

- [Agent API](docs/agent-api.md) - For AI agents
- [Architecture](docs/architecture.md) - Technical details
- [Contributing](CONTRIBUTING.md) - How to contribute

---

## License

MIT License - see [LICENSE](LICENSE) file.
