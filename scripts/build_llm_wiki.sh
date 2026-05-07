#!/usr/bin/env bash
set -euo pipefail

show_help() {
  cat <<'HELP'
kb-cli macOS/Linux quick-start helper

Usage:
  scripts/build_llm_wiki.sh /path/to/KnowledgeBase

Optional:
  scripts/build_llm_wiki.sh /path/to/KnowledgeBase /path/to/kb-cli

Flags:
  --copy             Copy files into raw/, default and safest.
  --move             Move files into raw/.
  --recursive        Also collect files from subfolders, excluding raw/wiki/rules/etc.
  --dry-run          Show planned actions without copying or moving files.
  --force-init       Run kb init --force.
  --force-metadata   Re-extract PDF metadata.
  --skip-metadata    Skip PDF metadata extraction.
  --skip-build       Skip wiki generation.
  --no-install       Do not auto-install kb with cargo install.
  --help             Show this help.

Examples:
  scripts/build_llm_wiki.sh "$HOME/github/LLM-wiki/quantum"
  scripts/build_llm_wiki.sh "$HOME/github/LLM-wiki/quantum" --move
  scripts/build_llm_wiki.sh "$HOME/github/LLM-wiki/quantum" --recursive --dry-run
HELP
}

if [[ $# -eq 0 ]]; then
  show_help
  exit 1
fi

case "${1:-}" in
  --help|-h)
    show_help
    exit 0
    ;;
esac

KB_ROOT="$1"
shift

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
KB_CLI_DIR=""
MODE="copy"
BOOTSTRAP_EXTRA_ARGS=()
NO_INSTALL=0

if [[ $# -gt 0 && "${1:0:2}" != "--" ]]; then
  KB_CLI_DIR="$1"
  shift
fi

while [[ $# -gt 0 ]]; do
  case "$1" in
    --copy)
      MODE="copy"
      ;;
    --move)
      MODE="move"
      ;;
    --recursive)
      BOOTSTRAP_EXTRA_ARGS+=("--recursive")
      ;;
    --dry-run|--preview)
      BOOTSTRAP_EXTRA_ARGS+=("--dry-run")
      ;;
    --force-init)
      BOOTSTRAP_EXTRA_ARGS+=("--force-init")
      ;;
    --force-metadata)
      BOOTSTRAP_EXTRA_ARGS+=("--force-metadata")
      ;;
    --skip-metadata)
      BOOTSTRAP_EXTRA_ARGS+=("--skip-metadata")
      ;;
    --skip-build)
      BOOTSTRAP_EXTRA_ARGS+=("--skip-build")
      ;;
    --no-install)
      NO_INSTALL=1
      ;;
    --help|-h)
      show_help
      exit 0
      ;;
    *)
      echo "[ERROR] Unknown argument: $1" >&2
      echo >&2
      show_help
      exit 1
      ;;
  esac
  shift
done

if [[ -z "$KB_CLI_DIR" ]]; then
  KB_CLI_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
  if [[ ! -f "$KB_CLI_DIR/Cargo.toml" ]]; then
    KB_CLI_DIR="$(pwd)"
  fi
fi

BOOTSTRAP_ARGS=("--$MODE" "${BOOTSTRAP_EXTRA_ARGS[@]}")

cat <<INFO

============================================================
 kb-cli LLM Wiki quick builder
============================================================
 target wiki root : $KB_ROOT
 kb-cli source    : $KB_CLI_DIR
 bootstrap args   : ${BOOTSTRAP_ARGS[*]}
============================================================
INFO

if ! command -v cargo >/dev/null 2>&1; then
  echo "[ERROR] cargo was not found. Install Rust first, then re-run this script." >&2
  exit 1
fi

if ! command -v kb >/dev/null 2>&1; then
  echo "[INFO] kb command not found."
  if [[ "$NO_INSTALL" == "1" ]]; then
    echo "[ERROR] --no-install was set, so this script will not run cargo install." >&2
    echo "Install manually first:" >&2
    echo "  cd '$KB_CLI_DIR'" >&2
    echo "  cargo install --path . --force" >&2
    exit 1
  fi

  if [[ -f "$KB_CLI_DIR/Cargo.toml" ]]; then
    echo "[INFO] Installing kb-cli from: $KB_CLI_DIR"
    (cd "$KB_CLI_DIR" && cargo install --path . --force)
  else
    echo "[ERROR] kb command is not installed, and Cargo.toml was not found in: $KB_CLI_DIR" >&2
    exit 1
  fi
else
  echo "[OK] kb command found."
fi

kb --help >/dev/null

echo "[OK] kb is runnable."
echo
echo "[RUN] kb --kb-path '$KB_ROOT' bootstrap ${BOOTSTRAP_ARGS[*]}"
echo

kb --kb-path "$KB_ROOT" bootstrap "${BOOTSTRAP_ARGS[@]}"

cat <<INFO

============================================================
 Done.
============================================================
 Wiki home:
 $KB_ROOT/wiki/Home.md

 Open this folder with Obsidian:
 $KB_ROOT
============================================================
INFO
