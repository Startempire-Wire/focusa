#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WORKFLOW="$ROOT/.github/workflows/nightly.yml"
SPEC="$ROOT/docs/177-focusa-release-channels-nightly-and-ci-spend-control-spec.md"

fail() { echo "FAIL: $*" >&2; exit 1; }
require() { grep -Fq -- "$1" "$2" || fail "$2 missing: $1"; }

require 'runs-on: [self-hosted, Linux, X64, ovh-build-2]' "$WORKFLOW"
if grep -Fq 'ubuntu-latest' "$WORKFLOW"; then
  fail 'nightly must not depend on billing-locked GitHub-hosted runners'
fi
require 'fetch-depth: 0' "$WORKFLOW"
require 'fetch-tags: true' "$WORKFLOW"
require 'FOCUSA_AUTHORITY_ROOT_KEYS_JSON' "$WORKFLOW"
require 'FOCUSA_RELEASE_ED25519_PRIVATE_KEY' "$WORKFLOW"
require 'scripts/release-trust-metadata.py' "$WORKFLOW"
require 'SHA256SUMS.txt.cosign.sig' "$WORKFLOW"
require 'release-manifest.json.sig' "$WORKFLOW"
require 'nightly-2026-08-28' "$WORKFLOW"
require 'x86_64-unknown-linux-musl' "$WORKFLOW"
require 'scripts/stamp-menubar-version.py "$tag"' "$WORKFLOW"
require 'scripts/verify-version-surfaces.py "$tag"' "$WORKFLOW"
for asset in focusa focusa-daemon focusa-tui focusa-session-runner; do
  require "$asset" "$WORKFLOW"
done
for asset in focusa-pi-extension focusa-installer focusa-agent-context SHA256SUMS.txt; do
  require "$asset" "$WORKFLOW"
done
require 'timeout-minutes: 20' "$WORKFLOW"
require "cron: '30 14 * * *'" "$WORKFLOW"
require 'workflow_dispatch:' "$WORKFLOW"
require 'force:' "$WORKFLOW"
require '${{ inputs.force }}' "$WORKFLOW"
require 'cancel-in-progress: true' "$WORKFLOW"
if grep -Fq '|| true' "$WORKFLOW"; then
  fail 'nightly must not suppress consequential failures'
fi

for marker in 'OVH self-hosted' 'x86_64-unknown-linux-musl' 'focusa-session-runner' 'FOCUSA_AUTHORITY_ROOT_KEYS_JSON' 'zero GitHub-hosted runner minutes'; do
  require "$marker" "$SPEC"
done

python3 - "$WORKFLOW" <<'PY'
from pathlib import Path
import sys
text = Path(sys.argv[1]).read_text()
stamp = text.index('scripts/stamp-menubar-version.py "$tag"')
build = text.index('cross build --release')
publish = text.index('gh release create "$TAG"')
if not stamp < build < publish:
    raise SystemExit('nightly must stamp before build and publish only after build')
PY

echo 'PASS Spec 177 nightly workflow contract'
