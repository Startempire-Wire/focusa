#!/usr/bin/env bash
# Focusa deploy smoke check.
#
# Verifies that an installed daemon matches the expected deployment
# state without mutating it. Intended for CI self-healing hooks and
# post-deploy verification.
set -euo pipefail

SERVICE_NAME="${FOCUSA_SERVICE_NAME:-focusa-daemon}"
BIN_NAME="${FOCUSA_BIN_NAME:-focusa-daemon}"
HEALTH_URL="${FOCUSA_DAEMON_URL:-http://127.0.0.1:8787/v1/health}"
EXPECTED_VERSION="${FOCUSA_EXPECTED_VERSION:-}"
AUDIT_LOG="${FOCUSA_DEPLOY_AUDIT_LOG:-/var/log/focusa/deploy-audit.jsonl}"
ALLOW_LOOPBACK=1

usage() {
  cat <<'USAGE'
Usage: scripts/deploy-smoke-check.sh [options]

Options:
  --expected-version VER   Require /v1/health version to match VER.
  --service-name NAME      systemd service base name (default focusa-daemon).
  --bin-name NAME          Process binary name (default focusa-daemon).
  --health-url URL         Health endpoint (default $FOCUSA_DAEMON_URL or http://127.0.0.1:8787/v1/health).
  --audit-log PATH         Append-only audit log path.
  --no-loopback            Refuse loopback endpoints (require a real public host).
  --help                   Show help.
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --expected-version) EXPECTED_VERSION="${2:?}"; shift 2 ;;
    --service-name) SERVICE_NAME="${2:?}"; shift 2 ;;
    --bin-name) BIN_NAME="${2:?}"; shift 2 ;;
    --health-url) HEALTH_URL="${2:?}"; shift 2 ;;
    --audit-log) AUDIT_LOG="${2:?}"; shift 2 ;;
    --no-loopback) ALLOW_LOOPBACK=0; shift ;;
    --help|-h) usage; exit 0 ;;
    *) printf 'unknown arg: %s\n' "$1" >&2; usage; exit 64 ;;
  esac
done

mkdir -p "$(dirname "$AUDIT_LOG")"

audit_event() {
  local event="$1"
  local outcome="$2"
  local note="${3:-}"
  local live="${4:-}"
  python3 - "$AUDIT_LOG" "$event" "$outcome" "$note" "$live" <<'PY'
import json, sys, time
path, event, outcome, note, live = sys.argv[1:6]
entry = {
    "ts": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
    "event": event,
    "outcome": outcome,
    "note": note,
    "live_version": live,
    "expected_version": __import__("os").environ.get("EXPECTED_VERSION", ""),
    "service_name": __import__("os").environ.get("SERVICE_NAME", ""),
    "health_url": __import__("os").environ.get("HEALTH_URL", ""),
}
with open(path, "a", encoding="utf-8") as fh:
    fh.write(json.dumps(entry, separators=(",", ":")) + "\n")
PY
}

check_failures=0
record_failure() {
  check_failances="${check_failures:-0}"
  check_failures=$((check_failures + 1))
  record_failure_event "$@"
}

record_failure_event() {
  local check="$1"
  local note="$2"
  audit_event "smoke_check" "failed" "${check}: ${note}" ""
}

record_success_event() {
  local check="$1"
  local note="$2"
  audit_event "smoke_check" "passed" "${check}: ${note}" "${live_version:-}"
}

if [[ "$ALLOW_LOOPBACK" -eq 0 ]]; then
  case "$HEALTH_URL" in
    http://localhost*|http://127.0.0.1*|http://::1*|http://[::1]*)
      audit_event "smoke_check" "failed" "loopback health url not allowed" ""
      echo "✗ loopback health url not allowed: $HEALTH_URL" >&2
      exit 1
      ;;
  esac
fi

if ! systemctl is-active --quiet "${SERVICE_NAME}.service"; then
  record_failure_event "service_active" "${SERVICE_NAME}.service not active"
  echo "✗ service not active: ${SERVICE_NAME}.service"
  check_failures=$((check_failures + 1))
else
  record_success_event "service_active" "${SERVICE_NAME}.service is active"
fi

live_pids="$(pgrep -x "$BIN_NAME" || true)"
pid_count=$(printf '%s\n' "$live_pids" | wc -l | tr -d ' ')
if [[ -z "$live_pids" ]]; then
  record_failure_event "process_count" "$BIN_NAME not running"
  echo "✗ daemon process not running"
  check_failures=$((check_failures + 1))
elif [[ "$pid_count" -gt 1 ]]; then
  record_failure_event "process_count" "duplicate ${BIN_NAME}: ${pid_count} pids"
  echo "✗ duplicate daemon pids: $pid_count"
  check_failures=$((check_failures + 1))
else
  record_success_event "process_count" "single pid=$live_pids"
fi

health_payload=""
if ! health_payload="$(curl -fsS --max-time 10 "$HEALTH_URL" 2>/dev/null)"; then
  record_failure_event "health_endpoint" "$HEALTH_URL unreachable"
  echo "✗ health endpoint unreachable: $HEALTH_URL"
  check_failures=$((check_failures + 1))
else
  live_version="$(printf '%s' "$health_payload" | python3 -c 'import json,sys; print(json.load(sys.stdin).get("version",""))')"
  if [[ -n "$EXPECTED_VERSION" && "$live_version" != "$EXPECTED_VERSION" ]]; then
    record_failure_event "version_match" "expected=$EXPECTED_VERSION live=$live_version"
    echo "✗ version mismatch: expected $EXPECTED_VERSION, got $live_version"
    check_failures=$((check_failures + 1))
  else
    record_success_event "version_match" "live=$live_version" "$live_version"
    echo "✓ version matches: $live_version"
  fi
fi

if (( check_failures > 0 )); then
  echo "smoke check: FAILED ($check_failures failure(s))"
  exit 1
fi
echo "smoke check: OK"
exit 0
