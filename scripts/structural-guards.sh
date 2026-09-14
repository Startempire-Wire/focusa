#!/usr/bin/env bash
# Structural guards — prevent the release failure modes cataloged in
# docs/current/RELEASE_FAILURE_MODE_CATALOG_2026-08-17.md.
# Fast, read-only, no network. Wired as pre-commit + pre-push + CI.
set -u
ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"
FAIL=0

fail() { echo "STRUCTURAL-GUARD: $1" >&2; FAIL=1; }

# Mode 3: conflict markers must never be committed.
markers="$(git grep -n -E '^(<<<<<<< |>>>>>>> )' -- ':!docs/current/RELEASE_FAILURE_MODE_CATALOG_2026-08-17.md' 2>/dev/null | head -5)"
if [ -n "$markers" ]; then
  fail "conflict markers present (catalog mode 3):"
  echo "$markers" | sed 's/^/  /' >&2
fi

# Mode 19/14: double visibility (pub pub) and items-after-test not greppable here;
# double-pub is a hard parse error — catch it cheaply.
pubpub="$(git grep -n 'pub pub ' -- '*.rs' 2>/dev/null | head -3)"
if [ -n "$pubpub" ]; then
  fail "double pub visibility (catalog mode 18):"
  echo "$pubpub" | sed 's/^/  /' >&2
fi

# Mode 21: every crates/*/Cargo.toml must be a workspace member (invisible crates = ungated code).
members="$(sed -n '/^members = \[/,/^\]/p' Cargo.toml | grep -oE '"crates/[a-z0-9-]+"' | tr -d '"')"
for manifest in crates/*/Cargo.toml; do
  crate="${manifest#crates/}"; crate="${crate%/Cargo.toml}"
  if ! printf '%s\n' "$members" | grep -qx "crates/$crate"; then
    fail "crate crates/$crate is not a workspace member (catalog mode 21)"
  fi
done

# Mode 23: release.yml tag patterns must be fnmatch globs, not regex.
if [ -f .github/workflows/release.yml ] && grep -q 'tags:' .github/workflows/release.yml; then
  if grep -E 'v\[0-9\]\+' .github/workflows/release.yml >/dev/null 2>&1; then
    fail "release.yml tag patterns use regex '+' — fnmatch globs required (catalog mode 23)"
  fi
fi

# Mode 2: staged deletions of files that exist on origin/main are squash-loss
# (only enforceable pre-push when origin/main is reachable).
if git rev-parse --verify -q origin/main >/dev/null 2>&1; then
  deleted="$(git diff --name-only --cached --diff-filter=D 2>/dev/null)"
  for f in $deleted; do
    if git cat-file -e "origin/main:$f" 2>/dev/null; then
      fail "staged deletion of main-owned file: $f (catalog mode 2 — restore from origin/main unless intentionally removed upstream)"
    fi
  done
fi

# Mode 12 reminder: no sleep-poll loops belong in committed scripts.
pollers="$(git diff --cached --name-only 2>/dev/null | xargs grep -l 'sleep [0-9]\+.*tail\|while.*sleep' 2>/dev/null | head -3)"
if [ -n "$pollers" ]; then
  fail "sleep-poll patterns in staged scripts (catalog mode 12, TBQ rule): $pollers"
fi

if [ "$FAIL" -eq 1 ]; then
  echo "STRUCTURAL-GUARD: blocked. See docs/current/RELEASE_FAILURE_MODE_CATALOG_2026-08-17.md" >&2
  exit 1
fi
exit 0
