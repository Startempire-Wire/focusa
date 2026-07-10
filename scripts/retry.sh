#!/usr/bin/env bash
# Shared retry helper for GitHub/network/transient workflow steps.
# Usage: scripts/retry.sh --attempts 3 --delay 5 -- command arg...
set -euo pipefail
attempts=3
delay=5
if [[ "${1:-}" == "--attempts" ]]; then
  attempts="$2"
  shift 2
fi
if [[ "${1:-}" == "--delay" ]]; then
  delay="$2"
  shift 2
fi
if [[ "${1:-}" == "--" ]]; then
  shift
fi
if [[ $# -eq 0 ]]; then
  echo "usage: scripts/retry.sh [--attempts N] [--delay SECONDS] -- command..." >&2
  exit 64
fi
for ((i=1; i<=attempts; i++)); do
  if "$@"; then
    exit 0
  fi
  status=$?
  if [[ "$i" -eq "$attempts" ]]; then
    exit "$status"
  fi
  echo "[retry] attempt $i/$attempts failed with $status; retrying in ${delay}s" >&2
  sleep "$delay"
done
