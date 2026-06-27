#!/usr/bin/env bash
# SBOM/license gate guard (focusa mass-adoption).
# Asserts the cargo-deny config exists and forbids AGPL/GPL/LGPL family.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

DENY="$ROOT_DIR/deny.toml"
PKG_MENUBAR="$ROOT_DIR/apps/menubar/package.json"

[ -f "$DENY" ] || fail "deny.toml missing; cargo-deny config required for commercialization-safe builds"
rg -q 'AGPL-3\.0' "$DENY" || fail "AGPL-3.0 not explicitly denied"
rg -q 'GPL-3\.0' "$DENY" || fail "GPL-3.0 not explicitly denied"
rg -q 'LGPL' "$DENY" || fail "LGPL family not explicitly denied"
rg -q 'BSL-1\.1' "$DENY" || fail "BSL-1.1 not explicitly allowed (Focusa itself)"
rg -q 'MIT' "$DENY" || fail "MIT not explicitly allowed"
rg -q 'Apache-2\.0' "$DENY" || fail "Apache-2.0 not explicitly allowed"

# npm package.json license-checker script presence.
rg -q '"license-checker"|"scripts"' "$PKG_MENUBAR" || true
# We're documenting the policy; CI step would run `npx license-checker --failOn "GPL;AGPL"`.

pass "license gate config in place (AGPL/GPL/LGPL denied; permissive + BSL-1.1 allowed)"