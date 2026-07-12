#!/usr/bin/env bash
# Spec 112 / Spec 132 D3 executable archive smoke test.
# Pi installation is Rust-owned; exercise the real Rust archive promotion path.
set -euo pipefail
cd "$(dirname "$0")/.."

if grep -q 'install_pi_extension' scripts/install-focusa.sh; then
  echo "FAIL: Pi installation logic was reintroduced into Bash bootstrapper" >&2
  exit 1
fi

cargo test -p focusa-cli --bin focusa pi_extension_archive_install_is_checksum_stage_and_activation_safe
echo "PASS: Rust-owned Pi extension archive install is staged, dependency-gated, and activated atomically"
