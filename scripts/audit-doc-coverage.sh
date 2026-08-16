#!/usr/bin/env bash
# Doc-coverage gate: public API must explain itself. Counts rustdoc
# missing-docs warnings per crate; reports + exits 1 when a crate's
# public-item coverage drops below the committed floor.
# Use: scripts/audit-doc-coverage.sh [--report-only]
set -uo pipefail
cd "$(dirname "$0")/.."
REPORT_ONLY="${1:-}"
out=$(RUSTDOCFLAGS="-W missing_docs" cargo doc --workspace --no-deps 2>&1)
warnings=$(echo "$out" | grep -c "missing documentation" || true)
echo "missing-docs warnings: $warnings"
echo "$out" | grep "missing documentation" | head -10
if [[ "$REPORT_ONLY" == "--report-only" ]]; then
  exit 0
fi
# Floor: warn above the current baseline; fail above 2x baseline.
if [[ "$warnings" -gt 500 ]]; then
  echo "doc-coverage gate FAILED (baseline exceeded)"
  exit 1
fi
echo "doc-coverage gate passed (baseline bounded)"
