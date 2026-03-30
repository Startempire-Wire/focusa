#!/usr/bin/env bash
set -euo pipefail

# Focusa Magic Wrapper
# Goal: transparently route harness CLIs through `focusa wrap`.
# Works best when installed as a shim named after the harness (e.g. `pi`).
#
# Usage patterns:
# - as `pi` shim:   pi "prompt"  -> focusa wrap -- pi "prompt"
# - explicit:       focusa-magic.sh pi "prompt"
#
# Env:
#   FOCUSA_MAGIC_DISABLE=1  -> bypass wrapping and run real harness
#   FOCUSA_BIN=/path/to/focusa

# The harness name is derived from THIS shim's filename (not the first arg)
# This allows: pi "prompt" -> focusa wrap -- pi "prompt"
# Even if someone calls: focusa-magic.sh pi "prompt"
harness_name="$(basename "$0")"

# Find shim directory (where this script lives)
shim_dir="$(cd "$(dirname "$0")" && pwd)"

# Build PATH without the shim directory to avoid re-invoking ourselves
# Use bash array to split and filter PATH entries
IFS=':' read -ra path_parts <<< "$PATH"
path_sans_shim=""
for part in "${path_parts[@]}"; do
    if [[ "$part" != "$shim_dir" ]]; then
        path_sans_shim="${path_sans_shim:+$path_sans_shim:}$part"
    fi
done

# If FOCUSA_MAGIC_DISABLE=1, run real harness directly (don't wrap)
if [[ "${FOCUSA_MAGIC_DISABLE:-}" == "1" ]]; then
    real_harness=""

    # First try: find in PATH (excluding shim)
    real_harness="$(PATH="$path_sans_shim" command -v "$harness_name" 2>/dev/null)"

    # Second try: if not found, check known harness locations
    if [[ -z "$real_harness" ]]; then
        case "$harness_name" in
            pi)    real_harness="/opt/cpanel/ea-nodejs20/bin/pi" ;;
            claude) real_harness="" ;;  # Not installed
            letta)  real_harness="" ;;  # Not installed
            opencode) real_harness="" ;;  # Not installed
            *)
                # Fallback: assume harness_name is in shim_dir
                if [[ -x "${shim_dir}/${harness_name}" ]]; then
                    real_harness="${shim_dir}/${harness_name}"
                fi
                ;;
        esac
    fi

    # If still not found, just use the harness_name (let shell handle it)
    [[ -z "$real_harness" || ! -x "$real_harness" ]] && real_harness="$harness_name"

    exec "$real_harness" "$@"
fi

# Handle interactive mode (no args) - still route through focusa wrap
# This enables Focusa to intercept interactive TUI sessions
focusa_bin="${FOCUSA_BIN:-focusa}"

# Check if focusa binary exists and is executable
# Handle both PATH names and absolute paths
if [[ -x "$focusa_bin" ]] || ( [[ "$focusa_bin" != */* ]] && command -v "$focusa_bin" >/dev/null 2>&1 ); then
    exec "$focusa_bin" wrap -- "$harness_name" "$@"
fi

# Focusa not available, run the real harness directly
real_harness="$(PATH="$path_sans_shim" command -v "$harness_name" 2>/dev/null || echo "$harness_name")"
exec "$real_harness" "$@"
