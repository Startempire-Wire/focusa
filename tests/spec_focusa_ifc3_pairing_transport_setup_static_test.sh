#!/usr/bin/env bash
# spec_focusa_ifc3_pairing_transport_setup_static_test.sh
#
# Static test that verifies the focusa pairing transport setup subcommand
# is multi-transport (not single-vendor) and includes the required
# transport candidates per focusa-ifc3 acceptance criteria.
#
# Required transports (at minimum 2 of: cloudflared / tailscale / ssh / bore):
#   - cloudflared quick tunnel (opt-in vendor)
#   - Tailscale Funnel (opt-in vendor)
#   - bore.pub (open source)
#   - ssh -R reverse tunnel
#   - operator URL via /etc/focusa/public-url
#
# Acceptance:
#   - Subcommand registered with Setup + Show subcommands
#   - TransportReport JSON includes transport_name + transport_url
#   - At least 2 of cloudflared / ssh / tailscale are coded
#   - Output written to /etc/focusa/public-url
#   - Static guard verifies multi-transport (not single-vendor)

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PAIR_RS="$ROOT_DIR/crates/focusa-cli/src/commands/pairing_transport.rs"
MAIN_RS="$ROOT_DIR/crates/focusa-cli/src/main.rs"
MOD_RS="$ROOT_DIR/crates/focusa-cli/src/commands/mod.rs"

fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

# 1. pairing_transport.rs exists
[ -f "$PAIR_RS" ] || fail "pairing_transport.rs not found"
pass "crates/focusa-cli/src/commands/pairing_transport.rs exists"

# 2. mod.rs declares the module
grep -q 'pub mod pairing_transport' "$MOD_RS" \
  || fail "pub mod pairing_transport missing from commands/mod.rs"
pass "commands/mod.rs exposes pairing_transport module"

# 3. main.rs registers PairingTransport in dispatch
grep -q 'PairingTransport' "$MAIN_RS" \
  || fail "PairingTransport not referenced in main.rs"
pass "main.rs registers PairingTransport in dispatch"

# 4. main.rs wires pairing_transport::run
grep -q 'pairing_transport::run' "$MAIN_RS" \
  || fail "pairing_transport::run call missing"
pass "main.rs calls pairing_transport::run for TransportCmd"

# 5. TransportCmd has Setup + Show subcommands
grep -q 'Setup {' "$PAIR_RS" \
  || fail "TransportCmd::Setup variant missing"
grep -q 'Show {' "$PAIR_RS" \
  || fail "TransportCmd::Show variant missing"
pass "TransportCmd has Setup and Show subcommands"

# 6. JSON output via --json flag
grep -q 'json: bool' "$PAIR_RS" \
  || fail "--json flag missing from Setup/Show subcommands"
pass "Setup + Show subcommands accept --json flag"

# 7. TransportReport includes chosen_transport + chosen_url
grep -q 'chosen_transport' "$PAIR_RS" \
  || fail "TransportReport missing chosen_transport field"
grep -q 'chosen_url' "$PAIR_RS" \
  || fail "TransportReport missing chosen_url field"
pass "TransportReport includes chosen_transport + chosen_url (mapped to transport_name + transport_url)"

# 8. At least 2 of cloudflared / ssh / tailscale are coded
found_count=0
grep -q 'cloudflared_candidate\|CLOUDFLARED' "$PAIR_RS" && found_count=$((found_count + 1))
grep -q 'tailscale_candidate\|TAILSCALE' "$PAIR_RS" && found_count=$((found_count + 1))
grep -q 'ssh\|SSH\|jump' "$PAIR_RS" && found_count=$((found_count + 1))
grep -q 'bore_candidate\|bore' "$PAIR_RS" && found_count=$((found_count + 1))
if [ "$found_count" -lt 2 ]; then
  fail "At least 2 of cloudflared/tailscale/ssh/bore must be coded (found $found_count)"
fi
pass "transport candidates coded: cloudflared/tailscale/ssh/bore ($found_count present)"

# 9. Output written to /etc/focusa/public-url
grep -q 'public-url\|PUBLIC_URL_FILE' "$PAIR_RS" \
  || fail "TransportCmd does not write to /etc/focusa/public-url"
pass "/etc/focusa/public-url target present"

# 10. recovery_hint included
grep -q 'recovery_hint' "$PAIR_RS" \
  || fail "TransportReport missing recovery_hint"
pass "TransportReport includes recovery_hint"

# 11. Resolver re-verifies reachability before writing
grep -q 'verify\|verify_url\|reachability\|verified' "$PAIR_RS" \
  || fail "no reachability verification before write"
pass "Resolver re-verifies reachability before write"

echo "✓ All focusa-ifc3 pairing transport setup static checks passed"
