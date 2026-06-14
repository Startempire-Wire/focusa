#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DOC="$ROOT_DIR/docs/current/MIGRATION_BACKUP_POLICY.md"
RETENTION="$ROOT_DIR/docs/current/DATA_RETENTION_BACKUP_DELETION_POLICY.md"
LOCAL="$ROOT_DIR/docs/current/LOCAL_FIRST_DATA_MODEL.md"
SPEC="$ROOT_DIR/docs/106-focusa-vision-tightening-spec.md"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

[ -f "$DOC" ] || fail "MIGRATION_BACKUP_POLICY.md missing"
[ -f "$RETENTION" ] || fail "DATA_RETENTION_BACKUP_DELETION_POLICY.md missing"
[ -f "$LOCAL" ] || fail "LOCAL_FIRST_DATA_MODEL.md missing"
for section in 'State classes' 'Backup checklist' 'Restore checklist' 'Migration checklist' 'Deletion / archive policy' 'Related docs' 'Proof'; do
  rg -n -F "$section" "$DOC" >/dev/null || fail "migration/backup policy missing section $section"
done
pass "migration/backup sections present"

for marker in 'daemon data directory' 'SQLite/event store' 'Focus State, Workpoints, Trajectory, HLT ledger' 'Evidence refs and proof handles' 'device pairing ledger' 'generated current docs and release proof artifacts'; do
  rg -n -F "$marker" "$DOC" >/dev/null || fail "state class missing $marker"
done
pass "migration/backup state classes present"

for marker in 'Stop or quiesce the daemon' 'append-only ledgers' 'API tokens, pairing tokens' 'backup timestamp' 'restore smoke test' 'Workpoint resume and Trajectory view'; do
  rg -n -F "$marker" "$DOC" >/dev/null || fail "backup/restore safeguard missing $marker"
done
pass "backup/restore safeguards present"

for marker in 'project_root + continuity_id' 'Re-pair devices' 'Regenerate current docs' 'release/version proof' 'Never delete `.beads/`'; do
  rg -n -F "$marker" "$DOC" >/dev/null || fail "migration/delete marker missing $marker"
done
pass "migration/delete boundaries present"

rg -n -F 'Append-only ledgers' "$LOCAL" >/dev/null || fail "local-first model missing append-only ledger wording"
for marker in 'MIGRATION_BACKUP_POLICY.md' 'migration_backup_policy_static_test.sh'; do
  rg -n -F "$marker" "$SPEC" >/dev/null || fail "Spec106 missing migration/backup proof marker $marker"
done
pass "Spec106 references migration/backup proof"

echo "migration backup policy static test: PASS"
