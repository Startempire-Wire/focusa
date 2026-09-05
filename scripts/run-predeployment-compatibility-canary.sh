#!/usr/bin/env bash
set -euo pipefail

: "${RELEASE_TAG:?RELEASE_TAG is required}"
: "${RELEASE_SHA:?RELEASE_SHA is required}"
: "${GITHUB_REPOSITORY:?GITHUB_REPOSITORY is required}"
: "${GITHUB_WORKSPACE:?GITHUB_WORKSPACE is required}"
: "${GITHUB_TOKEN:?GITHUB_TOKEN is required}"
: "${RUNNER_TEMP:?RUNNER_TEMP is required}"
: "${FOCUSA_COMPATIBILITY_CANARY_LICENSE_SOURCE:?legacy license source is required}"
: "${FOCUSA_COMPATIBILITY_CANARY_AUTHORITY_PROFILE:?provider-enrolled canary authority profile is required}"
: "${FOCUSA_COMPATIBILITY_CANARY_DATABASE_SOURCE:?legacy database source is required}"

PREVIOUS_TAG="${FOCUSA_COMPATIBILITY_CANARY_PREVIOUS_TAG:-v0.9.177}"
[[ "$RELEASE_TAG" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]] || {
  echo "candidate tag must be exact stable SemVer" >&2
  exit 1
}
[[ "$PREVIOUS_TAG" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]] || {
  echo "prior tag must be exact stable SemVer" >&2
  exit 1
}
[[ "$RELEASE_SHA" =~ ^[0-9a-f]{40}$ ]] || {
  echo "candidate SHA must contain 40 lowercase hexadecimal characters" >&2
  exit 1
}
[[ "$(id -u)" -ne 0 ]] || {
  echo "compatibility canary refuses root" >&2
  exit 1
}
for command in curl gh jq node npm python3 readlink setsid sha256sum systemctl; do
  command -v "$command" >/dev/null || {
    echo "compatibility canary dependency is missing: $command" >&2
    exit 1
  }
done
python3 -c 'import cryptography' || {
  echo "compatibility canary Python verifier dependency is missing: cryptography" >&2
  exit 1
}
NODE_BIN_DIR="$(dirname "$(readlink -f "$(command -v node)")")"
[[ -x "$NODE_BIN_DIR/node" && -x "$NODE_BIN_DIR/npm" ]] || {
  echo "compatibility canary requires node and npm in one trusted toolchain" >&2
  exit 1
}
[[ -f "$FOCUSA_COMPATIBILITY_CANARY_LICENSE_SOURCE" ]] || {
  echo "signed lease source is missing" >&2
  exit 1
}
for file in authority-lease.json node-identity.json; do
  [[ -f "$FOCUSA_COMPATIBILITY_CANARY_AUTHORITY_PROFILE/$file" && ! -L "$FOCUSA_COMPATIBILITY_CANARY_AUTHORITY_PROFILE/$file" ]] || {
    echo "provider-enrolled canary profile requires a regular $file" >&2
    exit 1
  }
done
[[ -f "$FOCUSA_COMPATIBILITY_CANARY_DATABASE_SOURCE" ]] || {
  echo "legacy database source is missing" >&2
  exit 1
}

CANARY_ROOT="$RUNNER_TEMP/focusa-compatibility-canary-${GITHUB_RUN_ID:-manual}-${GITHUB_RUN_ATTEMPT:-1}"
RECEIPT_PATH="${FOCUSA_COMPATIBILITY_CANARY_RECEIPT_PATH:-$RUNNER_TEMP/compatibility-canary-success.json}"
DAEMON_PORT="${FOCUSA_COMPATIBILITY_CANARY_PORT:-18787}"
DAEMON_PID=""

case "$CANARY_ROOT" in
  "$RUNNER_TEMP"/focusa-compatibility-canary-*) ;;
  *) echo "unsafe compatibility canary root" >&2; exit 1 ;;
esac
[[ ! -e "$CANARY_ROOT" ]] || {
  echo "compatibility canary root already exists" >&2
  exit 1
}

cleanup() {
  local status=$?
  set +e
  if [[ -n "$DAEMON_PID" ]] && kill -0 "$DAEMON_PID" 2>/dev/null; then
    local actual_exe=""
    actual_exe="$(readlink -f "/proc/$DAEMON_PID/exe" 2>/dev/null)"
    if [[ "$actual_exe" == "$CANARY_ROOT"/* ]]; then
      kill -TERM "$DAEMON_PID" 2>/dev/null
      local kill_status=$?
      wait "$DAEMON_PID" 2>/dev/null
      local wait_status=$?
      if [[ "$kill_status" -ne 0 || ( "$wait_status" -ne 0 && "$wait_status" -ne 143 ) ]]; then
        echo "canary daemon cleanup failed: kill=$kill_status wait=$wait_status" >&2
        status=1
      fi
    elif kill -0 "$DAEMON_PID" 2>/dev/null; then
      echo "refusing to terminate non-canary process $DAEMON_PID" >&2
      status=1
    fi
  fi
  DAEMON_PID=""
  if [[ -d "$CANARY_ROOT" ]]; then
    case "$CANARY_ROOT" in
      "$RUNNER_TEMP"/focusa-compatibility-canary-*)
        rm -rf --one-file-system "$CANARY_ROOT"
        [[ "$?" -eq 0 ]] || status=1
        ;;
      *) echo "refusing unsafe canary cleanup: $CANARY_ROOT" >&2; status=1 ;;
    esac
  fi
  set -e
  return "$status"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

mkdir -p \
  "$CANARY_ROOT/bootstrap/candidate" \
  "$CANARY_ROOT/.config/focusa" \
  "$CANARY_ROOT/.local/share" \
  "$CANARY_ROOT/.local/state" \
  "$CANARY_ROOT/.cache" \
  "$CANARY_ROOT/data" \
  "$CANARY_ROOT/pi/extensions" \
  "$CANARY_ROOT/evidence"
chmod 0700 "$CANARY_ROOT" "$CANARY_ROOT/.config/focusa"
cp -- "$FOCUSA_COMPATIBILITY_CANARY_LICENSE_SOURCE" \
  "$CANARY_ROOT/.config/focusa/license.json"
chmod 0600 "$CANARY_ROOT/.config/focusa/license.json"
# Transport only the explicitly supplied canary enrollment, never a derived ID
# or ambient production identity, credential store, or replacement trust roots.
for file in authority-lease.json node-identity.json; do
  cp -- "$FOCUSA_COMPATIBILITY_CANARY_AUTHORITY_PROFILE/$file" "$CANARY_ROOT/.config/focusa/$file"
  chmod 0600 "$CANARY_ROOT/.config/focusa/$file"
done
(cd "$CANARY_ROOT/.config/focusa" && sha256sum authority-lease.json node-identity.json) \
  > "$CANARY_ROOT/evidence/authority-fixture.sha256"
printf 'focusa compatibility canary user sentinel\n' > "$CANARY_ROOT/user-sentinel.txt"

python3 "$GITHUB_WORKSPACE/scripts/compatibility-canary-database-inventory.py" copy \
  --source "$FOCUSA_COMPATIBILITY_CANARY_DATABASE_SOURCE" \
  --destination "$CANARY_ROOT/data/focusa.sqlite" || {
  echo "legacy database canary copy failed quick_check" >&2
  exit 1
}

jq -n \
  --arg tag "$RELEASE_TAG" \
  --arg prior "$PREVIOUS_TAG" \
  --arg root "$CANARY_ROOT" \
  '{schema:"focusa.compatibility_canary_scope.v1",release_tag:$tag,required_previous_tag:$prior,root:$root,production:false}' \
  > "$CANARY_ROOT/.focusa-compatibility-canary-scope.json"

export HOME="$CANARY_ROOT"
export XDG_CONFIG_HOME="$CANARY_ROOT/.config"
export XDG_DATA_HOME="$CANARY_ROOT/.local/share"
export XDG_STATE_HOME="$CANARY_ROOT/.local/state"
export XDG_CACHE_HOME="$CANARY_ROOT/.cache"
export npm_config_cache="$CANARY_ROOT/.cache/npm"
export NPM_CONFIG_USERCONFIG="$CANARY_ROOT/.config/npmrc"
: > "$NPM_CONFIG_USERCONFIG"
unset NODE_OPTIONS NODE_PATH BUN_INSTALL BUN_INSTALL_CACHE_DIR COREPACK_HOME XDG_RUNTIME_DIR
export FOCUSA_DATA_DIR="$CANARY_ROOT/data"
export FOCUSA_PI_EXT_DIR="$CANARY_ROOT/pi/extensions"
export PI_CODING_AGENT_DIR="$CANARY_ROOT/pi"
export FOCUSA_PI_EXTENSION_PACKAGE_JSON="$CANARY_ROOT/pi/extensions/focusa/package.json"
export FOCUSA_INSTALLER_PATH="$CANARY_ROOT/bootstrap/candidate/focusa-installer.sh"
export FOCUSA_COMPATIBILITY_CANARY_PARENT="$RUNNER_TEMP"
export FOCUSA_BIND="127.0.0.1:$DAEMON_PORT"
export FOCUSA_DAEMON_URL="http://127.0.0.1:$DAEMON_PORT"
export PATH="$CANARY_ROOT/.focusa/bin:$CANARY_ROOT/.local/bin:$NODE_BIN_DIR:/usr/local/bin:/usr/bin:/bin"
unset INVOCATION_ID FOCUSA_DEV_MODE FOCUSA_RELEASE_BASE_URL FOCUSA_UPDATE_LATEST_TAG

production_fingerprint() {
  {
    for path in \
      /usr/local/bin/focusa \
      /usr/local/bin/focusa-daemon \
      /usr/local/bin/focusa-tui \
      /usr/local/bin/focusa-session-runner \
      /etc/systemd/system/focusa-daemon.service.d/operator-halt.conf; do
      if [[ -e "$path" ]]; then sha256sum "$path"; else printf 'missing  %s\n' "$path"; fi
    done
    if ! systemctl show focusa-daemon.service \
      -p MainPID -p ActiveState -p SubState 2>/dev/null; then
      printf 'production-service-status-unavailable\n'
    fi
  } | sha256sum | awk '{print $1}'
}

verify_bootstrap_asset() {
  local directory="$1"
  local asset="$2"
  local tag="$3"
  python3 "$GITHUB_WORKSPACE/scripts/verify-release-bootstrap-asset.py" \
    --asset "$directory/$asset" \
    --asset-signature "$directory/$asset.sig" \
    --checksums "$directory/SHA256SUMS.txt" \
    --checksums-signature "$directory/SHA256SUMS.txt.sig" \
    --trusted-keys "$directory/focusa-trusted-release-keys.json" \
    --trusted-keys-signature "$directory/focusa-trusted-release-keys.json.sig" \
    --pinned-trusted-keys "$GITHUB_WORKSPACE/config/focusa-trusted-release-keys.json" \
    --release-manifest "$directory/release-manifest.json" \
    --release-manifest-signature "$directory/release-manifest.json.sig" \
    --expected-tag "$tag"
}

download_bootstrap_bundle() {
  local tag="$1"
  local asset="$2"
  local directory="$3"
  gh release download "$tag" --repo "$GITHUB_REPOSITORY" --dir "$directory" --clobber \
    --pattern "$asset" \
    --pattern "$asset.sig" \
    --pattern SHA256SUMS.txt \
    --pattern SHA256SUMS.txt.sig \
    --pattern focusa-trusted-release-keys.json \
    --pattern focusa-trusted-release-keys.json.sig \
    --pattern release-manifest.json \
    --pattern release-manifest.json.sig
  verify_bootstrap_asset "$directory" "$asset" "$tag"
}

verify_version() {
  local binary="$1"
  local expected="${2#v}"
  "$binary" --version | grep -Fq -- "$expected"
}

stop_canary_daemon() {
  [[ -n "$DAEMON_PID" ]] || return 0
  local expected actual wait_status
  expected="$(readlink -f "$CANARY_ROOT/.focusa/bin/focusa-daemon")"
  actual="$(readlink -f "/proc/$DAEMON_PID/exe" 2>/dev/null)"
  [[ "$actual" == "$expected" ]] || {
    echo "canary daemon process identity changed" >&2
    return 1
  }
  kill -TERM "$DAEMON_PID"
  set +e
  wait "$DAEMON_PID"
  wait_status=$?
  set -e
  [[ "$wait_status" -eq 0 || "$wait_status" -eq 143 ]] || {
    echo "canary daemon exited unexpectedly during stop: $wait_status" >&2
    return 1
  }
  DAEMON_PID=""
}

verify_database_phase() {
  local phase="$1"
  local inventory="$CANARY_ROOT/evidence/database-${phase}.json"
  python3 "$GITHUB_WORKSPACE/scripts/compatibility-canary-database-inventory.py" capture \
    --database "$CANARY_ROOT/data/focusa.sqlite" \
    --output "$inventory"
  case "$phase" in
    prior-initial) ;;
    prior-interrupted-recovery)
      python3 "$GITHUB_WORKSPACE/scripts/compatibility-canary-database-inventory.py" compare \
        --baseline "$CANARY_ROOT/evidence/database-prior-initial.json" \
        --observed "$inventory" --same-schema
      ;;
    candidate-first)
      python3 "$GITHUB_WORKSPACE/scripts/compatibility-canary-database-inventory.py" compare \
        --baseline "$CANARY_ROOT/evidence/database-prior-interrupted-recovery.json" \
        --observed "$inventory"
      ;;
    prior-rollback)
      python3 "$GITHUB_WORKSPACE/scripts/compatibility-canary-database-inventory.py" compare \
        --baseline "$CANARY_ROOT/evidence/database-candidate-first.json" \
        --observed "$inventory" --same-schema
      ;;
    candidate-reapply)
      python3 "$GITHUB_WORKSPACE/scripts/compatibility-canary-database-inventory.py" compare \
        --baseline "$CANARY_ROOT/evidence/database-candidate-first.json" \
        --observed "$inventory" --same-schema
      ;;
    *) echo "unknown database verification phase: $phase" >&2; return 1 ;;
  esac
}

verify_authority_fixture() {
  local node_id
  node_id="$(jq -er 'select(.schema == "focusa.node_identity.v1" and .product == "focusa") | .node_id | strings | select(length > 0)' "$CANARY_ROOT/.config/focusa/node-identity.json")" || return 1
  [[ -n "$node_id" ]] || { echo "canary node identity is empty" >&2; return 1; }
  (cd "$CANARY_ROOT/.config/focusa" && sha256sum --check --status "$CANARY_ROOT/evidence/authority-fixture.sha256") || return 1
  # Cryptography, revocation, clock and node binding stay in the canonical guard.
  "$CANARY_ROOT/bootstrap/candidate/focusa-updater" license status --json \
    | jq -e --arg node "$node_id" '
      .authority.state == "active" and
      .authority.node_id == $node and
      (.authority.lease_id | type == "string" and length > 0) and
      (.authority.lease_digest | type == "string" and length > 0)
    ' >/dev/null || return 1
  (cd "$CANARY_ROOT/.config/focusa" && sha256sum --check --status "$CANARY_ROOT/evidence/authority-fixture.sha256")
}

verify_phase() {
  verify_authority_fixture || return 1
  local expected_tag="$1"
  local require_runner="$2"
  local phase="$3"
  local expected="${expected_tag#v}"
  verify_version "$CANARY_ROOT/.focusa/bin/focusa" "$expected_tag"
  verify_version "$CANARY_ROOT/.focusa/bin/focusa-daemon" "$expected_tag"
  "$CANARY_ROOT/.focusa/bin/focusa-tui" --headless-self-test \
    | jq -e --arg version "$expected" '.about_version == $version' >/dev/null
  jq -e --arg version "$expected" '.version == $version' \
    "$CANARY_ROOT/pi/extensions/focusa/package.json" >/dev/null
  [[ -f "$CANARY_ROOT/.focusa/agent-context/AGENTS.md" ]]
  [[ -n "$(find "$CANARY_ROOT/.focusa/agent-context/skills" \
    -type f -name SKILL.md -print -quit)" ]]
  if [[ "$require_runner" == "1" ]]; then
    verify_version "$CANARY_ROOT/.focusa/bin/focusa-session-runner" "$expected_tag"
    jq -e --arg version "$expected" '.release_version == $version' \
      "$CANARY_ROOT/.focusa/distribution-manifest.json" >/dev/null
  else
    [[ ! -e "$CANARY_ROOT/.focusa/bin/focusa-session-runner" ]]
  fi
  [[ "$(sha256sum "$CANARY_ROOT/.config/focusa/license.json" | awk '{print $1}')" == "$LICENSE_BEFORE" ]]
  [[ "$(sha256sum "$CANARY_ROOT/user-sentinel.txt" | awk '{print $1}')" == "$SENTINEL_BEFORE" ]]

  setsid "$CANARY_ROOT/.focusa/bin/focusa-daemon" \
    >"$CANARY_ROOT/daemon-${phase}.log" 2>&1 &
  DAEMON_PID=$!
  local health=""
  for _ in $(seq 1 30); do
    if health="$(curl -fsS --max-time 1 "http://127.0.0.1:$DAEMON_PORT/v1/health" 2>/dev/null)"; then
      break
    fi
    kill -0 "$DAEMON_PID" 2>/dev/null || {
      cat "$CANARY_ROOT/daemon-${phase}.log" >&2
      return 1
    }
    sleep 1
  done
  jq -e --arg version "$expected" '.status == "ok" and .version == $version' <<<"$health" >/dev/null
  if [[ "$require_runner" == "1" ]]; then
    FOCUSA_INSTALL_ROOT="$CANARY_ROOT/.focusa" \
    FOCUSA_DISTRIBUTION_MANIFEST="$CANARY_ROOT/.focusa/distribution-manifest.json" \
    FOCUSA_PI_EXT_DIR="$CANARY_ROOT/pi/extensions/focusa" \
    FOCUSA_RELEASE_MANIFEST="$CANARY_ROOT/bootstrap/candidate/release-manifest.json" \
    FOCUSA_RELEASE_ASSET_SUFFIX="x86_64-unknown-linux-musl" \
    FOCUSA_DAEMON_HEALTH_URL="http://127.0.0.1:$DAEMON_PORT/v1/health" \
      node "$GITHUB_WORKSPACE/scripts/audit-distribution-parity.mjs" --json \
        > "$CANARY_ROOT/evidence/distribution-parity-${phase}.json"
    jq -e '.parity_ok == true' \
      "$CANARY_ROOT/evidence/distribution-parity-${phase}.json" >/dev/null
  fi
  stop_canary_daemon
  verify_database_phase "$phase"
}

PRODUCTION_BEFORE="$(production_fingerprint)"
LICENSE_BEFORE="$(sha256sum "$CANARY_ROOT/.config/focusa/license.json" | awk '{print $1}')"
SENTINEL_BEFORE="$(sha256sum "$CANARY_ROOT/user-sentinel.txt" | awk '{print $1}')"

CANDIDATE_CLI="focusa-${RELEASE_TAG}-x86_64-unknown-linux-musl"
CANDIDATE_INSTALLER="focusa-installer-${RELEASE_TAG}.sh"
download_bootstrap_bundle "$RELEASE_TAG" "$CANDIDATE_CLI" "$CANARY_ROOT/bootstrap/candidate"
download_bootstrap_bundle "$RELEASE_TAG" "$CANDIDATE_INSTALLER" "$CANARY_ROOT/bootstrap/candidate"
# No downloaded executable receives repository credentials. Subsequent GitHub
# release reads are public and remain exact-tag/signature bound.
unset GITHUB_TOKEN GH_TOKEN
chmod 0755 \
  "$CANARY_ROOT/bootstrap/candidate/$CANDIDATE_CLI" \
  "$CANARY_ROOT/bootstrap/candidate/$CANDIDATE_INSTALLER"
cp "$CANARY_ROOT/bootstrap/candidate/$CANDIDATE_CLI" \
  "$CANARY_ROOT/bootstrap/candidate/focusa-updater"
chmod 0755 "$CANARY_ROOT/bootstrap/candidate/focusa-updater"

verify_authority_fixture

# Only the current, verified CLI installs the baseline. Its signed candidate
# manifest authorizes every historical digest before any installed probe runs.
"$CANARY_ROOT/bootstrap/candidate/focusa-updater" update compatibility-bootstrap \
  --channel stable \
  --latest-version "$RELEASE_TAG" \
  --daemon-health-url "http://127.0.0.1:$DAEMON_PORT/v1/health" \
  --compatibility-canary-root "$CANARY_ROOT" \
  --yes \
  --allow-apply \
  --dry-run=false
verify_phase "$PREVIOUS_TAG" 0 prior-initial

# The separately signed public candidate bootstrap is inert inside the canary;
# update inventory may inspect only its safe --version surface.
cp "$CANARY_ROOT/bootstrap/candidate/$CANDIDATE_INSTALLER" "$FOCUSA_INSTALLER_PATH"
chmod 0755 "$FOCUSA_INSTALLER_PATH"

run_candidate_apply() {
  verify_authority_fixture || return 1
  local output="$1"
  "$CANARY_ROOT/bootstrap/candidate/focusa-updater" update apply \
    --channel stable \
    --latest-version "$RELEASE_TAG" \
    --daemon-health-url "http://127.0.0.1:$DAEMON_PORT/v1/health" \
    --compatibility-canary-root "$CANARY_ROOT" \
    --yes \
    --allow-apply \
    --dry-run false \
    --json \
    > "$output"
}

apply_candidate() {
  run_candidate_apply "$CANARY_ROOT/update-apply.json"
  jq -e '.status == "completed" and .mutations_performed == true' \
    "$CANARY_ROOT/update-apply.json" >/dev/null
}

set +e
FOCUSA_COMPATIBILITY_CANARY_FAULT=after_asset_download \
  run_candidate_apply "$CANARY_ROOT/update-interrupted.json" \
  2> "$CANARY_ROOT/update-interrupted.err"
INTERRUPTED_STATUS=$?
set -e
[[ "$INTERRUPTED_STATUS" -ne 0 ]] || {
  echo "compatibility canary interruption unexpectedly succeeded" >&2
  exit 1
}
grep -Fq 'injected compatibility canary interruption after asset download' \
  "$CANARY_ROOT/update-interrupted.err"
[[ ! -e "$CANARY_ROOT/.focusa.stash" ]]
verify_phase "$PREVIOUS_TAG" 0 prior-interrupted-recovery

apply_candidate
verify_phase "$RELEASE_TAG" 1 candidate-first

verify_authority_fixture
"$CANARY_ROOT/.focusa/bin/focusa" update rollback \
  --part all --yes --dry-run false --json > "$CANARY_ROOT/update-rollback.json"
jq -e '.status == "completed" and .mutations_performed == true' \
  "$CANARY_ROOT/update-rollback.json" >/dev/null
verify_phase "$PREVIOUS_TAG" 0 prior-rollback

apply_candidate
verify_phase "$RELEASE_TAG" 1 candidate-reapply

verify_authority_fixture
LICENSE_AFTER="$(sha256sum "$CANARY_ROOT/.config/focusa/license.json" | awk '{print $1}')"
SENTINEL_AFTER="$(sha256sum "$CANARY_ROOT/user-sentinel.txt" | awk '{print $1}')"
PRODUCTION_AFTER="$(production_fingerprint)"
[[ "$LICENSE_AFTER" == "$LICENSE_BEFORE" ]] || {
  echo "signed lease fixture changed during compatibility canary" >&2
  exit 1
}
[[ "$SENTINEL_AFTER" == "$SENTINEL_BEFORE" ]] || {
  echo "user sentinel changed during compatibility canary" >&2
  exit 1
}
[[ "$PRODUCTION_AFTER" == "$PRODUCTION_BEFORE" ]] || {
  echo "production runtime changed during isolated compatibility canary" >&2
  exit 1
}

jq -n \
  --arg tag "$RELEASE_TAG" \
  --arg sha "$RELEASE_SHA" \
  --arg previous "$PREVIOUS_TAG" \
  --arg run_url "${GITHUB_SERVER_URL}/${GITHUB_REPOSITORY}/actions/runs/${GITHUB_RUN_ID}" \
  --slurpfile prior_database "$CANARY_ROOT/evidence/database-prior-initial.json" \
  --slurpfile interrupted_database "$CANARY_ROOT/evidence/database-prior-interrupted-recovery.json" \
  --slurpfile candidate_database "$CANARY_ROOT/evidence/database-candidate-first.json" \
  --slurpfile rollback_database "$CANARY_ROOT/evidence/database-prior-rollback.json" \
  --slurpfile reapply_database "$CANARY_ROOT/evidence/database-candidate-reapply.json" \
  --arg authority_profile_sha256 "$(sha256sum "$CANARY_ROOT/evidence/authority-fixture.sha256" | awk '{print $1}')" \
  --arg parity_first_sha256 "$(sha256sum "$CANARY_ROOT/evidence/distribution-parity-candidate-first.json" | awk '{print $1}')" \
  --arg parity_reapply_sha256 "$(sha256sum "$CANARY_ROOT/evidence/distribution-parity-candidate-reapply.json" | awk '{print $1}')" \
  '{
    schema:"focusa.compatibility_canary_success.v1",
    status:"passed",
    candidate:{tag:$tag,commit:$sha},
    previous_release_tag:$previous,
    environment:"isolated_preproduction",
    sequence:[
      "prior_release_healthy",
      "candidate_manifest_bound_apply_healthy",
      "prior_release_full_rollback_healthy",
      "candidate_manifest_bound_reapply_healthy"
    ],
    database_quick_check:"ok",
    database_evidence:{
      prior_initial:{schema_sha256:$prior_database[0].schema_sha256,row_counts_sha256:$prior_database[0].row_counts_sha256},
      prior_interrupted_recovery:{schema_sha256:$interrupted_database[0].schema_sha256,row_counts_sha256:$interrupted_database[0].row_counts_sha256},
      candidate_first:{schema_sha256:$candidate_database[0].schema_sha256,row_counts_sha256:$candidate_database[0].row_counts_sha256},
      prior_rollback:{schema_sha256:$rollback_database[0].schema_sha256,row_counts_sha256:$rollback_database[0].row_counts_sha256},
      candidate_reapply:{schema_sha256:$reapply_database[0].schema_sha256,row_counts_sha256:$reapply_database[0].row_counts_sha256}
    },
    distribution_parity:{
      candidate_first_sha256:$parity_first_sha256,
      candidate_reapply_sha256:$parity_reapply_sha256,
      status:"passed"
    },
    signed_lease_preserved:true,
    authority_profile:{status:"active_verified_each_phase",files_preserved:true,inventory_sha256:$authority_profile_sha256},
    user_sentinel_preserved:true,
    production_runtime_preserved:true,
    system_install_performed:false,
    service_mutation_performed:false,
    automatic_apply_performed:false,
    interrupted_install_recovered:true,
    run_url:$run_url
  }' > "$RECEIPT_PATH"
jq -e '.status == "passed" and .production_runtime_preserved == true' "$RECEIPT_PATH" >/dev/null
printf 'compatibility_canary=passed tag=%s sha=%s receipt=%s\n' \
  "$RELEASE_TAG" "$RELEASE_SHA" "$RECEIPT_PATH"
