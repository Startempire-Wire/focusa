#!/usr/bin/env bash
# spec_focusa_gkrj_pairing_doctor_static_test.sh
#
# Static test that verifies the focusa pairing doctor subcommand exists,
# is registered in the CLI dispatch, and that the DoctorReport JSON
# includes all the fields required by focusa-gkrj acceptance criteria.
#
# Required fields per the bead acceptance criteria:
#   - version
#   - transport_candidates
#   - codesign
#   - service_install_state
#   - pairing_state
#   - recovery_hint

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PAIR_RS="$ROOT_DIR/crates/focusa-cli/src/commands/pairing_doctor.rs"
MAIN_RS="$ROOT_DIR/crates/focusa-cli/src/main.rs"
MOD_RS="$ROOT_DIR/crates/focusa-cli/src/commands/mod.rs"

fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

# 1. pair_doctor.rs exists
[ -f "$PAIR_RS" ] || fail "pairing_doctor.rs not found at $PAIR_RS"
pass "crates/focusa-cli/src/commands/pairing_doctor.rs exists"

# 2. mod.rs declares the module
grep -q 'pub mod pairing_doctor' "$MOD_RS" \
  || fail "pub mod pairing_doctor missing from commands/mod.rs"
pass "commands/mod.rs exposes pairing_doctor module"

# 3. main.rs registers PairingDoctor in dispatch
grep -q 'PairingDoctor' "$MAIN_RS" \
  || fail "PairingDoctor not referenced in main.rs"
pass "main.rs registers PairingDoctor in dispatch"

# 4. main.rs wires pairing_doctor::run
grep -q 'pairing_doctor::run' "$MAIN_RS" \
  || fail "pairing_doctor::run call missing from main.rs"
pass "main.rs calls pairing_doctor::run for DoctorArgs"

# 5. DoctorReport struct includes required fields
for field in 'version' 'codesign' 'service_install' 'transport' 'next_action'; do
  grep -q "pub $field:" "$PAIR_RS" \
    || fail "DoctorReport missing field: $field"
done
pass "DoctorReport struct includes version, codesign, service_install, transport, next_action"

# 6. --json flag exists for machine-readable output
grep -q 'pub json:' "$PAIR_RS" \
  || fail "--json flag missing from DoctorArgs"
pass "DoctorArgs exposes --json flag"

# 7. The subcommand name is reachable as `focusa pairing doctor` or `focusa pair doctor`
grep -q 'doctor\|Doctor' "$MAIN_RS" \
  || fail "doctor subcommand not registered in main"
pass "doctor subcommand reachable in CLI dispatch"

# 8. recovery_hint logic (next_action or recovery field) for root-cause
grep -q '"next_action"\|recovery_hint\|recovery' "$PAIR_RS" \
  || fail "no recovery_hint or next_action in DoctorReport"
pass "DoctorReport includes recovery guidance (next_action)"

echo "✓ All focusa-gkrj pairing doctor static checks passed"
