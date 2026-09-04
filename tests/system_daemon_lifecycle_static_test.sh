#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
ADAPTER="$ROOT_DIR/scripts/install-daemon.sh"
RUST_LIFECYCLE="$ROOT_DIR/crates/focusa-cli/src/commands/system_service.rs"
RUST_PROCESS="$ROOT_DIR/crates/focusa-cli/src/commands/system_service_process.rs"
RUST_UPDATE="$ROOT_DIR/crates/focusa-cli/src/commands/update.rs"

fail() { echo "system daemon lifecycle test: FAIL: $*" >&2; exit 1; }

for forbidden in 'systemctl ' 'pgrep -x' 'kill -TERM' 'install -m' 'mv '; do
  ! grep -Fq "$forbidden" "$ADAPTER" || fail "adapter owns forbidden mutation: $forbidden"
done
for required in 'activate_and_verify' 'restore_prior_state' 'MemoryHigh=2G' 'MemoryMax=3G'; do
  grep -Fq "$required" "$RUST_LIFECYCLE" || fail "Rust lifecycle missing $required"
done
for required in 'RefuseManualStart=yes' 'validate_process_inventory' 'inspect_processes_before_update_restart'; do
  grep -Fq "$required" "$RUST_PROCESS" || fail "Rust process gate missing $required"
done
grep -Fq 'restart_existing_system_service' "$RUST_UPDATE" \
  || fail "Rust update path bypasses canonical system restart"

fixture="$(mktemp -d /tmp/focusa-system-lifecycle.XXXXXX)"
trap 'rm -rf "$fixture"' EXIT
mkdir -p "$fixture/scripts/lib" "$fixture/bin"
cp "$ADAPTER" "$fixture/scripts/install-daemon.sh"
cp "$ROOT_DIR/scripts/lib/release-version.sh" "$fixture/scripts/lib/release-version.sh"
cat >"$fixture/scripts/install-focusa.sh" <<'SH'
#!/usr/bin/env bash
printf '%s\n' "$@" >"${FOCUSA_ADAPTER_ARGS:?}"
SH
chmod 755 "$fixture/scripts/install-daemon.sh" "$fixture/scripts/install-focusa.sh"
candidate="$fixture/bin/focusa-daemon-v0.9.188-x86_64-unknown-linux-musl"
printf '#!/usr/bin/env sh\nexit 0\n' >"$candidate"
chmod 755 "$candidate"
args="$fixture/args"

FOCUSA_ADAPTER_ARGS="$args" FOCUSA_GITHUB_TAG=v0.9.188 \
  "$fixture/scripts/install-daemon.sh" \
  --binary "$candidate" --expected-version 0.9.188 --require-service
for expected in '--target=linux' '--channel=stable' '--release-tag=v0.9.188' '--system-install'; do
  grep -Fxq -- "$expected" "$args" || fail "delegation missing $expected"
done
! grep -Fxq -- '--no-service' "$args" || fail "required service unexpectedly skipped"

FOCUSA_ADAPTER_ARGS="$args" FOCUSA_GITHUB_TAG=v0.9.188 \
  "$fixture/scripts/install-daemon.sh" \
  --binary "$candidate" --expected-version 0.9.188 --no-restart
grep -Fxq -- '--no-service' "$args" || fail "--no-restart did not delegate --no-service"

if FOCUSA_ADAPTER_ARGS="$args" FOCUSA_GITHUB_TAG=v0.9.188 \
  "$fixture/scripts/install-daemon.sh" \
  --binary "$candidate" --expected-version 0.9.188 --no-verify; then
  fail "verification bypass was accepted"
fi
if FOCUSA_ADAPTER_ARGS="$args" FOCUSA_GITHUB_TAG=v0.9.188 FOCUSA_STATE_DIR=/tmp/wrong \
  "$fixture/scripts/install-daemon.sh" \
  --binary "$candidate" --expected-version 0.9.188; then
  fail "noncanonical state root was accepted"
fi

echo "system daemon lifecycle test: PASS"
