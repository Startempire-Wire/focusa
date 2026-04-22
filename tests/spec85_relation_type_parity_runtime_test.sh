#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

TMP_EXPECTED="/tmp/spec85_expected_links.txt"
TMP_API="/tmp/spec85_api_links.txt"
TMP_CLI="/tmp/spec85_cli_links.txt"

python3 - <<'PY' > "$TMP_EXPECTED"
import re, pathlib
p=pathlib.Path('crates/focusa-api/src/routes/ontology.rs')
text=p.read_text(encoding='utf-8')
m=re.search(r'const LINK_TYPES:\s*&\[&str\]\s*=\s*&\[(.*?)\];', text, re.S)
if not m:
    raise SystemExit('LINK_TYPES block missing')
vals=sorted(set(re.findall(r'"([a-z0-9_]+)"', m.group(1))))
print('\n'.join(vals))
PY

curl -sS http://127.0.0.1:8787/v1/ontology/primitives \
  | jq -r '.link_types[] | (.name // .)' | sort -u > "$TMP_API"

cargo run -q -p focusa-cli -- --json ontology primitives \
  | jq -r '.link_types[] | (.name // .)' | sort -u > "$TMP_CLI"

expected_n=$(wc -l < "$TMP_EXPECTED" | tr -d ' ')
api_n=$(wc -l < "$TMP_API" | tr -d ' ')
cli_n=$(wc -l < "$TMP_CLI" | tr -d ' ')

if ! diff -u "$TMP_EXPECTED" "$TMP_API" >/tmp/spec85_diff_expected_api.txt; then
  echo "SPEC85 parity mismatch: expected vs api"
  cat /tmp/spec85_diff_expected_api.txt
  exit 1
fi

if ! diff -u "$TMP_API" "$TMP_CLI" >/tmp/spec85_diff_api_cli.txt; then
  echo "SPEC85 parity mismatch: api vs cli"
  cat /tmp/spec85_diff_api_cli.txt
  exit 1
fi

echo "SPEC85 relation parity: PASS expected=$expected_n api=$api_n cli=$cli_n"
