#!/bin/bash
set -euo pipefail

BASE_URL="${FOCUSA_BASE_URL:-http://127.0.0.1:8787}"
cd "$(dirname "$0")/.."
FOCUSA_BASE_URL="$BASE_URL" bun tests/pi_extension_runtime_authority_test.mts
