#!/usr/bin/env bash
# Hard gate: block commit/push when closed beads lack evidence citations.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
ISSUES_FILE="${ROOT_DIR}/.beads/issues.jsonl"
CUTOFF_DATE="${BD_EVIDENCE_POLICY_CUTOFF:-2026-04-18}"

if [ ! -f "$ISSUES_FILE" ]; then
  echo "[bd-evidence] no .beads/issues.jsonl found; skipping" >&2
  exit 0
fi

missing=$(jq -r --arg cutoff "$CUTOFF_DATE" '
  select(.status=="closed")
  | select((.closed_at // .updated_at // "") >= $cutoff)
  | . as $i
  | (($i.close_reason // "") + "\n" + ($i.notes // "")) as $txt
  | ( ($txt | test("Evidence citations:"; "i")) and
      ($txt | test("tests/|docs/|crates/|/v1/|http"; "i")) ) as $ok
  | select($ok | not)
  | [.id, ($i.closed_at // $i.updated_at // ""), (($i.close_reason // "") | gsub("\n"; " "))] | @tsv
' "$ISSUES_FILE")

if [ -n "$missing" ]; then
  echo "[bd-evidence] BLOCKED: closed beads missing required evidence citations (cutoff ${CUTOFF_DATE})" >&2
  echo "$missing" | while IFS=$'\t' read -r id ts reason; do
    echo "  - ${id} (closed_at=${ts})" >&2
    if [ -n "$reason" ]; then
      echo "    close_reason=${reason}" >&2
    fi
  done
  echo "[bd-evidence] Required format in close_reason or notes:" >&2
  echo "  Evidence citations: tests/... ; docs/... ; crates/... ; /v1/..." >&2
  exit 1
fi

exit 0
