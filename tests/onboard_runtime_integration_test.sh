#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
source "$ROOT/tests/focusa_portable_bin.sh"
BINARY="$(focusa_resolve_test_cli_binary "$ROOT")"
API="${FOCUSA_DAEMON_URL:-http://127.0.0.1:8787}"
command -v jq >/dev/null || { echo 'FAIL: jq required' >&2; exit 1; }

systemctl is-active --quiet focusa-daemon || { echo 'FAIL: focusa-daemon inactive' >&2; exit 1; }
for command in focusa focusa-daemon focusa-tui pi; do
  command -v "$command" >/dev/null || { echo "FAIL: $command not on PATH" >&2; exit 1; }
  bash -lc "command -v $command" >/dev/null || { echo "FAIL: $command not on login-shell PATH" >&2; exit 1; }
done
pi --extension "$ROOT/apps/pi-extension/src/index.ts" --help | grep -q -- '--no-focusa' || {
  echo 'FAIL: Pi Focusa integration unavailable' >&2
  exit 1
}
curl -fsS --max-time 20 "$API/v1/health" | jq -e '.status=="ok" and (.version|type=="string")' >/dev/null

FIXTURE_ROOT="$HOME/focusa-onboard-tests"
mkdir -p "$FIXTURE_ROOT"
TMP="$(mktemp -d "$FIXTURE_ROOT/run.XXXXXX")"
cleanup() {
  python3 - "$TMP" <<'PY'
import shutil, sys
shutil.rmtree(sys.argv[1], ignore_errors=True)
PY
}
trap cleanup EXIT
PROJECT="$TMP/onboard-fixture"
mkdir -p "$PROJECT" "$TMP/home"
git -C "$PROJECT" init -q
printf '[package]\nname="onboard-fixture"\nversion="0.1.0"\n' >"$PROJECT/Cargo.toml"
printf 'fixture license\n' >"$PROJECT/LICENSE"
continuity="onboard-runtime-$(date +%s)-$$"
remote='https://example.test/focusa/onboard-fixture.git'

response="$(HOME="$TMP/home" "$BINARY" --json onboard --agent pi --scope project --project-root "$PROJECT" --remote "$remote" --continuity-id "$continuity")"
if grep -qi 'demo workpoint\|focusa-onboard-demo' <<<"$response"; then
  echo 'FAIL: demo-only substitution leaked into onboarding' >&2
  exit 1
fi
jq -e --arg continuity "$continuity" --arg remote "$remote" '
  .scope=="project" and
  .continuity_id==$continuity and
  .project_marker.repo_remote==$remote and
  .project_identity.status=="completed" and
  .project_identity.canonical==true and
  .trajectory.status=="completed" and
  .trajectory.canonical==true and
  (.workpoint.status=="accepted" or .workpoint.status=="completed") and
  .workpoint.canonical==true and
  .resume.status=="completed" and
  .resume.canonical==true
' <<<"$response" >/dev/null || { echo 'FAIL: onboarding response is not canonical' >&2; exit 1; }

encoded_root="$(python3 -c 'import sys,urllib.parse; print(urllib.parse.quote(sys.argv[1], safe=""))' "$PROJECT")"
trajectory="$(curl -fsS --max-time 20 "$API/v1/trajectory/view?project_root=$encoded_root&continuity_id=$continuity")"
workpoint="$(curl -fsS --max-time 20 "$API/v1/workpoint/current?project_root=$encoded_root&continuity_id=$continuity")"
jq -e '.canonical==true and (.trajectory.trajectory_id|type=="string")' <<<"$trajectory" >/dev/null || { echo 'FAIL: canonical Trajectory unavailable' >&2; exit 1; }
jq -e '.canonical==true and .status=="active" and .workpoint.work_item_id=="focusa-onboard-first-mission"' <<<"$workpoint" >/dev/null || { echo 'FAIL: canonical first Workpoint unavailable' >&2; exit 1; }

echo 'PASS: daemon/service, CLI/TUI/Pi PATH, project identity, canonical Trajectory and first Workpoint integration'
