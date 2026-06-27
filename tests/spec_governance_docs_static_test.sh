#!/usr/bin/env bash
# Governance + Security docs guard.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

GOV="$ROOT_DIR/GOVERNANCE.md"
SEC="$ROOT_DIR/SECURITY.md"
LIC="$ROOT_DIR/LICENSE.md"

[ -f "$GOV" ] || fail "GOVERNANCE.md missing"
[ -f "$SEC" ] || fail "SECURITY.md missing"
[ -f "$LIC" ] || fail "LICENSE.md missing"

rg -q 'BSL 1.1|Business Source License' "$LIC" || fail "LICENSE.md must reference BSL 1.1"
rg -q '90-day|coordinated disclosure' "$SEC" || fail "SECURITY.md must reference 90-day disclosure window"
rg -q 'Compatibility policy|Deprecation policy|Reproducibility' "$GOV" || fail "GOVERNANCE.md missing key sections"

pass "governance + security + license docs are in place"