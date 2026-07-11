#!/usr/bin/env bash
# Spec 112 §15A.5 — fail-closed install dry-run integration test.
#
# Verifies the real CLI exits successfully, emits a parseable InstallPlan, and
# mutates neither the isolated HOME/XDG fixture nor system install paths.

set -euo pipefail
cd "$(dirname "$0")/.."

echo "=== Spec 112 install smoke test ==="

if [[ ! -x target/debug/focusa && ! -x target/release/focusa ]]; then
    echo "Building focusa CLI..."
    cargo build -p focusa-cli --bin focusa
fi

if [[ -x target/debug/focusa ]]; then
    FOCUSA_BIN="$PWD/target/debug/focusa"
elif [[ -x target/release/focusa ]]; then
    FOCUSA_BIN="$PWD/target/release/focusa"
else
    echo "FAIL: focusa CLI build did not produce an executable" >&2
    exit 1
fi

FIXTURE="$(mktemp -d)"
trap 'rm -rf "$FIXTURE"' EXIT
mkdir -p "$FIXTURE/home" "$FIXTURE/xdg-config" "$FIXTURE/xdg-data" "$FIXTURE/xdg-state" "$FIXTURE/xdg-cache"

snapshot_system_paths() {
    for root in /usr/local/bin /usr/local/lib /usr/local/libexec; do
        [[ -d "$root" ]] || continue
        find "$root" -maxdepth 2 -type f -name 'focusa*' -print0 2>/dev/null
    done | sort -z | xargs -0 -r sha256sum
}

SYSTEM_BEFORE="$(snapshot_system_paths)"
set +e
OUTPUT="$(
    env \
        HOME="$FIXTURE/home" \
        XDG_CONFIG_HOME="$FIXTURE/xdg-config" \
        XDG_DATA_HOME="$FIXTURE/xdg-data" \
        XDG_STATE_HOME="$FIXTURE/xdg-state" \
        XDG_CACHE_HOME="$FIXTURE/xdg-cache" \
        "$FOCUSA_BIN" install --target=linux --dry-run --json 2>&1
)"
RC=$?
set -e

if [[ $RC -ne 0 ]]; then
    printf 'FAIL: dry-run exited %d\n%s\n' "$RC" "$OUTPUT" >&2
    exit 1
fi

PLAN_JSON="$OUTPUT" python3 - "$FIXTURE/home/.focusa" <<'PY'
import json
import os
import pathlib
import sys

expected_root = pathlib.Path(sys.argv[1])
try:
    plan = json.loads(os.environ["PLAN_JSON"])
except Exception as exc:
    raise SystemExit(f"FAIL: dry-run output is not valid JSON: {exc}")

required = {
    "target",
    "channel",
    "install_root",
    "assets_planned",
    "symlink_planned",
    "service_manager_planned",
    "shell_rc_plan",
    "license_mode",
    "notes",
}
missing = sorted(required - set(plan))
if missing:
    raise SystemExit(f"FAIL: InstallPlan missing fields: {missing}")
if plan["target"] != "linux":
    raise SystemExit(f"FAIL: expected linux target, got {plan['target']!r}")
if pathlib.Path(plan["install_root"]) != expected_root:
    raise SystemExit(
        f"FAIL: install_root escaped fixture: {plan['install_root']!r} != {str(expected_root)!r}"
    )
if not isinstance(plan["assets_planned"], list) or not plan["assets_planned"]:
    raise SystemExit("FAIL: InstallPlan has no planned assets")
PY

SYSTEM_AFTER="$(snapshot_system_paths)"
if [[ "$SYSTEM_BEFORE" != "$SYSTEM_AFTER" ]]; then
    echo "FAIL: dry-run mutated a system Focusa install path" >&2
    diff -u <(printf '%s\n' "$SYSTEM_BEFORE") <(printf '%s\n' "$SYSTEM_AFTER") || true
    exit 1
fi

if find "$FIXTURE" -type f -o -type l | grep -q .; then
    echo "FAIL: dry-run wrote files or symlinks inside the isolated fixture" >&2
    find "$FIXTURE" \( -type f -o -type l \) -print >&2
    exit 1
fi

if [[ -e "$FIXTURE/home/.focusa" ]]; then
    echo "FAIL: dry-run created the planned install root" >&2
    exit 1
fi

echo "PASS: isolated install dry-run emitted a valid plan with zero filesystem mutation"
