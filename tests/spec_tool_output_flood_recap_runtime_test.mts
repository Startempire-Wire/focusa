import {
  TOOL_OUTPUT_FLOOD_BYTES_THRESHOLD,
  TOOL_OUTPUT_FLOOD_RESULT_THRESHOLD,
  buildAttentionRecallVerdict,
  formatToolOutputVisibleRecapLines,
  getAttachmentRuntime,
  makeAttachmentKey,
  markVisibleRecapEmittedIfPresent,
  maybeCaptureReportSummaryFromAssistantOutput,
  recordToolOutputPressure,
  runWithAttachmentRuntime,
  toolOutputVisibleRecapReason,
} from "../apps/pi-extension/src/state.ts";

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(message);
}

const key = makeAttachmentKey({
  projectRoot: "/home/wirebot/focusa",
  continuityId: "cont-tool-flood-runtime",
  sessionId: "session-tool-flood-runtime",
});

await runWithAttachmentRuntime(key, async () => {
  Object.assign(getAttachmentRuntime(), {
    pi: null,
    focusaAvailable: false,
    sessionCwd: "/home/wirebot/focusa",
    continuityId: "cont-tool-flood-runtime",
    activeFrameGoal: "Tool flood memory refresh proof",
    activeFrameTitle: "Tool flood memory refresh proof",
    lastFocusSnapshot: { decisions: [], constraints: [], failures: [], intent: "", currentFocus: "" },
    latestReportSummary: null,
    toolOutputPressure: {
      windowStartedAt: Date.now(),
      resultCount: 0,
      totalBytes: 0,
      totalTokens: 0,
      largeResultCount: 0,
      recapRequired: false,
      recapReason: "",
      lastToolName: "",
      lastEventAt: 0,
      lastRecapAt: 0,
    },
  });

  const report = `Status: Done — tool flood memory seed report.\n\nProof:\n- Captured report summary handle for replay.\n- Will trigger tool-output pressure counters.\n- Expected refresh remains internal.\n\nNext action: continue without forced recap prose.\n\nThis intentionally has enough content to look like an implementation report and cross the report capture threshold.`;
  const captured = maybeCaptureReportSummaryFromAssistantOutput(report, "pi-turn-flood-report");
  assert(captured?.handle?.startsWith("[HANDLE:report-summary:"), "report handle was not captured");

  for (let i = 0; i < TOOL_OUTPUT_FLOOD_RESULT_THRESHOLD; i++) {
    recordToolOutputPressure(
      `tool-${i}`,
      Math.ceil(TOOL_OUTPUT_FLOOD_BYTES_THRESHOLD / TOOL_OUTPUT_FLOOD_RESULT_THRESHOLD),
      1200,
    );
  }

  const reason = toolOutputVisibleRecapReason();
  assert(reason.includes("tool_output_flood"), `flood reason missing: ${reason}`);
  const verdict = buildAttentionRecallVerdict({
    currentAskText: "continue",
    projectRoot: "/home/wirebot/focusa",
    visibleRecapReason: reason,
  });
  assert(verdict.visible_recap_required === false, "visible recap was forced");
  assert(verdict.status === "attention_risk", `verdict not attention_risk: ${verdict.status}`);
  assert(verdict.memory_anchor.latest_report_summary_ref === captured.handle, "report handle not retained");
  const lines = formatToolOutputVisibleRecapLines(reason).join("\n");
  assert(lines.includes("FOCUSA_MEMORY_REFRESH"), "internal memory refresh missing");
  assert(lines.includes("visibility=internal"), "memory refresh became visible enforcement");
  assert(lines.includes(captured.handle), "memory refresh missing report handle");

  const cleared = markVisibleRecapEmittedIfPresent("Continuing the operator request without forced recap prose.");
  assert(cleared === true, "normal assistant continuation did not consume memory refresh");
  assert(toolOutputVisibleRecapReason() === "", "pressure did not clear after continuation");
});

console.log("PASS: tool-output pressure refreshes internal memory without blocking flow");
