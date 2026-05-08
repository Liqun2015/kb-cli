# Release Checklist

This checklist is for small local releases such as `v0.4.x` or `v0.5.x`. It is deliberately conservative.

## 1. Confirm the scope

Before editing, write down the release purpose in one sentence.

Good examples:

```text
v0.5.4 cleans up scripts so build helpers compile kb-cli itself and broad knowledge-base workflow wrappers are removed.
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
version = "0.5.4"
```

## 3. Run local checks

Recommended:

```bash
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
```

## 4. Review file changes

Use:

```bash
git status
git diff
git add .
git diff --cached
```

Review both the unstaged and staged diff.

## 5. Commit and push

Manual form:

```bash
git commit -m "v0.5.4 clean script helpers"
git push
```

Or use the helper scripts:

```bat
scripts\git_safe_push.bat "v0.5.4 clean script helpers"
```

```bash
scripts/git_safe_push.sh "v0.5.4 clean script helpers"
```

## 6. Keep release notes honest

If `cargo` or another required tool was unavailable, write that limitation down.

Do not claim that tests passed unless they were actually run.
