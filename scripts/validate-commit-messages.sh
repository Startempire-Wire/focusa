#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage:
  scripts/validate-commit-messages.sh --message-file <path>
  scripts/validate-commit-messages.sh --range <git-revision-range>

Valid subjects use Conventional Commits (feat, fix, docs, test, refactor,
perf, build, ci, chore, revert, proof, or merge). Merge and generated Git revert subjects
are also accepted. Bead IDs belong in the body/trailers, never the subject.
USAGE
}

fail() {
  printf 'commit_message_policy: FAIL: %s\n' "$*" >&2
  return 1
}

validate_message_file() {
  local file="$1"
  local label="${2:-$file}"
  local subject
  local conventional_pattern='^(feat|fix|docs|test|refactor|perf|build|ci|chore|revert|proof|merge)(\([^)]+\))?!?:[[:space:]].{4,}$'
  subject=$(awk 'NF && $0 !~ /^[[:space:]]*#/ { sub(/\r$/, ""); print; exit }' "$file")

  [[ -n "$subject" ]] || { fail "${label}: subject is empty"; return 1; }

  # Git-generated merge/revert subjects carry branch/ref context and may exceed
  # the conventional subject limit; the policy explicitly accepts them.
  if [[ "$subject" =~ ^Merge[[:space:]] ]] || [[ "$subject" =~ ^Revert[[:space:]]\" ]]; then
    return 0
  fi

  [[ ${#subject} -le 100 ]] || { fail "${label}: subject exceeds 100 characters"; return 1; }

  case "$subject" in
    Beads:*|beads:*)
      fail "${label}: Beads IDs cannot be the commit subject; keep the human description first and move IDs to a body trailer"
      return 1
      ;;
  esac

  if [[ "$subject" =~ ^(focusa|workspace)-[[:alnum:]][[:alnum:]._-]*(,?[[:space:]]+(focusa|workspace)-[[:alnum:]][[:alnum:]._-]*)*$ ]]; then
    fail "${label}: ID-only commit subjects are forbidden"
    return 1
  fi

  if [[ "$subject" =~ ^(WIP|wip|update|updates|changes|misc|fix|test|commit)$ ]]; then
    fail "${label}: generic commit subject '${subject}' is not meaningful"
    return 1
  fi

  if [[ ! "$subject" =~ $conventional_pattern ]]; then
    fail "${label}: expected a meaningful Conventional Commit subject, e.g. 'fix: preserve compaction continuity'"
    return 1
  fi
}

is_exact_historical_exception() {
  # These two empty-subject commits accidentally added an empty file named
  # `nonexistent` directly to main on 2026-07-29. Each is immutable published
  # history and has a subsequent named cleanup commit. Keep this range-only
  # exception hash-pinned so no future malformed commit can pass.
  case "$1" in
    435f1cb9be6b91fb279c141408868e6c63d67e68|b0f0ebc20f50af17b4541ee4a279ea0b0d0d93ae)
      return 0
      ;;
    *)
      return 1
      ;;
  esac
}

validate_range() {
  local range="$1"
  local failed=0
  local commit tmp
  git rev-list --reverse "$range" >/dev/null
  while IFS= read -r commit; do
    [[ -n "$commit" ]] || continue
    if is_exact_historical_exception "$commit"; then
      printf 'commit_message_policy: exact historical exception: %s\n' "${commit:0:12}" >&2
      continue
    fi
    tmp=$(mktemp "${TMPDIR:-/tmp}/focusa-commit-message.XXXXXX")
    git show -s --format=%B "$commit" > "$tmp"
    if ! validate_message_file "$tmp" "${commit:0:12}"; then
      failed=1
    fi
    rm -f "$tmp"
  done < <(git rev-list --reverse "$range")
  [[ "$failed" -eq 0 ]]
}

case "${1:-}" in
  --message-file)
    [[ $# -eq 2 ]] || { usage >&2; exit 2; }
    validate_message_file "$2"
    ;;
  --range)
    [[ $# -eq 2 ]] || { usage >&2; exit 2; }
    validate_range "$2"
    ;;
  -h|--help)
    usage
    ;;
  *)
    usage >&2
    exit 2
    ;;
esac
