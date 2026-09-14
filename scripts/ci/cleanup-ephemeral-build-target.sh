#!/usr/bin/env bash
set -euo pipefail

target="${1:-}"
if [[ -z "$target" ]]; then
  echo "cleanup target is required" >&2
  exit 64
fi

resolved="$(realpath -m -- "$target")"
case "$resolved" in
  /tmp/focusa-ci-[0-9]*-[0-9]*|/tmp/focusa-ci-local-[0-9]*-[0-9]*) ;;
  *)
    echo "refusing non-ephemeral cleanup target: $resolved" >&2
    exit 65
    ;;
esac

if [[ ! -e "$resolved" ]]; then
  echo "ephemeral build target already absent: $resolved"
  exit 0
fi
if [[ ! -d "$resolved" || -L "$resolved" ]]; then
  echo "refusing non-directory or symlink cleanup target: $resolved" >&2
  exit 66
fi

owner_uid="$(stat -c %u -- "$resolved")"
current_uid="$(id -u)"
if [[ "$owner_uid" != "$current_uid" ]]; then
  echo "refusing target owned by uid $owner_uid (current uid $current_uid): $resolved" >&2
  exit 67
fi

before_bytes="$(du -sb -- "$resolved" | awk '{print $1}')"
python3 - "$resolved" <<'PY'
from pathlib import Path
import shutil
import sys

path = Path(sys.argv[1])
shutil.rmtree(path)
PY
[[ ! -e "$resolved" ]] || {
  echo "ephemeral build target still exists after cleanup: $resolved" >&2
  exit 68
}
echo "ephemeral build cleanup complete: path=$resolved reclaimed_bytes=$before_bytes"
