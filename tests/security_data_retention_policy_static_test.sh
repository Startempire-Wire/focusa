#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DOC="$ROOT_DIR/docs/current/DATA_RETENTION_BACKUP_DELETION_POLICY.md"
PRIVACY="$ROOT_DIR/docs/current/PERSISTED_STATE_PRIVACY_CLASSES.md"
[[ -f "$DOC" ]] || { echo "missing data retention backup deletion policy doc" >&2; exit 1; }
[[ -f "$PRIVACY" ]] || { echo "missing persisted-state privacy classes doc" >&2; exit 1; }

for marker in \
  "Store inventory" \
  "Backup rules" \
  "Deletion rules" \
  "Retention rules" \
  "Restore rules" \
  "event_hash_chain" \
  "peer tokens P4" \
  "No deletion of Focusa data directories while the daemon is running" \
  "Project privacy erasure"; do
  if ! grep -Fq "$marker" "$DOC"; then
    echo "retention policy missing marker: $marker" >&2
    exit 1
  fi
done

for marker in "P4 Secret" "Peer sync tokens" "PERSISTED_STATE_PRIVACY_CLASSES"; do
  if ! grep -Fq "$marker" "$PRIVACY" "$DOC"; then
    echo "privacy/retention cross-reference missing marker: $marker" >&2
    exit 1
  fi
done

echo "✓ data retention/backup/deletion policy static markers present"
