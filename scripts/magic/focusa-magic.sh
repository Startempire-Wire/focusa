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
#   FOCUSA_MAGIC_DISABLE=1  -> bypass wrapping
#   FOCUSA_BIN=/path/to/focusa

if [[ "${FOCUSA_MAGIC_DISABLE:-}" == "1" ]]; then
  exec "$@"
fi

if [[ $# -lt 1 ]]; then
  echo "usage: $(basename "$0") <harness> [args...]" >&2
  exit 2
fi

harness="$1"
shift

focusa_bin="${FOCUSA_BIN:-focusa}"

# If focusa isn't available, fail open.
if ! command -v "$focusa_bin" >/dev/null 2>&1; then
  exec "$harness" "$@"
fi

# Avoid recursion if shim shadows the real binary.
# Resolve the underlying harness from PATH *after* removing this script's dir.
shim_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
path_sans_shim="$(echo ":$PATH:" | sed "s/:${shim_dir//\//\\/}://g" | sed 's/^://; s/:$//')"

real_harness="$(PATH="$path_sans_shim" command -v "$harness" || true)"
if [[ -z "$real_harness" ]]; then
  # If not found, still try to run as-is (maybe it's a shell function).
  real_harness="$harness"
fi

exec "$focusa_bin" wrap -- "$real_harness" "$@"
