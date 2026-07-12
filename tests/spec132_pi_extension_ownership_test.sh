#!/usr/bin/env bash
# Spec 132 D3 regression guard: Pi integration is Rust-owned.
set -euo pipefail
cd "$(dirname "$0")/.."

if grep -q 'install_pi_extension' scripts/install-focusa.sh; then
  echo "FAIL: Pi installation logic was reintroduced into Bash bootstrapper" >&2
  exit 1
fi
bash tests/spec_install_pi_integration_truth_test.sh
echo "PASS: Pi extension ownership remains Rust-only"
