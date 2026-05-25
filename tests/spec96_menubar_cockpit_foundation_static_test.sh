#!/bin/bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
MENUBAR="${ROOT_DIR}/apps/menubar"

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

assert_has apps/menubar/src/lib/api.ts 'postJson|normalizeToolResult|isDegraded|summarizeError|DEFAULT_API_URL' 'shared menubar API client exposes runtime-cockpit helpers'
assert_has apps/menubar/src/routes/+page.svelte '/v1/project/identity|/v1/trajectory/view\?mode=summary|/v1/workpoint/resume|/v1/work-loop/health|/v1/telemetry/memory|/v1/doctor' 'menubar poll includes current Focusa cockpit hot surfaces'
assert_has apps/menubar/src/lib/components/MissionControl.svelte 'PROJECT|TRAJECTORY|POST /v1/workpoint/resume|GET /v1/work-loop/health|GET /v1/telemetry/memory|GET /v1/doctor' 'mission panel started cockpit surface expansion'
assert_has apps/menubar/src/lib/components/MissionControl.svelte 'envelopeLabel|envelopeTone|evidenceCount|class:watch|class:bad|class="chip"' 'mission panel renders calm result-envelope/status chips'
assert_has apps/menubar/src/lib/components/TrajectoryPeek.svelte 'Active gap|Long-term goal|Desired end state|Next Workpoint|Evidence refs|Checks / risks' 'trajectory peek renders current trajectory surfaces calmly'
assert_has apps/menubar/src/routes/+page.svelte "TrajectoryPeek|activeTab === 'trajectory'" 'trajectory peek is reachable from menubar shell'
assert_has apps/menubar/src/lib/components/WorkpointPeek.svelte 'Continuation contract|Current action|Next action|Target objects|Evidence refs|Blockers|Do not drift' 'workpoint peek renders canonical continuation surfaces calmly'
assert_has apps/menubar/src/routes/+page.svelte "WorkpointPeek|activeTab === 'workpoint'" 'workpoint peek is reachable from menubar shell'
assert_has apps/menubar/src/routes/+page.svelte '/v1/predictions/recent|/v1/predictions/stats|/v1/metacognition/evaluations/recent|/v1/focus/snapshots/recent|/v1/lineage/head' 'menubar poll includes proof hot surfaces'
assert_has apps/menubar/src/lib/components/ProofPeek.svelte 'Workpoint evidence|Predictions|Metacognition|Snapshots|Lineage head' 'proof peek renders evidence/prediction/metacog/snapshot surfaces calmly'
assert_has apps/menubar/src/routes/+page.svelte "ProofPeek|activeTab === 'proof'" 'proof peek is reachable from menubar shell'
assert_has apps/menubar/package.json 'svelte-kit sync && svelte-check' 'menubar check script generates SvelteKit tsconfig first'

if rg -n 'http://127\.0\.0\.1:8787' "${MENUBAR}/src" | rg -v 'src/lib/api.ts|placeholder=' >/tmp/menubar-hardcoded-api.txt; then
  echo "✗ FAIL: hardcoded API base outside shared default/placeholder" >&2
  cat /tmp/menubar-hardcoded-api.txt >&2
  exit 1
else
  echo "✓ PASS: no hardcoded API base outside shared default/placeholder"
fi

echo "SPEC96 Menubar cockpit foundation static test: PASS"
