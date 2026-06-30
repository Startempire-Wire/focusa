#!/usr/bin/env bash
# Scoped safe disk cleanup for Focusa automation.
# Only removes rebuildable or temporary Focusa-specific paths.
set -euo pipefail

PROJECT_ROOT="${FOCUSA_CLEANUP_PROJECT_ROOT:-$PWD}"
BACKUP_DIR="${FOCUSA_CLEANUP_BACKUP_DIR:-/usr/local/lib/focusa/backups}"
RUNNER_ROOT="${FOCUSA_CLEANUP_RUNNER_ROOT:-}"
MIN_FREE_GB="${FOCUSA_CLEANUP_MIN_FREE_GB:-15}"
MAX_USAGE_PCT="${FOCUSA_CLEANUP_MAX_USAGE_PCT:-92}"
TMP_GLOB_1="/tmp/focusa-release-*"
TMP_GLOB_2="/tmp/focusa-deploy-*"
APPLY=0
VERBOSE=0
RETENTION_DAYS="${FOCUSA_CLEANUP_RETENTION_DAYS:-14}"
BACKUP_KEEP="${FOCUSA_CLEANUP_BACKUP_KEEP:-5}"

log() { printf '[focusa-cleanup] %s\n' "$*"; }
warn() { printf '[focusa-cleanup][warn] %s\n' "$*" >&2; }
die() { printf '[focusa-cleanup][error] %s\n' "$*" >&2; exit 1; }

usage() {
  cat <<'USAGE'
Usage: scripts/safe-disk-cleanup.sh [options]

Options:
  --apply                 Actually remove safe cruft. Default is dry-run.
  --project-root PATH     Focusa repo root whose target/.tmp can be cleaned.
  --backup-dir PATH       Deploy backup dir to prune by age.
  --runner-root PATH      Optional self-hosted runner root to prune temp files.
  --min-free-gb N         Required free space after cleanup (default 15).
  --max-usage-pct N       Required max disk usage after cleanup (default 92).
  --retention-days N      Age cutoff for pruning backup/temp files (default 14).
  --backup-keep N         Keep only the N most recent backups (default 5).
  --verbose               Show every removal candidate.
  --help                  Show help.
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --apply) APPLY=1; shift ;;
    --project-root) PROJECT_ROOT="${2:?}"; shift 2 ;;
    --backup-dir) BACKUP_DIR="${2:?}"; shift 2 ;;
    --runner-root) RUNNER_ROOT="${2:?}"; shift 2 ;;
    --min-free-gb) MIN_FREE_GB="${2:?}"; shift 2 ;;
    --max-usage-pct) MAX_USAGE_PCT="${2:?}"; shift 2 ;;
    --retention-days) RETENTION_DAYS="${2:?}"; shift 2 ;;
    --backup-keep) BACKUP_KEEP="${2:?}"; shift 2 ;;
    --verbose) VERBOSE=1; shift ;;
    --help|-h) usage; exit 0 ;;
    *) die "Unknown argument: $1" ;;
  esac
done

have() { command -v "$1" >/dev/null 2>&1; }
bytes_to_gb() { python3 - "$1" <<'PY'
import sys
print(round(int(sys.argv[1]) / (1024**3), 2))
PY
}

disk_state() {
  df -Pk "$PROJECT_ROOT" | awk 'NR==2 {gsub(/%/,"",$5); printf "%s %s\n", $4, $5}'
}

free_kb_and_usage() {
  local raw
  raw="$(disk_state)"
  FREE_KB="${raw%% *}"
  USAGE_PCT="${raw##* }"
}

report_state() {
  free_kb_and_usage
  local free_gb
  free_gb="$(python3 - "$FREE_KB" <<'PY'
import sys
print(round(int(sys.argv[1]) / (1024*1024), 2))
PY
)"
  log "disk state: free=${free_gb}GiB usage=${USAGE_PCT}% project_root=$PROJECT_ROOT"
}

remove_path() {
  local path="$1"
  [[ -e "$path" ]] || return 0
  if [[ "$VERBOSE" -eq 1 ]]; then
    du -sh "$path" 2>/dev/null || true
  fi
  if [[ "$APPLY" -eq 1 ]]; then
    rm -rf --one-file-system "$path"
    log "removed $path"
  else
    log "would remove $path"
  fi
}

prune_find() {
  local base="$1"
  local expr="$2"
  [[ -d "$base" ]] || return 0
  while IFS= read -r path; do
    [[ -n "$path" ]] || continue
    remove_path "$path"
  done < <(find "$base" -mindepth 1 $expr 2>/dev/null | sort)
}

cleanup_targets() {
  remove_path "$PROJECT_ROOT/target"
  remove_path "$PROJECT_ROOT/.tmp"
  remove_path "$PROJECT_ROOT/apps/menubar/src-tauri/target"
}

cleanup_tmp_dirs() {
  local path
  for path in $TMP_GLOB_1 $TMP_GLOB_2; do
    [[ -e "$path" ]] || continue
    if [[ -d "$path" ]]; then
      if find "$path" -maxdepth 0 -mtime +1 >/dev/null 2>&1; then
        remove_path "$path"
      fi
    fi
  done
}

cleanup_backups() {
  [[ -d "$BACKUP_DIR" ]] || return 0
  # V2: keep only the N most recent backups; prune everything else.
  # This replaces the older "keep for N days" rule so the list is bounded
  # even if the install path is hit many times per day.
  if (( BACKUP_KEEP < 0 )); then
    return 0
  fi
  while IFS= read -r path; do
    [[ -n "$path" ]] || continue
    remove_path "$path"
  done < <(
    find "$BACKUP_DIR" -type f -name '*.bak' -printf '%T@ %p\n' 2>/dev/null \
      | sort -rn \
      | tail -n +$((BACKUP_KEEP + 1)) \
      | sed 's/^[0-9.]* //'
  )
}

cleanup_runner_temp() {
  [[ -n "$RUNNER_ROOT" ]] || return 0
  [[ -d "$RUNNER_ROOT" ]] || return 0
  prune_find "$RUNNER_ROOT" "\\( -path '*/_work/_temp/*' -o -path '*/_diag/*' \\) -mtime +$RETENTION_DAYS"
}

report_state
cleanup_targets
cleanup_tmp_dirs
cleanup_backups
cleanup_runner_temp
report_state

free_kb_and_usage
FREE_GB_INT=$(( FREE_KB / 1024 / 1024 ))
if (( FREE_GB_INT < MIN_FREE_GB )); then
  die "free disk ${FREE_GB_INT}GiB is below required threshold ${MIN_FREE_GB}GiB"
fi
if (( USAGE_PCT > MAX_USAGE_PCT )); then
  die "disk usage ${USAGE_PCT}% exceeds allowed threshold ${MAX_USAGE_PCT}%"
fi

if [[ "$APPLY" -eq 1 ]]; then
  log "cleanup applied successfully"
else
  log "dry-run cleanup check passed"
fi
