#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
SCRIPT="${ROOT_DIR}/scripts/phone-bridge-transport.sh"
SHIM="${ROOT_DIR}/scripts/setup-phone-bridge-url.sh"
GLOSSARY="${ROOT_DIR}/docs/00-glossary.md"
PLAN="${ROOT_DIR}/docs/54-focusa-pairing-room-plan.md"
PAIR_RS="${ROOT_DIR}/crates/focusa-cli/src/commands/pair.rs"

assert_has() {
  local file="$1"
  local pattern="$2"
  local label="$3"
  if rg -n "$pattern" "$file" >/dev/null; then
    echo "✓ PASS: ${label}"
  else
    echo "✗ FAIL: ${label}" >&2
    echo "Missing pattern '${pattern}' in ${file}" >&2
    exit 1
  fi
}

assert_not_has() {
  local file="$1"
  local pattern="$2"
  local label="$3"
  if rg -n "$pattern" "$file" >/dev/null; then
    echo "✗ FAIL: ${label}" >&2
    echo "Forbidden pattern '${pattern}' in ${file}" >&2
    exit 1
  fi
  echo "✓ PASS: ${label}"
}

bash -n "$SCRIPT" "$SHIM"
assert_has "$SCRIPT" 'detect\|check\|write\|options\|proxy-snippets' 'transport resolver exposes adaptive modes'
assert_has "$SCRIPT" 'FOCUSA_PAIRING_URL' 'transport resolver includes configured URL candidates'
assert_has "$SCRIPT" 'private_or_tailscale_ip' 'transport resolver includes private/Tailscale candidates'
assert_has "$SCRIPT" '/connect/\*' 'transport resolver documents /connect route'
assert_has "$SCRIPT" '/v1/connect/\*' 'transport resolver documents /v1/connect route'
assert_has "$SCRIPT" '/etc/focusa/public-url' 'transport resolver writes canonical public URL file'
assert_has "$SCRIPT" 'Focusa Connect' 'transport resolver validates actual Focusa Connect page'
assert_has "$SCRIPT" 'room/start' 'transport resolver validates Bridge Room API'
assert_has "$SHIM" 'phone-bridge-transport\.sh' 'legacy setup helper delegates to transport resolver'
assert_has "$GLOSSARY" 'Phone Bridge Flow' 'glossary defines Phone Bridge Flow'
assert_has "$GLOSSARY" 'Public Focusa URL' 'glossary defines Public Focusa URL'
assert_has "$PLAN" 'Phone Bridge Transport Resolver' 'Phone Bridge plan defines transport resolver'
assert_has "$PLAN" 'scripts/phone-bridge-transport\.sh' 'Phone Bridge plan references transport resolver helper'
assert_has "$PAIR_RS" 'daemon::start\(\)\.await.*resolve_server_url|daemon_started = daemon::start' 'focusa pair starts daemon before transport auto-detect'
assert_has "$PAIR_RS" 'private_or_tailscale_daemon_port' 'focusa pair auto-detects private/Tailscale daemon routes'
assert_has "$PAIR_RS" 'bridge_api_probe' 'focusa pair validates Bridge Room API route'
assert_has "$PAIR_RS" 'bridge_api_reachable' 'focusa pair reports Bridge Room API candidate status'
assert_has "$PAIR_RS" 'checked_candidates' 'focusa pair reports auto-detect candidate probes'
assert_has "$PAIR_RS" 'connect_probe_diagnostic' 'focusa pair reports structured Connect page probe diagnostics'
assert_has "$PAIR_RS" 'bridge_api_probe_diagnostic' 'focusa pair reports structured Bridge Room API probe diagnostics'
assert_has "$PAIR_RS" 'first_rejection' 'focusa pair reports first rejected candidate reason'
assert_has "$PAIR_RS" 'probe_timeout|probe_connect_failed|connect_page_signature_missing|bridge_api_signature_missing' 'focusa pair classifies auto-detect probe failures'
assert_has "${ROOT_DIR}/crates/focusa-api/src/routes/device_pairing.rs" 'phone bridge room started' 'daemon logs Bridge Room start lifecycle'
assert_has "${ROOT_DIR}/crates/focusa-api/src/routes/device_pairing.rs" 'phone bridge mac offer accepted' 'daemon logs Mac offer lifecycle'
assert_has "${ROOT_DIR}/crates/focusa-api/src/routes/device_pairing.rs" 'approval_completed' 'daemon reports approval completion diagnostics'
assert_has "${ROOT_DIR}/crates/focusa-api/src/routes/device_pairing.rs" 'next_step_hint' 'daemon responses include operator next-step diagnostics'
assert_has "${ROOT_DIR}/crates/focusa-cli/src/commands/daemon.rs" 'running_version_matches' 'daemon start detects stale daemon version'
assert_has "${ROOT_DIR}/crates/focusa-cli/src/commands/daemon.rs" 'DaemonShutdownRequest::new' 'daemon start uses typed exact-daemon shutdown for stale versions'
assert_has "${ROOT_DIR}/crates/focusa-cli/src/commands/daemon.rs" 'daemon health and lock process identities do not match' 'daemon start fails closed on stale identity mismatch'
assert_has "${ROOT_DIR}/crates/focusa-cli/src/commands/daemon.rs" 'exact stale-daemon shutdown failed; refusing broad process repair' 'daemon start fails closed when exact stale shutdown fails'
assert_not_has "${ROOT_DIR}/crates/focusa-cli/src/commands/daemon.rs" 'kill_daemon_processes' 'daemon start has no process-name fallback'
assert_has "${ROOT_DIR}/crates/focusa-cli/src/commands/daemon.rs" 'focusa-daemon' 'daemon discovery keeps CLI and daemon paired'

echo "Phone Bridge transport static test: PASS"
