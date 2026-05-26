#!/bin/bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"

assert_has() {
  local file="$1"
  local pattern="$2"
  local label="$3"
  if rg -n "$pattern" "${ROOT_DIR}/${file}" >/dev/null; then
    echo "✓ PASS: ${label}"
  else
    echo "✗ FAIL: ${label}" >&2
    echo "Missing pattern '${pattern}' in ${file}" >&2
    exit 1
  fi
}

assert_has README.md 'persisted evaluations|dispatch-readiness diagnostics|Business Source License 1\.1' 'README reflects metacog/work-loop/license current state'
assert_has docs/README.md 'evaluation readback/promotion|ontology memory-pipeline artifacts|work-loop dispatch-readiness health|source-available/commercial licensing boundary' 'docs README reflects current runtime surfaces'
assert_has docs/current/CURRENT_RUNTIME_STATUS.md 'evaluations persist as first-class records|/v1/work-loop/health exposes dispatch readiness|ontology memory-pipeline promotions' 'runtime status reflects latest shipped functionality'
assert_has docs/current/CLI_REFERENCE_CURRENT.md 'recent-evaluations|focusa metacognition recent-evaluations' 'CLI reference includes metacognition evaluation readback'
assert_has docs/current/RUNTIME_CONFIG_KEYS.md 'evaluation_memory|/v1/metacognition/evaluations/recent|adjustment records and evaluation records' 'runtime config docs include evaluation retention/readback'
assert_has docs/focusa-tools/work-loop.md '/v1/work-loop/health.*dispatch readiness' 'work-loop tool docs include health readiness semantics'
assert_has docs/focusa-tools/tools/focusa_metacog_evaluate_outcome.md 'promote.*retrieval memory|evaluations/recent|recent-evaluations' 'metacog evaluate tool doc includes persisted evaluation/promotion readback'
assert_has docs/current/FOCUSA_TOOL_IMPLEMENTATION_SPEC_AUDIT.md 'Metacognition evaluation.*durable readback|Ontology memory pipeline.*prediction follow-up|Public licensing metadata' 'tool audit documents latest gap closures'
assert_has README.md 'focusa status --operator|focusa workpoint resume --copy-prompt|scripts/demo-workpoint-happy-path.sh' 'README reflects Operator Preview command surface'
assert_has docs/current/CLI_REFERENCE_CURRENT.md 'status --operator|workpoint resume --copy-prompt|focusa onboard --agent manual' 'CLI reference reflects Operator Preview commands'
assert_has docs/current/FOCUSA_OPERATOR_PREVIEW_PROOF.md 'focusa onboard --agent pi|focusa status --operator|focusa workpoint resume --copy-prompt|scripts/demo-workpoint-happy-path.sh' 'Operator Preview proof keeps golden path commands'
assert_has docs/current/NON_PI_AGENT_FOCUSA_USAGE.md 'focusa workpoint resume --copy-prompt|focusa status --operator' 'Non-Pi docs include manual continuation commands'
assert_has docs/current/VALIDATION_AND_RELEASE_PROOF.md 'focusa status --operator|focusa workpoint resume --copy-prompt|scripts/demo-workpoint-happy-path.sh' 'Release proof includes Operator Preview validation commands'

 echo "SPEC96 Public docs current features static test: PASS"
