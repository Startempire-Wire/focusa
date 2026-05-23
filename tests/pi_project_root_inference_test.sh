#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
bun "$ROOT_DIR/tests/pi_project_root_inference_test.mts"
