#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
source "$ROOT/tests/focusa_portable_bin.sh"
BINARY="$(focusa_resolve_test_cli_binary "$ROOT")"
command -v jq >/dev/null || { echo 'FAIL: jq required' >&2; exit 1; }

FIXTURE_ROOT="$HOME/focusa-lifecycle-tests"
mkdir -p "$FIXTURE_ROOT"
TMP="$(mktemp -d "$FIXTURE_ROOT/run.XXXXXX")"
HOME_DIR="$TMP/home"
RELEASE_DIR="$TMP/release"
PORT_FILE="$TMP/server.port"
mkdir -p "$HOME_DIR/.focusa/state" "$HOME_DIR/.config/focusa" "$RELEASE_DIR"
SERVER_PID=''
cleanup() {
  if [[ -n "$SERVER_PID" ]]; then
    kill "$SERVER_PID" >/dev/null 2>&1 || true
    wait "$SERVER_PID" 2>/dev/null || true
  fi
  [[ "${FOCUSA_KEEP_FIXTURE:-0}" == 1 ]] && { echo "fixture preserved: $TMP" >&2; return; }
  python3 - "$TMP" <<'PY'
import shutil, sys
shutil.rmtree(sys.argv[1], ignore_errors=True)
PY
}
trap cleanup EXIT

printf 'customer-state-v1\n' >"$HOME_DIR/.focusa/state/customer.txt"
cat >"$HOME_DIR/.config/focusa/license.json" <<'JSON'
{
  "key_hash": "",
  "key_prefix": "",
  "product": "focusa",
  "tier": "evaluation",
  "status": "active",
  "commercial_use": false,
  "customer_email": "",
  "features": [],
  "offline_valid_until": null,
  "expires_at": null,
  "eval": true,
  "registry": "",
  "issued_at": 1
}
JSON

write_release() {
  local tag="$1" version="$2" focusa_exit="${3:-0}"
  local triple='x86_64-unknown-linux-musl'
  rm -f "$RELEASE_DIR"/*
  for name in focusa focusa-daemon focusa-tui; do
    local asset="$RELEASE_DIR/${name}-${tag}-${triple}"
    local exit_code=0
    [[ "$name" == focusa ]] && exit_code="$focusa_exit"
    cat >"$asset" <<EOF
#!/bin/sh
if [ "\${1:-}" = "--version" ]; then
  printf '%s %s\\n' '$name' '$version'
  exit $exit_code
fi
exit 0
EOF
    chmod 0755 "$asset"
  done

  local package="$TMP/agent-package/focusa-agent-context"
  rm -rf "$TMP/agent-package"
  mkdir -p "$package/skills/focusa-getting-started"
  printf '# Focusa fixture context\n' >"$package/AGENTS.md"
  printf '# Getting started fixture\n' >"$package/skills/focusa-getting-started/SKILL.md"
  tar -czf "$RELEASE_DIR/focusa-agent-context-${tag}.tar.gz" \
    -C "$TMP/agent-package" focusa-agent-context

  local pi_package="$TMP/pi-package/pi-extension"
  rm -rf "$TMP/pi-package"
  mkdir -p "$pi_package"
  printf '{"name":"focusa-pi-bridge","version":"%s"}\n' "$version" >"$pi_package/package.json"
  tar -czf "$RELEASE_DIR/focusa-pi-extension-${tag}.tar.gz" \
    -C "$TMP/pi-package" pi-extension

  (
    cd "$RELEASE_DIR"
    : >SHA256SUMS.txt
    for asset in focusa-*; do
      sha256sum "$asset" >>SHA256SUMS.txt
    done
  )
}

python3 - "$RELEASE_DIR" "$PORT_FILE" <<'PY' &
import http.server, pathlib, socketserver, sys
root, port_file = sys.argv[1:]
class QuietHandler(http.server.SimpleHTTPRequestHandler):
    def log_message(self, format, *args):
        pass
handler = lambda *args, **kwargs: QuietHandler(*args, directory=root, **kwargs)
with socketserver.TCPServer(("127.0.0.1", 0), handler) as server:
    pathlib.Path(port_file).write_text(str(server.server_address[1]))
    server.serve_forever()
PY
SERVER_PID=$!
for _ in $(seq 1 50); do
  [[ -s "$PORT_FILE" ]] && break
  sleep 0.1
done
[[ -s "$PORT_FILE" ]] || { echo 'FAIL: fixture release server did not start' >&2; exit 1; }
BASE_URL="http://127.0.0.1:$(cat "$PORT_FILE")"
# This lifecycle fixture validates Focusa transitions, not third-party package
# installation. Provide deterministic capability shims so dependency preflight
# remains truthful without touching the host or network.
mkdir -p "$TMP/bin"
for dependency in node npm pi; do
  case "$dependency" in
    node) version='v22.0.0' ;;
    npm) version='10.0.0' ;;
    pi) version='pi 1.0.0' ;;
  esac
  cat >"$TMP/bin/$dependency" <<SH
#!/bin/sh
if [ "\${1:-}" = "--version" ]; then printf '%s\n' '$version'; fi
exit 0
SH
  chmod +x "$TMP/bin/$dependency"
done
cat >"$TMP/bin/npm" <<'SH'
#!/bin/sh
if [ "${1:-}" = "--version" ]; then printf '%s\n' '10.0.0'; exit 0; fi
mkdir -p node_modules
printf '%s\n' fixture >node_modules/.focusa-smoke
exit 0
SH
chmod +x "$TMP/bin/npm"
TEST_PATH="$TMP/bin:/usr/bin:/bin"

install_release() {
  local tag="$1" output="$2"
  HOME="$HOME_DIR" PATH="$TEST_PATH" FOCUSA_RELEASE_TAG="$tag" \
    FOCUSA_RELEASE_BASE_URL="$BASE_URL" \
    "$BINARY" install --target linux --eval --no-service --no-persist-path --json \
    >"$output"
}

write_release v1.0.0 1.0.0
install_release v1.0.0 "$TMP/install-v1.json"
jq -e '.ok==true and .license_status=="eval"' "$TMP/install-v1.json" >/dev/null || {
  echo 'FAIL: initial install JSON invalid or incomplete' >&2
  cat "$TMP/install-v1.json" >&2
  exit 1
}
[[ "$(cat "$HOME_DIR/.focusa/state/customer.txt")" == 'customer-state-v1' ]] \
  || { echo 'FAIL: first install lost existing customer state' >&2; exit 1; }
first_sha="$(sha256sum "$HOME_DIR/.focusa/bin/focusa" | cut -d' ' -f1)"

install_release v1.0.0 "$TMP/rerun-v1.json"
[[ "$(sha256sum "$HOME_DIR/.focusa/bin/focusa" | cut -d' ' -f1)" == "$first_sha" ]] \
  || { echo 'FAIL: idempotent rerun changed the verified binary' >&2; exit 1; }
[[ "$(cat "$HOME_DIR/.focusa/state/customer.txt")" == 'customer-state-v1' ]] \
  || { echo 'FAIL: idempotent rerun lost customer state' >&2; exit 1; }
[[ ! -e "$HOME_DIR/.focusa.stash" ]] \
  || { echo 'FAIL: successful rerun left an install stash' >&2; exit 1; }

write_release v2.0.0 2.0.0 23
set +e
HOME="$HOME_DIR" PATH="$TEST_PATH" FOCUSA_RELEASE_TAG=v2.0.0 \
  FOCUSA_RELEASE_BASE_URL="$BASE_URL" \
  "$BINARY" install --target linux --eval --no-service --no-persist-path --json \
  >"$TMP/interrupted.json" 2>"$TMP/interrupted.err"
interrupted_rc=$?
set -e
[[ "$interrupted_rc" -ne 0 ]] || { echo 'FAIL: broken replacement unexpectedly installed' >&2; exit 1; }
jq -e '
  .status=="blocked" and
  (.details.raw_error|contains("prior installation restored")) and
  .safe_recovery=="focusa doctor"
' "$TMP/interrupted.json" >/dev/null || {
  echo 'FAIL: interrupted install lacks actionable JSON recovery error' >&2
  cat "$TMP/interrupted.json" >&2
  exit 1
}
[[ "$(sha256sum "$HOME_DIR/.focusa/bin/focusa" | cut -d' ' -f1)" == "$first_sha" ]] \
  || { echo 'FAIL: interrupted install did not restore the prior binary' >&2; exit 1; }
[[ "$(cat "$HOME_DIR/.focusa/state/customer.txt")" == 'customer-state-v1' ]] \
  || { echo 'FAIL: interrupted install lost customer state' >&2; exit 1; }

write_release v2.0.0 2.0.0
install_release v2.0.0 "$TMP/repair-v2.json"
"$HOME_DIR/.focusa/bin/focusa" --version | grep -q '2.0.0' \
  || { echo 'FAIL: repair rerun did not install v2' >&2; exit 1; }

write_release v3.0.0 3.0.0
HOME="$HOME_DIR" PATH="$TEST_PATH" FOCUSA_RELEASE_TAG=v3.0.0 \
  FOCUSA_RELEASE_BASE_URL="$BASE_URL" \
  "$BINARY" --json upgrade --no-service --no-persist-path >"$TMP/upgrade-v3.json"
jq -e '.ok==true and .status=="completed" and .license_preserved==true' "$TMP/upgrade-v3.json" >/dev/null
"$HOME_DIR/.focusa/bin/focusa" --version | grep -q '3.0.0' \
  || { echo 'FAIL: upgrade did not install v3' >&2; exit 1; }
[[ "$(cat "$HOME_DIR/.focusa/state/customer.txt")" == 'customer-state-v1' ]] \
  || { echo 'FAIL: upgrade lost customer state' >&2; exit 1; }
[[ -f "$HOME_DIR/.config/focusa/license.json" ]] \
  || { echo 'FAIL: upgrade lost the reusable license record' >&2; exit 1; }

HOME="$HOME_DIR" PATH="$TEST_PATH" \
  "$BINARY" uninstall --target linux --yes --keep-data --keep-license \
  --keep-path-modifications --json >"$TMP/uninstall.json"
jq -e '.ok==true and ([.steps_executed[].kind]|index("remove_install_artifacts")!=null)' \
  "$TMP/uninstall.json" >/dev/null
[[ ! -e "$HOME_DIR/.focusa/bin" && ! -e "$HOME_DIR/.focusa/share" \
   && ! -e "$HOME_DIR/.focusa/agent-context" && ! -e "$HOME_DIR/.focusa/.focusa-version" ]] \
  || { echo 'FAIL: uninstall left managed software artifacts' >&2; exit 1; }
[[ "$(cat "$HOME_DIR/.focusa/state/customer.txt")" == 'customer-state-v1' ]] \
  || { echo 'FAIL: uninstall did not preserve customer state' >&2; exit 1; }
[[ -f "$HOME_DIR/.config/focusa/license.json" ]] \
  || { echo 'FAIL: uninstall did not preserve license record' >&2; exit 1; }

HOME="$HOME_DIR" PATH="$TEST_PATH" \
  "$BINARY" uninstall --target linux --yes --keep-data --keep-license \
  --keep-path-modifications --json >"$TMP/uninstall-rerun.json"
jq -e '.ok==true' "$TMP/uninstall-rerun.json" >/dev/null

echo 'PASS: interrupted rollback, idempotent rerun, repair, upgrade license reuse, and software-complete preserve-data uninstall'
