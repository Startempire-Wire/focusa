#!/usr/bin/env bash
# Commercialization-safe pairing transport setup guard (focusa-ifc3).
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

MOD="$ROOT_DIR/crates/focusa-cli/src/commands/pairing_transport.rs"
MAIN="$ROOT_DIR/crates/focusa-cli/src/main.rs"

# Required: candidate helpers + opt-in check + serialization helpers.
rg -n 'pub enum TransportCmd|cloudflared_candidate|tailscale_candidate|ngrok_candidate|opt_in_enabled|FOCUSA_TUNNEL_|frp_candidate|bore_candidate|ssh_reverse_candidate|extract_trycloudflare_url' "$MOD" >/dev/null \
  || fail "pairing_transport.rs missing commercialization-safe candidate surface"
rg -n 'Commands::PairingTransport' "$MAIN" >/dev/null \
  || fail "main.rs not dispatching focusa pairing transport"

# Policy: vendor transports must be opt-in (gated by env var), never auto.
rg -q 'opt_in_enabled\("CLOUDFLARED"\)' "$MOD" || fail "cloudflared must be opt-in via FOCUSA_TUNNEL_CLOUDFLARED"
rg -q 'opt_in_enabled\("TAILSCALE"\)' "$MOD" || fail "Tailscale must be opt-in via FOCUSA_TUNNEL_TAILSCALE"
rg -q 'opt_in_enabled\("NGROK"\)' "$MOD" || fail "ngrok must be opt-in via FOCUSA_TUNNEL_NGROK"

# Policy: localhost.run must NOT appear as a candidate anywhere in the transport surface.
# (Comments mentioning it as "not supported" are allowed.)
if rg -n 'localhost_run_candidate|fn localhost_run' "$MOD" >/dev/null; then
  fail "localhost.run is not permitted (unclear license, single-operator SSH relay)"
fi

# Policy: defaults must include self-hostable transports.
for t in ssh_reverse frp bore operator_url; do
  rg -q "${t}_candidate|${t}_url|${t}" "$MOD" || fail "default transport missing: $t"
done

pass "pairing transport setup enforces commercialization-safe defaults + opt-in vendor interop"
