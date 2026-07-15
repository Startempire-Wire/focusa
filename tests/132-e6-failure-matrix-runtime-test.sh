#!/usr/bin/env bash
# 132 E6: failure matrix runtime proof harness.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
source "$ROOT/tests/focusa_portable_bin.sh"

if ! command -v jq >/dev/null 2>&1; then
  echo "FAIL: jq is required for JSON command validation" >&2
  exit 1
fi

resolve_tui_binary() {
  local root="$1"
  local -a candidates
  local explicit=0
  local -a tried

  if [[ -n "${FOCUSA_TUI_BIN+x}" ]]; then
    explicit=1
    if [[ -z "$FOCUSA_TUI_BIN" ]]; then
      candidates=("")
    else
      candidates=("$FOCUSA_TUI_BIN")
    fi
  else
    candidates=(
      "$root/target/debug/focusa-tui"
      "$root/target/release/focusa-tui"
    )
  fi

  for binary in "${candidates[@]}"; do
    [[ -n "$binary" ]] || continue
    tried+=("$binary")
    if [[ ! -x "$binary" ]]; then
      continue
    fi

    if focusa_is_host_compatible_binary "$binary"; then
      printf '%s\n' "$binary"
      return 0
    fi
  done

  if [[ "$explicit" -eq 1 ]]; then
    echo "explicit FOCUSA_TUI_BIN is not executable or host-incompatible: ${FOCUSA_TUI_BIN}" >&2
  else
    if [[ ${#tried[@]} -gt 0 ]]; then
      echo "no host-compatible focusa-tui binary found in candidates: ${tried[*]}" >&2
    else
      echo "no candidate focusa-tui binary found under: $root/target/{debug,release}/focusa-tui" >&2
    fi
  fi

  return 1
}

BIN="$(focusa_resolve_test_cli_binary "$ROOT")"
TUI_BIN="$(resolve_tui_binary "$ROOT")"

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

HOST_PROFILE="${FOCUSA_HOST_PROFILE:-$(uname -s)-$(uname -m)-native}"
EVIDENCE_ROOT="${FOCUSA_E6_EVIDENCE_DIR:-$TMP/evidence}"
LOG_DIR="$EVIDENCE_ROOT/logs"
mkdir -p "$LOG_DIR"

ORIGINAL_HOME="${HOME-}"
if [[ -z "${ORIGINAL_HOME}" ]]; then
  ORIGINAL_HOME="$(pwd -P)"
fi
ORIGINAL_CARGO_HOME="${CARGO_HOME-}"
if [[ -z "${ORIGINAL_CARGO_HOME}" ]]; then
  ORIGINAL_CARGO_HOME="${ORIGINAL_HOME}/.cargo"
fi
ORIGINAL_RUSTUP_HOME="${RUSTUP_HOME-}"
if [[ -z "${ORIGINAL_RUSTUP_HOME}" ]]; then
  ORIGINAL_RUSTUP_HOME="${ORIGINAL_HOME}/.rustup"
fi
E6_CARGO_TARGET_DIR="${FOCUSA_E6_CARGO_TARGET_DIR:-$TMP/e6-cargo-target}"
mkdir -p "$E6_CARGO_TARGET_DIR"

EVIDENCE_FILE="$EVIDENCE_ROOT/132-e6-failure-matrix-runtime-proof.md"
TIMESTAMP="$(date -u +"%Y%m%dT%H%M%SZ")"
TARGET_TRIPLE="${FOCUSA_TARGET_TRIPLE:-unknown}"

BIN_VERSION="$(focusa_probe_version "$BIN" 2>/dev/null | head -n 1 || true)"
BIN_IDENTITY="$(focusa_binary_identity "$BIN" 2>/dev/null || true)"
BIN_FILE_INFO="$(file -b "$BIN" 2>/dev/null || echo "file unavailable")"
BIN_SHA256="$(sha256sum "$BIN" 2>/dev/null | awk '{print $1}')"
if [[ -z "$BIN_SHA256" ]]; then
  BIN_SHA256="$(shasum -a 256 "$BIN" 2>/dev/null | awk '{print $1}')"
fi

TUI_VERSION="$($TUI_BIN --version 2>/dev/null | head -n 1 || echo 'unavailable')"
TUI_IDENTITY="$(focusa_binary_identity "$TUI_BIN" 2>/dev/null || echo 'identity unavailable')"
TUI_FILE_INFO="$(file -b "$TUI_BIN" 2>/dev/null || echo 'file unavailable')"
TUI_SHA256="$(sha256sum "$TUI_BIN" 2>/dev/null | awk '{print $1}')"
if [[ -z "$TUI_SHA256" ]]; then
  TUI_SHA256="$(shasum -a 256 "$TUI_BIN" 2>/dev/null | awk '{print $1}')"
fi

cat > "$EVIDENCE_FILE" <<EOF2
# 132-E6 failure matrix runtime proof

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
  local expected_rc="$3"
  local actual_rc="$4"
  local out="$5"
  local err="$6"
  printf '| %s | `%s` | %s | %s | %s | %s |\n' "$label" "$command" "$expected_rc" "$actual_rc" "$out" "$err" >>"$EVIDENCE_FILE"
}

fail_case() {
  local label="$1"
  local reason="$2"
  echo "FAIL: ${label}: ${reason}" >&2
  if [[ -n "${LAST_CASE_STDOUT:-}" && -f "$LAST_CASE_STDOUT" ]]; then
    echo "--- stdout (${LAST_CASE_STDOUT}) ---" >&2
    cat "$LAST_CASE_STDOUT" >&2
  fi
  if [[ -n "${LAST_CASE_STDERR:-}" && -f "$LAST_CASE_STDERR" ]]; then
    echo "--- stderr (${LAST_CASE_STDERR}) ---" >&2
    cat "$LAST_CASE_STDERR" >&2
  fi
  exit 1
}

assert_no_ansi() {
  local label="$1"
  local path="$2"
  if grep -q $'\033' "$path"; then
    fail_case "$label" "ANSI escape bytes found in output"
  fi
}

assert_contains() {
  local label="$1"
  local path="$2"
  shift 2
  local marker
  for marker in "$@"; do
    if ! grep -Fq "$marker" "$path"; then
      fail_case "$label" "expected marker missing: ${marker}"
    fi
  done
}

assert_json_plan() {
  local label="$1"
  local path="$2"
  local install_root="$3"
  jq -e --arg root "$install_root" '
    .target == "linux" and
    .channel == "stable" and
    .license_mode == "missing" and
    .install_root == $root and
    (.assets_planned | type == "array" and length >= 4) and
    has("first_install_walkthrough_v1") and
    has("service_manager_planned") and
    has("symlink_planned") and
    has("shell_rc_plan") and
    has("notes")
  ' "$path" >/dev/null || fail_case "$label" "JSON dry-run contract check failed"
}

assert_install_plan_markers() {
  local label="$1"
  local path="$2"
  assert_no_ansi "$label" "$path"
  assert_contains "$label" "$path" \
    "Focusa install plan (dry-run)" \
    "Target:" \
    "License mode:" \
    "Assets to install:" \
    "Symlink:" \
    "Service manager:"
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

  LAST_CASE_LABEL="$label"
  LAST_CASE_EXPECTED_RC="$expected_rc"
  LAST_CASE_ACTUAL_RC="$rc"
  LAST_CASE_STDOUT="$out"
  LAST_CASE_STDERR="$err"

  append_case "$label" "$cmd_line" "$expected_rc" "$rc" "$out" "$err"
  if [[ "$rc" -ne "$expected_rc" ]]; then
    fail_case "$label" "command failed (exit=${rc}, expected=${expected_rc})"
  fi
}

run_pty_case() {
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
  local script_cmd=""
  local sep=""
  for arg in "${cmd[@]}"; do
    script_cmd+="$sep$(printf '%q' "$arg")"
    sep=" "
  done

  set +e
  HOME="$home_dir" script -qec "$script_cmd" "$out" >/dev/null 2>"$err"
  local rc=$?
  set -e

  LAST_CASE_LABEL="$label"
  LAST_CASE_EXPECTED_RC="$expected_rc"
  LAST_CASE_ACTUAL_RC="$rc"
  LAST_CASE_STDOUT="$out"
  LAST_CASE_STDERR="$err"

  append_case "$label" "$script_cmd" "$expected_rc" "$rc" "$out" "$err"
  if [[ "$rc" -ne "$expected_rc" ]]; then
    fail_case "$label" "PTY command failed (exit=${rc}, expected=${expected_rc})"
  fi
}

run_cargo_case() {
  local label="$1"
  local test_name="$2"
  local expected_marker="$3"
  local home_dir="$4"
  local -a cmd=(
    env
    CARGO_HOME="$ORIGINAL_CARGO_HOME"
    RUSTUP_HOME="$ORIGINAL_RUSTUP_HOME"
    CARGO_TARGET_DIR="$E6_CARGO_TARGET_DIR"
    "${CARGO_PREFIX[@]}" test -p focusa-cli --bin focusa "$test_name" -- --nocapture
  )

  run_case "$label" "$home_dir" "${cmd[@]}"
  if ! grep -Eq '^running [1-9][0-9]* tests?' "$LAST_CASE_STDOUT"; then
    fail_case "$label" "cargo test did not execute matching test case(s)"
  fi
  assert_contains "$label" "$LAST_CASE_STDOUT" "test result: ok." "$test_name" "$expected_marker"
}

run_terminal_ui_case() {
  local label="$1"
  local test_name="$2"
  local home_dir="$3"
  shift 3
  local -a markers=("$@")
  local -a cmd=(
    env
    CARGO_HOME="$ORIGINAL_CARGO_HOME"
    RUSTUP_HOME="$ORIGINAL_RUSTUP_HOME"
    CARGO_TARGET_DIR="$E6_CARGO_TARGET_DIR"
    "${CARGO_PREFIX[@]}" test -p focusa-terminal-ui --test 132-e6-renderer-transcripts "$test_name" -- --nocapture
  )

  run_case "$label" "$home_dir" "${cmd[@]}"
  if ! grep -Eq '^running [1-9][0-9]* tests?' "$LAST_CASE_STDOUT"; then
    fail_case "$label" "cargo test did not execute matching terminal-ui test case(s)"
  fi
  assert_contains "$label" "$LAST_CASE_STDOUT" "test result: ok." "$test_name" "${markers[@]}"
}

if ! command -v script >/dev/null 2>&1; then
  echo "FAIL: script is required for PTY acceptance cases" >&2
  exit 1
fi

CARGO_PREFIX_INPUT="${FOCUSA_E6_CARGO_PREFIX:-cargo +nightly}"
if [[ -z "${CARGO_PREFIX_INPUT//[[:space:]]/}" ]]; then
  CARGO_PREFIX_INPUT='cargo +nightly'
fi
read -r -a CARGO_PREFIX <<< "$CARGO_PREFIX_INPUT"
if [[ -z "${CARGO_PREFIX[*]-}" ]]; then
  echo "FAIL: invalid FOCUSA_E6_CARGO_PREFIX: '${FOCUSA_E6_CARGO_PREFIX}'" >&2
  exit 1
fi
if ! command -v "${CARGO_PREFIX[0]}" >/dev/null 2>&1; then
  echo "FAIL: cargo prefix executable not found: ${CARGO_PREFIX[0]}" >&2
  exit 1
fi

printf '\n| case | command | expected exit | actual exit | stdout | stderr |\n' >>"$EVIDENCE_FILE"
echo "|---|---|---:|---:|---|---|" >>"$EVIDENCE_FILE"

run_pty_case "install-dry-run-pty-truecolor-plan" "$TMP/e6-truecolor/home" 0 \
  TERM=xterm-truecolor COLORTERM=truecolor "$BIN" install --dry-run
assert_install_plan_markers "$LAST_CASE_LABEL" "$LAST_CASE_STDOUT"

run_case "install-dry-run-json" "$TMP/e6-json/home" 0 "$BIN" --json install --dry-run
assert_no_ansi "$LAST_CASE_LABEL" "$LAST_CASE_STDOUT"
assert_json_plan "$LAST_CASE_LABEL" "$LAST_CASE_STDOUT" "$TMP/e6-json/home/.focusa"

run_pty_case "install-dry-run-pty-no-color-plain" "$TMP/e6-nocolor/home" 0 \
  TERM=xterm NO_COLOR=1 FOCUSA_INSTALL_UI=plain "$BIN" install --dry-run
assert_install_plan_markers "$LAST_CASE_LABEL" "$LAST_CASE_STDOUT"

mkdir -p "$TMP/e6-pi-skipped/no-pi" "$TMP/e6-pi-skipped/pi"
run_case "install-dry-run-pi-skipped" "$TMP/e6-pi-skipped/home" 0 \
  env PATH="$TMP/e6-pi-skipped/no-pi" FOCUSA_PI_EXT_DIR="$TMP/e6-pi-skipped/pi" "$BIN" install --dry-run
assert_install_plan_markers "$LAST_CASE_LABEL" "$LAST_CASE_STDOUT"
if grep -qi "pi extension" "$LAST_CASE_STDOUT"; then
  fail_case "$LAST_CASE_LABEL" "non-mutating Pi-skipped dry-run unexpectedly mentioned Pi extension output"
fi

run_cargo_case "cargo-test-pi-activation-success" "pi_extension_archive_install_is_checksum_stage_and_activation_safe" "E6_PI_PRESENT_SUCCESS" "$TMP/e6-cargo/pi-activation"
run_cargo_case "cargo-test-pi-missing" "phase_pi_extension_download_returns_none_when_pi_binary_is_missing" "E6_PI_ABSENT" "$TMP/e6-cargo/pi-missing"
run_cargo_case "cargo-test-pi-malformed-archive" "malformed_pi_extension_archive_rejects_and_keeps_existing_destination" "E6_PI_FAILURE_SAFE" "$TMP/e6-cargo/pi-malformed"
run_cargo_case "cargo-test-checksum-mismatch" "verify_checksum_rejects_mismatched_hash_from_local_http_fixture" "E6_INTEGRITY_FAILURE" "$TMP/e6-cargo/checksum-mismatch"
run_cargo_case "cargo-test-windows-service-warning" "delegate_service_render_windows_target_returns_warning_outcome" "E6_SERVICE_WARNING" "$TMP/e6-cargo/windows-service"
run_cargo_case "cargo-test-atomic-cleanup" "phase_atomic_cleanup_preserves_new_install_content_and_removes_stash" "E6_UPGRADE_CLEANUP" "$TMP/e6-cargo/atomic-cleanup"
run_cargo_case "cargo-test-cancel-rollback" "cancellation_result_restores_stash_with_prior_state_and_emits_rollback_events" "E6_CANCELLATION_ROLLBACK" "$TMP/e6-cargo/cancel-rollback"

run_terminal_ui_case "cargo-test-renderer-truecolor-transcript" \
  "truecolor_transcript_moves_through_health_checks_and_finalize_then_finishes" \
  "$TMP/e6-cargo/renderer-transcripts" \
  "[truecolor transcript]" \
  "✓ Finalize" \
  "phase completion"

run_terminal_ui_case "cargo-test-renderer-monochrome-transcript" \
  "monochrome_transcript_shows_verify_failure_then_rollback_and_keeps_recovery" \
  "$TMP/e6-cargo/renderer-transcripts" \
  "[monochrome transcript - failed pre-rollback]" \
  "[monochrome transcript - rollback started]" \
  "[monochrome transcript - rollback succeeded]" \
  "✗ Verify checksums and trust" \
  "↶ Rolling back safely" \
  "✗ Installation failed"

echo "PASS: 132-E6 failure matrix runtime proof passed"
echo "EVIDENCE_FILE=${EVIDENCE_FILE}"
