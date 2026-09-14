#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT
CALLS="$TMP_DIR/git-calls"
PUSH_COUNT="$TMP_DIR/push-count"
printf '0\n' > "$PUSH_COUNT"

FUNCTION_SOURCE="$(awk '
  /^push_candidate_main_with_auto_rebase\(\) \{/ { capture = 1 }
  capture { print }
  capture && /^}$/ { exit }
' "$ROOT_DIR/scripts/create-dev-release-tag.sh")"
[[ -n "$FUNCTION_SOURCE" ]] || {
  echo "missing push_candidate_main_with_auto_rebase function" >&2
  exit 1
}

git() {
  printf '%s\n' "$*" >> "$CALLS"
  case "$*" in
    "push origin HEAD:main")
      count="$(cat "$PUSH_COUNT")"
      count=$((count + 1))
      printf '%s\n' "$count" > "$PUSH_COUNT"
      [[ "$count" -gt 1 ]]
      ;;
    "pull --rebase origin main")
      return 0
      ;;
    *)
      echo "forbidden pre-CI git operation: git $*" >&2
      return 99
      ;;
  esac
}

eval "$FUNCTION_SOURCE"
push_candidate_main_with_auto_rebase

[[ "$(grep -c '^push origin HEAD:main$' "$CALLS")" -eq 2 ]] || {
  echo "candidate main was not retried exactly once" >&2
  exit 1
}
[[ "$(grep -c '^pull --rebase origin main$' "$CALLS")" -eq 1 ]] || {
  echo "candidate race did not perform one bounded rebase" >&2
  exit 1
}
if grep -Eq '(^| )tag( |$)|push origin v[0-9]' "$CALLS"; then
  echo "pre-CI retry created, retargeted, or pushed a release tag" >&2
  exit 1
fi

printf 'PASS: pre-CI candidate race retries main without touching the release tag\n'
