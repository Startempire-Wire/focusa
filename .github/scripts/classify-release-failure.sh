#!/usr/bin/env bash
# Compatibility wrapper for the DRY Focusa release-path classifier.
# Workflows consume shell-safe KEY=value lines; agents/tests can call the
# Python classifier directly for structured JSON.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
exec python3 "$ROOT_DIR/scripts/classify-ci-failure.py" --format env "${1:--}"
