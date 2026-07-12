#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SAN="$ROOT/crates/focusa-terminal-ui/src/sanitize.rs"
EVENT="$ROOT/crates/focusa-terminal-ui/src/install/event.rs"
CLI="$ROOT/crates/focusa-cli/src/commands/install.rs"
grep -Fq 'sanitize' "$SAN"
grep -Fq 'redact_url' "$CLI"
grep -Fq 'AssetProgress' "$EVENT"
grep -Fq 'total_bytes: Option<u64>' "$EVENT"
! grep -RInE 'https?://[^" ]*(token|secret|key)=' "$ROOT/crates/focusa-terminal-ui/src/install" >/dev/null
echo "Spec 132 installer animation security contract: PASS"
