#!/usr/bin/env bash
set -euo pipefail

SOURCE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REPO_ROOT="$(git rev-parse --show-toplevel)"
HOOKS_DIR="$(git rev-parse --git-path hooks)"
mkdir -p "$HOOKS_DIR"

install_hook() {
  local name="$1"
  local source="$SOURCE_ROOT/scripts/git-hooks/$name"
  local target="$HOOKS_DIR/$name"
  local backup="$target.focusa-upstream"

  if [[ -f "$target" ]] && ! grep -q "FOCUSA_COMMIT_MESSAGE_HOOK" "$target"; then
    cp "$target" "$backup"
    chmod +x "$backup"
  fi
  cp "$source" "$target"
  chmod +x "$target"
}

cd "$REPO_ROOT"
install_hook prepare-commit-msg
install_hook commit-msg
install_hook pre-push

printf 'Installed Focusa commit-message hooks in %s\n' "$HOOKS_DIR"
