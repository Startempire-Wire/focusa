#!/usr/bin/env bash
# Contract: project Beads files and daemon must be owned by the project owner.
# A root-owned bd daemon rewrites .beads/issues.jsonl as root and breaks
# evidence policy/pre-push gates for non-root agents.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

project_uid="$(stat -c '%u' .)"
project_user="$(stat -c '%U' .)"
failed=0

check_owner() {
  local path="$1"
  [[ -e "$path" ]] || return 0
  local uid user
  uid="$(stat -c '%u' "$path")"
  user="$(stat -c '%U' "$path")"
  if [[ "$uid" != "$project_uid" ]]; then
    echo "✗ $path owned by $user(uid=$uid), expected $project_user(uid=$project_uid)" >&2
    failed=1
  else
    echo "✓ $path owner=$user"
  fi
}

check_owner .beads
check_owner .beads/issues.jsonl
check_owner .git/beads-worktrees/beads-sync/.beads/issues.jsonl

if [[ -f .beads/daemon.pid ]]; then
  pid="$(cat .beads/daemon.pid || true)"
  if [[ -n "$pid" ]] && ps -p "$pid" >/dev/null 2>&1; then
    proc_uid="$(ps -o uid= -p "$pid" | tr -d ' ')"
    proc_user="$(ps -o user= -p "$pid" | tr -d ' ')"
    if [[ "$proc_uid" != "$project_uid" ]]; then
      echo "✗ bd daemon pid=$pid user=$proc_user(uid=$proc_uid), expected $project_user(uid=$project_uid)" >&2
      failed=1
    else
      echo "✓ bd daemon pid=$pid owner=$proc_user"
    fi
  else
    echo "✓ no running bd daemon from .beads/daemon.pid"
  fi
fi

if [[ "$failed" -ne 0 ]]; then
  echo "BD sync ownership policy: FAIL" >&2
  exit 1
fi

echo "BD sync ownership policy: PASS"
