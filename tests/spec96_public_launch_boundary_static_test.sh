#!/bin/bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
PUBLIC_FILES=(
  "README.md"
  "docs/README.md"
  "docs/INDEX.md"
  "docs/PRD.md"
  "docs/current/CURRENT_RUNTIME_STATUS.md"
  "docs/current/CLI_REFERENCE_CURRENT.md"
  "docs/current/VALIDATION_AND_RELEASE_PROOF.md"
  "COMMERCIAL.md"
  "CONTRIBUTING.md"
  "SUPPORT_TERMS.md"
  "TRADEMARKS.md"
)

for file in "${PUBLIC_FILES[@]}"; do
  path="${ROOT_DIR}/${file}"
  if [ ! -f "$path" ]; then
    echo "✗ FAIL: public launch file missing: $file" >&2
    exit 1
  fi
  if rg -n 'v0\.9\.11-dev|Architecture Locked|Pre-Implementation|Specifications Complete|license\s*=\s*"MIT"|Proprietary — Startempire Wire' "$path" >/dev/null; then
    echo "✗ FAIL: stale/conflicting public launch marker in $file" >&2
    rg -n 'v0\.9\.11-dev|Architecture Locked|Pre-Implementation|Specifications Complete|license\s*=\s*"MIT"|Proprietary — Startempire Wire' "$path" >&2 || true
    exit 1
  fi
done

if [ ! -f "${ROOT_DIR}/legal/COMMERCIAL_LICENSE_TEMPLATE.md" ] || [ ! -f "${ROOT_DIR}/legal/CONTRIBUTOR_LICENSE_AGREEMENT_TEMPLATE.md" ]; then
  echo "✗ FAIL: commercial license or CLA template missing" >&2
  exit 1
fi

if [ ! -f "${ROOT_DIR}/.github/PULL_REQUEST_TEMPLATE.md" ] || [ ! -f "${ROOT_DIR}/.github/ISSUE_TEMPLATE/config.yml" ]; then
  echo "✗ FAIL: GitHub contribution/licensing templates missing" >&2
  exit 1
fi

echo "✓ PASS: public launch files have license/support/contribution boundaries and no stale public markers"
echo "SPEC96 Public launch boundary static test: PASS"
