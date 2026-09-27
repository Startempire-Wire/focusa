#!/usr/bin/env bash
# Guardian-gated cleanup for Focusa release-only regenerable artifacts.
set -euo pipefail
cd "$(dirname "$0")/.."

MODE="${1:-pre}"
case "$MODE" in
  pre|post) ;;
  *) echo "usage: $0 [pre|post]" >&2; exit 64 ;;
esac

command -v guardian >/dev/null 2>&1 || {
  echo "Server Guardian CLI is required for release cleanup routing." >&2
  exit 1
}
systemctl is-active --quiet server-guardian.service || {
  echo "Server Guardian daemon is not active; refusing automated cleanup." >&2
  exit 1
}

disk_percent() {
  df -P / | awk 'NR==2 {gsub(/%/, "", $5); print $5}'
}

# Reserve a private receipt location before any cleanup. Keep prior runs intact.
umask 077
artifact_dir="$(mktemp -d "${TMPDIR:-/tmp}/focusa-guardian-release-cleanup-${MODE}.XXXXXXXX")"
artifact="${artifact_dir}/receipt.json"

before="$(disk_percent)"
guardian_status="$(guardian check disk 2>&1 || true)"
cleaned=()

if [[ "$MODE" == "post" || "$before" -ge 90 ]]; then
  if [[ -d target ]]; then
    cargo clean >/dev/null
    cleaned+=("cargo-target")
  fi
fi

if [[ "$MODE" == "post" && -d apps/pi-extension/node_modules ]]; then
  find apps/pi-extension/node_modules -depth -delete
  cleaned+=("pi-extension-node_modules")
fi

after="$(disk_percent)"
python3 - "$artifact" "$MODE" "$before" "$after" "$guardian_status" "${cleaned[*]:-none}" <<'PY'
import json, sys
path, mode, before, after, guardian_status, cleaned = sys.argv[1:]
value = {
    "schema": "focusa.guardian_release_cleanup.v1",
    "status": "completed",
    "mode": mode,
    "artifact_path": path,
    "disk_percent_before": int(before),
    "disk_percent_after": int(after),
    "cleaned": [] if cleaned == "none" else cleaned.split(),
    "guardian": guardian_status.splitlines()[0] if guardian_status else "checked",
    "scope": "regenerable_release_artifacts_only",
}
with open(path, "x") as handle:
    json.dump(value, handle, indent=2, sort_keys=True)
    handle.write("\n")
print(json.dumps(value, sort_keys=True))
PY
