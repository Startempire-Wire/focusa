#!/bin/bash
# Thin compatibility wrapper; consolidated semantic checks now live in doc70_shared_interfaces_lifecycle_contract_test.sh
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
bash "${ROOT_DIR}/tests/doc70_shared_interfaces_lifecycle_contract_test.sh"
