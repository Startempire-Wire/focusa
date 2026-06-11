#!/usr/bin/env bash
# Backward-compatible shim. Prefer scripts/phone-bridge-transport.sh.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"

args=()
mode=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --check) mode="check"; shift ;;
    --write) mode="write"; shift ;;
    --print-proxy) mode="proxy-snippets"; shift ;;
    *) args+=("$1"); shift ;;
  esac
done

if [[ -z "$mode" ]]; then
  mode="options"
fi

exec "${ROOT_DIR}/scripts/phone-bridge-transport.sh" "$mode" "${args[@]}"
