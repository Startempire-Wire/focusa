#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
UI="$ROOT/crates/focusa-terminal-ui/src/install"
CLI="$ROOT/crates/focusa-cli/src/commands/install.rs"
for path in "$UI/canvas.rs" "$UI/continuity_core.rs" "$UI/matrix_rain.rs" "$UI/glow_base.rs" "$UI/renderer.rs" "$UI/presenter.rs" "$CLI"; do
  test -f "$path" || { echo "FAIL: missing $path" >&2; exit 1; }
done
grep -Fq '"▄"' "$UI/renderer.rs"
grep -Fq 'FOCUSA INSTALL' "$UI/renderer.rs"
grep -Fq 'Continuity Core' "$UI/renderer.rs"
grep -Fq 'Matrix field' "$UI/renderer.rs"
grep -Fq 'phase completion' "$UI/renderer.rs"
grep -Fq 'InstallEvent' "$CLI"
echo "Spec 132 installer animation static contract: PASS"
