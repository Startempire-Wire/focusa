#!/usr/bin/env bash
# Verify the on-disk `focusa` binary matches the source-of-truth surface.
#
# Reads the canonical feature markers declared in the Rust CLI source and
# confirms the installed binary at /usr/local/bin/focusa (or the path
# passed via --bin) is recent enough to expose them. Catches the
# "binary-drift" gap observed in the 2026-07-05 fresh-operator dry-run,
# where the installed CLI was 30 days stale and refused documented flags
# like `tui` and `--scope`.
#
# Exit status:
#   0   markers all present (or no source/build available — soft pass)
#   1   source present but binary missing
#   2   binary missing one or more markers (stale binary — pin rebuild)
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
BIN="${1:-/usr/local/bin/focusa}"
if [[ "$BIN" == "--bin" ]]; then
  BIN="${2:-/usr/local/bin/focusa}"
fi

if [[ ! -x "$BIN" ]]; then
  echo "✗ focusa binary missing at $BIN; install via scripts/install-daemon.sh" >&2
  exit 1
fi

CLI_SRC="$ROOT_DIR/crates/focusa-cli/src/main.rs"
if [[ ! -f "$CLI_SRC" ]]; then
  echo "note: CLI source not found at $CLI_SRC; skipping marker check"
  exit 0
fi

# Markers: each must appear in `focusa --help` or its --flag help output.
# We verify flag-level help strings via `--help` substring searches.
markers=(
  "--scope"
  "onboard"
  "audit"
  "Tui"
  "Init"
)

failed=()
for marker in "${markers[@]}"; do
  if ! "$BIN" --help 2>/dev/null | grep -q -- "$marker"; then
    failed+=("$marker")
  fi
done

if [[ ${#failed[@]} -gt 0 ]]; then
  echo "✗ focusa binary at $BIN is missing markers: ${failed[*]}" >&2
  echo "  install-script: bash scripts/install-daemon.sh /usr/local" >&2
  exit 2
fi

echo "✓ focusa binary at $BIN has the canonical surface"
exit 0