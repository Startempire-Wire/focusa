#!/usr/bin/env bash
# Establish the repository's isolated, hash-locked Python test environment.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
PYTHON_BIN="${PYTHON_BIN:-python3}"
VENV_DIR="${FOCUSA_PYTHON_TEST_VENV:-$ROOT/.focusa-python-test-venv}"

"$PYTHON_BIN" -m venv "$VENV_DIR"
"$VENV_DIR/bin/python" -m pip install \
  --disable-pip-version-check \
  --only-binary=:all: \
  --require-hashes \
  --requirement "$ROOT/requirements-test.txt"
"$VENV_DIR/bin/python" "$ROOT/scripts/ci/verify-python-test-env.py"

if [[ -n "${GITHUB_PATH:-}" ]]; then
  printf '%s\n' "$VENV_DIR/bin" >> "$GITHUB_PATH"
else
  printf 'Python test environment: %s\n' "$VENV_DIR"
fi
