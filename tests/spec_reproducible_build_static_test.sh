#!/usr/bin/env bash
# Reproducible-build + signature guard (focusa mass-adoption).
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

WORKFLOW="$ROOT_DIR/.github/workflows/release.yml"
PORTABILITY="$ROOT_DIR/docs/current/PORTABILITY_AUDIT.md"

# Toolchain pinning.
rg -q 'dtolnay/rust-toolchain@nightly' "$WORKFLOW" || fail "release.yml must pin dtolnay/rust-toolchain"
rg -q 'toolchain: nightly-2026-08-28' "$WORKFLOW" || fail "release.yml must pin nightly-2026-08-28"

# Cosign signature job.
rg -q 'sigstore/cosign-installer' "$WORKFLOW" || fail "release.yml must install cosign"
rg -q 'cosign sign-blob' "$WORKFLOW" || fail "release.yml must sign SHA256SUMS.txt"

# Docs.
rg -q 'Reproducible Builds & Release Signatures' "$PORTABILITY" || fail "PORTABILITY_AUDIT.md missing Reproducible Builds addendum"
rg -q 'SOURCE_DATE_EPOCH' "$PORTABILITY" || fail "Reproducibility addendum missing SOURCE_DATE_EPOCH policy"

pass "release workflow pins toolchain, signs SHA256SUMS, documents reproducibility"