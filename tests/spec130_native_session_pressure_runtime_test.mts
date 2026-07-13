import {
  measureNativeSessionPressure,
  nativeSessionBudgets,
} from "../apps/pi-extension/src/session-pressure.ts";
import { mkdtempSync, rmSync, truncateSync, writeFileSync } from "fs";
import { tmpdir } from "os";
import { join } from "path";

function assert(condition: any, message: string): asserts condition {
  if (!condition) throw new Error(message);
}

const MIB = 1024 * 1024;
const heapLimit = 4 * 1024 * MIB;
const root = mkdtempSync(join(tmpdir(), "focusa-spec130-pressure-"));

try {
  const fixture = (name: string, bytes: number) => {
    const path = join(root, name);
    writeFileSync(path, "");
    truncateSync(path, bytes);
    return path;
  };

  const budgets = nativeSessionBudgets(heapLimit);
  assert(budgets.native_segment_soft_bytes === 64 * MIB, "wrong soft segment budget");
  assert(budgets.native_segment_hard_bytes === 128 * MIB, "wrong hard segment budget");
  assert(budgets.native_startup_migration_bytes === 256 * MIB, "wrong startup migration budget");
  assert(budgets.focusa_custom_soft_bytes === 8 * MIB, "wrong custom soft budget");
  assert(budgets.focusa_custom_hard_bytes === 16 * MIB, "wrong custom hard budget");

  const entries = [
    {
      type: "custom",
      customType: "focusa-state",
      data: { semanticDigest: "sha256:same", payload: "x".repeat(200) },
    },
    {
      type: "custom",
      customType: "focusa-wbm-state",
      data: { semanticDigest: "sha256:same", payload: "x".repeat(200) },
    },
    { type: "message", message: { role: "user", content: "ordinary" } },
  ];

  const normal = measureNativeSessionPressure({
    adapter: "pi",
    sessionFile: fixture("normal.jsonl", 1 * MIB),
    entries,
    heapUsedBytes: 512 * MIB,
    heapLimitBytes: heapLimit,
    measuredAt: new Date("2026-07-13T00:00:00Z"),
  });
  assert(normal.posture === "normal", `expected normal, got ${normal.posture}`);
  assert(normal.recommended_action === "continue", "normal action mismatch");
  assert(normal.focusa_custom_entries === 2, "Focusa custom entries not counted");
  assert(normal.duplicate_anchor_count === 1, "duplicate anchor not detected");
  assert(normal.sample_complete === true, "small entry sample should be complete");

  const soft = measureNativeSessionPressure({
    adapter: "pi",
    sessionFile: fixture("soft.jsonl", 70 * MIB),
    heapUsedBytes: 512 * MIB,
    heapLimitBytes: heapLimit,
  });
  assert(soft.posture === "soft_pressure", `expected soft, got ${soft.posture}`);
  assert(soft.recommended_action === "checkpoint", "soft action mismatch");

  const hard = measureNativeSessionPressure({
    adapter: "pi",
    sessionFile: fixture("hard.jsonl", 140 * MIB),
    heapUsedBytes: 512 * MIB,
    heapLimitBytes: heapLimit,
  });
  assert(hard.posture === "hard_pressure", `expected hard, got ${hard.posture}`);
  assert(hard.recommended_action === "rollover", "hard action mismatch");

  const oversized = measureNativeSessionPressure({
    adapter: "pi",
    sessionFile: fixture("oversized.jsonl", 300 * MIB),
    heapUsedBytes: 512 * MIB,
    heapLimitBytes: heapLimit,
  });
  assert(
    oversized.posture === "oversized_at_start",
    `expected oversized, got ${oversized.posture}`
  );
  assert(oversized.recommended_action === "refuse_full_load", "oversized action mismatch");

  const emergency = measureNativeSessionPressure({
    adapter: "pi",
    sessionFile: fixture("emergency.jsonl", 1 * MIB),
    heapUsedBytes: 950 * MIB,
    heapLimitBytes: 1024 * MIB,
  });
  assert(emergency.posture === "emergency", `expected emergency, got ${emergency.posture}`);
  assert(emergency.recommended_action === "stream_migrate", "emergency action mismatch");

  const manyEntries = Array.from({ length: 3_000 }, (_, i) => ({
    type: "custom",
    customType: i % 2 ? "focusa-state" : "other-state",
    data: { semanticDigest: `sha256:${i}` },
  }));
  const sampled = measureNativeSessionPressure({
    entries: manyEntries,
    heapUsedBytes: 1,
    heapLimitBytes: heapLimit,
  });
  assert(sampled.sampled_entry_count === 2_048, "entry sample was not bounded");
  assert(sampled.sample_complete === false, "partial sample not labeled");

  console.log("PASS: Spec 130 native session pressure policy");
} finally {
  rmSync(root, { recursive: true, force: true });
}
