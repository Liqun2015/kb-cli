# `kb audit-wiki`

`kb audit-wiki` is a deterministic proof-audit command for the central claim of this project:

> `kb-cli` can turn a pile of source materials into a reviewable, human/AI-maintainable research Markdown knowledge base.

The command does not prove that every scientific claim in the Wiki is true. Instead, it checks whether the local folder contains the reviewable layers needed for a serious human/AI maintenance workflow.

## Command

```bash
kb audit-wiki
kb audit-wiki --json
kb audit-wiki --dry-run
kb audit-wiki --strict --min-score 70
kb audit-wiki --no-llm-prompt
```

## What it checks

The audit checks the transformation chain from source pile to maintainable LLM Wiki:

| Layer | What is checked |
|---|---|
| `raw/` | Whether original source materials exist. |
| `rules/` | Whether the AI/human maintenance contract exists. |
| `processing/manifest.json` | Whether source files are registered. |
| `processing/text/`, `processing/proposals/`, `processing/keywords/` | Whether deterministic intermediate artifacts exist. |
| `wiki/` | Whether Markdown knowledge pages exist. |
| Wiki traceability | Whether wiki pages contain `source_ids` or `source_files`. |
| `processing/refs/` | Whether reference/index/graph artifacts exist. |
| `topics/` | Whether topic workspaces, importance candidates, and review queues exist. |
| `LLM/tasks/` and `LLM/memory/` | Whether LLM work is explicit and auditable. |
| `interfaces/` | Whether lint, health, and viewer outputs exist. |
| `topics/*/review/review_queue.md` | Whether review queue format is actually usable. |

## Interfaces

By default, the command writes two files:

```text
interfaces/reports/wiki_audit_<timestamp>.md
LLM/tasks/wiki_audit_llm_review_<timestamp>.md
```

The first file is the deterministic audit report. The second file is an explicit LLM review prompt that can be given to a Manager LLM for independent evaluation.

Use `--no-llm-prompt` when you only want the deterministic report.

## LLM involvement

`kb audit-wiki` does not hide a model call inside the command. Instead, it creates a bounded LLM audit task under `LLM/tasks/`.

This keeps the project boundary clean:

```text
Rust command performs deterministic checks
        ↓
Rust command writes an LLM review prompt
        ↓
Manager LLM independently reviews the proof chain
        ↓
human accepts or rejects the conclusion under Git review
```

The LLM prompt asks the model to evaluate whether the KB is reviewable and maintainable, not whether every scientific claim is correct.

## Review queue format audit

The command also checks whether each topic review queue has the fields needed for real review work:

```text
review_id
candidate_id
source_item
proposed_decision
final_decision
status
reviewer
confidence
evidence
uncertainty
review_notes
accepted_record_path
```

A good review queue should not merely list candidates. It should also give human/LLM reviewers a place to record final decisions, confidence, uncertainty, notes, and the path where an accepted record should be stored.

## Score interpretation

| Score | Interpretation |
|---:|---|
| `85-100` | Strong evidence that the folder has become a reviewable LLM Wiki. |
| `70-84` | Sufficient evidence, but some workflow links may still be weak. |
| `45-69` | Partial evidence; the folder has some KB structure but is not yet a convincing LLM Wiki. |
| `<45` | Insufficient evidence; it is still closer to a material pile than a maintainable Wiki. |

## Strict mode

```bash
kb audit-wiki --strict --min-score 70
```

Strict mode returns a non-zero error when the score is below the threshold. This is useful before a release or before asking an LLM agent to maintain the Wiki.

## Boundary

`kb audit-wiki` must not:

- call an LLM implicitly;
- claim scientific correctness;
- rewrite wiki pages;
- accept or reject topic candidates;
- modify topic decisions;
- hide uncertainty.

It only checks whether the workflow is reviewable, traceable, and ready for human/AI collaboration.
