# `kb links`

`kb links` is a Rust-native deterministic WikiLink scanner.

It scans Markdown pages, extracts Obsidian-style links such as `[[Target]]`, `[[Target|Alias]]`, and `[[Target#Heading|Alias]]`, and tries to resolve each target against pages under `wiki/`.

It does not call an LLM. It does not rewrite pages. It does not create missing pages.

## Purpose

Wiki links are the nervous system of the local knowledge base. `kb links` is the first scouting pass for link structure:

```text
find WikiLinks
resolve what can be resolved deterministically
mark unresolved and ambiguous links
return task hints for future Wiki Link Repair Agent work
```

## Usage

```bash
kb links
kb links --unresolved
kb links --ambiguous
kb links --resolved
kb links --limit 20
kb links --json
```

With an explicit knowledge-base path:

```bash
kb --lib ./quantum links
kb --lib "D:\\github\\llm-wiki\\quantum" links --unresolved
```

## Options

| Option | Meaning |
|---|---|
| `--path PATH` | Scan a path relative to the knowledge base. Defaults to `wiki/`. Target resolution still uses pages under `wiki/`. |
| `--limit N` | Limit returned link records. Use `0` for no limit. |
| `--resolved` | Return only resolved links. |
| `--unresolved` | Return only unresolved links. |
| `--ambiguous` | Return only ambiguous links. |
| `--json` | Print a machine-readable JSON report. |

## Output categories

`kb links` classifies each link into one of three states:

```text
resolved   = exactly one matching target page was found
unresolved = no matching target page was found
ambiguous  = multiple matching target pages were found
```

Example output:

```text
wiki/concepts/Thermal_Cloak.md:42 [resolved] [[Transformation Thermodynamics]] -> wiki/concepts/Transformation_Thermodynamics.md
wiki/papers/example.md:88 [unresolved] [[Unknown Concept]] -> -
wiki/topics/thermal.md:12 [ambiguous] [[Cloak]] -> wiki/concepts/Cloak.md, wiki/topics/Cloak.md
```

## Deferred LLM / agent task hints

`kb links` can deterministically identify unresolved and ambiguous links, but it should not guess the correct semantic repair.

When unresolved or ambiguous links exist, the report includes task hints for a future **Wiki Link Repair Agent**:

```text
target_agent: Wiki Link Repair Agent
goal: decide whether each unresolved link should be created, redirected, renamed, or converted to plain text
requirements: preserve aliases, return confidence, avoid broad structure rewrites without Manager LLM approval
files: source pages that contain problematic links
evidence: source page, line number, and original WikiLink
```

For a consolidated task handoff file, run:

```bash
kb tasks
```

That writes task groups under:

```text
LLM/tasks/
```

## Boundary

`kb links` does not:

```text
call an LLM
rewrite wiki pages
create missing pages
rename files
resolve semantic ambiguity
repair links automatically
build a full knowledge graph
```

It performs one narrow ability: **scan and classify WikiLinks**. Semantic repair belongs to a future explicit LLM/agent skill.
