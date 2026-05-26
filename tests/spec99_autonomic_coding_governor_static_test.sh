#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DOC="$ROOT_DIR/docs/current/AUTONOMIC_CODING_WORKFLOW_GOVERNOR.md"

[[ -f "$DOC" ]] || { echo "missing autonomic coding governor doc" >&2; exit 1; }

for needle in \
  "project_vitals" \
  "focusa.project_vitals.v1" \
  "focusa.coding_governor.v1" \
  "Stuck detector" \
  "Safety immune check" \
  "Resource/homeostasis detector" \
  "Evidence/proof detector" \
  "approval_required" \
  "focusa_project_vitals" \
  "focusa_coding_governor_assess" \
  "tool_result_v1"; do
  if ! grep -Fq "$needle" "$DOC"; then
    echo "autonomic governor doc missing: $needle" >&2
    exit 1
  fi
done

for index in "$ROOT_DIR/docs/README.md" "$ROOT_DIR/docs/INDEX.md"; do
  if ! grep -Fq "AUTONOMIC_CODING_WORKFLOW_GOVERNOR.md" "$index"; then
    echo "docs index missing autonomic governor link: $index" >&2
    exit 1
  fi
done

echo "✓ autonomic coding governor static spec present"
