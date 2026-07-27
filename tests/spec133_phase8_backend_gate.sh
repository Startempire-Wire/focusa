#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$ROOT"
export CARGO_INCREMENTAL=${CARGO_INCREMENTAL:-0}

# Phase 8 is capability-truth first: optional adapters must either prove their
# declared behavior on this platform or report unsupported without fallback.
cargo test -q -p focusa-harness-adapters generic
cargo test -q -p focusa-harness-adapters
cargo test -q -p focusa-core silent_sessions::platform_backends
cargo test -q -p focusa-session-runner backend

case "$(uname -s)" in
  Linux)
    cargo test -q -p focusa-session-runner process_posix
    ;;
  Darwin)
    cargo test -q -p focusa-session-runner process_posix
    ;;
  MINGW*|MSYS*|CYGWIN*|Windows_NT)
    cargo test -q -p focusa-session-runner
    ;;
  *)
    echo "unsupported Phase 8 platform: $(uname -s)" >&2
    exit 1
    ;;
esac

python3 tests/spec133_platform_backends_static_test.py
printf '%s\n' 'PASS: Spec133 Phase 8 generic RPC/PTY, tmux migration, Herdr, and platform capability-truth matrix'
