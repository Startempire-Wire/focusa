#!/usr/bin/env bash
# Focusa daemon installer / deployer.
#
# Supports local installs from a checkout build or from a release artifact,
# enforces a deploy lock, kills stray duplicate daemons, backs up the current
# binary, verifies health/version, and rolls back automatically on failure.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
INSTALL_ROOT="${FOCUSA_INSTALL_ROOT:-/usr/local}"
BIN_NAME="${FOCUSA_BIN_NAME:-focusa-daemon}"
SERVICE_NAME="${FOCUSA_SERVICE_NAME:-focusa-daemon}"
HEALTH_URL_DEFAULT="${FOCUSA_DAEMON_URL:-http://127.0.0.1:8787}"
EXPECTED_VERSION="${FOCUSA_EXPECTED_VERSION:-}"
BINARY=""
NO_RESTART=0
NO_VERIFY=0
REQUIRE_SERVICE=0
LOCK_FILE="${FOCUSA_DEPLOY_LOCK_FILE:-/tmp/focusa-daemon-deploy.lock}"
STATE_DIR="${FOCUSA_STATE_DIR:-${INSTALL_ROOT}/lib/focusa}"
BACKUP_DIR="${FOCUSA_BACKUP_DIR:-${STATE_DIR}/backups}"
HEALTH_URL=""

log() { printf '[focusa-deploy] %s\n' "$*"; }
warn() { printf '[focusa-deploy][warn] %s\n' "$*" >&2; }
die() { printf '[focusa-deploy][error] %s\n' "$*" >&2; exit 1; }

usage() {
  cat <<'USAGE'
Usage: scripts/install-daemon.sh [options]

Options:
  --binary PATH            Install this binary instead of auto-detecting a local build.
  --expected-version VER   Require binary and /v1/health version to match VER.
  --install-root PATH      Install root (default: /usr/local).
  --bin-name NAME          Installed binary name (default: focusa-daemon).
  --service-name NAME      systemd service base name (default: focusa-daemon).
  --health-url URL         Health endpoint or base URL (default: $FOCUSA_DAEMON_URL or http://127.0.0.1:8787).
  --require-service        Fail if the systemd service does not exist.
  --no-restart             Install only; do not restart the service.
  --no-verify              Skip curl health verification.
  --help                   Show this help.

Environment overrides:
  FOCUSA_INSTALL_ROOT, FOCUSA_BIN_NAME, FOCUSA_SERVICE_NAME,
  FOCUSA_DAEMON_URL, FOCUSA_EXPECTED_VERSION, FOCUSA_DEPLOY_LOCK_FILE,
  FOCUSA_STATE_DIR, FOCUSA_BACKUP_DIR
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --binary)
      BINARY="${2:?--binary requires a path}"
      shift 2
      ;;
    --expected-version)
      EXPECTED_VERSION="${2:?--expected-version requires a version}"
      shift 2
      ;;
    --install-root)
      INSTALL_ROOT="${2:?--install-root requires a path}"
      shift 2
      ;;
    --bin-name)
      BIN_NAME="${2:?--bin-name requires a value}"
      shift 2
      ;;
    --service-name)
      SERVICE_NAME="${2:?--service-name requires a value}"
      shift 2
      ;;
    --health-url)
      HEALTH_URL="${2:?--health-url requires a value}"
      shift 2
      ;;
    --require-service)
      REQUIRE_SERVICE=1
      shift
      ;;
    --no-restart)
      NO_RESTART=1
      shift
      ;;
    --no-verify)
      NO_VERIFY=1
      shift
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      die "Unknown argument: $1"
      ;;
  esac
done

normalize_health_url() {
  local value="$1"
  if [[ "$value" == */v1/health ]]; then
    printf '%s\n' "$value"
  else
    printf '%s/v1/health\n' "${value%/}"
  fi
}

HEALTH_URL="$(normalize_health_url "${HEALTH_URL:-$HEALTH_URL_DEFAULT}")"
INSTALL_PATH="${INSTALL_ROOT}/bin/${BIN_NAME}"
SERVICE_UNIT="${SERVICE_NAME}.service"

mkdir -p "$(dirname "$LOCK_FILE")"
exec 9>"$LOCK_FILE"
if ! flock -n 9; then
  die "another deploy is already running (lock: $LOCK_FILE)"
fi

mkdir -p "$BACKUP_DIR" "$STATE_DIR" "$(dirname "$INSTALL_PATH")"

have_cmd() { command -v "$1" >/dev/null 2>&1; }

binary_version() {
  local path="$1"
  local out=""
  out="$($path --version 2>/dev/null || true)"
  if [[ -z "$out" ]]; then
    printf '\n'
    return 0
  fi
  printf '%s\n' "$out" | grep -oE '[0-9]+\.[0-9]+\.[0-9]+([-.+][0-9A-Za-z._-]+)?' | head -1 || true
}

json_field() {
  local field="$1"
  python3 - "$field" <<'PY'
import json, sys
field = sys.argv[1]
try:
    payload = json.load(sys.stdin)
except Exception:
    sys.exit(1)
value = payload.get(field, "")
print(value if value is not None else "")
PY
}

service_exists() {
  systemctl list-unit-files "$SERVICE_UNIT" >/dev/null 2>&1
}

stop_service_and_strays() {
  if service_exists; then
    log "stopping service $SERVICE_UNIT"
    systemctl stop "$SERVICE_UNIT" || true
    sleep 2
  fi

  local pids=""
  pids="$(pgrep -x "$BIN_NAME" || true)"
  if [[ -n "$pids" ]]; then
    warn "found stray ${BIN_NAME} pid(s): $(tr '\n' ' ' <<<"$pids")"
    kill -TERM $pids || true
    sleep 2
  fi

  pids="$(pgrep -x "$BIN_NAME" || true)"
  if [[ -n "$pids" ]]; then
    warn "forcing remaining ${BIN_NAME} pid(s) down: $(tr '\n' ' ' <<<"$pids")"
    kill -KILL $pids || true
    sleep 1
  fi
}

start_service() {
  if ! service_exists; then
    if [[ "$REQUIRE_SERVICE" -eq 1 ]]; then
      die "required service missing: $SERVICE_UNIT"
    fi
    warn "service $SERVICE_UNIT not found; install complete but no restart performed"
    return 0
  fi
  systemctl daemon-reload
  systemctl start "$SERVICE_UNIT"
}

assert_single_process() {
  local count
  count="$(pgrep -x "$BIN_NAME" | wc -l | tr -d ' ')"
  if [[ "$count" -gt 1 ]]; then
    die "duplicate ${BIN_NAME} processes detected after restart: $count"
  fi
}

wait_for_health() {
  local attempts="${1:-30}"
  local expected_version="${2:-}"
  local payload version
  for _ in $(seq 1 "$attempts"); do
    if payload="$(curl -fsS "$HEALTH_URL" 2>/dev/null)"; then
      if [[ -n "$expected_version" ]]; then
        version="$(printf '%s' "$payload" | json_field version || true)"
        if [[ "$version" != "$expected_version" ]]; then
          sleep 1
          continue
        fi
      fi
      printf '%s\n' "$payload"
      return 0
    fi
    sleep 1
  done
  return 1
}

if [[ -z "$BINARY" ]]; then
  if [[ -x "$ROOT_DIR/target/release/$BIN_NAME" ]]; then
    BINARY="$ROOT_DIR/target/release/$BIN_NAME"
  elif [[ -x "$ROOT_DIR/target/debug/$BIN_NAME" ]]; then
    BINARY="$ROOT_DIR/target/debug/$BIN_NAME"
    warn "using debug binary (run cargo build --release for production)"
  else
    log "building daemon (debug fallback)"
    export PATH="/root/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin:$PATH"
    cd "$ROOT_DIR"
    cargo build -p focusa-api --bin "$BIN_NAME"
    BINARY="$ROOT_DIR/target/debug/$BIN_NAME"
  fi
fi

[[ -x "$BINARY" ]] || die "binary not executable: $BINARY"

if [[ -n "$EXPECTED_VERSION" ]]; then
  candidate_version="$(binary_version "$BINARY")"
  if [[ -n "$candidate_version" && "$candidate_version" != "$EXPECTED_VERSION" ]]; then
    die "binary version mismatch: expected $EXPECTED_VERSION, got $candidate_version from $BINARY --version"
  fi
fi

log "installing $BIN_NAME from $BINARY to $INSTALL_PATH"
BACKUP_PATH=""
if [[ -f "$INSTALL_PATH" ]]; then
  stamp="$(date +%Y%m%d-%H%M%S)"
  BACKUP_PATH="$BACKUP_DIR/${BIN_NAME}.${stamp}.bak"
  cp -p "$INSTALL_PATH" "$BACKUP_PATH"
  ln -sfn "$BACKUP_PATH" "$STATE_DIR/${BIN_NAME}.previous"
  log "backup saved to $BACKUP_PATH"
fi

rollback() {
  local why="$1"
  warn "$why"
  if [[ -z "$BACKUP_PATH" || ! -f "$BACKUP_PATH" ]]; then
    die "rollback unavailable; no prior binary backup exists"
  fi
  warn "rolling back to $BACKUP_PATH"
  stop_service_and_strays
  install -m 0755 "$BACKUP_PATH" "$INSTALL_PATH"
  if [[ "$NO_RESTART" -eq 0 ]]; then
    start_service
    if [[ "$NO_VERIFY" -eq 0 ]]; then
      wait_for_health 20 "" >/dev/null || die "rollback restart failed; daemon still unhealthy"
    fi
    assert_single_process
  fi
  die "deploy failed and rollback was applied"
}

stop_service_and_strays
install -m 0755 "$BINARY" "$INSTALL_PATH.new"
mv "$INSTALL_PATH.new" "$INSTALL_PATH"
echo "${EXPECTED_VERSION:-unknown}" > "$STATE_DIR/live-version"

if [[ "$NO_RESTART" -eq 1 ]]; then
  log "installed without restart (--no-restart)"
  exit 0
fi

if ! start_service; then
  rollback "service start failed"
fi
assert_single_process

if [[ "$NO_VERIFY" -eq 0 ]]; then
  payload="$(wait_for_health 30 "$EXPECTED_VERSION" || true)"
  if [[ -z "$payload" ]]; then
    rollback "health verification failed for $HEALTH_URL"
  fi
  version="$(printf '%s' "$payload" | json_field version || true)"
  if [[ -n "$EXPECTED_VERSION" && "$version" != "$EXPECTED_VERSION" ]]; then
    rollback "health version mismatch: expected $EXPECTED_VERSION, got ${version:-<empty>}"
  fi
  log "daemon healthy at $HEALTH_URL"
  if [[ -n "$version" ]]; then
    log "live version=$version"
  fi
fi

log "deploy complete"
