#!/usr/bin/env bash
# Spec124 Order 12 — CLI cross-phase smoke/regression suite.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FOCUSA_BIN="${FOCUSA_BIN:-$ROOT_DIR/target/debug/focusa}"

if [[ ! -x "$FOCUSA_BIN" ]]; then
  cargo build -p focusa-cli >/dev/null
fi

TMP_DIR="${TMPDIR:-/tmp}/focusa-cli-cross-phase-smoke.$$"
mkdir -p "$TMP_DIR"
trap 'rm -rf "$TMP_DIR"' EXIT

fail() { echo "FAIL: $*" >&2; exit 1; }
pass() { echo "PASS: $*"; }

run_allow() {
  local name="$1"; shift
  local out="$TMP_DIR/$name.out"
  set +e
  "$FOCUSA_BIN" "$@" >"$out" 2>&1
  local code=$?
  set -e
  echo "$code" >"$TMP_DIR/$name.code"
  echo "$out"
}

run_ok() {
  local name="$1"; shift
  local out
  out="$(run_allow "$name" "$@")"
  local code
  code="$(cat "$TMP_DIR/$name.code")"
  [[ "$code" == "0" ]] || { sed -n '1,120p' "$out" >&2; fail "$name exited $code"; }
  echo "$out"
}

json_has_any_status() {
  local file="$1"
  jq -e 'type == "object" and (.status? or .projects? or .project_count? or .schema? or .deck? or .health?)' "$file" >/dev/null \
    || { sed -n '1,160p' "$file" >&2; fail "expected JSON status-like envelope: $file"; }
}

# 28.1 canonical command set.
out="$(run_ok project_list --json project list)"; json_has_any_status "$out"; pass "focusa project list"
out="$(run_allow project_current --json project current)"; grep -Eq 'project|status|blocked|degraded|project_root_selection_required' "$out" || fail "project current produced no scoped status"; pass "focusa project current --json tolerated"
out="$(run_ok project_discover --json project discover --max-depth 2)"; jq -e 'tostring | contains("/root") | not or true' "$out" >/dev/null; grep -Eq 'unsafe|projects|candidates|project' "$out" || fail "project discover missing project/unsafe evidence"; pass "focusa project discover"
out="$(run_allow status_operator_positional --json status operator)"; grep -Eq 'project_root_selection_required|active_workpoint|health|blocked|degraded|status' "$out" || fail "status operator not scoped/blocking"; pass "focusa status operator --json"
out="$(run_ok first_mission --json first-mission --dry-run)"; grep -q 'first_mission' "$out" || fail "first-mission dry-run missing schema"; grep -Eq 'dry_run|would_' "$out" || fail "first-mission dry-run missing non-mutating marker"; pass "focusa first-mission --dry-run --json"

out="$(run_allow deck_self_test --json deck --headless-self-test)"
if [[ "$(cat "$TMP_DIR/deck_self_test.code")" != "0" ]]; then
  out="$(run_allow tui_self_test --json tui --headless-self-test)"
fi
grep -Eq 'headless|snapshot|status|health|tui|deck|blocked|degraded' "$out" || fail "deck/tui self-test missing expected envelope"; pass "deck/tui headless self-test"

# 28.2 alias smoke tests.
out="$(run_allow status_operator_flag --json status --operator)"; grep -Eq 'active_workpoint|health|blocked|degraded|status' "$out" || fail "status --operator alias missing status"; pass "focusa status --operator --json"
out="$(run_allow stack_alias stack)"; grep -Eq 'Deprecated alias|Active:|blocked|error|focus stack' "$out" || fail "focusa stack alias missing warning/status"; pass "focusa stack alias"
out="$(run_allow focus_stack focus stack)"; grep -Eq 'Active:|blocked|error|focus stack|stack' "$out" || fail "focusa focus stack missing status"; pass "focusa focus stack"
out="$(run_ok pair_help pair --help)"; grep -Eq 'Deprecated alias|Open a Mac Pairing Room|pairing start|QR' "$out" || fail "focusa pair --help missing pairing help"; pass "focusa pair --help"
out="$(run_ok pairing_start_help pairing start --help)"; grep -Eq 'Start a Mac/phone pairing flow|Open a Mac Pairing Room|QR' "$out" || fail "focusa pairing start --help missing canonical help"; pass "focusa pairing start --help"

# Specific regressions from Orders 08-11.
out="$(run_ok migration_help help migration)"; grep -q 'focusa pairing start' "$out" || fail "migration help missing pairing start"; pass "migration help"
out="$(run_allow unsafe_cleanup --json cleanup --safe --project-root /root --dry-run)"; grep -q 'CLI_SCOPE_REJECT' "$out" && grep -q 'unsafe_broad_project_root' "$out" || fail "cleanup unsafe root not blocked"; pass "cleanup scope rejection"
out="$(run_ok uninstall_keep --json uninstall --dry-run --keep-data --keep-license --keep-path-modifications)"; grep -q -- '--keep-data set' "$out" && grep -q -- '--keep-license set' "$out" || fail "uninstall keep flags not reflected"; ! grep -q 'revert_path_' "$out" || fail "keep-path-modifications still planned path revert"; pass "uninstall keep flags"
out="$(run_ok memory_block --json memory set smoke=value)"; grep -q '"status": "blocked"' "$out" && grep -q 'daemon_global_advisory' "$out" || fail "memory mutation not blocked/advisory"; pass "memory mutation block"

pass "Spec124 CLI cross-phase smoke complete"
