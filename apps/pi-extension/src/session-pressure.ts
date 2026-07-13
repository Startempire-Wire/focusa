// Native coding-agent session pressure policy for Spec 130 §§44–48.

import { statSync } from "fs";
import { getHeapStatistics } from "v8";

export type NativeSessionPressurePosture =
  | "normal"
  | "soft_pressure"
  | "hard_pressure"
  | "emergency"
  | "oversized_at_start";

export interface NativeSessionBudgetsV1 {
  focusa_custom_soft_bytes: number;
  focusa_custom_hard_bytes: number;
  native_segment_soft_bytes: number;
  native_segment_hard_bytes: number;
  native_startup_migration_bytes: number;
  soft_headroom_floor: number;
  hard_headroom_floor: number;
  emergency_headroom_floor: number;
}

export interface NativeSessionPressureV1 {
  schema: "focusa.native_session_pressure.v1";
  adapter: string;
  native_session_ref: string;
  session_bytes: number;
  entry_count: number;
  sampled_entry_count: number;
  sample_complete: boolean;
  focusa_custom_bytes: number;
  focusa_custom_entries: number;
  duplicate_anchor_count: number;
  heap_used_bytes: number;
  heap_limit_bytes: number;
  headroom_ratio: number;
  budgets: NativeSessionBudgetsV1;
  posture: NativeSessionPressurePosture;
  recommended_action:
    | "continue"
    | "checkpoint"
    | "compact"
    | "rollover"
    | "stream_migrate"
    | "refuse_full_load";
  measured_at: string;
}

const MIB = 1024 * 1024;
const SAMPLE_LIMIT = 2_048;

function boundedFraction(heapLimit: number, absolute: number, fraction: number): number {
  return Math.max(1, Math.floor(Math.min(absolute, heapLimit * fraction)));
}

export function nativeSessionBudgets(heapLimitBytes: number): NativeSessionBudgetsV1 {
  const heapLimit = Math.max(64 * MIB, heapLimitBytes || 0);
  return {
    focusa_custom_soft_bytes: boundedFraction(heapLimit, 8 * MIB, 0.005),
    focusa_custom_hard_bytes: boundedFraction(heapLimit, 16 * MIB, 0.01),
    native_segment_soft_bytes: boundedFraction(heapLimit, 64 * MIB, 0.05),
    native_segment_hard_bytes: boundedFraction(heapLimit, 128 * MIB, 0.1),
    native_startup_migration_bytes: boundedFraction(heapLimit, 256 * MIB, 0.2),
    soft_headroom_floor: 0.35,
    hard_headroom_floor: 0.2,
    emergency_headroom_floor: 0.1,
  };
}

function entryCustomType(entry: any): string {
  return String(entry?.customType || entry?.custom_type || "");
}

function entryData(entry: any): any {
  return entry?.data ?? entry?.details ?? null;
}

function entryBytes(entry: any): number {
  try {
    return Buffer.byteLength(JSON.stringify(entry), "utf8");
  } catch {
    return 0;
  }
}

export function measureNativeSessionPressure(input: {
  adapter?: string;
  sessionFile?: string;
  entries?: any[];
  heapUsedBytes?: number;
  heapLimitBytes?: number;
  measuredAt?: Date;
}): NativeSessionPressureV1 {
  const heapStats = getHeapStatistics();
  const heapUsed = Math.max(0, input.heapUsedBytes ?? process.memoryUsage().heapUsed);
  const heapLimit = Math.max(1, input.heapLimitBytes ?? heapStats.heap_size_limit);
  const budgets = nativeSessionBudgets(heapLimit);
  let sessionBytes = 0;
  if (input.sessionFile) {
    try {
      sessionBytes = statSync(input.sessionFile).size;
    } catch {
      sessionBytes = 0;
    }
  }

  const entries = Array.isArray(input.entries) ? input.entries : [];
  const sample = entries.slice(-SAMPLE_LIMIT);
  let focusaCustomBytes = 0;
  let focusaCustomEntries = 0;
  let duplicateAnchorCount = 0;
  const anchorDigests = new Set<string>();
  for (const entry of sample) {
    const customType = entryCustomType(entry);
    if (!customType.startsWith("focusa-")) continue;
    focusaCustomEntries += 1;
    focusaCustomBytes += entryBytes(entry);
    const digest = String(entryData(entry)?.semanticDigest || "");
    if (digest) {
      if (anchorDigests.has(digest)) duplicateAnchorCount += 1;
      else anchorDigests.add(digest);
    }
  }

  const headroomRatio = Math.max(0, Math.min(1, (heapLimit - heapUsed) / heapLimit));
  let posture: NativeSessionPressurePosture = "normal";
  let recommendedAction: NativeSessionPressureV1["recommended_action"] = "continue";

  if (sessionBytes >= budgets.native_startup_migration_bytes) {
    posture = "oversized_at_start";
    recommendedAction = "refuse_full_load";
  } else if (headroomRatio <= budgets.emergency_headroom_floor) {
    posture = "emergency";
    recommendedAction = "stream_migrate";
  } else if (
    sessionBytes >= budgets.native_segment_hard_bytes ||
    focusaCustomBytes >= budgets.focusa_custom_hard_bytes ||
    headroomRatio <= budgets.hard_headroom_floor
  ) {
    posture = "hard_pressure";
    recommendedAction = "rollover";
  } else if (
    sessionBytes >= budgets.native_segment_soft_bytes ||
    focusaCustomBytes >= budgets.focusa_custom_soft_bytes ||
    headroomRatio <= budgets.soft_headroom_floor
  ) {
    posture = "soft_pressure";
    recommendedAction = "checkpoint";
  }

  return {
    schema: "focusa.native_session_pressure.v1",
    adapter: input.adapter || "unknown",
    native_session_ref: input.sessionFile || "unavailable",
    session_bytes: sessionBytes,
    entry_count: entries.length,
    sampled_entry_count: sample.length,
    sample_complete: entries.length <= SAMPLE_LIMIT,
    focusa_custom_bytes: focusaCustomBytes,
    focusa_custom_entries: focusaCustomEntries,
    duplicate_anchor_count: duplicateAnchorCount,
    heap_used_bytes: heapUsed,
    heap_limit_bytes: heapLimit,
    headroom_ratio: headroomRatio,
    budgets,
    posture,
    recommended_action: recommendedAction,
    measured_at: (input.measuredAt || new Date()).toISOString(),
  };
}
