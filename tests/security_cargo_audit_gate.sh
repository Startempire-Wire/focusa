#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
AUDIT_BIN="${CARGO_AUDIT:-cargo-audit}"
OUT="${TMPDIR:-/tmp}/focusa-cargo-audit-gate.json"
ERR="${OUT}.stderr"

if ! command -v "$AUDIT_BIN" >/dev/null 2>&1; then
  echo "cargo-audit not found; install cargo-audit or set CARGO_AUDIT=/path/to/cargo-audit" >&2
  exit 2
fi

cd "$ROOT_DIR"
"$AUDIT_BIN" audit --json >"$OUT.raw" 2>"$ERR" || true
python3 - "$OUT.raw" "$OUT" <<'PY'
from pathlib import Path
import sys
raw=Path(sys.argv[1]).read_text(errors='ignore')
idx=raw.find('{"database"')
if idx < 0:
    print('cargo-audit JSON payload not found', file=sys.stderr)
    sys.exit(3)
Path(sys.argv[2]).write_text(raw[idx:])
PY
vulns="$(jq -r '.vulnerabilities.count // 0' "$OUT")"
if [[ "$vulns" != "0" ]]; then
  jq -r '.vulnerabilities.list[] | "RUSTSEC: \(.advisory.id) \(.advisory.package) \(.package.version) — \(.advisory.title)"' "$OUT" >&2
  exit 1
fi
warnings="$(jq -r '((.warnings.unmaintained // []) | length) + ((.warnings.unsound // []) | length) + ((.warnings.yanked // []) | length)' "$OUT")"
echo "✓ cargo-audit vulnerabilities=0 warnings=$warnings report=$OUT"
