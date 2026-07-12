#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CLI="$ROOT/crates/focusa-cli/src/commands/install.rs"
grep -Fq 'phase_pi_extension_download' "$CLI"
grep -Fq 'integrate_pi_extension' "$CLI"
grep -Fq 'Pi extension archive' "$CLI"
grep -Fq 'verify_checksum' "$CLI"
grep -Fq 'extract' "$CLI"
grep -Fq 'npm' "$CLI"
echo "Spec 132 Rust-owned Pi integration static contract: PASS"
