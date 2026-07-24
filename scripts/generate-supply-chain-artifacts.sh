#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="${1:-$ROOT/release-proof/supply-chain}"
mkdir -p "$OUT"

command -v cargo-deny >/dev/null || { echo "cargo-deny is required" >&2; exit 2; }
command -v cargo-about >/dev/null || { echo "cargo-about is required" >&2; exit 2; }
command -v syft >/dev/null || { echo "syft is required" >&2; exit 2; }

cd "$ROOT"
cargo deny --all-features check
cargo about generate --config about.toml config/cargo-about.hbs \
  > "$OUT/THIRD_PARTY_NOTICES.generated.html"
syft "dir:$ROOT" -o "cyclonedx-json=$OUT/focusa.cdx.json"
syft "dir:$ROOT" -o "spdx-json=$OUT/focusa.spdx.json"

python3 - "$OUT" <<'PY'
import hashlib, json, pathlib, sys
root = pathlib.Path(sys.argv[1])
artifacts = []
for path in sorted(root.glob("*")):
    if path.name == "manifest.json" or not path.is_file():
        continue
    artifacts.append({
        "path": path.name,
        "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
        "bytes": path.stat().st_size,
    })
(root / "manifest.json").write_text(json.dumps({
    "schema": "focusa.supply_chain_manifest.v1",
    "artifacts": artifacts,
}, indent=2, sort_keys=True) + "\n")
PY

echo "Supply-chain artifacts written to $OUT"
