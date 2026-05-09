# Relation Graph Examples

Current version: `v0.5.10.2`

These examples show how third-party skills and tools should display bibliographic index relations.

## 1. Confirmed DOI relation

Input situation:

```text
Paper A reference entry contains DOI 10.1234/example.
Local Paper B metadata contains DOI 10.1234/example.
```

Graph rendering:

```text
Paper A ━━━━━ Paper B
```

Status:

```text
status = confirmed
edge_style = solid
needs_human_review = false
```

## 2. Candidate title/author/year relation

Input situation:

```text
Paper A reference entry has a title and year similar to local Paper B.
No exact DOI match is available.
```

Graph rendering:

```text
Paper A - - - Paper B
```

Status:

```text
status = candidate
edge_style = dashed
needs_human_review = true
```

## 3. Ambiguous relation

Input situation:

```text
Paper A reference entry may match Paper B1 or Paper B2.
The deterministic scanner cannot choose one safely.
```

Graph rendering:

```text
Paper A - - - Paper B1
Paper A - - - Paper B2
```

Status:

```text
status = ambiguous
edge_style = dashed
needs_human_review = true
```

## 4. Missing local paper

Input situation:

```text
Paper A contains a reference entry, but no local paper appears to match it.
```

Graph rendering:

```text
Paper A - - - ○ Unresolved Reference 001
```

Status:

```text
status = missing
edge_style = dashed
node_style = hollow
needs_human_review = true
```

## 5. Keyword or scientific idea relation

Keyword and scientific idea relations should not reuse bibliographic certainty styles without explanation.

A topic relation can use a separate relation type:

```json
{
  "source": "paper_a",
  "target": "paper_c",
  "relation_type": "keyword_topic",
  "status": "candidate",
  "evidence": ["both mention transformation thermodynamics"],
  "needs_human_review": true
}
```

A scientific idea relation should normally be created by a Manager LLM / Worker LLM workflow and should keep source evidence.
