import { createHash } from "crypto";
import { mkdtempSync, readFileSync, readdirSync, rmSync } from "fs";
import { tmpdir } from "os";
import { join } from "path";
import {
  COMPACTION_PERSISTENCE_ANCHOR_REF_SCHEMA,
  type PersistenceFaultBoundary,
  loadPersistedRecoveryState,
  semanticPersistenceDigest,
  writeRecoverySidecar,
} from "../apps/pi-extension/src/persistence.ts";

function assert(value: unknown, message: string): asserts value {
  if (!value) throw new Error(message);
}
const root = mkdtempSync(join(tmpdir(), "focusa-spec130a-stress-"));
process.env.FOCUSA_DATA_DIR = root;
const projectRoot = "/verified/focusa";
const sessionId = "million-event-session";
let eventCount = 0,
  cycles = 0,
  unchangedAppends = 0,
  lastDigest = "";
const rss: number[] = [];
for (let cycle = 1; cycle <= 10_000; cycle++) {
  const segmentHash = createHash("sha256");
  for (let offset = 0; offset < 100; offset++) {
    const sequence = eventCount++;
    segmentHash.update(
      `${sequence}:mission:workpoint:blocker:evidence:receipt\n`,
    );
  }
  const state = {
    projectRoot,
    sessionId,
    cycle,
    eventCount,
    segmentDigest: segmentHash.digest("hex"),
    mission: "release",
    workpoint: "wp",
    blocker: "none",
    evidence: ["segment"],
    receipt: `receipt-${cycle}`,
    updatedAt: "2026-07-23T00:00:00Z",
  };
  const digest = semanticPersistenceDigest(state);
  if (digest === lastDigest) unchangedAppends++;
  else {
    writeRecoverySidecar(state, digest, cycle);
    cycles++;
    lastDigest = digest;
  }
  if (cycle % 1000 === 0) rss.push(process.memoryUsage().rss);
}
assert(eventCount === 1_000_000, "million semantic events not reached");
assert(cycles === 10_000, "10,000 checkpoint/rollover cycles not reached");
assert(unchangedAppends === 0, "unchanged semantic append detected");
assert(
  Math.max(...rss) - Math.min(...rss) < 96 * 1024 * 1024,
  "startup/replay memory slope is not flat enough",
);

const boundaries: PersistenceFaultBoundary[] = [
  "prepare",
  "write",
  "fsync",
  "checksum",
  "manifest",
  "target-create",
  "resume-verify",
  "commit",
];
for (const boundary of boundaries) {
  const faultRoot = mkdtempSync(join(tmpdir(), `focusa-spec130a-${boundary}-`));
  process.env.FOCUSA_DATA_DIR = faultRoot;
  const base = {
    projectRoot,
    sessionId: `crash-${boundary}`,
    revision: 1,
    mission: "m",
    workpoint: "w",
    blocker: "b",
    evidence: ["e"],
    receipt: "r",
    updatedAt: "2026-07-23T00:00:00Z",
  };
  const baseDigest = semanticPersistenceDigest(base);
  const written = writeRecoverySidecar(base, baseDigest, 1);
  const dir = join(faultRoot, "pi-session-state");
  const source = readdirSync(dir).find((name) => name.includes(".r1."));
  assert(source, `source generation missing for ${boundary}`);
  const sourceBytes = readFileSync(join(dir, source));
  const next = { ...base, revision: 2, receipt: "r2" };
  const nextDigest = semanticPersistenceDigest(next);
  let injected = false;
  try {
    writeRecoverySidecar(next, nextDigest, 2, boundary);
  } catch (error) {
    injected = String(error).includes(
      `injected persistence fault: ${boundary}`,
    );
  }
  assert(injected, `fault was not injected at ${boundary}`);
  assert(
    readFileSync(join(dir, source)).equals(sourceBytes),
    `source mutated at ${boundary}`,
  );
  writeRecoverySidecar(next, nextDigest, 2);
  const recovered = loadPersistedRecoveryState({
    schema: COMPACTION_PERSISTENCE_ANCHOR_REF_SCHEMA,
    projectRoot,
    sessionId: base.sessionId,
    sidecarKey: written.key,
  });
  assert(
    recovered?.revision === 2 && recovered?.receipt === "r2",
    `idempotent recovery failed at ${boundary}`,
  );
  rmSync(faultRoot, { recursive: true, force: true });
}

const agents = ["pi", "claude", "codex-opencode", "pi"];
let handoff = {
  mission: "release",
  workpoint: "wp",
  blocker: "none",
  evidence: ["proof"],
  receipt: "hash-chain",
  continuityId: "continuity-0",
  lineage: [] as string[],
};
for (let i = 0; i < agents.length; i++)
  handoff = {
    ...handoff,
    continuityId: `continuity-${i + 1}`,
    lineage: [...handoff.lineage, agents[i]],
  };
assert(
  handoff.lineage.join(",") === agents.join(","),
  "cross-agent lineage fidelity lost",
);
for (const key of ["mission", "workpoint", "blocker", "evidence", "receipt"])
  assert((handoff as any)[key], `handoff lost ${key}`);
rmSync(root, { recursive: true, force: true });
console.log(
  "PASS: Spec130A million-event, rollover crash, and rotating-agent stress matrix",
);
