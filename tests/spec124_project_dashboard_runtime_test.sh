#!/usr/bin/env bash
# Spec 124 / focusa-ux2qx.8 — project dashboard and safe selection runtime proof.
set -euo pipefail
cd "$(dirname "$0")/.."
ROOT="$PWD"
FIXTURE="$(mktemp -d)"
PORT="${FOCUSA_PROJECT_DASHBOARD_TEST_PORT:-18796}"
BASE="http://127.0.0.1:${PORT}"
DAEMON_PID=""
cleanup() {
  if [[ -n "$DAEMON_PID" ]]; then
    kill "$DAEMON_PID" 2>/dev/null || true
    wait "$DAEMON_PID" 2>/dev/null || true
  fi
  rm -rf "$FIXTURE"
}
trap cleanup EXIT

fail() { echo "FAIL: $*" >&2; exit 1; }

cargo build -q -p focusa-api -p focusa-cli --bin focusa-daemon --bin focusa
HOME="$FIXTURE/home" \
FOCUSA_DATA_DIR="$FIXTURE/data" \
FOCUSA_BIND="127.0.0.1:${PORT}" \
  "$ROOT/target/debug/focusa-daemon" >"$FIXTURE/daemon.log" 2>&1 &
DAEMON_PID=$!

for _ in $(seq 1 120); do
  curl -fsS --max-time 1 "$BASE/v1/health" >/dev/null 2>&1 && break
  sleep 0.1
done
curl -fsS --max-time 2 "$BASE/v1/health" >/dev/null \
  || { tail -100 "$FIXTURE/daemon.log" >&2; fail "isolated daemon did not become healthy"; }

# Templates and project creation must provide safe, usable defaults.
curl -fsS "$BASE/v1/project/templates" > "$FIXTURE/templates.json"
jq -e '.schema=="focusa.project_templates.v1" and .status=="ok" and .count>0' \
  "$FIXTURE/templates.json" >/dev/null \
  || { cat "$FIXTURE/templates.json" >&2; fail "project templates unavailable"; }
NEW_ROOT="$FIXTURE/projects/new-project"
curl -fsS -X POST "$BASE/v1/project/new" \
  -H 'content-type: application/json' \
  --data "$(jq -cn --arg root "$NEW_ROOT" '{project_root:$root,project_id:"new-project",canonical_name:"New Project",template:"blank",workspace_kind:"rust-monorepo",create_git:false,use_selected:false,force:false}')" \
  > "$FIXTURE/new.json"
jq -e '.status=="ok"' "$FIXTURE/new.json" >/dev/null \
  || { cat "$FIXTURE/new.json" >&2; fail "safe project creation failed"; }
[[ -f "$NEW_ROOT/.focusa-project.json" && -f "$NEW_ROOT/.focusa/settings.json" ]] \
  || fail "project creation omitted marker/settings defaults"

# Settings must be readable and safely mutable within the created project.
curl -fsS -G "$BASE/v1/project/settings" --data-urlencode "project_root=$NEW_ROOT" \
  > "$FIXTURE/settings.json"
jq -e '.schema=="focusa.project_settings.v1" and .status=="ok"' \
  "$FIXTURE/settings.json" >/dev/null \
  || { cat "$FIXTURE/settings.json" >&2; fail "project settings unavailable"; }
curl -fsS -X POST "$BASE/v1/project/settings" \
  -H 'content-type: application/json' \
  --data "$(jq -cn --arg root "$NEW_ROOT" '{action:"set",project_root:$root,key:"dashboard_mode",value:"compact"}')" \
  > "$FIXTURE/settings-set.json"
jq -e '.status=="ok"' "$FIXTURE/settings-set.json" >/dev/null \
  || { cat "$FIXTURE/settings-set.json" >&2; fail "project setting update failed"; }
curl -fsS -G "$BASE/v1/project/settings" \
  --data-urlencode "project_root=$NEW_ROOT" --data-urlencode 'key=dashboard_mode' \
  > "$FIXTURE/settings-key.json"
jq -e '.status=="ok" and .value=="compact"' "$FIXTURE/settings-key.json" >/dev/null \
  || { cat "$FIXTURE/settings-key.json" >&2; fail "project setting did not persist"; }

# Unsafe broad caller cwd must not inherit the daemon install/build root.
(
  cd /root
  FOCUSA_API_URL="$BASE" "$ROOT/target/debug/focusa" --json project current
) > "$FIXTURE/current-unselected.json"
jq -e '
  .schema == "focusa.project_dashboard.v1" and
  .status == "degraded" and
  .failure_class == "project_root_selection_required" and
  .selected == null and
  .effective_project == null and
  .runtime.status == "invalid"
' "$FIXTURE/current-unselected.json" >/dev/null \
  || { cat "$FIXTURE/current-unselected.json" >&2; fail "unsafe-home dashboard did not fail closed"; }
if grep -q '/usr/local/lib/focusa' "$FIXTURE/current-unselected.json"; then
  fail "dashboard leaked stale daemon install root as caller project"
fi

# Explicitly select a verified project in isolated state.
curl -fsS -X POST "$BASE/v1/project/use" \
  -H 'content-type: application/json' \
  --data "$(jq -cn --arg root "$ROOT" '{project_root:$root,selected_by:"runtime-test",note:"isolated proof"}')" \
  > "$FIXTURE/use.json"
jq -e --arg root "$ROOT" '.status=="ok" and .selected.project_root==$root' \
  "$FIXTURE/use.json" >/dev/null \
  || { cat "$FIXTURE/use.json" >&2; fail "safe project selection failed"; }

# The selected project must win over an unsafe caller cwd without rewriting it.
(
  cd /root
  FOCUSA_API_URL="$BASE" "$ROOT/target/debug/focusa" --json project current
) > "$FIXTURE/current-selected.json"
jq -e --arg root "$ROOT" '
  .status == "ok" and
  .failure_class == null and
  .selected.project_root == $root and
  .effective_project.project_root == $root and
  .runtime.status == "invalid"
' "$FIXTURE/current-selected.json" >/dev/null \
  || { cat "$FIXTURE/current-selected.json" >&2; fail "selected-project dashboard did not override unsafe runtime hint"; }

# From the project itself, runtime identity is verified and remains consistent.
(
  cd "$ROOT"
  FOCUSA_API_URL="$BASE" "$ROOT/target/debug/focusa" --json project status
) > "$FIXTURE/status-project.json"
jq -e --arg root "$ROOT" '
  .status == "ok" and
  .runtime.project_identity.status == "verified" and
  .selected.project_root == $root
' "$FIXTURE/status-project.json" >/dev/null \
  || { cat "$FIXTURE/status-project.json" >&2; fail "verified project runtime dashboard failed"; }

echo "PASS: project dashboard fails closed at unsafe home and honors persisted safe selection"
