# Release Checklist

This checklist is for small local releases such as `v0.4.x` or `v0.5.x`. It is deliberately conservative.

## 1. Confirm the scope

Before editing, write down the release purpose in one sentence.

Good examples:

```text
v0.5.10 documents the literature relationship core and Manager/Worker hierarchy while keeping OCR, LLM calls, citation-graph generation, and autonomous editing out of scope.
v0.4.7 only improves cross-platform onboarding docs.
```

Avoid mixing unrelated work into the same small release.

## 2. Update version markers

Check:

```text
Cargo.toml
Cargo.lock
README.md
CHANGELOG.md
```

For standard releases, use normal SemVer such as:

```toml
version = "0.5.7"
```

## 3. Run local checks

Recommended:

```bash
cargo fmt
cargo fmt --check
cargo test
cargo check
cargo build --release
```

Or use the bundled build helper:

```bat
scripts\build_release.bat
```

```bash
scripts/build_release.sh
```

Also check the CLI help text:

```bash
kb --help
kb prepare --help
kb lint-static --help
kb query --help
kb links --help
kb grep --help
kb extract-text --help
kb refs --help
kb refs-index --help
kb refs-graph --help
kb keywords --help
kb health --help
kb shell --help
kb tasks --help
kb memory --help
```

## 4. Review file changes

Use:

```bash
git --no-pager status
git --no-pager diff
git add .
git --no-pager diff --cached
```

Review both the unstaged and staged diff.

## 5. Commit and push

Manual form:

```bash
git commit -m "v0.5.10 literature relationship core"
git push
```

Or use the helper scripts:

```bat
scripts\git_safe_push.bat "v0.5.10 literature relationship core"
```

```bash
scripts/git_safe_push.sh "v0.5.10 literature relationship core"
```

## 6. Keep release notes honest

If `cargo` or another required tool was unavailable, write that limitation down.

Do not claim that tests passed unless they were actually run.


For v0.5.10 and later, also smoke-test:

```bash
kb refs-index --help
kb refs-graph --help
kb keywords --help
kb health --help
kb shell --help
kb refs-index --dry-run
```


## Third-party skill documentation check

When relationship status, graph export, or visualization conventions change, review:

```text
docs/third-party-skills/README.md
docs/third-party-skills/relation-graph-visual-protocol.md
docs/third-party-skills/relation-graph-json-schema.md
docs/third-party-skills/relation-graph-examples.md
docs/third-party-skills/developer-contract.md
docs/third-party-skills/claude-code-generation-skill.md
```

## Documentation convergence check

For documentation convergence releases, especially `v0.5.10.3` and later, review:

```text
docs/commands.md
docs/task-lifecycle.md
docs/command-classification.md
docs/llm-command-guide.md
docs/llm-hierarchy.md
docs/literature-relationships.md
```

Confirm that each command has a clear ability, input area, output area, LLM boundary, and deferred-work rule.

For graph-export releases, smoke-test:

```bash
kb refs-graph --dry-run
kb keywords --dry-run
kb refs-graph --json --dry-run
```

## Cargo.lock note

If the downloaded source package does not include `Cargo.lock`, run `cargo check` or `cargo build --release` once to generate a fresh local lockfile. This avoids stale transitive locks from fast-moving Windows crates. Review the generated `Cargo.lock` with `git --no-pager diff` before committing.


## Topic relationship overlay check

Before adding topic-specific relationship features, confirm that `docs/topic-relationships.md` still distinguishes global bibliographic index relations from topic-local causal, method, evidence, idea, and importance overlays.
