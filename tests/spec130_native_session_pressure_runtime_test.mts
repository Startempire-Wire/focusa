import {
  measureNativeSessionPressure,
  migrateNativeSessionBounded,
  nativeSessionBudgets,
} from "../apps/pi-extension/src/session-pressure.ts";
import {
  existsSync,
  mkdtempSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  rmSync,
  statSync,
  truncateSync,
  writeFileSync,
} from "fs";
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

  const migrationSource = join(root, "migration-source.jsonl");
  const migrationOutput = join(root, "migration-output");
  const oversizedEntry = JSON.stringify({ type: "tool", payload: "x".repeat(40 * 1024) });
  const sourceBody = [
    JSON.stringify({ type: "session", id: "spec130" }),
    oversizedEntry,
    JSON.stringify({ type: "compaction", id: "cmp-final", summary: "bounded recovery" }),
  ].join("\n") + "\n";
  writeFileSync(migrationSource, sourceBody);
  const sourceBefore = readFileSync(migrationSource);
  const scope = {
    root_scope: {
      scope_kind: "project" as const,
      scope_id: "project:spec130",
      root_path: root,
      canonical_name: "spec130-fixture",
      fingerprint: "sha256:spec130",
    },
    continuity_id: "focusa-cont-spec130",
  };

  const dryRun = await migrateNativeSessionBounded({
    source_path: migrationSource,
    output_dir: migrationOutput,
    scope,
    mode: "dry_run",
    recovery_max_bytes: 64 * 1024,
    entry_max_bytes: 16 * 1024,
  });
  assert(dryRun.archive === null, "dry-run unexpectedly wrote archive metadata");
  assert(!existsSync(migrationOutput), "dry-run mutated output directory");

  const migrated = await migrateNativeSessionBounded({
    source_path: migrationSource,
    output_dir: migrationOutput,
    scope,
    mode: "execute",
    recovery_max_bytes: 64 * 1024,
    entry_max_bytes: 16 * 1024,
  });
  assert(migrated.integrity.source_unchanged, "migration mutated source session");
  assert(migrated.integrity.archive_matches_source, "archive checksum mismatch");
  assert(migrated.integrity.recovery_within_budget, "recovery segment exceeded budget");
  assert(migrated.archive?.immutable === true, "archive is not immutable");
  assert((statSync(migrated.archive!.path).mode & 0o777) === 0o400, "archive mode is not read-only");
  assert(migrated.recovery_segment!.omitted_oversized_entries === 1, "oversized entry was not externalized");
  assert(
    readFileSync(migrated.recovery_segment!.path, "utf8").includes("focusa-migration-omitted-entry"),
    "recovery segment lacks oversized-entry handle"
  );
  assert(existsSync(migrated.manifest_path!), "migration manifest missing");
  assert(readFileSync(migrationSource).equals(sourceBefore), "source bytes changed after migration");

  const faultSteps = [
    "after_prepare",
    "after_archive_write",
    "after_archive_checksum",
    "after_archive_seal",
    "after_recovery_write",
    "after_recovery_checksum",
    "after_source_verify",
    "after_manifest_write",
    "after_manifest_commit",
  ] as const;
  for (const step of faultSteps) {
    const faultOutput = join(root, `fault-${step}`);
    let fault = "";
    try {
      await migrateNativeSessionBounded({
        source_path: migrationSource,
        output_dir: faultOutput,
        scope,
        mode: "execute",
        recovery_max_bytes: 64 * 1024,
        entry_max_bytes: 16 * 1024,
        fault_injection_step: step,
      });
    } catch (error) {
      fault = error instanceof Error ? error.message : String(error);
    }
    assert(fault === `native_session_migration_fault:${step}`, `wrong ${step} failure`);
    assert(readFileSync(migrationSource).equals(sourceBefore), `${step} mutated source session`);
    assert(
      !existsSync(faultOutput) || readdirSync(faultOutput).length === 0,
      `${step} left committed or temporary migration files`
    );

    const retried = await migrateNativeSessionBounded({
      source_path: migrationSource,
      output_dir: faultOutput,
      scope,
      mode: "execute",
      recovery_max_bytes: 64 * 1024,
      entry_max_bytes: 16 * 1024,
    });
    assert(retried.integrity.source_unchanged, `${step} retry lost source integrity`);
    assert(retried.integrity.archive_matches_source, `${step} retry lost archive integrity`);
    assert(retried.integrity.recovery_within_budget, `${step} retry exceeded recovery budget`);
    rmSync(faultOutput, { recursive: true, force: true });
  }

  const rollbackOutput = join(root, "rollback-output");
  const rollbackPlan = await migrateNativeSessionBounded({
    source_path: migrationSource,
    output_dir: rollbackOutput,
    scope,
    mode: "dry_run",
    recovery_max_bytes: 64 * 1024,
    entry_max_bytes: 16 * 1024,
  });
  const rollbackDigest = rollbackPlan.migration_id.replace("native-session-", "");
  const rollbackBase = "migration-source.jsonl";
  const collidingRecovery = join(
    rollbackOutput,
    `${rollbackBase}.${rollbackDigest}.recovery.jsonl`
  );
  const rollbackArchive = join(
    rollbackOutput,
    `${rollbackBase}.${rollbackDigest}.immutable.jsonl`
  );
  mkdirSync(rollbackOutput, { recursive: true });
  writeFileSync(collidingRecovery, "preexisting\n");
  let rollbackError = "";
  try {
    await migrateNativeSessionBounded({
      source_path: migrationSource,
      output_dir: rollbackOutput,
      scope,
      mode: "execute",
      recovery_max_bytes: 64 * 1024,
      entry_max_bytes: 16 * 1024,
    });
  } catch (error) {
    rollbackError = error instanceof Error ? error.message : String(error);
  }
  assert(Boolean(rollbackError), "collision did not fail migration");
  assert(!existsSync(rollbackArchive), "failed migration left a partial archive");
  assert(readFileSync(collidingRecovery, "utf8") === "preexisting\n", "rollback removed preexisting file");
  assert(readFileSync(migrationSource).equals(sourceBefore), "rollback mutated source session");

  console.log("PASS: Spec 130 native session pressure and streaming migration policy");
} finally {
  rmSync(root, { recursive: true, force: true });
}
