import {
  S,
  adoptWorkpointScopeForFrameRecovery,
  getActiveWorkpointPacket,
  resolveFocusWriteProjectRoot,
  setActiveWorkpointPacket,
} from "../apps/pi-extension/src/state.ts";
import { mkdirSync, mkdtempSync, rmSync } from "fs";
import { tmpdir } from "os";
import { join } from "path";

function assert(condition: any, message: string): asserts condition {
  if (!condition) throw new Error(message);
}

const root = mkdtempSync(join(tmpdir(), "focusa-decide-scope-"));
const projectA = join(root, "project-a");
const projectB = join(root, "project-b");
mkdirSync(join(projectA, ".git"), { recursive: true });
mkdirSync(join(projectA, ".beads"), { recursive: true });
mkdirSync(join(projectB, ".git"), { recursive: true });
mkdirSync(join(projectB, ".beads"), { recursive: true });

try {
  const recovered = resolveFocusWriteProjectRoot("/root", projectA);
  assert(recovered === projectA, `safe cached scope not preferred over /root: ${recovered}`);

  const liveWins = resolveFocusWriteProjectRoot(projectB, projectA);
  assert(liveWins === projectB, `verified live project did not win: ${liveWins}`);

  const stillUnsafe = resolveFocusWriteProjectRoot("/root", "/tmp");
  assert(stillUnsafe === "/root", `unsafe fallback was unexpectedly promoted: ${stillUnsafe}`);

  S.sessionFrameKey = "pi-session-scope-test";
  S.continuityId = "continuity-scope-test";
  S.sessionCwd = "/root";
  setActiveWorkpointPacket(null);
  const packet = {
    status: "active",
    canonical: true,
    project_root: projectA,
    continuity_id: "continuity-scope-test",
    session_id: "pi-session-scope-test",
    mission: "Prove project-bound Focus State writes",
  };
  const adopted = adoptWorkpointScopeForFrameRecovery(packet, "runtime_test");
  assert(adopted === projectA, "canonical same-session Workpoint scope was not adopted");
  assert(
    getActiveWorkpointPacket()?.project_root === projectA,
    "adopted Workpoint was not cached for frame recovery"
  );
  assert(
    getActiveWorkpointPacket()?.pi_session_frame_key === "pi-session-scope-test",
    "adopted Workpoint was not stamped to the current Pi session"
  );

  const mismatch = adoptWorkpointScopeForFrameRecovery(
    { ...packet, continuity_id: "different-continuity" },
    "runtime_test_mismatch"
  );
  assert(mismatch === null, "cross-continuity Workpoint was adopted");

  console.log("PASS: focusa_decide project-bound scope recovery runtime contract");
} finally {
  setActiveWorkpointPacket(null);
  rmSync(root, { recursive: true, force: true });
}
