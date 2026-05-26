#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DOC="$ROOT_DIR/docs/current/PATH_TRAVERSAL_SECURITY_TESTS.md"
CLEANUP="$ROOT_DIR/crates/focusa-cli/src/commands/cleanup.rs"
[[ -f "$DOC" ]] || { echo "missing path traversal security tests doc" >&2; exit 1; }

for marker in \
  "tmp_glob_match_is_prefix_suffix_only_under_tmp" \
  "safe_target_keeps_absolute_paths_inside_trash_root" \
  "../focusa-audit.json" \
  "root.join(\"tmp/focusa-audit.json\")"; do
  if ! grep -Fq "$marker" "$CLEANUP"; then
    echo "cleanup path traversal test marker missing: $marker" >&2
    exit 1
  fi
done

for marker in \
  "CWE-22" \
  "Path-sensitive route inventory" \
  "Attachments" \
  "Work-loop silent sessions"; do
  if ! grep -Fq "$marker" "$DOC"; then
    echo "path traversal doc marker missing: $marker" >&2
    exit 1
  fi
done

echo "✓ path traversal static markers present"
