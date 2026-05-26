#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DOC="$ROOT_DIR/docs/current/FOCUSA_SECURITY_STANDARD_MATRIX_REVIEW_2026-05-26.md"
[[ -f "$DOC" ]] || { echo "missing standard matrix review doc" >&2; exit 1; }

for needle in \
  "OWASP Application Security Verification Standard" \
  "OWASP API Security Top 10" \
  "MITRE CWE Top 25" \
  "STRIDE threat model" \
  "CIS Controls v8" \
  "Route scopes and API permission matrix" \
  "not yet ready for broad network exposure"; do
  if ! grep -Fq "$needle" "$DOC"; then
    echo "matrix review missing marker: $needle" >&2
    exit 1
  fi
done

echo "✓ standard security matrix review present"
