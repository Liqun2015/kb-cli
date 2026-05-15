# Git Workflow

`kb-cli` is designed around reviewable local files. Every meaningful change to the knowledge base should be visible through Git before it is accepted.

The preferred workflow is:

```text
edit files
  ↓
git --no-pager status
  ↓
git --no-pager diff
  ↓
git add .
  ↓
git --no-pager diff --cached
  ↓
git commit -m "clear message"
  ↓
git push
```

This is intentionally a little slower than a blind commit. The extra review step helps keep the project maintainable, auditable, and easy to roll back.

## Windows helper

From the project root:

```bat
scripts\git_safe_push.bat "v0.5.10 literature relationship core"
```

The script pauses twice:

1. before `git add .`, after showing `git --no-pager diff`;
2. before `git commit` and `git push`, after showing `git --no-pager diff --cached`.

## macOS / Linux helper

From the project root:

```bash
chmod +x scripts/git_safe_push.sh
scripts/git_safe_push.sh "v0.5.10 literature relationship core"
```

The Unix helper follows the same two-review process as the Windows helper.

## Manual commands

If you do not want to use the helper scripts, run:

```bash
git --no-pager status
git --no-pager diff
git add .
git --no-pager diff --cached
git commit -m "your commit message"
git push
```


## Pager behavior

The helper scripts intentionally use `git --no-pager ...` for review output. This avoids Git opening `less` or another pager that requires page-by-page key presses. If you run commands manually and want the same behavior, use `git --no-pager diff` and `git --no-pager diff --cached`.

## What not to do

Avoid this pattern for normal work:

```bash
git add . && git commit -m "update" && git push
```

It is too easy to push temporary files, local notes, generated test outputs, or experimental changes by accident.

## Commit message style

Use short, specific messages:

```text
v0.5.10 literature relationship core
fix lint-static missing source report
update Unix quickstart path notes
```

Avoid vague messages:

```text
update
fix
misc
final
```

## Before pushing a release tag

Run the local checks that are available on your machine:

```bash
cargo fmt --check
cargo test
cargo check
cargo build --release
kb --help
kb build --help
```

If a check cannot be run because a tool is not installed, say so explicitly in the release notes instead of pretending it passed.
