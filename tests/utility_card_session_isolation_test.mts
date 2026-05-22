import { buildFocusaUtilityCard } from "../apps/pi-extension/src/awareness.ts";
import { S, adoptPersistedContinuityForSession, ensurePiFrame } from "../apps/pi-extension/src/state.ts";

function assert(cond: any, msg: string) {
  if (!cond) throw new Error(msg);
}

Object.assign(S, {
  focusaAvailable: true,
  sessionCwd: "/tmp/project-a",
  sessionFrameKey: "new-session",
  continuityId: "current-continuity",
  activeWorkpointPacket: {
    mission: "STALE MISSION SHOULD NOT LEAK",
    next_slice: "DO_NOT_DRIFT: stale drift boundary",
    project_root: "/tmp/project-a",
    continuity_id: "old-continuity",
    canonical: true,
    status: "active",
  },
  activeWorkpointSummary: "STALE SUMMARY",
  currentAsk: { text: "fresh unrelated ask", kind: "instruction", sourceTurnId: "t", updatedAt: Date.now() },
  activeFrameGoal: "Fresh frame goal",
  activeFrameTitle: "Fresh frame title",
  lastCompactDecision: "stale compact decision",
});

const mismatched = buildFocusaUtilityCard("visible");
assert(!mismatched.includes("STALE MISSION SHOULD NOT LEAK"), "mismatched Utility Card leaked stale mission");
assert(!mismatched.includes("DO_NOT_DRIFT"), "mismatched Utility Card leaked stale drift boundary");
assert(mismatched.includes("none verified"), "mismatched Utility Card should declare no scoped Workpoint");

Object.assign(S, {
  focusaAvailable: true,
  sessionCwd: "/root",
  sessionFrameKey: "other-session",
  continuityId: "spec96-lowmem-surgical",
  activeWorkpointPacket: {
    mission: "SPEC96 MISSION MUST NOT LEAK TO UNSAFE CWD",
    next_slice: "prove evidence link remains bounded under LowMem",
    project_root: "/home/wirebot/focusa",
    continuity_id: "spec96-lowmem-surgical",
    session_id: "spec96-lowmem-surgical",
    canonical: true,
    status: "active",
  },
});
await ensurePiFrame("/root", "other-session", "test-unsafe-cwd");
const unsafeCwdCard = buildFocusaUtilityCard("visible");
assert(!unsafeCwdCard.includes("SPEC96 MISSION MUST NOT LEAK"), "unsafe-cwd Utility Card adopted global active Workpoint");
assert(!unsafeCwdCard.includes("spec96-lowmem-surgical"), "unsafe-cwd Utility Card leaked stale continuity id");
assert(unsafeCwdCard.includes("none verified"), "unsafe-cwd Utility Card should declare no scoped Workpoint");

Object.assign(S, {
  focusaAvailable: true,
  sessionCwd: "/tmp/project-a",
  sessionFrameKey: "new-session",
  continuityId: "current-continuity",
  activeWorkpointPacket: null,
  activeWorkpointSummary: "",
});

adoptPersistedContinuityForSession({
  sessionId: "old-session",
  continuityId: "old-continuity",
  activeWorkpointPacket: {
    mission: "OLD SESSION PACKET",
    project_root: "/tmp/project-a",
    continuity_id: "old-continuity",
    canonical: true,
    status: "active",
  },
}, "new-session", "/tmp/project-a");
assert(S.continuityId === "current-continuity", "mismatched persisted session adopted continuity");
assert(S.activeWorkpointPacket === null, "mismatched persisted session retained active Workpoint packet");

S.activeWorkpointPacket = {
  mission: "MATCHED MISSION",
  next_slice: "matched next",
  project_root: "/tmp/project-a",
  continuity_id: "current-continuity",
  canonical: true,
  status: "active",
};
const matched = buildFocusaUtilityCard("visible");
assert(matched.includes("MATCHED MISSION"), "matched Utility Card did not show scoped mission");
assert(matched.includes("verified project_root + continuity_id match"), "matched Utility Card did not declare verified scope");
console.log("utility card session isolation proof passed");
