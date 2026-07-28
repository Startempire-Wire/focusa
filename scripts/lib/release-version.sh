#!/usr/bin/env bash
# Parse a Focusa release version from a canonical target-qualified asset name.

release_version_from_asset_name() {
  local path="${1:-}"
  local expected="${2:-}"
  local base=""
  base="$(basename "$path")"

  # Exact expected-version matching is authoritative and prevents a stable
  # version from consuming the first character of `-x86_64` as a suffix.
  if [[ -n "$expected" && "$base" == *"-v${expected}-"* ]]; then
    printf '%s\n' "$expected"
    return 0
  fi

  # Supported tags: stable, dev, rc(.N), and nightly.N. The final hyphen is
  # the required boundary before the target triple.
  if [[ "$base" =~ -v([0-9]+\.[0-9]+\.[0-9]+(-(dev|rc)(\.[0-9]+)?|-nightly\.[0-9]+)?)-[A-Za-z0-9_] ]]; then
    printf '%s\n' "${BASH_REMATCH[1]}"
    return 0
  fi
  printf '\n'
}
