#!/usr/bin/env bash
# Focusa bd guard shim — wraps bd to intercept close-shaped commands.
# Replaces `bd` in PATH. When close-shaped args are detected, redirects
# to `focusa work-item close` instead.
#
# Install: cp scripts/bd-guard-shim.sh /usr/local/bin/bd (or $HOME/.local/bin/bd)
# Verify: which bd && bd --version

set -euo pipefail

# Real bd binary location. Adjust if bd is elsewhere.
REAL_BD="${FOCUSA_REAL_BD:-$(command -v br 2>/dev/null || echo /usr/local/bin/br)}"

# Inspect for close-shaped commands
args=("$@")
contains_close=false
for a in "${args[@]}"; do
  case "$a" in
    close|--status|--status=closed|--status=done)
      contains_close=true
      ;;
  esac
done

# Detect: bd close <id> [...], bd update ... --status closed/done
if [[ "${args[0]:-}" == "close" ]] || [[ "${args[0]:-}" == "update" && " ${args[*]} " =~ --status[\ =](closed|done) ]]; then
  contains_close=true
fi

if [[ "$contains_close" == "true" && "${FOCUSA_BD_GUARD_BYPASS:-0}" != "1" ]]; then
  cat <<EOF
[bd-guard shim] Intercepted close-shaped command: bd $*
[bd-guard shim] Raw close bypasses evidence validation.
[bd-guard shim] Use 'focusa work-item close <id> --from-workpoint <WP>' instead.
[bd-guard shim] To bypass this guard (NOT recommended), set FOCUSA_BD_GUARD_BYPASS=1
EOF
  exit 4  # blocked by guard
fi

# Pass through to real bd (or br, since bd is a symlink)
exec "$REAL_BD" "$@"
