#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
core="$root/crates/focusa-core/src/runtime"
backup_io="$core/backup_io.rs"
api="$root/crates/focusa-api/src"

require() {
  local pattern="$1" file="$2" message="$3"
  if ! grep -Eq "$pattern" "$file"; then
    printf 'FAIL: %s (%s)\n' "$message" "$file" >&2
    exit 1
  fi
}

require 'rusqlite = .*"backup"' "$root/crates/focusa-core/Cargo.toml" 'rusqlite online backup feature missing'
require '^pub mod backup;' "$core/mod.rs" 'backup runtime not exported'
require '^pub mod backup_contracts;' "$core/mod.rs" 'backup contracts not exported'
require '^pub mod event_retention;' "$core/mod.rs" 'event retention runtime not exported'
require 'focusa\.backup_policy\.v1' "$core/backup_contracts.rs" 'versioned backup policy missing'
require 'focusa\.backup_generation_manifest\.v1' "$core/backup_contracts.rs" 'versioned manifest missing'
require 'focusa\.backup_receipt\.v1' "$core/backup_contracts.rs" 'versioned receipt missing'
require 'breach_incremental_not_implemented' "$core/backup.rs" 'nonconforming incremental RPO breach missing'
require 'breach_restore_unproven' "$core/backup.rs" 'restore-proof RPO gate missing'
require 'Backup::new' "$backup_io" 'SQLite online backup API missing'
require 'online backup did not converge' "$backup_io" 'hot-writer liveness fallback missing'
require 'PRAGMA quick_check' "$core/backup.rs" 'SQLite integrity check missing'
require 'create_incremental_generation' "$core/backup_incremental.rs" 'incremental recovery-point runtime missing'
require 'restore_generation' "$core/backup_restore.rs" 'isolated restore runtime missing'
require 'settle_generation_off_host' "$core/backup_offhost.rs" 'off-host settlement runtime missing'
require 'execute_retention' "$core/backup_retention.rs" 'GFS retention runtime missing'
require 'latest_off_host_receipt' "$core/backup_retention.rs" 'off-host prune gate missing'
require 'newer_restore_safe' "$core/backup_retention.rs" 'newer restore prune gate missing'
require 'content_sha256' "$core/backup_incremental.rs" 'content-addressed chunk integrity missing'
require 'create_new\(true\)' "$backup_io" 'overlap lock missing'
require 'manifest_sha256' "$core/backup.rs" 'manifest hash verification missing'
require '^pub mod backups;' "$api/routes/mod.rs" 'backup API route not exported'
require '^pub mod events_retention;' "$api/routes/mod.rs" 'event-retention API route not exported'
require 'merge\(routes::backups::router\(\)\)' "$api/server.rs" 'backup router not mounted'
require 'maintenance_loop' "$api/routes/backup_maintenance.rs" 'backup maintenance coordinator not wired'
require 'stack_size\(32 \* 1024 \* 1024\)' "$api/routes/backups.rs" 'large SQLite backup worker stack is not bounded explicitly'
require 'verification_cache_hit' "$core/backup.rs" 'unchanged backup artifacts are rehashed on every health cycle'
require 'Perform a deep cryptographic verification even when metadata is unchanged' "$core/backup.rs" 'explicit verification no longer guarantees a deep artifact hash'
require 'spawn_backup_operation\("scheduled-full"' "$api/routes/backup_maintenance.rs" 'scheduled full backup bypasses the bounded worker'
require 'merge\(routes::events_retention::router\(\)\)' "$api/server.rs" 'event-retention router not mounted'
require 'backup_recovery_gate_not_met' "$api/routes/events_retention.rs" 'event-retention recovery gate missing'
require 'focusa.event_retention_receipt.v1' "$api/routes/events_retention.rs" 'event-retention receipt missing'
require 'file.sync_all' "$core/event_retention.rs" 'cold-export fsync missing'

if grep -Eq 'focusa stop|systemctl stop|pkill' "$core/backup.rs" "$backup_io" "$core/backup_incremental.rs" "$core/backup_restore.rs" "$api/routes/backups.rs" "$api/routes/backup_maintenance.rs"; then
  echo 'FAIL: backup implementation contains daemon-stop authority' >&2
  exit 1
fi

printf 'PASS: Spec 181 backup runtime/consumer wiring present; RPO remains breached for the nonconforming snapshot-chunk prototype.\n'
