#!/usr/bin/env bash
set -euo pipefail

# Installs Focusa "magic" shims into ~/.local/bin.
# These shims transparently route harness invocations through `focusa wrap`.
#
# Example:
#   ./scripts/magic/install.sh pi claude
# Then:
#   pi "hello"  -> focusa wrap -- pi "hello"

if [[ $# -lt 1 ]]; then
  echo "usage: $0 <harness> [harness...]" >&2
  exit 2
fi

dest="${HOME}/.local/bin"
mkdir -p "$dest"

src_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
src="$src_dir/focusa-magic.sh"

for h in "$@"; do
  ln -sf "$src" "$dest/$h"
  echo "installed shim: $dest/$h -> $src"
done

echo "ensure ~/.local/bin is on PATH"