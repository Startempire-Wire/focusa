#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
INSTALLER="$ROOT/scripts/install-focusa.sh"
WORK="$(mktemp -d /tmp/focusa-public-uninstall.XXXXXX)"
trap 'rm -rf "$WORK"' EXIT

mkdir -p "$WORK/home/.focusa/bin" "$WORK/home/.pi/agent/extensions/focusa"
cat > "$WORK/home/.focusa/bin/focusa" <<'FAKE'
#!/usr/bin/env bash
printf '%s\n' "$*" > "$FOCUSA_TEST_ARGS"
FAKE
chmod +x "$WORK/home/.focusa/bin/focusa"

FOCUSA_TEST_ARGS="$WORK/preserve.args" HOME="$WORK/home" bash "$INSTALLER" --uninstall >/dev/null
[[ "$(cat "$WORK/preserve.args")" == "uninstall --yes --keep-data" ]] \
  || { echo "FAIL: public uninstall did not preserve data by default" >&2; exit 1; }

mkdir -p "$WORK/home/.pi/agent/extensions/focusa"
FOCUSA_TEST_ARGS="$WORK/purge.args" HOME="$WORK/home" bash "$INSTALLER" --uninstall --purge-data >/dev/null
[[ "$(cat "$WORK/purge.args")" == "uninstall --yes" ]] \
  || { echo "FAIL: explicit purge did not select destructive CLI posture" >&2; exit 1; }

set +e
HOME="$WORK/home" bash "$INSTALLER" --purge-data >"$WORK/invalid.out" 2>&1
status=$?
set -e
[[ $status -eq 64 ]] || { echo "FAIL: --purge-data without --uninstall exited $status, expected 64" >&2; exit 1; }
grep -qF -- '--purge-data requires --uninstall' "$WORK/invalid.out" \
  || { echo "FAIL: invalid purge posture lacked actionable error" >&2; exit 1; }

echo "PASS: public uninstall preserves data by default and purge requires explicit intent"
