import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const projectDir = fileURLToPath(new URL("..", import.meta.url));
const outDir = mkdtempSync(join(tmpdir(), "focusa-bloatgaurd-pressure-circuit-"));
const compactionSource = readFileSync(new URL("../src/compaction.ts", import.meta.url), "utf8");

const sessionCompactStart = compactionSource.indexOf('pi.on("session_compact"');
const sessionCompactEnd = compactionSource.indexOf("// ── Compaction tier check", sessionCompactStart);
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
  const contextWindow = 200_000;
  let usagePct = 0;
  const message = (id) => ({
    type: "message",
    id,
    parentId: null,
    timestamp: 1,
    message: { role: "user", content: "x".repeat(100_000), timestamp: 1 },
  });
  const branch = [message("a"), message("b"), message("c"), message("d")];
  const ctx = {
    cwd: "/tmp/focusa-bloatgaurd-pressure",
    getContextUsage: () => ({
      percent: usagePct,
      tokens: Math.ceil((contextWindow * usagePct) / 100),
      contextWindow,
    }),
    isIdle: () => true,
    hasPendingMessages: () => false,
    sessionManager: {
      getBranch: () => branch,
      getSessionId: () => "spec101-pressure-session",
    },
    ui: {
      notify() {},
      setStatus() {},
    },
    hasUI: true,
    compact(payload) {
      compact.push(payload);
      payload.onComplete?.({
        summary: "bounded summary",
        firstKeptEntryId: "d",
        tokensBefore: Math.ceil((contextWindow * usagePct) / 100),
      });
    },
  };
  return { compact, ctx, setUsagePct: (pct) => (usagePct = pct) };
}

async function runEmergencyInput(autoCompaction, state, attachmentKey) {
  const ctxBundle = buildCtx();
  const handlers = new Map();
  const entries = [];
  const sentMessages = [];
  const pi = {
    on(name, handler) {
      handlers.set(name, handler);
    },
    appendEntry(type, data) {
      entries.push({ type, data });
    },
    sendUserMessage(message) {
      sentMessages.push(message);
    },
  };
  assert.equal(
    autoCompaction.registerAutoCompaction(pi, () => ({
      ...autoCompaction.DEFAULT_PROACTIVE_COMPACTION_POLICY,
      cooldownMs: 0,
    })),
    true
  );
  state.runWithAttachmentRuntime(attachmentKey, () => {
    const runtime = state.getAttachmentRuntime();
    runtime.cfg = { ...runtimeCfg };
    runtime.focusaAvailable = false;
  });
  ctxBundle.setUsagePct(129.5);
  const result = await state.runWithAttachmentRuntime(attachmentKey, () =>
    handlers.get("input")(
      {
        text: "Preserve this operator steering",
        images: [{ type: "image", source: { type: "base64", data: "fixture" } }],
        source: "interactive",
      },
      ctxBundle.ctx
    )
  );
  await handlers.get("session_shutdown")({ type: "session_shutdown" }, ctxBundle.ctx);
  return { result, entries, sentMessages };
}

async function runCheck(autoCompaction, compaction, state, attachmentKey, pct, canCompact) {
  const ctxBundle = buildCtx();
  const handlers = new Map();
  const pi = {
    on(name, handler) {
      handlers.set(name, handler);
    },
    appendEntry() {},
  };
  assert.equal(
    autoCompaction.registerAutoCompaction(pi, () => ({
      ...autoCompaction.DEFAULT_PROACTIVE_COMPACTION_POLICY,
      cooldownMs: 0,
    })),
    true
  );
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
  ctxBundle.setUsagePct(pct);

  await state.runWithAttachmentRuntime(attachmentKey, async () => {
    await compaction.checkCompactionTier(ctxBundle.ctx);
  });

  const runtime = state.runWithAttachmentRuntime(attachmentKey, () => state.getAttachmentRuntime());
  await handlers.get("session_shutdown")({ type: "session_shutdown" }, ctxBundle.ctx);
  return {
    compactCalls: ctxBundle.compact,
    runtime,
  };
}

try {
  symlinkSync(join(projectDir, "node_modules"), join(outDir, "node_modules"), "dir");
  execFileSync(
    "./node_modules/.bin/tsc",
    ["-p", "tsconfig.json", "--outDir", outDir, "--noEmit", "false", "--module", "ES2022"],
    { cwd: projectDir, stdio: "pipe" }
  );
  writeFileSync(join(outDir, "package.json"), '{"type":"module"}\n');

  const autoCompaction = await import(pathToFileURL(join(outDir, "auto-compaction.js")).href);
  const compaction = await import(pathToFileURL(join(outDir, "compaction.js")).href);
  const state = await import(pathToFileURL(join(outDir, "state.js")).href);

  assert(checkStart >= 0, "checkCompactionTier source must be discoverable");
  assert(checkEnd > checkStart, "checkCompactionTier source block must include micro-compact boundary");
  const checkBlock = compactionSource.slice(checkStart, checkEnd);
  assert(
    checkBlock.includes("classifyBloatgaurdPressureAction"),
    "checkCompactionTier should call the bloatgaurd pressure classifier"
  );
  assert(
    checkBlock.includes("requestCoordinatedCompaction(ctx"),
    "checkCompactionTier must route native compaction through the process-wide coordinator"
  );
  assert(
    !checkBlock.includes("ctx.compact({"),
    "checkCompactionTier must not own an independent native compaction path"
  );
  assert(
    sessionCompactStart >= 0 && sessionCompactEnd > sessionCompactStart,
    "session_compact block must exist"
  );
  const sessionCompactBlock = compactionSource.slice(sessionCompactStart, sessionCompactEnd);
  assert(
    sessionCompactBlock.includes("resetLiveContextPressureAfterCompaction()"),
    "saved manual compaction must reset live pressure"
  );
  assert(
    sessionCompactBlock.includes('setContextStatus(ctx, "")'),
    "saved manual compaction must clear the live-context UI status"
  );

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

  const scopeObservations = state.runWithAttachmentRuntime(attachmentKey, () =>
    state.observeProjectThreadHintsFromText(
      "Focusa pressure was 129.5; evidence is focusa-final-ci-spec-pirpc.log",
      "pi-turn-spec142",
      "current_ask"
    )
  );
  assert(
    scopeObservations.every(
      (entry) => entry.project_alias !== "129.5" && !entry.project_alias.endsWith(".log")
    ),
    "numeric pressure and log artifacts must not become project aliases"
  );

  const emergency = await runEmergencyInput(autoCompaction, state, attachmentKey);
  assert.deepEqual(emergency.result, { action: "continue" });
  assert.deepEqual(emergency.sentMessages, []);
  assert(
    emergency.entries.some((entry) => entry.data?.kind === "input_passthrough_native_overflow_recovery"),
    "129.5% input should record native overflow recovery without intercepting the prompt"
  );

  const nativePressure = { posture: "hard_pressure", recommended_action: "rollover" };
  state.runWithAttachmentRuntime(attachmentKey, () => {
    const runtime = state.getAttachmentRuntime();
    runtime.currentTier = "hard";
    runtime.currentContextPct = 97;
    runtime.turnsSinceCompact = 11;
    runtime.lastCompactTime = 0;
    runtime.forkSuggested = true;
    runtime.lastNativeSessionPressure = nativePressure;
    runtime.lastNativeSessionPressureNoticeKey = "hard_pressure:rollover";
    compaction.resetLiveContextPressureAfterCompaction(123_456);
  });
  const manualReset = state.runWithAttachmentRuntime(attachmentKey, () => state.getAttachmentRuntime());
  assert.equal(manualReset.currentTier, "", "manual compaction must clear live tier");
  assert.equal(manualReset.currentContextPct, null, "manual compaction must clear live percentage");
  assert.equal(manualReset.turnsSinceCompact, 0, "manual compaction must reset turn cooldown");
  assert.equal(manualReset.lastCompactTime, 123_456, "manual compaction must start time cooldown");
  assert.equal(manualReset.forkSuggested, false, "manual compaction must reset stale fork suggestion");
  assert.equal(
    manualReset.lastNativeSessionPressure,
    nativePressure,
    "native hard pressure must remain authoritative"
  );
  assert.equal(
    manualReset.lastNativeSessionPressureNoticeKey,
    "hard_pressure:rollover",
    "native pressure notice must not be cleared by prompt compaction"
  );

  const hardByCooldown = await runCheck(autoCompaction, compaction, state, attachmentKey, 85, false);
  assert.equal(
    hardByCooldown.compactCalls.length,
    1,
    "hard tier should trigger ctx.compact even during cooldown"
  );
  assert.equal(hardByCooldown.runtime.currentTier, "", "hard compaction completion must clear live tier");
  assert.equal(
    hardByCooldown.runtime.currentContextPct,
    null,
    "hard compaction completion must clear percentage"
  );

  const highWithCooldown = await runCheck(autoCompaction, compaction, state, attachmentKey, 75, false);
  assert.equal(highWithCooldown.compactCalls.length, 0, "auto-tier should be suppressed during cooldown");
  assert.equal(
    highWithCooldown.runtime.currentTier,
    "warn",
    "cooldown-suppressed high pressure should remain warn"
  );

  const highWithoutCooldown = await runCheck(autoCompaction, compaction, state, attachmentKey, 75, true);
  assert.equal(
    highWithoutCooldown.compactCalls.length,
    1,
    "auto tier should compact when cooldown is satisfied"
  );
  assert.equal(
    highWithoutCooldown.runtime.currentTier,
    "",
    "auto compaction completion must clear live tier"
  );
  assert.equal(
    highWithoutCooldown.runtime.currentContextPct,
    null,
    "auto compaction completion must clear percentage"
  );

  console.log("spec101 bloatgaurd pressure circuit runtime test passed");
} finally {
  rmSync(outDir, { recursive: true, force: true });
}
