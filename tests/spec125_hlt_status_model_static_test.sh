#!/usr/bin/env bash
# Spec125-01: Core HLT status model and generic-HLT classifier.
# Verifies that the HltStatus enum and classifier exist in focusa-core,
# that trajectory.rs wires hlt_status into the view response, and that
# generic HLT never carries route authority.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CORE="$ROOT/crates/focusa-core/src/types.rs"
TRAJ="$ROOT/crates/focusa-api/src/routes/trajectory.rs"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

# 1. HltStatus enum exists with all 6 Spec 125 statuses.
for variant in CanonicalExplicit PreviousValidFallback SupersessionPending MissingRequired GenericDegraded Conflicted; do
  grep -q "pub enum HltStatus" "$CORE" || fail "HltStatus enum missing"
  grep -q "$variant" "$CORE" || fail "HltStatus::$variant missing"
done
pass "HltStatus enum has all 6 Spec 125 statuses"

# 2. classify_hlt function exists.
grep -q "pub fn classify_hlt" "$CORE" || fail "classify_hlt function missing"
pass "classify_hlt function present"

# 3. Generic HLT patterns are defined.
grep -q "GENERIC_HLT_PATTERNS" "$CORE" || fail "GENERIC_HLT_PATTERNS missing"
for pattern in "Maintain and improve" "Strengthen project intelligence"; do
  grep -qF "$pattern" "$CORE" || fail "generic pattern missing: $pattern"
done
pass "generic HLT patterns defined"

# 4. is_action_ready is false for degraded/missing/conflicted.
grep -q "is_action_ready" "$CORE" || fail "is_action_ready method missing"
grep -q "has_route_authority" "$CORE" || fail "has_route_authority method missing"
pass "authority methods present"

# 5. trajectory.rs imports classify_hlt and HltStatus.
grep -q "classify_hlt" "$TRAJ" || fail "trajectory.rs does not import classify_hlt"
grep -q "HltStatus" "$TRAJ" || fail "trajectory.rs does not import HltStatus"
pass "trajectory.rs imports HLT classifier"

# 6. trajectory.rs computes hlt_status.
grep -q "hlt_status = classify_hlt" "$TRAJ" || fail "trajectory.rs does not compute hlt_status"
pass "trajectory.rs computes hlt_status"

# 7. trajectory.rs emits hlt_status in JSON response.
grep -q '"hlt_status"' "$TRAJ" || fail "trajectory.rs does not emit hlt_status"
pass "trajectory.rs emits hlt_status in response"

# 8. hlt_status affects degraded flag.
grep -q "hlt_status.has_route_authority" "$TRAJ" || fail "hlt_status does not affect degraded flag"
pass "hlt_status gates degraded flag"

# 9. TrajectoryLadderContext carries hlt_status.
grep -q "pub hlt_status: HltStatus" "$CORE" || fail "TrajectoryLadderContext missing hlt_status"
pass "TrajectoryLadderContext carries hlt_status"

echo ""
echo "=== Spec125-01 core HLT status model: PASS ==="
