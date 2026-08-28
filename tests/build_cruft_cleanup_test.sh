#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CLEANUP="$ROOT_DIR/scripts/ci/cleanup-ephemeral-build-target.sh"
WORKFLOW="$ROOT_DIR/.github/workflows/ci.yml"
target="/tmp/focusa-ci-local-$$-99"

cleanup_test_target() {
  if [[ -d "$target" ]]; then
    "$CLEANUP" "$target" >/dev/null
  fi
}
trap cleanup_test_target EXIT

mkdir -p "$target/debug/incremental"
printf 'cruft\n' > "$target/debug/incremental/probe"
"$CLEANUP" "$target" >/dev/null
[[ ! -e "$target" ]] || {
  echo "ephemeral target was not deleted" >&2
  exit 1
}

if "$CLEANUP" /tmp/not-a-focusa-build-target >/dev/null 2>&1; then
  echo "cleanup accepted a non-allowlisted target" >&2
  exit 1
fi

grep -q 'CARGO_TARGET_DIR: /tmp/focusa-ci-${{ github.run_id }}-${{ github.run_attempt }}' "$WORKFLOW"
grep -q 'name: Immediate ephemeral build cleanup' "$WORKFLOW"
grep -q 'if: ${{ always() }}' "$WORKFLOW"

echo "build cruft cleanup test: PASS"
