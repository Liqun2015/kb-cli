# Literature Relationship Core Principle

Current version: `v0.7.40`

LLM Wiki is not merely a local document archive.

Its central purpose is to maintain an evolving, reviewable, and evidence-backed relationship network among references.

Source files, extracted text, grep results, reference scans, WikiLinks, task handoffs, and memory records are all supporting structures for this relationship network.

## Core statement

```text
LLM Wiki maintains relationships among references.
Everything else exists to support that relationship network.
```

This principle should guide future command design. Before adding a command, ask:

```text
Which reference relationship does this command discover, confirm, repair, explain, or delegate?
```

## Global vs topic-specific relations

LLM Wiki separates relationship records into two layers:

```text
processing/refs/
    Global bibliographic index relations.
    These apply across all topics.

topics/<topic>/
    Topic-specific interpretive relations.
    These include topic-local importance, causal candidates, method relations, evidence relations, and scientific idea relations.
```

The global layer should remain conservative and reusable. Topic overlays may contain richer, topic-dependent interpretation, but these records must preserve evidence and review status. See `docs/topic-relationships.md`.

## Three relationship levels

### 1. Bibliographic index relations

These are direct reference-index relations:

```text
Paper A cites Paper B.
Paper A's reference list contains Paper B.
A reference entry in Paper A matches a local source file.
Two reference records share DOI / title / author / journal / year / volume / pages.
```

The evidence can include:

- title;
- author names;
- journal name;
- year and date;
- volume, issue, and pages;
- DOI;
- reference list entries;
- numbered citations;
- author-year citations.

Rust-native commands should extract candidate relations wherever possible.

Human review is the final guarantee for this level, because different journals format references differently and the same paper may appear in multiple incomplete or inconsistent forms.

The Manager LLM may organize candidates and prepare review work, but it should not pretend that uncertain bibliographic matches are confirmed.

### 2. Keyword / topic relations

These are relations based on shared terms, topics, and repeated technical expressions:

```text
Two papers mention the same method.
Two papers share important technical keywords.
Two papers discuss similar materials, models, devices, or experiments.
```

Rust-native commands should find candidates through `kb query`, `kb grep`, and future keyword/statistics commands.

The Manager LLM may decide whether the keyword relation is scientifically meaningful and may delegate bounded analysis to a Worker LLM.

### 3. Scientific idea relations

These are conceptual research relations:

```text
method transfer
mechanism similarity
theory inheritance
problem-method relation
assumption-counterexample relation
research-gap relation
cross-domain inspiration
```

These relations are the most valuable part of LLM Wiki and should be handled through explicit LLM workflows.

The Manager LLM should use deterministic command evidence first, then assign bounded tasks to Worker LLMs for synthesis, comparison, and draft writing.

Important idea relations should remain traceable to source files, evidence lines, and human review records.

## Role responsibilities

```text
Rust-native kb commands
    Discover evidence and candidate relations.
    Do not make unsupported semantic claims.

Manager LLM
    Uses kb commands.
    Reads evidence.
    Prioritizes relationship work.
    Delegates bounded semantic tasks to Worker LLMs.
    Records completed work through kb memory.

Worker LLM
    Handles a specific bounded task.
    Produces reviewable outputs with evidence.
    Does not redefine project direction.

Human reviewer
    Provides the final guarantee for bibliographic index relations and important scientific claims.
```

## Command implications

Existing and future commands should be interpreted through this relationship-first model:

```text
kb extract-text
    Converts sources into searchable text so relationships can be discovered.

kb grep
    Finds exact textual evidence and structural traces.

kb refs
    Finds reference headings, entries, DOI values, and citation markers.

kb links
    Maintains explicit wiki-level links.

kb tasks
    Turns unresolved relationship work into structured handoff tasks.

kb memory
    Records which relationship tasks were completed and what remains unresolved.
```

`kb keywords` begins the keyword/topic relation layer by finding deterministic co-occurrence candidates. Future commands such as `kb refs-review`, `kb relation-tasks`, and explicit Worker LLM workflows should continue this pattern.

## Non-goals

Do not turn uncertain evidence into confirmed relationships automatically.

Do not let a Worker LLM decide global relationship policy.

Do not hide bibliographic uncertainty behind model confidence.

Do not treat keyword co-occurrence as scientific idea equivalence.

Do not skip human review for high-impact bibliographic matches or scientific claims.


## `kb refs-index`

`kb refs-index` is the first skeleton command dedicated to Level 1 bibliographic index relations. It connects extracted reference entries with local papers when deterministic evidence is available.

The command is intentionally conservative: DOI-exact matches can be marked `confirmed`, while title/filename matches are only `candidate`; multiple equal matches are `ambiguous`; and unmatched entries are `missing`. Candidate, ambiguous, and missing rows become review material for humans and for Manager LLM routing.

## Third-party graph visualization protocol

Third-party skills and tools should render literature relationship certainty in a stable way:

```text
confirmed bibliographic relation -> solid edge
candidate or ambiguous relation  -> dashed edge
missing / unresolved reference   -> hollow node
needs human review               -> explicit evidence and review marker
```

The detailed third-party skill guidance lives in `docs/third-party-skills/`.

This visual protocol is not cosmetic. It is part of the evidence and review model. Graph tools must not silently upgrade uncertain relations into confirmed relations.

## v0.6.3 topic schema foundation

The topic relationship layer now has a conservative schema foundation:

```text
docs/topic-relation-schema.md
    Directed relation records such as uses_method, improves, supports, contradicts, causal_or_motivates, and unknown.

docs/literature-importance-schema.md
    Global and topic-local importance levels such as core, important, background, peripheral, and unknown.

docs/topic-v2-roadmap.md
    A practical route for topic-centered relationship overlays.
```

The schema does not change the global `processing/refs/` layer. Global bibliographic index relations remain shared across all topics. Topic-local method, causal, evidence, idea, and importance records belong under `topics/<topic>/`.
