#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
INSTALLER="$ROOT/scripts/install-focusa.sh"
WORK="$(mktemp -d /tmp/focusa-public-uninstall.XXXXXX)"
trap 'rm -rf "$WORK"' EXIT

mkdir -p "$WORK/home/.focusa/bin" "$WORK/home/.pi/agent/extensions/focusa"
printf '%s\n' 'PRESERVE_PI_SKILLS' > "$WORK/home/.pi/agent/extensions/focusa/operator.txt"
cat > "$WORK/home/.focusa/bin/focusa" <<'FAKE'
#!/usr/bin/env bash
printf '%s\n' "$*" > "$FOCUSA_TEST_ARGS"
FAKE
chmod +x "$WORK/home/.focusa/bin/focusa"

expect_refusal() {
  local name="$1" match="$2"
  shift 2
  local output="$WORK/$name.log" marker="$WORK/$name.args"
  set +e
  FOCUSA_TEST_ARGS="$marker" HOME="$WORK/home" bash "$INSTALLER" "$@" >"$output" 2>&1
  local status=$?
  set -e
  [[ "$status" -eq 64 ]] || { echo "FAIL: $name returned $status, wanted 64" >&2; exit 1; }
  grep -qF -- "$match" "$output" || { echo "FAIL: $name lacked denial" >&2; exit 1; }
  [[ ! -e "$marker" ]] || { echo "FAIL: $name reached an uninstall executable" >&2; exit 1; }
}

run_delegated() {
  local name="$1" expected="$2"
  shift 2
  local marker="$WORK/$name.args"
  FOCUSA_TEST_ARGS="$marker" HOME="$WORK/home" bash "$INSTALLER" "$@" >/dev/null
  [[ -f "$marker" && "$(cat "$marker")" == "$expected" ]] || {
    echo "FAIL: $name delegated unsafe or unexpected Rust flags" >&2; exit 1;
  }
  grep -qF PRESERVE_PI_SKILLS "$WORK/home/.pi/agent/extensions/focusa/operator.txt" ||
    { echo "FAIL: existing Pi customer data was modified" >&2; exit 1; }
}

# The old wrapper passed --yes before checking --dry-run, deleting customer
# state despite a dry-run request. Preview must reach Rust's dry-run only.
run_delegated preview 'uninstall --dry-run --keep-license --keep-data' --uninstall --dry-run
run_delegated purge_preview 'uninstall --dry-run --keep-license' --uninstall --purge-data --dry-run

# No unconfirmed operation may reach the canonical product-owned Rust command.
expect_refusal unconfirmed '--uninstall requires --confirm' --uninstall
expect_refusal unconfirmed_purge '--uninstall requires --confirm' --uninstall --purge-data
expect_refusal one_confirmation '--purge-data requires separate --confirm-purge-data' --uninstall --purge-data --confirm
expect_refusal implicit_yes '--uninstall requires --confirm' --uninstall --assume-yes
expect_refusal orphan_purge '--purge-data requires --uninstall' --purge-data
expect_refusal orphan_confirm '--confirm requires --uninstall' --confirm
expect_refusal orphan_purge_confirm '--confirm-purge-data requires --purge-data' --uninstall --confirm-purge-data

# Explicitly authorized normal removal still preserves independent license and
# data. Explicit purge removes Focusa-owned data only, never Pi agent skills.
run_delegated preserve 'uninstall --yes --keep-license --keep-data' --uninstall --confirm
run_delegated purge 'uninstall --yes --keep-license --confirm-purge-data' --uninstall --purge-data --confirm --confirm-purge-data

echo "PASS: public Focusa uninstall dry-run is nonmutating, confirmations and licenses preserved"
