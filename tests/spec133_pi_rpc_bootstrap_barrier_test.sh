#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

export CARGO_INCREMENTAL=0
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-${TMPDIR:-/tmp}/focusa-spec133-pi-rpc-bootstrap-target}"

cargo fmt --all -- --check
cargo test -p focusa-core silent_session_bootstrap::tests --locked
cargo test -p focusa-session-runner mutation_posix::tests --locked
cargo test -p focusa-harness-adapters --test adapter_contract_runtime --locked
cargo clippy \
  -p focusa-core \
  -p focusa-session-runner \
  -p focusa-harness-adapters \
  --lib --tests --locked -- -D warnings

printf 'PASS: Spec 133 Pi RPC and verified AgentBootstrap mutation barrier\n'
