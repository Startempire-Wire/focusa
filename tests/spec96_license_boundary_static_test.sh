#!/bin/bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"

for file in LICENSE.md COMMERCIAL.md TRADEMARKS.md CONTRIBUTING.md; do
  if [ ! -s "${ROOT_DIR}/${file}" ]; then
    echo "✗ FAIL: ${file} missing or empty" >&2
    exit 1
  fi
done

if rg -n 'license\s*=\s*"MIT"|Proprietary — Startempire Wire' "${ROOT_DIR}/Cargo.toml" "${ROOT_DIR}/crates" "${ROOT_DIR}/README.md" >/dev/null; then
  echo "✗ FAIL: conflicting MIT/proprietary licensing marker remains in active package/readme files" >&2
  exit 1
fi

if rg -n 'license-file\s*=\s*"LICENSE.md"|license-file\.workspace\s*=\s*true|license-file\s*=\s*"../../LICENSE.md"' "${ROOT_DIR}/Cargo.toml" "${ROOT_DIR}/crates" >/dev/null \
  && rg -n 'Business Source License 1\.1|Commercial, production, hosted-service, client-delivery' "${ROOT_DIR}/LICENSE.md" >/dev/null \
  && rg -n 'Commercial, team/company, hosted-service, client-delivery' "${ROOT_DIR}/README.md" >/dev/null; then
  echo "✓ PASS: package metadata and README point to source-available commercial license boundary"
else
  echo "✗ FAIL: license boundary not wired through metadata/readme" >&2
  exit 1
fi

echo "SPEC96 License boundary static test: PASS"
