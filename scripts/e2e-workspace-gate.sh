#!/usr/bin/env bash
# E2E workspace gate — runs the full test workspace serially; any failure
# exits non-zero (pipefail-safe). Use: scripts/e2e-workspace-gate.sh
set -euo pipefail
cd "$(dirname "$0")/.."
cargo test --workspace --all-targets -- --test-threads=1
echo "E2E-WORKSPACE-GREEN"
