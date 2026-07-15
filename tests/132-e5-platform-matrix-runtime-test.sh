#!/usr/bin/env bash
# 132 E5: native runtime matrix proof with CLI/TUI updater contracts.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
source "$ROOT/tests/focusa_portable_bin.sh"

if ! command -v jq >/dev/null 2>&1; then
  echo "FAIL: jq is required for JSON command validation" >&2
  exit 1
fi

BIN="$(focusa_resolve_test_cli_binary "$ROOT")"
TUI_BIN="${FOCUSA_TUI_BIN:-$ROOT/target/debug/focusa-tui}"
if [[ -z "$TUI_BIN" || ! -x "$TUI_BIN" ]]; then
  echo "FAIL: focusa-tui executable missing or non-executable: ${FOCUSA_TUI_BIN:-$TUI_BIN}" >&2
  exit 1
fi

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

HOST_PROFILE="${FOCUSA_HOST_PROFILE:-$(uname -s)-$(uname -m)-native}"
EVIDENCE_ROOT="${FOCUSA_E5_EVIDENCE_DIR:-$TMP/evidence}"
LOG_DIR="$EVIDENCE_ROOT/logs"
mkdir -p "$LOG_DIR"
EVIDENCE_FILE="$EVIDENCE_ROOT/132-e5-platform-matrix-proof.md"
TIMESTAMP="$(date -u +"%Y%m%dT%H%M%SZ")"
TARGET_TRIPLE="${FOCUSA_TARGET_TRIPLE:-unknown}"

BIN_VERSION="$(focusa_probe_version "$BIN" 2>/dev/null | head -n 1 || true)"
BIN_IDENTITY="$(focusa_binary_identity "$BIN" 2>/dev/null || true)"
BIN_FILE_INFO="$(file -b "$BIN" 2>/dev/null || echo "file unavailable")"
BIN_SHA256="$(sha256sum "$BIN" 2>/dev/null | awk '{print $1}')"
if [[ -z "$BIN_SHA256" ]]; then
  BIN_SHA256="$(shasum -a 256 "$BIN" 2>/dev/null | awk '{print $1}')"
fi

TUI_VERSION="$("$TUI_BIN" --version 2>/dev/null | head -n 1 || echo 'unavailable')"
TUI_IDENTITY="$(stat -Lc '%d:%i %h %s %y %n' "$TUI_BIN" 2>/dev/null || stat -f '%d:%i %h %s %m %N' "$TUI_BIN" 2>/dev/null || echo 'identity unavailable')"
TUI_FILE_INFO="$(file -b "$TUI_BIN" 2>/dev/null || echo 'file unavailable')"
TUI_SHA256="$(sha256sum "$TUI_BIN" 2>/dev/null | awk '{print $1}')"
if [[ -z "$TUI_SHA256" ]]; then
  TUI_SHA256="$(shasum -a 256 "$TUI_BIN" 2>/dev/null | awk '{print $1}')"
fi

cat > "$EVIDENCE_FILE" <<EOF2
# 132-E5 platform matrix runtime proof

Timestamp: $TIMESTAMP
Host profile: $HOST_PROFILE
Configured target triple: $TARGET_TRIPLE

CLI binary path:
$BIN
CLI binary version: ${BIN_VERSION:-unavailable}
CLI binary file identity: ${BIN_IDENTITY:-unavailable}
CLI binary file detail: ${BIN_FILE_INFO:-unavailable}
CLI binary sha256: ${BIN_SHA256:-unavailable}

TUI binary path:
$TUI_BIN
TUI binary version: ${TUI_VERSION}
TUI binary file identity: ${TUI_IDENTITY}
TUI binary file detail: ${TUI_FILE_INFO}
TUI binary sha256: ${TUI_SHA256:-unavailable}
EOF2

append_case() {
  local label="$1"
  local command="$2"
  local exit_code="$3"
  local out="$4"
  local err="$5"
  printf '| %s | `%s` | %s | %s | %s |\n' "$label" "$command" "$exit_code" "$out" "$err" >>"$EVIDENCE_FILE"
}

run_case() {
  local label="$1"
  local home_dir="$2"
  local expected_rc=0
  local -a cmd

  if [[ "${3:-}" =~ ^[0-9]+$ ]]; then
    expected_rc="$3"
    shift 3
  else
    shift 2
  fi
  cmd=("$@")

  local out="$LOG_DIR/$label.out"
  local err="$LOG_DIR/$label.err"
  mkdir -p "$home_dir"
  local cmd_line=""
  local sep=""
  for arg in "${cmd[@]}"; do
    cmd_line+="$sep$(printf '%q' "$arg")"
    sep=" "
  done

  set +e
  HOME="$home_dir" "${cmd[@]}" >"$out" 2>"$err"
  local rc=$?
  set -e

  LAST_CASE_RC=$rc
  LAST_CASE_STDOUT="$out"
  LAST_CASE_STDERR="$err"

  append_case "$label" "$cmd_line" "$rc" "$out" "$err"
  if [[ "$rc" -ne "$expected_rc" ]]; then
    echo "FAIL: ${label} command failed (exit=$rc, expected=$expected_rc)" >&2
    cat "$out" "$err" >&2
    exit 1
  fi

  if [[ "$label" == *json* || "$label" == *headless-self-test* ]]; then
    ! grep -q $'\033' "$out" || {
      echo "FAIL: ${label} emitted ANSI control bytes" >&2
      cat "$out" >&2
      exit 1
    }
  fi

  case "$label" in
    focusa-install-plain|focusa-install-no-color|focusa-install-no-animation)
      jq -e '.schema == "focusa.install_preflight.v1" and .read_only == true and .mutations_performed == false' "$out" >/dev/null
      ;;
    focusa-update-status)
      jq -e '.schema == "focusa.update_inventory.v1" and .read_only == true' "$out" >/dev/null
      ;;
    focusa-update-plan)
      jq -e '.schema == "focusa.update_plan.v1" and .read_only == true and .mutations_performed == false' "$out" >/dev/null
      ;;
    tui-headless-self-test)
      jq -e '.schema == "focusa.tui_headless_self_test.v1" and has("about_version")' "$out" >/dev/null
      ;;
  esac
}

printf '\n| case | command | exit | stdout | stderr |\n' >>"$EVIDENCE_FILE"
echo "|---|---|---:|---|---|" >>"$EVIDENCE_FILE"

run_case focusa-install-plain "$TMP/focusa-install-plain/home" env CI=1 TERM=dumb NO_COLOR=1 "$BIN" --json install --preflight --quiet --no-animation
run_case focusa-install-no-color "$TMP/focusa-install-no-color/home" env TERM=xterm NO_COLOR=1 FOCUSA_REDUCE_MOTION=1 "$BIN" --json install --preflight --quiet
run_case focusa-install-no-animation "$TMP/focusa-install-no-animation/home" env TERM=xterm FOCUSA_INSTALL_UI=plain "$BIN" --json install --preflight --quiet
run_case focusa-update-status "$TMP/focusa-update-status/home" env FOCUSA_LATEST_VERSION=0.9.99-dev "$BIN" --json update status --latest-version 0.9.99-dev
run_case focusa-update-plan "$TMP/focusa-update-plan/home" env FOCUSA_LATEST_VERSION=0.9.99-dev "$BIN" --json update plan --latest-version 0.9.99-dev

run_case tui-version "$TMP/tui-version/home" "$TUI_BIN" --version
run_case tui-headless-self-test "$TMP/tui-headless-self-test/home" "$TUI_BIN" --headless-self-test
run_case tui-ordinary-launch-fail-fast "$TMP/tui-ordinary-launch-fail-fast/home" 64 "$TUI_BIN"

if ! grep -q 'FOCUSA_TUI_NON_TTY' "$LAST_CASE_STDERR"; then
  echo "FAIL: focusa-tui fail-fast exit did not emit expected diagnostic marker" >&2
  cat "$LAST_CASE_STDOUT" "$LAST_CASE_STDERR" >&2
  exit 1
fi
if ! grep -q 'focusa tui --headless-self-test' "$LAST_CASE_STDERR"; then
  echo "FAIL: focusa-tui fail-fast exit did not include recovery guidance" >&2
  cat "$LAST_CASE_STDOUT" "$LAST_CASE_STDERR" >&2
  exit 1
fi

echo "PASS: CLI and focusa-tui proof ran across installer/update and interactive guard contracts"
echo "EVIDENCE_FILE=${EVIDENCE_FILE}"
