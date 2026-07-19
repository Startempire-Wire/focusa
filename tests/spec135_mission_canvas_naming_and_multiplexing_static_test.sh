#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

MISSION_CANVAS_VIEW="$ROOT_DIR/apps/menubar/src/lib/components/MissionCanvasView.svelte"
RUNTIME_VIEW="$ROOT_DIR/apps/menubar/src/lib/components/RuntimeView.svelte"
PAGE="$ROOT_DIR/apps/menubar/src/routes/+page.svelte"
MASTER="$ROOT_DIR/docs/135-focusa-professional-workspaces-and-crist-project-genesis-master-spec.md"
SPEC_A="$ROOT_DIR/docs/135a-workspace-projection-pi-sidebar-work-rail-and-vertical-ux-spec.md"
SPEC_C="$ROOT_DIR/docs/135c-uiai-rich-artifact-live-refresh-and-research-bridge-spec.md"
SPEC_D="$ROOT_DIR/docs/135d-complete-implementation-order-framework-reuse-performance-and-no-deferral-spec.md"
SPEC_E="$ROOT_DIR/docs/135e-cross-spec-amendments-migration-and-closure-matrix.md"
SPEC_G="$ROOT_DIR/docs/135g-multiplexed-mission-canvas-work-surfaces-session-attachments-and-browser-context-isolation-spec.md"

[[ -f "$MISSION_CANVAS_VIEW" ]] || fail "MissionCanvasView.svelte is missing"
[[ ! -e "$ROOT_DIR/apps/menubar/src/lib/components/CockpitView.svelte" ]] || fail "obsolete Focusa CockpitView still exists"

if rg -ni 'cockpit' "$ROOT_DIR/apps/menubar/src" >/tmp/focusa-active-ui-cockpit.txt; then
  cat /tmp/focusa-active-ui-cockpit.txt >&2
  fail "active Focusa UI code still contains generic Cockpit terminology"
fi
pass "active Focusa UI uses Mission Canvas terminology"

rg -n "MissionCanvasView|activeTab === 'mission-canvas'|title=\"Mission Canvas\"" "$PAGE" >/dev/null || fail "menubar route does not expose Mission Canvas"
rg -n 'mission-canvas-grid|Focusa Mission Canvas runtime summary' "$RUNTIME_VIEW" >/dev/null || fail "runtime summary does not use Mission Canvas naming"
pass "menubar route and runtime view use Mission Canvas"

for file in "$MASTER" "$SPEC_A" "$SPEC_C" "$SPEC_D" "$SPEC_E" "$SPEC_G"; do
  rg -n 'UIAI Engine Cockpit' "$file" >/dev/null || fail "$(basename "$file") does not preserve the UIAI Engine Cockpit boundary"
done
pass "Spec 135 integration documents preserve UIAI Engine Cockpit as the rich shell"

for needle in \
  'Focusa Mission Canvas' \
  'Work Surface' \
  'ProjectRootKey' \
  'WorkstreamKey' \
  'AttachmentKey' \
  'browser_context_id' \
  'browser_target_id' \
  'shared_authenticated' \
  'isolated_authenticated' \
  'steering_queue_ref' \
  'writer_lease_ref' \
  'Close view' \
  'Terminate session'; do
  rg -n -F "$needle" "$SPEC_G" >/dev/null || fail "Spec 135G missing multiplexing contract marker: $needle"
done
pass "Spec 135G contains Work Surface, scope, browser isolation, routing, and lifecycle contracts"

rg -n 'Specs 38–41, 43, 98, 104, and 133|Specs 38–41, 43, 98, 104, 133' "$SPEC_G" "$SPEC_D" >/dev/null || fail "multiplexing foundation dependencies are not explicit"
rg -n '135G' "$MASTER" "$SPEC_D" "$SPEC_E" "$ROOT_DIR/docs/INDEX.md" >/dev/null || fail "Spec 135G is not integrated across the series/index"
pass "Spec 135G is mandatory in the master, build order, migration matrix, and index"

echo "Spec 135 Mission Canvas naming and multiplexing static test: PASS"
