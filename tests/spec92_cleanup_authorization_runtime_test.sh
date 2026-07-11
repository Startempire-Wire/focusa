#!/usr/bin/env bash
# Spec 92 / focusa-ux2qx.11 — safe cleanup authorization and execution proof.
set -euo pipefail
cd "$(dirname "$0")/.."
ROOT="$PWD"
FIXTURE="$(mktemp -d)"
GLOBAL_TMP="/tmp/focusa-cleanup-runtime-$$.log"
trap 'rm -rf "$FIXTURE"; rm -f "$GLOBAL_TMP"' EXIT
fail() { echo "FAIL: $*" >&2; exit 1; }

cargo build -q -p focusa-cli --bin focusa
BIN="$ROOT/target/debug/focusa"
PROJECT="$FIXTURE/project"
TRASH="$FIXTURE/trash"
mkdir -p "$PROJECT/.tmp/nested" "$PROJECT/.beads" "$PROJECT/data" "$PROJECT/target" "$FIXTURE/home"
printf '%s\n' '{"schema":"focusa.project.v1","project_id":"cleanup-test","canonical_name":"Cleanup Test"}' \
  > "$PROJECT/.focusa-project.json"
printf 'generated\n' > "$PROJECT/.tmp/nested/generated.txt"
printf 'preserve\n' > "$PROJECT/.beads/issues.jsonl"
printf 'preserve\n' > "$PROJECT/data/state.json"
printf 'preserve\n' > "$PROJECT/target/artifact"
printf 'global\n' > "$GLOBAL_TMP"

# Missing --safe is a hard error and performs no mutation.
set +e
HOME="$FIXTURE/home" FOCUSA_TRASH_DIR="$TRASH" \
  "$BIN" cleanup --project-root "$PROJECT" >"$FIXTURE/unsafe.out" 2>&1
unsafe_status=$?
set -e
[[ $unsafe_status -ne 0 ]] || fail "cleanup without --safe exited zero"
[[ -f "$PROJECT/.tmp/nested/generated.txt" ]] || fail "unsafe cleanup mutated project"

# Default dry-run previews project residue only; global /tmp remains excluded.
HOME="$FIXTURE/home" FOCUSA_TRASH_DIR="$TRASH" \
  "$BIN" --json cleanup --safe --dry-run --project-root "$PROJECT" \
  > "$FIXTURE/preview.json"
jq -e '
  .status=="completed" and
  (.details.actions|any(.status=="would_move" and (.path|endswith("/.tmp")))) and
  (.warnings|index("global /tmp cleanup excluded; pass --include-global-tmp explicitly"))!=null
' "$FIXTURE/preview.json" >/dev/null \
  || { cat "$FIXTURE/preview.json" >&2; fail "project cleanup preview incomplete"; }
[[ -f "$PROJECT/.tmp/nested/generated.txt" ]] || fail "dry-run mutated project"
if grep -qF "$GLOBAL_TMP" "$FIXTURE/preview.json"; then
  fail "default cleanup preview included global /tmp without consent"
fi

# Explicit global opt-in is visible in dry-run but still non-mutating.
HOME="$FIXTURE/home" FOCUSA_TRASH_DIR="$TRASH" \
  "$BIN" --json cleanup --safe --dry-run --include-global-tmp --project-root "$PROJECT" \
  > "$FIXTURE/global-preview.json"
grep -qF "$GLOBAL_TMP" "$FIXTURE/global-preview.json" \
  || fail "explicit global /tmp preview omitted matching residue"
[[ -f "$GLOBAL_TMP" ]] || fail "global dry-run mutated residue"

# Authorized execution moves generated project residue and preserves critical paths.
HOME="$FIXTURE/home" FOCUSA_TRASH_DIR="$TRASH" \
  "$BIN" --json cleanup --safe --project-root "$PROJECT" > "$FIXTURE/execute.json"
jq -e '
  .status=="completed" and
  (.details.actions|any(.status=="completed" and (.path|endswith("/.tmp"))))
' "$FIXTURE/execute.json" >/dev/null \
  || { cat "$FIXTURE/execute.json" >&2; fail "authorized cleanup did not complete"; }
[[ ! -e "$PROJECT/.tmp" ]] || fail "generated project residue remains"
for path in "$PROJECT/.beads/issues.jsonl" "$PROJECT/data/state.json" "$PROJECT/target/artifact"; do
  [[ -f "$path" ]] || fail "cleanup removed preserved path: $path"
done
trash_target="$(jq -r '.details.actions[] | select(.status=="completed" and (.path|endswith("/.tmp"))) | .target' "$FIXTURE/execute.json")"
[[ -f "$trash_target/nested/generated.txt" ]] || fail "recoverable trash copy missing"

# Repeat is idempotent and remains successful.
HOME="$FIXTURE/home" FOCUSA_TRASH_DIR="$TRASH" \
  "$BIN" --json cleanup --safe --project-root "$PROJECT" > "$FIXTURE/repeat.json"
jq -e '.status=="completed" and ([.details.actions[].status]|all(.=="skipped"))' \
  "$FIXTURE/repeat.json" >/dev/null \
  || { cat "$FIXTURE/repeat.json" >&2; fail "repeat cleanup is not idempotent"; }

echo "PASS: cleanup requires explicit authorization, is recoverable/idempotent, and scopes global tmp separately"
