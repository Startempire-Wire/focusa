#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
INSTALLER="$ROOT_DIR/scripts/install-focusa.sh"

bash -n "$INSTALLER"

require() {
  local pattern="$1"
  local message="$2"
  grep -Fq -- "$pattern" "$INSTALLER" || {
    printf 'FAIL: %s\n' "$message" >&2
    exit 1
  }
}

require 'linux)' 'missing explicit linux alias'
require 'RUST_TARGET="linux"' 'linux alias does not bind Rust target'
require 'TRIPLE="x86_64-unknown-linux-musl"' 'linux x64 alias lacks release triple'
require 'TRIPLE="aarch64-unknown-linux-gnu"' 'linux arm64 alias lacks release triple'
require 'RUST_TARGET="darwin"' 'darwin alias does not bind Rust target'
require 'TRIPLE="x86_64-apple-darwin"' 'darwin x64 alias lacks release triple'
require 'TRIPLE="aarch64-apple-darwin"' 'darwin arm64 alias lacks release triple'
require 'RUST_TARGET="windows-x64"' 'windows x64 alias does not bind Rust target'
require 'TRIPLE="x86_64-pc-windows-msvc"' 'windows x64 alias lacks release triple'
require 'RUST_TARGET="windows-arm64"' 'windows arm64 alias does not bind Rust target'
require 'TRIPLE="aarch64-pc-windows-msvc"' 'windows arm64 alias lacks release triple'
require 'TARGET="$TRIPLE"' 'resolved release triple is not assigned to asset target'

printf 'PASS: explicit installer aliases map to release triples and Rust targets\n'
