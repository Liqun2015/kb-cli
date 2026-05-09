# LLM Hierarchy

Current version: `v0.5.10`

This project separates LLM roles by level.

The LLM that **uses** structured `kb` commands is not at the same level as the LLM that later **executes** the tasks produced by those commands.

## Role hierarchy

```text
Highest level: Manager LLM
    Uses deterministic kb commands.
    Reads command outputs, reports, and LLM/tasks/.
    Decides what should be delegated.
    Assigns one clear task at a time to lower-level Worker LLMs.
    Reviews results and records completion through kb memory.

Lower level: Worker LLM
    Receives a specific task from LLM/tasks/ or from the Manager LLM.
    Works on a bounded file list with explicit requirements.
    Produces reviewable changes or reports.
    Does not redefine the overall workflow.

Deterministic layer: Rust-native kb commands
    Search, inspect, extract, count, lint, and generate handoff materials.
    They do not call LLM APIs by default.
```

## Why this matters

The purpose of `kb-cli` is not to let every LLM wander freely through the repository.

The command-using LLM is the top-level coordinator. It should first run deterministic commands such as `kb query`, `kb grep`, `kb refs`, `kb lint-static`, and `kb tasks`. These commands scout the project, organize evidence, and reduce the search space.

Only after that should hard semantic work be handed to lower-level LLM workers.

This keeps the system:

- fast;
- auditable;
- cheaper to run;
- easier to review;
- less dependent on hidden model judgment;
- safer for long-term knowledge-base maintenance.

## Manager LLM responsibilities

The Manager LLM may:

- run or request deterministic `kb` commands;
- inspect JSON or Markdown command output;
- choose which task from `LLM/tasks/` should be executed next;
- decide which lower-level worker task is appropriate;
- prepare bounded instructions for the worker;
- require evidence, file lists, and non-goals;
- run checks after work is completed;
- record completed work with `kb memory`.

The Manager LLM should not:

- directly perform every semantic task when a worker task handoff is clearer;
- skip deterministic inspection commands;
- ask worker LLMs to discover the whole file universe from scratch;
- allow lower-level workers to silently expand scope;
- treat chat history as durable project memory.

## Worker LLM responsibilities

A Worker LLM may:

- repair a specific broken-link task;
- clean a bounded extracted-text file;
- reconcile a bounded set of reference hints;
- draft or revise a specific wiki page;
- normalize source metadata for a named file list;
- produce a concise report for review.

A Worker LLM should not:

- decide global project direction;
- invent additional source files;
- bypass the task file it was given;
- modify unrelated files;
- silently call other agents;
- close its own task without review or memory recording.

## Task handoff rule

A task handed from the Manager LLM to a Worker LLM should include:

- target worker or skill name;
- goal;
- requirements;
- exact file list;
- evidence from deterministic commands;
- non-goals;
- expected output;
- completion-memory requirements.

This is why `kb tasks` writes structured handoff files under `LLM/tasks/`.

## Completion rule

After a worker finishes meaningful work, the Manager LLM or human operator should record the outcome:

```bash
kb memory --task-id TASK_ID --summary "What was completed" --source-task LLM/tasks/llm_tasks_YYYYMMDD_HHMMSS.md
```

`LLM/memory/` is the project-local audit memory. It is not hidden model memory.

## Relationship to command classification

This hierarchy complements `docs/command-classification.md`:

```text
Rust commands          = deterministic scouting and evidence organization.
Manager LLM          = top-level planner and dispatcher that uses those commands.
Worker LLM           = bounded executor for deferred semantic tasks.
LLM/tasks/             = handoff queue.
LLM/memory/            = completed-task audit memory.
```

The highest-level LLM should use the command layer first. The lower-level LLM should receive a bounded task after the command layer has already narrowed the work.
