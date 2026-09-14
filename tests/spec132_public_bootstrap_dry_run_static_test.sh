#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SCRIPT="$ROOT/scripts/install-focusa.sh"
python3 - "$SCRIPT" <<'PY'
from pathlib import Path
import sys

text = Path(sys.argv[1]).read_text()

# The shell installer is a verified delegation bootstrap (Spec 132/152E):
# it never writes license files itself — the Rust installer performs all
# mutations. Dry-run must print a truthful plan and exit before any mutation.
assert 'if [ "$DRY_RUN" = 1 ]; then' in text
dry_run_start = text.index('if [ "$DRY_RUN" = 1 ]; then')
dry_run_end = text.index('exit 0\nfi', dry_run_start)
dry_run_block = text[dry_run_start:dry_run_end]
assert 'mutations: none' in dry_run_block, "dry-run plan is not truthful about mutations"
assert 'entitlement: signed authority lease' in dry_run_block
assert 'system install: %s' in dry_run_block

# System promotion is explicit, Linux-only, and delegated to Rust. The shell
# bootstrap never mutates /usr/local/bin itself.
assert '--system-install) SYSTEM_INSTALL=1' in text
assert '[ "$SYSTEM_INSTALL" = 0 ] || ARGS+=(--system-install)' in text
assert 'RUST_TARGET" != linux' in text
assert 'cp ' not in text and 'ln -s' not in text

# The delegation call must be constructed only after the dry-run guard.
delegate_index = text.index('ARGS=(install ')
assert delegate_index > dry_run_end, "delegation precedes dry-run guard"
delegate_tail = text[delegate_index:]
assert 'if "$BOOTSTRAP_BIN" "${ARGS[@]}"; then' in delegate_tail, "delegation does not run the Rust installer"
assert delegate_tail.count('if "$BOOTSTRAP_BIN" "${ARGS[@]}"; then') == 1, "delegation must execute exactly once"

# Uninstall delegation preserves data by default (Spec 132).
assert 'uninstall --yes' in text
assert '--keep-data' in text

print("Spec132 public bootstrap dry-run mutation fence: PASS")
PY
