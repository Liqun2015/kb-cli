# v0.7.1 Release Checklist

Release name: Topic Review Command Skeleton

## Required checks

```bash
cargo fmt --check
cargo test
cargo check
cargo build --release
```

## Manual behavior checks

After implementation, test with a temporary knowledge base:

```bash
kb --wiki /tmp/kb init
kb --wiki /tmp/kb topic init thermal-metamaterials
mkdir -p /tmp/kb/topics/thermal-metamaterials/importance
cp examples/topic-review/importance_candidate_example.md /tmp/kb/topics/thermal-metamaterials/importance/
kb --wiki /tmp/kb topic review thermal-metamaterials
```

Expected outputs:

```text
/tmp/kb/topics/thermal-metamaterials/review/review_queue.md
/tmp/kb/topics/thermal-metamaterials/review/review_summary.md
```

## Non-goals

Do not add:

- accept/reject/defer subcommands
- LLM calls
- vector search
- automatic scientific judgment
- automatic writes to `reviewed/`
- automatic writes to `LLM/memory/`
