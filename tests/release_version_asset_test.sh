#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
# shellcheck source=scripts/lib/release-version.sh
source "$ROOT_DIR/scripts/lib/release-version.sh"

assert_version() {
  local expected="$1"
  local asset="$2"
  local hint="${3:-}"
  local actual=""
  actual="$(release_version_from_asset_name "$asset" "$hint")"
  [[ "$actual" == "$expected" ]] || {
    echo "version parse mismatch: asset=$asset hint=$hint expected=$expected actual=$actual" >&2
    exit 1
  }
}

assert_version 0.9.137 focusa-daemon-v0.9.137-x86_64-unknown-linux-musl
assert_version 0.9.137 focusa-daemon-v0.9.137-x86_64-unknown-linux-musl 0.9.137
assert_version 0.9.137-dev focusa-daemon-v0.9.137-dev-x86_64-unknown-linux-musl
assert_version 0.9.137-rc.2 focusa-daemon-v0.9.137-rc.2-aarch64-apple-darwin
assert_version 0.9.137-nightly.42 focusa-daemon-v0.9.137-nightly.42-x86_64-unknown-linux-gnu
assert_version '' focusa-daemon-v0.9.137

grep -Fq 'release_version_from_asset_name "$BINARY" "$EXPECTED_VERSION"' "$ROOT_DIR/scripts/install-daemon.sh"
echo "release asset version parsing: PASS"
