#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -lt 1 ]; then
  echo 'Usage: scripts/git_safe_push.sh "commit message"'
  echo 'Example: scripts/git_safe_push.sh "v0.4.8 developer workflow helpers"'
  exit 1
fi

commit_message="$*"

echo
echo "===== Current status ====="
git --no-pager status

echo
echo "===== Unstaged diff: changes not yet added ====="
git --no-pager diff

echo
read -r -p "Review the changes above. Press Enter to continue with: git add ."

git add .

echo
echo "===== Staged diff: changes ready to commit ====="
git --no-pager diff --cached

echo
read -r -p "Review the staged changes above. Press Enter to commit and push."

git commit -m "$commit_message"
git push

echo
echo "Done."
