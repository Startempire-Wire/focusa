import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const projectDir = fileURLToPath(new URL("..", import.meta.url));
const outDir = mkdtempSync(join(tmpdir(), "focusa-bloatgaurd-pressure-circuit-"));
const compactionSource = readFileSync(new URL("../src/compaction.ts", import.meta.url), "utf8");

const checkStart = compactionSource.indexOf("export async function checkCompactionTier");
const checkEnd = compactionSource.indexOf("// ── Periodic micro-compact", checkStart);

const pressureCfg = {
  warnPct: 50,
  compactPct: 70,
  hardPct: 85,
};

const runtimeCfg = {
  ...pressureCfg,
  cooldownMs: 120_000,
  maxCompactionsPerHour: 8,
  minTurnsBetweenCompactions: 2,
  autoSuggestForkPct: 90,
  autoSuggestHandoffAfterNCompactions: 3,
};

function buildCtx() {
  const compact = [];
  return {
    compact,
    ctx: {
      getContextUsage: () => ({ percent: 0, tokens: 1_000 }),
      ui: {
        notify() {},
        setStatus() {},
      },
      compact(payload) {
        compact.push(payload);
        payload.onComplete?.();
      },
    },
  };
}

async function runCheck(compaction, state, attachmentKey, pct, canCompact) {
  const ctxBundle = buildCtx();
  state.runWithAttachmentRuntime(attachmentKey, () => {
    const runtime = state.getAttachmentRuntime();
    runtime.cfg = {
      ...runtimeCfg,
      warnPct: pressureCfg.warnPct,
      compactPct: pressureCfg.compactPct,
      hardPct: pressureCfg.hardPct,
    };
    runtime.focusaAvailable = false;
    runtime.lastCompactTime = canCompact ? Date.now() - (runtimeCfg.cooldownMs + 10_000) : Date.now();
    runtime.compactsThisHour = 0;
    runtime.compactHourStart = Date.now() - 5_000;
    runtime.turnsSinceCompact = Math.max(runtimeCfg.minTurnsBetweenCompactions - 1, 0);
  });
  ctxBundle.ctx.getContextUsage = () => ({ percent: pct, tokens: 1_000 });

  await state.runWithAttachmentRuntime(attachmentKey, async () => {
    await compaction.checkCompactionTier(ctxBundle.ctx);
  });

  const runtime = state.runWithAttachmentRuntime(attachmentKey, () => state.getAttachmentRuntime());
  return {
    compactCalls: ctxBundle.compact,
    runtime,
  };
}

try {
  symlinkSync(join(projectDir, "node_modules"), join(outDir, "node_modules"), "dir");
  execFileSync("./node_modules/.bin/tsc", [
    "-p",
    "tsconfig.json",
    "--outDir",
    outDir,
    "--noEmit",
    "false",
    "--module",
    "ES2022",
  ], { cwd: projectDir, stdio: "pipe" });
  writeFileSync(join(outDir, "package.json"), "{\"type\":\"module\"}\n");

  const compaction = await import(pathToFileURL(join(outDir, "compaction.js")).href);
  const state = await import(pathToFileURL(join(outDir, "state.js")).href);

  assert(checkStart >= 0, "checkCompactionTier source must be discoverable");
  assert(checkEnd > checkStart, "checkCompactionTier source block must include micro-compact boundary");
  const checkBlock = compactionSource.slice(checkStart, checkEnd);
  assert(
    checkBlock.includes("classifyBloatgaurdPressureAction"),
    "checkCompactionTier should call the bloatgaurd pressure classifier"
  );
  assert(checkBlock.includes("ctx.compact({"), "checkCompactionTier should preserve existing ctx.compact path");

  assert.equal(compaction.classifyBloatgaurdPressureAction(49, pressureCfg, true), "none");
  assert.equal(compaction.classifyBloatgaurdPressureAction(50, pressureCfg, true), "warn");
  assert.equal(compaction.classifyBloatgaurdPressureAction(69, pressureCfg, true), "warn");
  assert.equal(compaction.classifyBloatgaurdPressureAction(70, pressureCfg, true), "auto");
  assert.equal(compaction.classifyBloatgaurdPressureAction(70, pressureCfg, false), "warn");
  assert.equal(compaction.classifyBloatgaurdPressureAction(85, pressureCfg, true), "hard");
  assert.equal(compaction.classifyBloatgaurdPressureAction(85, pressureCfg, false), "hard");

  state.attachmentRuntimeRegistry.reset();
  const attachmentKey = state.makeAttachmentKey({
    projectRoot: "/tmp/focusa-bloatgaurd-pressure",
    continuityId: "cont-pressure",
    sessionId: "session-pressure",
    attachmentId: "attach-pressure",
  });
  state.attachmentRuntimeRegistry.bindSessionAttachment(attachmentKey);

  const hardByCooldown = await runCheck(compaction, state, attachmentKey, 85, false);
  assert.equal(hardByCooldown.compactCalls.length, 1, "hard tier should trigger ctx.compact even during cooldown");

  const highWithCooldown = await runCheck(compaction, state, attachmentKey, 75, false);
  assert.equal(highWithCooldown.compactCalls.length, 0, "auto-tier should be suppressed during cooldown");
  assert.equal(highWithCooldown.runtime.currentTier, "warn", "cooldown-suppressed high pressure should remain warn");

  const highWithoutCooldown = await runCheck(compaction, state, attachmentKey, 75, true);
  assert.equal(highWithoutCooldown.compactCalls.length, 1, "auto tier should compact when cooldown is satisfied");

  console.log("spec101 bloatgaurd pressure circuit runtime test passed");
} finally {
  rmSync(outDir, { recursive: true, force: true });
}
