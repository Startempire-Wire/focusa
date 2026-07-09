#!/usr/bin/env bash
# Focusa provider guard shim template.
# Used for asana / linear / github / gitlab / jira (and similar task providers).
# Replaces each provider CLI in PATH. Detects close-shaped commands and blocks.
#
# Usage: rename this file to the provider name (e.g., copy to /usr/local/bin/linear)
# The PROVIDER name can also be passed via FOCUSA_GUARD_PROVIDER env var.

set -euo pipefail

# Provider name (override via env or rename the file)
PROVIDER="${FOCUSA_GUARD_PROVIDER:-$(basename "${0##*/}")}"

# Find the real binary
REAL="${PROVIDER}.real"
for path in /usr/local/bin /usr/bin $HOME/.local/bin; do
  if [ -x "$path/$REAL" ]; then
    REAL="$path/$REAL"
    break
  fi
done

# Inspect args
args=("$@")
contains_close=false
for a in "${args[@]}"; do
  case "$a" in
    close|done|complete|archive|--status|status:*"done"*|state:*"closed"*)
      contains_close=true
      ;;
  esac
done

# CLI-specific patterns
case "$PROVIDER" in
  github|gitlab)
    if [[ "${args[0]:-}" == "issue" ]] || [[ "${args[0]:-}" == "pr" ]]; then
      if [[ "${args[1]:-}" == "close" ]]; then
        contains_close=true
      fi
    fi
    ;;
  asana|linear|jira)
    if [[ "${args[0]:-}" == "complete" ]] || [[ "${args[0]:-}" == "done" ]] || [[ "${args[0]:-}" == "close" ]]; then
      contains_close=true
    fi
    ;;
esac

if [[ "$contains_close" == "true" && "${FOCUSA_GUARD_BYPASS:-0}" != "1" ]]; then
  cat <<EOF
[$PROVIDER-guard shim] Intercepted close-shaped command: $PROVIDER $*
[$PROVIDER-guard shim] Raw close bypasses evidence validation.
[$PROVIDER-guard shim] Use 'focusa work-item close <id> --from-workpoint <WP>' with provider=$PROVIDER instead.
[$PROVIDER-guard shim] To bypass this guard (NOT recommended), set FOCUSA_GUARD_BYPASS=1
EOF
  exit 4
fi

# Pass through to real binary if installed
if [ -x "$REAL" ]; then
  exec "$REAL" "$@"
else
  # Real binary not installed — emit a stub command so user understands
  echo "[$PROVIDER-guard shim] Real '$PROVIDER' binary not installed (expected at $REAL)."
  echo "[$PROVIDER-guard shim] Use 'focusa work-item close <id> --from-workpoint <WP>' with provider=$PROVIDER."
  exit 0
fi
