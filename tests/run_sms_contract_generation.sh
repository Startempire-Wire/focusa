#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
if [[ "$(id -u)" == 0 ]]; then
  exec /usr/local/bin/as-user wirebot "$ROOT/tests/run_sms_contract_generation.sh"
fi
cd "$ROOT"
bun scripts/generate-agent-capability-descriptors.ts --write
bun scripts/generate-agent-tool-docs.ts
bun scripts/generate-agent-capability-descriptors.ts --check
bun scripts/generate-agent-tool-docs.ts --check
