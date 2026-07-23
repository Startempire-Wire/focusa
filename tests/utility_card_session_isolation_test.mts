import { buildFocusaUtilityCard } from "../apps/pi-extension/src/awareness.ts";
import {
  adoptPersistedContinuityForSession,
  getActiveWorkpointPacket,
  getAttachmentRuntime,
  makeAttachmentKey,
  resetPiSessionScopedState,
  runWithAttachmentRuntime,
  setActiveWorkpointPacket,
  setActiveWorkpointSummary,
} from "../apps/pi-extension/src/state.ts";

function assert(cond: unknown, msg: string): asserts cond {
  if (!cond) throw new Error(msg);
}

const attachmentKey = makeAttachmentKey({
  projectRoot: "/tmp/project-a",
  continuityId: "current-continuity",
  sessionId: "new-session",
});

await runWithAttachmentRuntime(attachmentKey, async () => {
  const runtime = getAttachmentRuntime();
  Object.assign(runtime, {
    focusaAvailable: true,
    sessionCwd: "/tmp/project-a",
    sessionFrameKey: "new-session",
    continuityId: "current-continuity",
    currentAsk: {
      text: "fresh unrelated ask",
      kind: "instruction",
      sourceTurnId: "t",
      updatedAt: Date.now(),
    },
    activeFrameGoal: "Fresh frame goal",
    activeFrameTitle: "Fresh frame title",
  });

  setActiveWorkpointPacket({
    mission: "STALE MISSION SHOULD NOT LEAK",
    next_slice: "DO_NOT_DRIFT: stale drift boundary",
    project_root: "/tmp/project-a",
    continuity_id: "old-continuity",
    canonical: true,
    status: "active",
  } as any);
  setActiveWorkpointSummary("STALE SUMMARY");
  const mismatched = buildFocusaUtilityCard("visible");
  assert(!mismatched.includes("STALE MISSION SHOULD NOT LEAK"), "mismatched Utility Card leaked stale mission");
  assert(!mismatched.includes("DO_NOT_DRIFT"), "mismatched Utility Card leaked stale drift boundary");
  assert(mismatched.includes("none verified"), "mismatched Utility Card did not fail closed");

  runtime.sessionCwd = "/root";
  runtime.sessionFrameKey = "other-session";
  runtime.continuityId = "spec96-lowmem-surgical";
  setActiveWorkpointPacket({
    mission: "SPEC96 MISSION MUST NOT LEAK TO UNSAFE CWD",
    project_root: "/home/wirebot/focusa",
    continuity_id: "spec96-lowmem-surgical",
    canonical: true,
    status: "active",
  } as any);
  const unsafe = buildFocusaUtilityCard("visible");
  assert(!unsafe.includes("SPEC96 MISSION MUST NOT LEAK"), "unsafe-cwd Utility Card adopted stale Workpoint");
  assert(!unsafe.includes("spec96-lowmem-surgical"), "unsafe-cwd Utility Card leaked stale continuity");
  assert(unsafe.includes("none verified"), "unsafe-cwd Utility Card did not fail closed");
  assert(unsafe.includes("broad/unsafe"), "unsafe-cwd Utility Card omitted recovery posture");

  resetPiSessionScopedState("test-session-switch");
  const reset = buildFocusaUtilityCard("visible");
  assert(!reset.includes("STALE"), "session reset leaked stale Workpoint content");

  adoptPersistedContinuityForSession(
    {
      continuityId: "old-continuity",
      activeWorkpointPacket: {
        mission: "PERSISTED OTHER SESSION",
        project_root: "/tmp/project-b",
        continuity_id: "old-continuity",
        canonical: true,
        status: "active",
      },
    },
    "new-session",
    "/tmp/project-a",
  );
  assert(getActiveWorkpointPacket() === null, "mismatched persisted session retained Workpoint packet");

  Object.assign(runtime, {
    focusaAvailable: true,
    sessionCwd: "/tmp/project-a",
    sessionFrameKey: "new-session",
    continuityId: "current-continuity",
  });
  setActiveWorkpointPacket({
    mission: "MATCHED MISSION",
    next_slice: "matched next",
    project_root: "/tmp/project-a",
    continuity_id: "current-continuity",
    canonical: true,
    status: "active",
  } as any);
  setActiveWorkpointSummary("MATCHED MISSION");
  const matched = buildFocusaUtilityCard("visible");
  assert(matched.includes("MATCHED MISSION"), "matched Utility Card did not show scoped mission");
  assert(matched.includes("verified project_root + continuity_id match"), "matched Utility Card omitted verified scope");

  console.log("utility card session isolation proof passed");
});
