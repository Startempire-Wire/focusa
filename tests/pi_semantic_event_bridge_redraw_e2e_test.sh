#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FOCUSA_BIN="${FOCUSA_BIN:-$ROOT/target/debug/focusa}"
BASE_URL="${FOCUSA_BASE_URL:-http://127.0.0.1:8787}"
TMP="$(mktemp -d /tmp/focusa-redraw-e2e.XXXXXX)"
STALE="/tmp/focusa-session-999999-1.txt"
trap 'rm -rf "$TMP"; rm -f "$STALE"' EXIT

[[ -x "$FOCUSA_BIN" ]] || { echo "missing focusa binary: $FOCUSA_BIN" >&2; exit 2; }
curl -fsS "$BASE_URL/v1/health" >/dev/null

cat > "$TMP/redraw.py" <<'PY'
#!/usr/bin/env python3
import sys
sys.stdout.write("api_key=VERYSECRET-FOCUSA-REDRAW-E2E\n")
payload = "x" * 2048
for i in range(10_000):
    sys.stdout.write(f"\x1b[H\x1b[2Jframe={i}\n{payload}\n")
    if i % 16 == 0:
        sys.stdout.flush()
PY
chmod +x "$TMP/redraw.py"

curl -fsS "$BASE_URL/v1/ecs/handles?limit=100&cursor=0&summary_only=false" \
  | jq -r '.handles[] | .id' | sort > "$TMP/before.ids"
recordings_before="$(find /tmp -maxdepth 1 -name 'focusa-session-*.txt' | wc -l)"

set +e
FOCUSA_RAW_PTY_CAPTURE=1 FOCUSA_MAGIC_DISABLE=1 \
  /usr/bin/time -f '%M' -o "$TMP/rss-kb" \
  timeout 120 "$FOCUSA_BIN" wrap -- "$TMP/redraw.py" >/dev/null 2>"$TMP/stderr"
rc=$?
set -e
[[ "$rc" -eq 124 ]] || { cat "$TMP/stderr" >&2; echo "expected bounded exit 124, got $rc" >&2; exit 1; }
rss_kb="$(tail -1 "$TMP/rss-kb")"
[[ "$rss_kb" =~ ^[0-9]+$ ]] || { cat "$TMP/rss-kb" >&2; exit 1; }
[[ "$rss_kb" -lt 262144 ]] || { echo "RSS exceeded 256 MiB" >&2; exit 1; }
[[ "$(find /tmp -maxdepth 1 -name 'focusa-session-*.txt' | wc -l)" -eq "$recordings_before" ]] \
  || { echo "stale PTY recording remained" >&2; exit 1; }

curl -fsS "$BASE_URL/v1/ecs/handles?limit=100&cursor=0&summary_only=false" > "$TMP/after.json"
jq -r '.handles[] | .id' "$TMP/after.json" | sort > "$TMP/after.ids"
comm -13 "$TMP/before.ids" "$TMP/after.ids" > "$TMP/new.ids"
new_count="$(while read -r id; do jq -r --arg id "$id" '.handles[] | select(.id==$id and (.label|startswith("bounded-pty-diagnostic-"))) | .id' "$TMP/after.json"; done < "$TMP/new.ids" | sed '/^$/d' | wc -l)"
[[ "$new_count" -eq 16 ]] || { echo "expected 16 ECS chunks, got $new_count" >&2; exit 1; }
part_one="$(jq -r '.handles[] | select((.label|startswith("bounded-pty-diagnostic-")) and (.label|endswith("part-01"))) | .id' "$TMP/after.json" | head -1)"
[[ -n "$part_one" ]]
curl -fsS -X POST "$BASE_URL/v1/ecs/rehydrate/$part_one" \
  -H 'content-type: application/json' -d '{"max_chars":2000}' > "$TMP/content.json"
! grep -q 'VERYSECRET-FOCUSA-REDRAW-E2E' "$TMP/content.json"
grep -q '\[REDACTED LINE\]' "$TMP/content.json"

# Governed stale-recording migration: dry-run receipt first, then auditable apply.
printf 'legacy stale recording\n' > "$STALE"
touch -d '2 hours ago' "$STALE"
cat > "$TMP/tiny.sh" <<'SH'
#!/usr/bin/env bash
printf 'semantic fixture ok\n'
SH
chmod +x "$TMP/tiny.sh"
FOCUSA_RAW_PTY_CAPTURE=1 FOCUSA_PTY_SCAVENGE_MODE=dry-run FOCUSA_DATA_DIR="$TMP/data" \
  "$FOCUSA_BIN" wrap -- "$TMP/tiny.sh" >/dev/null 2>/dev/null
[[ -f "$STALE" ]]
dry_receipt="$(find "$TMP/data/receipts" -name 'pty-scavenge-*.json' | head -1)"
[[ -n "$dry_receipt" ]]
jq -e '.schema=="focusa.pty_scavenge_receipt.v1" and .mode=="dry-run" and (.actions|any(.status=="would_remove"))' "$dry_receipt" >/dev/null
sleep 0.01
FOCUSA_RAW_PTY_CAPTURE=1 FOCUSA_PTY_SCAVENGE_MODE=apply FOCUSA_DATA_DIR="$TMP/data" \
  "$FOCUSA_BIN" wrap -- "$TMP/tiny.sh" >/dev/null 2>/dev/null
[[ ! -e "$STALE" ]]
apply_receipt="$(grep -l '"mode":"apply"' "$TMP"/data/receipts/pty-scavenge-*.json | head -1)"
[[ -n "$apply_receipt" ]]
jq -e '.mode=="apply" and (.actions|any(.status=="removed"))' "$apply_receipt" >/dev/null

echo "Pi semantic bridge 10,000-redraw E2E: PASS (8 MiB cap, 16 ECS chunks, redacted, RSS bounded, no stale file, dry-run/apply receipts)"
