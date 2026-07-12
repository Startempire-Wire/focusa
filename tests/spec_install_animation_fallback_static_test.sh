#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CAP="$ROOT/crates/focusa-terminal-ui/src/capabilities.rs"
PRES="$ROOT/crates/focusa-terminal-ui/src/install/presenter.rs"
RENDER="$ROOT/crates/focusa-terminal-ui/src/install/renderer.rs"
grep -Fq 'InstallRendererMode::Plain' "$CAP"
grep -Fq 'InstallRendererMode::Silent' "$CAP"
grep -Fq 'InstallRendererMode::ReducedMotion' "$CAP"
grep -Fq 'PlainPresenter' "$PRES"
grep -Fq 'SilentPresenter' "$PRES"
grep -Fq 'TerminalGuard' "$RENDER"
grep -Fq 'guard.restore' "$RENDER"
echo "Spec 132 installer animation fallback/restoration contract: PASS"
