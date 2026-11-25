#!/bin/bash
set -euo pipefail
# Cheap, blocking source checks. Only exact-commit CI can authorize a tag.
# --strict remains a compatibility spelling for the same source check.
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
case "${1:-}" in
  ''|--check|--strict) ;;
  *) echo 'usage: local-release-preflight.sh [--strict]' >&2; exit 2 ;;
esac

python3 scripts/release-git-admin-preflight.py --repo "$ROOT"

# Git path quoting is avoided; NUL-delimited records preserve exact names.
git ls-files -z | while IFS= read -r -d '' path; do
  if [[ "$path" == *[':?*"<>|']* ]]; then
    printf 'FAIL Windows lint: illegal character in tracked path %q\n' "$path" >&2
    exit 1
  fi
done

echo '=== local preflight: release versions, source digests and provenance ==='
VERSION="$(tr -d '[:space:]' < docs/current/.release-version-stamp)"
python3 scripts/verify-version-surfaces.py "v${VERSION}"
scripts/verify-doc-version-consistency
scripts/generate-current-runtime-status --check
python3 scripts/generate-pi-menu-audit.py --check
python3 scripts/distribution_manifest.py --check --source-ancestor

echo '=== local preflight: formatting ==='
if command -v cargo >/dev/null 2>&1; then
  cargo fmt --all -- --check
fi

echo '=== local preflight: PASS (source only; exact-commit CI still required) ==='
