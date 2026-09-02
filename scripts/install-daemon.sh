#!/usr/bin/env bash
# Focusa daemon installer / deployer.
#
# Supports local installs from a checkout build or from a release artifact,
# enforces a deploy lock, kills stray duplicate daemons, backs up the current
# binary, verifies health/version, and rolls back automatically on failure.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
# shellcheck source=scripts/lib/release-version.sh
source "$ROOT_DIR/scripts/lib/release-version.sh"
INSTALL_ROOT="${FOCUSA_INSTALL_ROOT:-/usr/local}"
BIN_NAME="${FOCUSA_BIN_NAME:-focusa-daemon}"
SERVICE_NAME="${FOCUSA_SERVICE_NAME:-focusa-daemon}"
HEALTH_URL_DEFAULT="${FOCUSA_DAEMON_URL:-http://127.0.0.1:8787}"
EXPECTED_VERSION="${FOCUSA_EXPECTED_VERSION:-}"
BINARY=""
CURRENT_VERSION=""
CURRENT_CHECKSUM=""
NEW_CHECKSUM=""
NO_RESTART=0
NO_VERIFY=0
REQUIRE_SERVICE=0
LOCK_FILE="${FOCUSA_DEPLOY_LOCK_FILE:-/tmp/focusa-daemon-deploy.lock}"
STATE_DIR="${FOCUSA_STATE_DIR:-${INSTALL_ROOT}/lib/focusa}"
BACKUP_DIR="${FOCUSA_BACKUP_DIR:-${STATE_DIR}/backups}"
AUDIT_LOG="${FOCUSA_DEPLOY_AUDIT_LOG:-/var/log/focusa/deploy-audit.jsonl}"
GITHUB_RUN_ID="${FOCUSA_GITHUB_RUN_ID:-}"
GITHUB_RUN_ATTEMPT="${FOCUSA_GITHUB_RUN_ATTEMPT:-}"
GITHUB_ACTOR="${FOCUSA_GITHUB_ACTOR:-}"
GITHUB_SHA="${FOCUSA_GITHUB_SHA:-}"
GITHUB_TAG="${FOCUSA_GITHUB_TAG:-}"
GITHUB_WORKFLOW="${FOCUSA_GITHUB_WORKFLOW:-}"
GITHUB_REPOSITORY="${FOCUSA_GITHUB_REPOSITORY:-}"
HEALTH_URL=""

log() { printf '[focusa-deploy] %s\n' "$*"; }
warn() { printf '[focusa-deploy][warn] %s\n' "$*" >&2; }
die() { printf '[focusa-deploy][error] %s\n' "$*" >&2; exit 1; }

audit_event() {
  local event="$1"
  local outcome="$2"
  local note="${3:-}"
  local live_version="${4:-}"
  mkdir -p "$(dirname "$AUDIT_LOG")"
  python3 - "$AUDIT_LOG" "$event" "$outcome" "$note" "$live_version" <<'PY'
import json, os, sys, time
path, event, outcome, note, live_version = sys.argv[1:6]
entry = {
    "ts": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
    "event": event,
    "outcome": outcome,
    "note": note,
    "expected_version": os.environ.get("EXPECTED_VERSION", ""),
    "live_version": live_version,
    "install_root": os.environ.get("INSTALL_ROOT", ""),
    "install_path": os.environ.get("INSTALL_PATH", ""),
    "service_name": os.environ.get("SERVICE_NAME", ""),
    "health_url": os.environ.get("HEALTH_URL", ""),
    "backup_path": os.environ.get("BACKUP_PATH", ""),
    "github": {
        "run_id": os.environ.get("GITHUB_RUN_ID", ""),
        "run_attempt": os.environ.get("GITHUB_RUN_ATTEMPT", ""),
        "actor": os.environ.get("GITHUB_ACTOR", ""),
        "sha": os.environ.get("GITHUB_SHA", ""),
        "tag": os.environ.get("GITHUB_TAG", ""),
        "workflow": os.environ.get("GITHUB_WORKFLOW", ""),
        "repository": os.environ.get("GITHUB_REPOSITORY", ""),
    },
}
with open(path, "a", encoding="utf-8") as fh:
    fh.write(json.dumps(entry, separators=(",", ":")) + "\n")
PY
}

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
audit_event "deploy_start" "started" "installer invoked"

# ---- Self-healing safety net -----------------------------------------
# Two failure classes have shown up in production:
#   1) runner OOM kill (exit 137) after >20 minutes of apparent hang
#   2) the script wedging in wait_for_health curl loop
# Both are caught below by:
#   - a memory watchdog that polls our own RSS every 5s; if it
#     crosses RSS_LIMIT_MB we audit + die (the runner will see the
#     audit row and the auto-retry workflow can re-dispatch)
#   - a wall-clock budget via DEPLOY_WALL_CLOCK_SEC (default 600) that
#     aborts the deploy if the script runs that long without reaching
#     deploy_complete
WALL_CLOCK_SEC="${FOCUSA_DEPLOY_WALL_CLOCK_SEC:-600}"
RSS_LIMIT_MB="${FOCUSA_DEPLOY_RSS_LIMIT_MB:-768}"
SCRIPT_START_EPOCH="$(date +%s)"

watchdog_check() {
  set +e
  local now elapsed rss_kb rss_mb pid
  pid="$$"
  now="$(date +%s)"
  elapsed=$(( now - SCRIPT_START_EPOCH ))
  if (( elapsed > WALL_CLOCK_SEC )); then
    audit_event "deploy_oom_killed" "timeout" "wall clock exceeded ${WALL_CLOCK_SEC}s at elapsed=${elapsed}s"
    die "wall clock budget exceeded (${WALL_CLOCK_SEC}s); aborting deploy"
  fi
  if have_cmd ps; then
    rss_kb="$(ps -o rss= -p "$pid" 2>/dev/null | tr -d ' ' || true)"
    if [[ -n "$rss_kb" ]]; then
      rss_mb=$(( rss_kb / 1024 ))
      if (( rss_mb > RSS_LIMIT_MB )); then
        audit_event "deploy_oom_killed" "rss_exceeded" "RSS=${rss_mb}MB exceeds limit=${RSS_LIMIT_MB}MB at elapsed=${elapsed}s"
        die "memory budget exceeded (${rss_mb}MB > ${RSS_LIMIT_MB}MB); aborting deploy"
      fi
    fi
  fi
  set -e
}

# Background watchdog: poll every 5s. The trap below makes sure it is
# cleaned up on exit.
watchdog_loop() {
  while :; do
    sleep 5
    watchdog_check || {
      audit_event "deploy_oom_killed" "watchdog_exit" "watchdog_check died at $(date -u +%s); exit=$?"
      kill -TERM "$$" 2>/dev/null || true
      exit 1
    }
  done
}
watchdog_loop >/dev/null 2>&1 &
WATCHDOG_PID=$!
trap 'kill "$WATCHDOG_PID" 2>/dev/null || true' EXIT
# ---- end self-healing safety net -------------------------------------

have_cmd() { command -v "$1" >/dev/null 2>&1; }

# Extract version from a binary without actually executing it.
#
# Invoking the daemon with `--version` is dangerous: when the binary is
# dynamically linked and incompatible (AlmaLinux 8 + Ubuntu-built gnu),
# --version segfaults inside libc and leaves zombie processes that
# pin port 8787, breaking the systemd service and the next install.
#
# Preferred: parse the canonical release-asset filename
# (`focusa-daemon-v0.9.42-dev-x86_64-unknown-linux-musl`). Fallback:
# run the binary under `timeout 3` and parse the first vX.Y.Z token.
binary_version() {
  local path="$1"
  local base=""
  base="$(basename "$path")"
  # The target-triple boundary is explicit. Stable tags must not consume the
  # first character of `-x86_64` as a prerelease suffix.
  local from_name=""
  from_name="$(release_version_from_asset_name "$path" "$EXPECTED_VERSION")"
  if [[ -n "$from_name" ]]; then
    printf '%s\n' "$from_name"
    return 0
  fi
  # Fallback: actually run --version, but with a hard 3s timeout so
  # we never wedge here even if the binary is broken.
  local out=""
  out="$(timeout 3 "$path" --version 2>/dev/null || true)"
  if [[ -z "$out" ]]; then
    printf '\n'
    return 0
  fi
  printf '%s\n' "$out" | grep -oE '[0-9]+\.[0-9]+\.[0-9]+([-.+][0-9A-Za-z._-]+)?' | head -1 || true
}

binary_checksum() {
  local path="$1"
  if have_cmd sha256sum; then
    sha256sum "$path" | awk '{print $1}'
  elif have_cmd shasum; then
    shasum -a 256 "$path" | awk '{print $1}'
  else
    printf '\n'
  fi
}

json_field() {
  local field="$1"
  local code='import json,sys; d=json.load(sys.stdin); print(d.get(sys.argv[1],"") if d.get(sys.argv[1]) is not None else "")'
  python3 -c "$code" "$field" 2>/dev/null || true
}

service_exists() {
  systemctl list-unit-files "$SERVICE_UNIT" >/dev/null 2>&1
}

validate_service_execstart() {
  service_exists || return 0
  local execstart=""
  execstart="$(systemctl show -p ExecStart --value "$SERVICE_UNIT" 2>/dev/null || true)"
  if [[ -z "$execstart" ]]; then
    warn "could not read ExecStart for $SERVICE_UNIT"
    return 0
  fi
  if [[ "$execstart" != *"$INSTALL_PATH"* ]]; then
    warn "service ExecStart=$execstart does not reference $INSTALL_PATH; auto-patching unit"
    if ! patch_service_unit_execstart; then
      audit_event "deploy_preflight" "failed" "service ExecStart does not reference install path: $INSTALL_PATH"
      die "service ExecStart mismatch for $SERVICE_UNIT; expected reference to $INSTALL_PATH"
    fi
    audit_event "deploy_preflight" "patched" "service ExecStart rewritten to $INSTALL_PATH"
    # Reload so the next start_service picks up the new ExecStart.
    sudo -n systemctl daemon-reload 2>/dev/null || systemctl daemon-reload 2>/dev/null || true
  fi
}

# Patch the installed systemd unit so ExecStart points at the canonical
# install path. Handles two failure modes that hit production deploys:
#   1) Unit references the in-tree build artifact (target/release/...) which
#      gets pruned by safe-disk-cleanup.
#   2) Unit was created against a previous install location.
patch_service_unit_execstart() {
  set +e
  local unit_path="/etc/systemd/system/${SERVICE_UNIT}"
  if [[ ! -f "$unit_path" ]]; then
    warn "systemd unit file not found at $unit_path; cannot auto-patch"
    return 1
  fi
  # Replace any existing ExecStart= line; if none exists, insert one
  # directly under [Service].
  if grep -Eq '^[[:space:]]*ExecStart=' "$unit_path"; then
    sudo -n sed -i -E "s|^[[:space:]]*ExecStart=.*|ExecStart=${INSTALL_PATH}|" "$unit_path" || \
      sed -i -E "s|^[[:space:]]*ExecStart=.*|ExecStart=${INSTALL_PATH}|" "$unit_path"
  else
    sudo -n sed -i -E "/^[[:space:]]*\\[Service\\]/a ExecStart=${INSTALL_PATH}" "$unit_path" || \
      sed -i -E "/^[[:space:]]*\\[Service\\]/a ExecStart=${INSTALL_PATH}" "$unit_path"
  fi
  # Also align WorkingDirectory and ReadWritePaths to canonical install root
  # when they reference the in-tree path.
  sudo -n sed -i -E "s|^[[:space:]]*WorkingDirectory=.*|WorkingDirectory=${STATE_DIR}|" "$unit_path" || \
    sed -i -E "s|^[[:space:]]*WorkingDirectory=.*|WorkingDirectory=${STATE_DIR}|" "$unit_path"
  set -e
  log "patched $unit_path: ExecStart=$INSTALL_PATH WorkingDirectory=$STATE_DIR"
  return 0
}

stop_service_and_strays() {
  # V2 deploy: explicitly disable set -e for the kill block so a single
  # permission error or missing pid does not abort the whole deploy.
  set +e
  if service_exists; then
    log "stopping service $SERVICE_UNIT"
    sudo -n systemctl stop "$SERVICE_UNIT" 2>/dev/null || systemctl stop "$SERVICE_UNIT"
  fi

  local pids=""
  pids="$(pgrep -x "$BIN_NAME" || true)"
  if [[ -n "$pids" ]]; then
    warn "found stray ${BIN_NAME} pid(s): $(tr '\n' ' ' <<<"$pids")"
    # V3 deploy: use systemctl kill to signal stray processes without
    # triggering Restart=always. Raw kill would cause systemd to see
    # an unexpected exit and restart the daemon, creating a race with
    # binary installation.
    sudo -n systemctl kill -s SIGTERM "$SERVICE_UNIT" 2>/dev/null
    systemctl kill -s SIGTERM "$SERVICE_UNIT" 2>/dev/null
    true
    sleep 2
  fi

  pids="$(pgrep -x "$BIN_NAME" || true)"
  if [[ -n "$pids" ]]; then
    warn "forcing remaining ${BIN_NAME} pid(s) down: $(tr '\n' ' ' <<<"$pids")"
    sudo -n systemctl kill -s SIGKILL "$SERVICE_UNIT" 2>/dev/null
    systemctl kill -s SIGKILL "$SERVICE_UNIT" 2>/dev/null
    true
    sleep 1
  fi
  set -e
}

start_service() {
  if ! service_exists; then
    if [[ "$REQUIRE_SERVICE" -eq 1 ]]; then
      die "required service missing: $SERVICE_UNIT"
    fi
    warn "service $SERVICE_UNIT not found; install complete but no restart performed"
    return 0
  fi
  sudo -n systemctl daemon-reload 2>/dev/null || systemctl daemon-reload
  sudo -n systemctl start "$SERVICE_UNIT" 2>/dev/null || systemctl start "$SERVICE_UNIT"
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
  local payload version i
  # Extract host and port from health URL
  local host_port="${HEALTH_URL#http://}"
  host_port="${host_port%%/*}"
  local host="${host_port%:*}"
  local port="${host_port##*:}"
  if [[ -z "$port" ]]; then
    port=80
  fi
  # Use python3 for health check (avoids bash /dev/tcp and curl issues in sudo env)
  local py_script='
import json, sys, urllib.request, socket, time

health_url = sys.argv[1]
expected_version = sys.argv[2] if len(sys.argv) > 2 else ""
host_port = health_url.replace("http://", "").split("/")[0]
host = host_port.split(":")[0]
port = int(host_port.split(":")[1]) if ":" in host_port else 80

for i in range('"$attempts"'):
    # TCP connect probe
    try:
        s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        s.settimeout(3)
        s.connect((host, port))
        s.close()
    except Exception:
        time.sleep(1)
        continue

    # Port is open, try HTTP health check
    try:
        req = urllib.request.Request(health_url, method="GET")
        with urllib.request.urlopen(req, timeout=5) as resp:
            body = resp.read().decode()
            if expected_version:
                data = json.loads(body)
                if data.get("version") != expected_version:
                    time.sleep(1)
                    continue
            print(body)
            sys.exit(0)
    except Exception:
        time.sleep(1)
        continue

sys.exit(1)
'
  local py_stdout py_stderr py_exit
  py_stderr="$(mktemp)"
  py_stdout="$(python3 -c "$py_script" "$HEALTH_URL" "$expected_version" 2>"$py_stderr")"
  py_exit=$?
  if [[ "$py_exit" -eq 0 && -n "$py_stdout" ]]; then
    rm -f "$py_stderr"
    printf '%s\n' "$py_stdout"
    return 0
  fi
  if [[ -s "$py_stderr" ]]; then
    log "health check python stderr: $(head -3 "$py_stderr" | tr '\n' ' ')"
  fi
  rm -f "$py_stderr"
  audit_event "deploy_health" "timeout" "wait_for_health exhausted ${attempts} attempts for ${HEALTH_URL}"
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

validate_service_execstart
NEW_CHECKSUM="$(binary_checksum "$BINARY")"
log "installing $BIN_NAME from $BINARY to $INSTALL_PATH"
BACKUP_PATH=""
if [[ -f "$INSTALL_PATH" ]]; then
  CURRENT_VERSION="$(binary_version "$INSTALL_PATH")"
  CURRENT_CHECKSUM="$(binary_checksum "$INSTALL_PATH")"
  stamp="$(date +%Y%m%d-%H%M%S)"
  BACKUP_PATH="$BACKUP_DIR/${BIN_NAME}.${stamp}.bak"
  cp -p "$INSTALL_PATH" "$BACKUP_PATH"
  ln -sfn "$BACKUP_PATH" "$STATE_DIR/${BIN_NAME}.previous"
  log "backup saved to $BACKUP_PATH"
fi
audit_event "deploy_preflight" "ready" "current_version=${CURRENT_VERSION:-unknown} current_checksum=${CURRENT_CHECKSUM:-unknown} new_checksum=${NEW_CHECKSUM:-unknown}"

rollback() {
  local why="$1"
  warn "$why"
  audit_event "deploy_rollback" "started" "$why"
  if [[ -z "$BACKUP_PATH" || ! -f "$BACKUP_PATH" ]]; then
    audit_event "deploy_rollback" "failed" "rollback unavailable; no prior binary backup exists"
    die "rollback unavailable; no prior binary backup exists"
  fi
  warn "rolling back to $BACKUP_PATH"
  sudo -n install -m 0755 "$BACKUP_PATH" "$INSTALL_PATH" 2>/dev/null || install -m 0755 "$BACKUP_PATH" "$INSTALL_PATH"
  if [[ "$NO_RESTART" -eq 0 ]]; then
    log "restarting service for rollback"
    sudo -n systemctl restart "$SERVICE_UNIT" 2>/dev/null || systemctl restart "$SERVICE_UNIT"
    sleep 2
    if [[ "$NO_VERIFY" -eq 0 ]]; then
      sleep 3
      payload="$(wait_for_health 20 "" || true)"
      if [[ -z "$payload" ]] && ! systemctl is-active "$SERVICE_UNIT" >/dev/null 2>&1; then
        audit_event "deploy_rollback" "failed" "rollback restart failed; daemon still unhealthy"
        die "rollback restart failed; daemon still unhealthy"
      fi
    fi
    assert_single_process
  fi
  audit_event "deploy_rollback" "applied" "$why"
  die "deploy failed and rollback was applied"
}

# Install new binary atomically
sudo -n install -m 0755 "$BINARY" "$INSTALL_PATH.new" 2>/dev/null || install -m 0755 "$BINARY" "$INSTALL_PATH.new"
sudo -n mv -f "$INSTALL_PATH.new" "$INSTALL_PATH" 2>/dev/null || mv -f "$INSTALL_PATH.new" "$INSTALL_PATH"
echo "${EXPECTED_VERSION:-unknown}" > "$STATE_DIR/live-version"

if [[ "$NO_RESTART" -eq 1 ]]; then
  log "installed without restart (--no-restart)"
  audit_event "deploy_install" "completed" "installed without restart" ""
  exit 0
fi

# Restart the daemon to pick up the new binary.
# HARD RESTART: kill the process directly (not through systemctl)
# so systemd auto-restart brings it back immediately with the new binary.
# This avoids the systemd 239 + Restart=always race where systemctl
# restart causes an extra auto-restart cycle.
if service_exists; then
  log "restarting service $SERVICE_UNIT"
  pids="$(pgrep -x "$BIN_NAME" || true)"
  if [[ -n "$pids" ]]; then
    sudo -n kill -TERM $pids 2>/dev/null || kill -TERM $pids 2>/dev/null || true
    # wait for systemd restart to pick up new binary
    sleep 3
  else
    # Service not running; start it
    sudo -n systemctl start "$SERVICE_UNIT" 2>/dev/null || systemctl start "$SERVICE_UNIT"
    sleep 2
  fi
fi

if ! systemctl is-active "$SERVICE_UNIT" >/dev/null 2>&1; then
  rollback "service restart failed"
fi

if [[ "$NO_VERIFY" -eq 0 ]]; then
  payload="$(wait_for_health 60 "$EXPECTED_VERSION" || true)"
  if [[ -z "$payload" ]]; then
    rollback "health verification failed for $HEALTH_URL (empty or unavailable response)"
  fi

  version="$(printf '%s' "$payload" | json_field version || true)"
  log "health payload received: len=${#payload}"
  if [[ -n "$EXPECTED_VERSION" && "$version" != "$EXPECTED_VERSION" ]]; then
    rollback "health version mismatch: expected $EXPECTED_VERSION, got ${version:-<empty>}"
  fi
  log "daemon healthy at $HEALTH_URL"
  if [[ -n "$version" ]]; then
    log "live version=$version"
  fi

  validator_url="${FOCUSA_CALLGRAPH_VALIDATOR_URL:-}"
  if [[ -z "$validator_url" ]]; then
    case "$HEALTH_URL" in
      */v1/health) validator_url="${HEALTH_URL%/v1/health}/v1/callgraphs/validate" ;;
      *) rollback "CallGraph validator URL cannot be derived from health URL: $HEALTH_URL" ;;
    esac
  fi
  if ! python3 "$ROOT_DIR/scripts/verify-callgraph-validator.py" --url "$validator_url"; then
    rollback "CallGraph validator verification failed for $validator_url"
  fi
  audit_event "deploy_capability" "verified" "canonical CallGraph validator available at $validator_url"

  log "daemon is running with expected binary and required CallGraph validator"
  audit_event "deploy_complete" "success" "deploy complete current_version=${CURRENT_VERSION:-unknown} current_checksum=${CURRENT_CHECKSUM:-unknown} new_checksum=${NEW_CHECKSUM:-unknown}" "${version:-}"
  log "deploy complete"
  exit 0
fi
