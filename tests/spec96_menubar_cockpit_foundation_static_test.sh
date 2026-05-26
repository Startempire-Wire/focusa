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
assert_has apps/menubar/src/lib/components/CockpitView.svelte 'PROJECT|TRAJECTORY|POST /v1/workpoint/resume|GET /v1/work-loop/health|GET /v1/telemetry/memory|GET /v1/doctor' 'cockpit panel includes current Focusa surfaces'
assert_has apps/menubar/src/lib/components/CockpitView.svelte 'envelopeLabel|envelopeTone|evidenceCount|class:watch|class:bad|class="chip"' 'cockpit panel renders calm result-envelope/status chips'
assert_has apps/menubar/src/lib/components/TrajectoryPeek.svelte 'Active gap|Long-term goal|Desired end state|Next Workpoint|Evidence refs|Checks / risks' 'trajectory peek renders current trajectory surfaces calmly'
assert_has apps/menubar/src/routes/+page.svelte "TrajectoryPeek|activeTab === 'trajectory'" 'trajectory peek is reachable from menubar shell'
assert_has apps/menubar/src/lib/components/WorkpointPeek.svelte 'Continuation contract|Current action|Next action|Target objects|Evidence refs|Blockers|Do not drift' 'workpoint peek renders canonical continuation surfaces calmly'
assert_has apps/menubar/src/routes/+page.svelte "WorkpointPeek|activeTab === 'workpoint'" 'workpoint peek is reachable from menubar shell'
assert_has apps/menubar/src/routes/+page.svelte '/v1/predictions/recent|/v1/predictions/stats|/v1/metacognition/evaluations/recent|/v1/focus/snapshots/recent|/v1/lineage/head' 'menubar poll includes proof hot surfaces'
assert_has apps/menubar/src/lib/components/ProofPeek.svelte 'Workpoint evidence|Predictions|Metacognition|Snapshots|Lineage head' 'proof peek renders evidence/prediction/metacog/snapshot surfaces calmly'
assert_has apps/menubar/src/routes/+page.svelte "ProofPeek|activeTab === 'proof'" 'proof peek is reachable from menubar shell'
assert_has apps/menubar/src/routes/+page.svelte '/v1/work-loop/checkpoints' 'menubar poll includes work-loop checkpoints surface'
assert_has apps/menubar/src/lib/components/WorkLoopPeek.svelte 'Dispatch posture|Active task|Pause flags|Recent checkpoints|writer' 'work-loop peek renders readiness and checkpoint surfaces calmly'
assert_has apps/menubar/src/routes/+page.svelte "WorkLoopPeek|activeTab === 'workloop'" 'work-loop peek is reachable from menubar shell'
assert_has apps/menubar/src/routes/+page.svelte 'aria-label="Focusa peeks"|tab-mark|icon-only|quiet|scrollbar-width: none' 'menubar navigation uses calm aesthetic peek tabs only where needed'
assert_has apps/menubar/src/routes/+page.svelte "activeTab === 'cockpit'|CockpitView" 'menubar uses cockpit naming instead of mission naming'
assert_has apps/menubar/src/lib/components/CockpitView.svelte 'cockpit-grid|Focusa cockpit' 'cockpit component naming is polished'
assert_has apps/menubar/src/lib/components/Settings.svelte 'v0\.9\.13-dev' 'settings polish shows current menubar version'
assert_has apps/menubar/src/lib/components/Settings.svelte 'Direct network binding exposes Focusa' 'settings polish includes remote security copy'
assert_has apps/menubar/src/routes/+page.svelte 'manual_proof_required' 'release proof card avoids hardcoded ready state'
assert_has apps/menubar/src/lib/components/CockpitView.svelte 'manual gate|manual_proof_required' 'release proof UI renders manual proof gate'
assert_has apps/menubar/src/lib/components/FocusView.svelte 'CURRENT BUBBLE|BACKGROUND CLOUDS|ambient-orbit|thought-cloud|focus-bubble|Quiet surface|Focusa is out of view' 'default focus view preserves original bubble/cloud hierarchy and calm empty states'
assert_has apps/menubar/src/lib/components/GatePanel.svelte 'Gate is quiet|SOFT CANDIDATES|AMBIENT SIGNALS|do not switch focus for you' 'gate panel copy remains ambient awareness, not control UI'
assert_has apps/menubar/src/lib/components/SyncPanel.svelte 'Peer awareness; no automatic ownership changes|Local-first mode is fine|Pull gently|Listening for peers' 'sync panel copy remains calm local-first awareness'
assert_has docs/current/TAURI_MENUBAR_IMPLEMENTATION_GAPS.md 'Implemented in the current menubar slice|Remaining Phase 0 gaps|Next recommended implementation slices|git:9de260c' 'menubar implementation gaps doc reflects current post-audit status'
assert_has docs/current/TAURI_MENUBAR_UP_TO_SPEED_SPEC.md '✅ Implemented|Partial: read-only Trajectory peek implemented|Read-only Work Loop peek implemented' 'up-to-speed spec tracks implemented vs remaining slices'
assert_has apps/menubar/package.json 'svelte-kit sync && svelte-check' 'menubar check script generates SvelteKit tsconfig first'
assert_has apps/menubar/package.json '"@sveltejs/vite-plugin-svelte": "\^5\.0\.0"' 'menubar npm parity uses Vite-6-compatible Svelte plugin'
assert_has apps/menubar/src-tauri/tauri.conf.json '"beforeBuildCommand": "npm run build"' 'Tauri package proof uses npm build command available in CI'
assert_has .github/workflows/ci.yml 'name: Menubar|npm run tauri build -- --bundles app|Validate macOS bundle|apps/menubar/package-lock.json' 'root CI owns menubar npm and Tauri package proof'
assert_has .github/workflows/ci.yml 'Info\.plist|plutil -lint' 'root CI validates macOS bundle metadata that Tauri emits'
assert_has docs/current/TAURI_MENUBAR_IMPLEMENTATION_GAPS.md 'npm ci.*resolved|wirebot.*no accessible.*cargo|root-only.*/root/.cargo/bin/cargo' 'menubar packaging gaps distinguish resolved npm parity from Cargo blocker'

if rg -n 'http://127\.0\.0\.1:8787' "${MENUBAR}/src" | rg -v 'src/lib/api.ts|placeholder=' >/tmp/menubar-hardcoded-api.txt; then
  echo "✗ FAIL: hardcoded API base outside shared default/placeholder" >&2
  cat /tmp/menubar-hardcoded-api.txt >&2
  exit 1
else
  echo "✓ PASS: no hardcoded API base outside shared default/placeholder"
fi

echo "SPEC96 Menubar cockpit foundation static test: PASS"
