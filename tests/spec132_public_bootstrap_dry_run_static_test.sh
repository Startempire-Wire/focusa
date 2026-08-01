#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SCRIPT="$ROOT/scripts/install-focusa.sh"
python3 - "$SCRIPT" <<'PY'
from pathlib import Path
import sys

text = Path(sys.argv[1]).read_text()
commercial_start = text.index('log "license valid: tier=${TIER}"')
eval_start = text.index('elif [ "$EVAL" = 1 ]; then', commercial_start)
commercial = text[commercial_start:eval_start]
eval_end = text.index('\nelse\n  # Should be unreachable', eval_start)
evaluation = text[eval_start:eval_end]

for name, block in (("commercial", commercial), ("evaluation", evaluation)):
    guard = block.index('if [ "$DRY_RUN" = 1 ]; then')
    first_write = min(
        position
        for token in ("write_license_authority", "write_license_json", "write_license_receipt")
        if (position := block.index(token)) >= 0
    )
    assert guard < first_write, f"{name} license writes precede dry-run guard"
    assert "DRY RUN: would write" in block, f"{name} dry-run receipt is not truthful"

assert 'if [ "$DRY_RUN" = 0 ]; then\n  migrate_legacy_license\nfi' in text
assert (
    'if [ "$DRY_RUN" = 0 ]; then\n'
    '  mkdir -p "$BIN_DIR" "$STATE_DIR" "$CONFIG_DIR" "$LIBEXEC_DIR"\n'
    'fi'
) in text
assert '--dry-run                print the install plan; do not write anything' in text
print("Spec132 public bootstrap dry-run mutation fence: PASS")
PY
