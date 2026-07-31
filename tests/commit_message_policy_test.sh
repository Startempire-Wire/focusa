#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VALIDATOR="$ROOT/scripts/validate-commit-messages.sh"
TMP="$(mktemp -d "${TMPDIR:-/tmp}/focusa-commit-policy.XXXXXX")"
trap 'rm -rf "$TMP"' EXIT

assert_pass() {
  local message="$1"
  printf '%s\n' "$message" > "$TMP/message"
  "$VALIDATOR" --message-file "$TMP/message" >/dev/null
}

assert_fail() {
  local message="$1"
  printf '%s\n' "$message" > "$TMP/message"
  if "$VALIDATOR" --message-file "$TMP/message" >/dev/null 2>&1; then
    printf 'expected rejection: %s\n' "$message" >&2
    exit 1
  fi
}

assert_pass $'fix: preserve compaction continuity\n\nBeads: focusa-im74e'
assert_pass 'proof(spec135): close verified Mission Canvas gate'
assert_pass 'merge: synchronize concurrent implementation lane'
assert_pass 'Merge pull request #123 from example/branch'
assert_pass "Merge remote-tracking branch 'origin/feature-spec135-mission-canvas' into feature/spec-135-mission-canvas"
assert_fail $'Beads: focusa-im74e\n\nfix: hidden real subject'
assert_fail 'focusa-im74e'
assert_fail 'WIP'
assert_fail 'made changes'

# Published malformed history is accepted only by exact full hash during range
# validation; commit-msg validation remains strict for every new commit.
grep -q '435f1cb9be6b91fb279c141408868e6c63d67e68' "$VALIDATOR"
grep -q 'b0f0ebc20f50af17b4541ee4a279ea0b0d0d93ae' "$VALIDATOR"
[[ "$(grep -Eo '[0-9a-f]{40}' "$VALIDATOR" | sort -u | wc -l)" -eq 2 ]]

mkdir -p "$TMP/repo"
cd "$TMP/repo"
git init -q
git config user.name "Focusa Test"
git config user.email "focusa-test@example.invalid"
printf 'seed\n' > seed.txt
git add seed.txt
mkdir -p scripts
cp "$VALIDATOR" scripts/validate-commit-messages.sh
chmod +x scripts/validate-commit-messages.sh
cat > .git/hooks/pre-push <<'UPSTREAM'
#!/usr/bin/env sh
# TEST_UPSTREAM_PRE_PUSH
exit 0
UPSTREAM
chmod +x .git/hooks/pre-push
"$ROOT/scripts/install-commit-message-hooks.sh" >/dev/null
"$ROOT/scripts/install-commit-message-hooks.sh" >/dev/null
grep -q 'TEST_UPSTREAM_PRE_PUSH' .git/hooks/pre-push.focusa-upstream
if grep -q 'validate-commit-messages.sh' .git/hooks/pre-push.focusa-upstream; then
  echo 'reinstall replaced preserved upstream pre-push hook' >&2
  exit 1
fi

git commit -q -m 'test: preserve meaningful subject'
[[ "$(git log -1 --format=%s)" == "test: preserve meaningful subject" ]]
if git commit --allow-empty -q -m 'Beads: focusa-im74e' 2>/dev/null; then
  echo 'commit-msg hook accepted a Beads-only subject' >&2
  exit 1
fi

git commit --allow-empty -q --no-verify -m 'Beads: focusa-im74e'
if "$VALIDATOR" --range HEAD~1..HEAD >/dev/null 2>&1; then
  echo 'range validation accepted a Beads-only subject' >&2
  exit 1
fi

rg -q 'validate-commit-messages.sh --range' "$ROOT/.github/workflows/ci.yml"
rg -q 'validate-commit-messages.sh --range' "$ROOT/scripts/create-dev-release-tag.sh"

printf 'PASS: meaningful commit subjects preserved; Beads-only and generic subjects rejected\n'
