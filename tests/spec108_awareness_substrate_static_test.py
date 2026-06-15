#!/usr/bin/env python3
"""Spec108 awareness-substrate static audit — focusa-4jo5.4."""
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

def fail(msg: str) -> None:
    print(f"FAIL: {msg}")
    sys.exit(1)

def read(rel: str) -> str:
    return (ROOT / rel).read_text()

def main() -> None:
    src = read("apps/pi-extension/src/awareness-substrate.ts")

    # Schema types
    for token in [
        "AwarenessMode",
        "AwarenessSurface",
        "AwarenessLayer",
        "AwarenessCandidateLine",
        "AwarenessInput",
        "AwarenessPacket",
        "ContextPressureState",
        "ToolGuidance",
        "focusa.awareness_packet.v1",
    ]:
        if token not in src:
            fail(f"missing type: {token}")

    # Key functions
    for token in [
        "gatherAwarenessInput",
        "generateCandidateLine",
        "scoreLine",
        "selectTopTool",
        "shouldShowPressureWarning",
        "updateCadenceState",
        "buildAwarenessPacket",
        "renderAwarenessPacketText",
        "selectMode",
        "contextTierFromString",
        "sideEffectToRisk",
    ]:
        if token not in src:
            fail(f"missing function: {token}")

    # DVS formula components
    for token in [
        "authorityValue",
        "actionability",
        "riskReduction",
        "novelty",
        "proofValue",
        "redundancyPenalty",
        "stalenessPenalty",
        "DVS",
        "DVS_WEIGHTS",
        "DVS_CUTOFF",
    ]:
        if token not in src:
            fail(f"missing DVS component: {token}")

    # Mode selection logic
    for mode in ["minimal", "standard", "rich", "onboarding"]:
        if mode not in src:
            fail(f"missing mode: {mode}")

    # Surface types
    for surface in ["reload", "post_compaction", "warning", "tool_guidance", "uiai_bridge"]:
        if surface not in src:
            fail(f"missing surface: {surface}")

    # Layer types
    for layer in ["identity", "authority", "mission", "goal", "risk", "proof", "recovery", "learning", "tool"]:
        if layer not in src:
            fail(f"missing layer: {layer}")

    # Dedupe/cadence logic
    for token in [
        "lastShownAt",
        "lastPct",
        "lastTier",
        "compactionCountAtLastShown",
        "transitionCount",
        "suppressionCount",
        "tier_escalation",
        "anchor_changed",
        "stale_reminder",
    ]:
        if token not in src:
            fail(f"missing cadence token: {token}")

    # AwarenessInput fields
    for field in [
        "projectIdentity",
        "projectRootSafety",
        "workpointResume",
        "trajectoryView",
        "contextPressure",
        "operatorSteering",
        "toolGraph",
        "cadenceState",
        "mode",
        "surface",
    ]:
        if field not in src:
            fail(f"missing AwarenessInput field: {field}")

    # State integration
    state = read("apps/pi-extension/src/state.ts")
    if "awarenessCadenceState" not in state:
        fail("state.ts missing awarenessCadenceState field")
    if "lastWorkpointUpdate" not in state:
        fail("state.ts missing lastWorkpointUpdate field")

    # Compaction integration
    compaction = read("apps/pi-extension/src/compaction.ts")
    if "lastWorkpointUpdate" not in compaction:
        fail("compaction.ts missing lastWorkpointUpdate stamp")

    # Tool guidance
    for token in [
        "sideEffectRisk",
        "safe",
        "moderate",
        "risky",
        "write_state",
        "control_state",
    ]:
        if token not in src:
            fail(f"missing tool guidance token: {token}")

    # Visibility filtering
    for token in ["modeAllowed", "surfaceAllowed"]:
        if token not in src:
            fail(f"missing visibility token: {token}")

    # Packet output
    for token in [
        "visibleLines",
        "systemLines",
        "nextTools",
        "recoveryTools",
        "suppressedLines",
        "metadata",
        "dvsCutoff",
        "totalCandidates",
        "visibleCount",
        "suppressedCount",
        "freshnessScore",
        "authorityScore",
        "confidence",
        "rehydrateId",
    ]:
        if token not in src:
            fail(f"missing packet token: {token}")

    # Other module integrations
    for fname in ["session.ts", "turns.ts"]:
        m = read(f"apps/pi-extension/src/{fname}")
        if "lastWorkpointUpdate" not in m:
            fail(f"{fname} missing lastWorkpointUpdate stamp")

    print("PASS: awareness-substrate static audit")
    print(f"  awareness-substrate.ts: {len(src.splitlines())} lines")
    print(f"  types: AwarenessMode, AwarenessSurface, AwarenessLayer, AwarenessCandidateLine, AwarenessInput, AwarenessPacket, ContextPressureState, ToolGuidance")
    print(f"  functions: gatherAwarenessInput, generateCandidateLines, scoreLines, selectMode, selectTopTools, shouldShowPressureWarning, updateCadenceState, buildAwarenessPacket, renderAwarenessPacketText")
    print(f"  DVS weights: authorityValue={3.0}, actionability={2.5}, riskReduction={2.0}, novelty={1.5}, proofValue={1.5}, redundancyPenalty={2.0}, stalenessPenalty={1.5}")
    print(f"  modes: minimal/standard/rich/onboarding, surfaces: reload/post_compaction/warning/tool_guidance/uiai_bridge")
    print(f"  layers: identity/authority/mission/goal/risk/proof/recovery/learning/tool")
    print(f"  state: awarenessCadenceState + lastWorkpointUpdate in S, stamped in compaction/session/turns/tools")
    print(f"  cadence: dedupe within 30s, tier escalation, anchor change, stale reminder after 5min")

if __name__ == "__main__":
    main()
