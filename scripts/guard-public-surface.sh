#!/usr/bin/env bash
# Public-surface guard: blocks private paths, private URLs, transcripts, and runtime leakage in buyer/public surfaces.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

HARD_PATTERNS=(
  '/home/wirebot'
  '/root/'
  'root@host'
  'wpuiai.com/wp-admin'
  'signalos.pro'
  '$384,330'
  '.focusa-private'
  'raw transcript'
  'raw transcripts'
  'docs/evidence/transcripts'
)

CAUTION_PATTERNS=(
  'MemoryMax'
  '.corrupt'
  'xShmMap'
  'production VPS'
  'tmux session'
  'Founders Forge'
  'dev_mode'
  'Stripe webhook'
  'license row'
)

TARGETS=(
  README.md
  SECURITY.md
  LICENSE-FAQ.md
  COMMERCIAL.md
  SUPPORT_TERMS.md
  docs/118-focusa-license-tiers-spec.md
  docs/current/CLI_REFERENCE_CURRENT.md
  docs/current/PRODUCTION_RELEASE_COMMANDS.md
  docs/current/TROUBLESHOOTING_CURRENT.md
  scripts/install-focusa.sh
  scripts/install-focusa.ps1
  crates/focusa-cli/src/commands/license.rs
  crates/focusa-cli/src/commands/install.rs
)

existing_targets=()
for target in "${TARGETS[@]}"; do
  [ -e "$target" ] && existing_targets+=("$target")
done

for pattern in "${HARD_PATTERNS[@]}"; do
  if rg -n --fixed-strings "$pattern" "${existing_targets[@]}" 2>/dev/null; then
    echo "public-surface guard failed: hard private/internal pattern: $pattern" >&2
    exit 1
  fi
done

for pattern in "${CAUTION_PATTERNS[@]}"; do
  if rg -n --fixed-strings "$pattern" "${existing_targets[@]}" 2>/dev/null; then
    echo "public-surface guard warning: review pattern: $pattern" >&2
  fi
done

echo "public-surface guard completed"
