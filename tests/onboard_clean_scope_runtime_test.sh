#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
source "$ROOT/tests/focusa_portable_bin.sh"
command -v jq >/dev/null || { echo "FAIL: jq required" >&2; exit 1; }
BINARY="$(focusa_resolve_test_cli_binary "$ROOT")"
TMP="$(mktemp -d /tmp/focusa-onboard-scopes.XXXXXX)"
cleanup() { python3 - "$TMP" <<'PY'
import shutil, sys
shutil.rmtree(sys.argv[1], ignore_errors=True)
PY
}
trap cleanup EXIT
mkdir -p "$TMP/home" "$TMP/git-project" "$TMP/no-git" "$TMP/remote-project"
git -C "$TMP/git-project" init -q

run_json() {
  HOME="$TMP/home" "$BINARY" --json onboard --no-demo-workpoint "$@"
}

project="$(run_json --scope project --project-root "$TMP/git-project")"
jq -e --arg root "$TMP/git-project" '.scope=="project" and .project_root==$root and .checks.git_repo=="ok"' <<<"$project" >/dev/null

no_git="$(run_json --scope project --project-root "$TMP/no-git")"
jq -e --arg root "$TMP/no-git" '.scope=="project" and .project_root==$root and .checks.git_repo=="needs_attention"' <<<"$no_git" >/dev/null

host="$(run_json --scope host --project-root "$TMP/home")"
jq -e '.scope=="host" and .project_root=="" and .project_identity.status=="skipped" and .workpoint==null' <<<"$host" >/dev/null
test ! -e "$TMP/home/.focusa-project.json"

remote='https://example.test/acme/demo.git'
first="$(run_json --scope project --project-root "$TMP/remote-project" --remote "$remote")"
marker="$TMP/remote-project/.focusa-project.json"
jq -e --arg remote "$remote" '.project_marker.status=="created" and .project_marker.repo_remote==$remote' <<<"$first" >/dev/null
test -f "$marker"
before="$(sha256sum "$marker" | awk '{print $1}')"
second="$(run_json --scope project --project-root "$TMP/remote-project" --remote "$remote")"
after="$(sha256sum "$marker" | awk '{print $1}')"
jq -e --arg remote "$remote" '.project_marker.status=="exists" and .project_marker.repo_remote==$remote' <<<"$second" >/dev/null
test "$before" = "$after"

unsafe_log="$TMP/unsafe.log"
if run_json --scope project --project-root /tmp --remote "$remote" >"$unsafe_log" 2>&1; then
  echo "FAIL: unsafe broad project root was accepted" >&2
  exit 1
fi
grep -q 'unsafe project root' "$unsafe_log"

echo 'PASS: clean project, host, remote-marker, no-git, idempotency, and unsafe-root onboarding flows'
