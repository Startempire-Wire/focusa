#!/usr/bin/env bash
# Focusa development script.
#
# Usage: ./scripts/dev.sh [daemon|cli|test|check]

set -euo pipefail
cd "$(dirname "$0")/.."

case "${1:-check}" in
  daemon)
    echo "Starting Focusa daemon in dev mode..."
    cargo run --bin focusa-daemon
    ;;
  cli)
    shift
    echo "Running Focusa CLI..."
    cargo run --bin focusa -- "$@"
    ;;
  test)
    echo "Running tests..."
    cargo test --workspace
    ;;
  check)
    echo "Checking workspace..."
    cargo check --workspace
    cargo clippy --workspace -- -W clippy::all
    echo "✓ All checks passed"
    ;;
  *)
    echo "Usage: $0 {daemon|cli|test|check}"
    exit 1
    ;;
esac
