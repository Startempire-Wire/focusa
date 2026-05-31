import {
  S,
  TOOL_OUTPUT_FLOOD_BYTES_THRESHOLD,
  TOOL_OUTPUT_FLOOD_RESULT_THRESHOLD,
  buildAttentionRecallVerdict,
  formatToolOutputVisibleRecapLines,
  markVisibleRecapEmittedIfPresent,
  maybeCaptureReportSummaryFromAssistantOutput,
  recordToolOutputPressure,
  toolOutputVisibleRecapReason,
} from "../apps/pi-extension/src/state.ts";

function assert(cond: any, msg: string) {
  if (!cond) throw new Error(msg);
}

Object.assign(S, {
  pi: null,
  focusaAvailable: false,
  sessionCwd: "/home/wirebot/focusa",
  continuityId: "cont-tool-flood-runtime",
  activeFrameGoal: "Tool flood recap proof",
  activeFrameTitle: "Tool flood recap proof",
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

const report = `Status: Done — tool flood recap seed report.

Proof:
- Captured report summary handle for replay.
- Will trigger tool-output flood counters.
- Expected visible recap comes from MEMORY_ANCHOR/latest report.

Next action: continue with bounded recap enforcement proof.

This intentionally has enough content to look like an implementation report and cross the report capture threshold.`;
const captured = maybeCaptureReportSummaryFromAssistantOutput(report, "pi-turn-flood-report");
assert(captured?.handle?.startsWith("[HANDLE:report-summary:"), "report handle was not captured");

for (let i = 0; i < TOOL_OUTPUT_FLOOD_RESULT_THRESHOLD; i++) {
  recordToolOutputPressure(`tool-${i}`, Math.ceil(TOOL_OUTPUT_FLOOD_BYTES_THRESHOLD / TOOL_OUTPUT_FLOOD_RESULT_THRESHOLD), 1200);
}

const reason = toolOutputVisibleRecapReason();
assert(reason.includes("tool_output_flood"), `flood reason missing: ${reason}`);
const verdict = buildAttentionRecallVerdict({ currentAskText: "continue", projectRoot: "/home/wirebot/focusa", visibleRecapReason: reason });
assert(verdict.visible_recap_required === true, `visible recap not required: ${JSON.stringify(verdict)}`);
assert(verdict.status === "attention_risk", `verdict not attention_risk: ${verdict.status}`);
assert(verdict.attention_risks.includes("tool_output_flood"), `risk missing: ${JSON.stringify(verdict.attention_risks)}`);
assert(verdict.memory_anchor.latest_report_summary_ref === captured!.handle, "latest report handle not replayed");
const lines = formatToolOutputVisibleRecapLines(reason).join("\n");
assert(lines.includes("VISIBLE_RECAP_REQUIRED"), "visible recap block missing");
assert(lines.includes(captured!.handle), "visible recap block missing report handle");

const cleared = markVisibleRecapEmittedIfPresent(`Recap: ${verdict.memory_anchor.task}; latest report ${captured!.handle}; next ${verdict.memory_anchor.next_action}.`);
assert(cleared === true, "recap emission did not clear flood pressure");
assert(toolOutputVisibleRecapReason() === "", "flood reason did not clear after recap");

console.log("SPEC tool-output flood recap runtime proof passed");
