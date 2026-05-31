import { S, maybeCaptureReportSummaryFromAssistantOutput, buildAttentionRecallVerdict, getEcsArtifact, extractHandles } from "../apps/pi-extension/src/state.ts";

function assert(cond: any, msg: string) {
  if (!cond) throw new Error(msg);
}

Object.assign(S, {
  pi: null,
  sessionCwd: "/home/wirebot/focusa",
  continuityId: "cont-report-runtime",
  activeFrameGoal: "Report replay proof",
  activeFrameTitle: "Report replay proof",
  lastFocusSnapshot: { decisions: [], constraints: [], failures: [], intent: "", currentFocus: "" },
  latestReportSummary: null,
});

const report = `Status: Done — report replay proof complete.

Proof:
- Captured report summary handle.
- Replayed latest_report_summary_ref through AttentionRecallVerdict.
- Avoided raw transcript-tail dependence.

Next action: continue with bounded implementation.

This intentionally has enough content to cross the report capture threshold and resembles a final implementation report with proof and next action.`;

const captured = maybeCaptureReportSummaryFromAssistantOutput(report, "pi-turn-report-runtime");
assert(captured, "report summary was not captured");
assert(captured!.handle.startsWith("[HANDLE:report-summary:"), `bad handle: ${captured!.handle}`);
const handle = extractHandles(captured!.handle)[0];
assert(handle?.kind === "report-summary", `bad extracted handle: ${JSON.stringify(handle)}`);
const stored = getEcsArtifact(handle.kind, handle.id);
assert(stored && stored.includes("Status: Done"), `stored summary missing: ${stored}`);
const verdict = buildAttentionRecallVerdict({ currentAskText: "continue", projectRoot: "/home/wirebot/focusa" });
assert(verdict.memory_anchor.latest_report_summary_ref === captured!.handle, `verdict did not replay latest handle: ${JSON.stringify(verdict.memory_anchor)}`);

console.log("SPEC report summary capture/replay runtime proof passed");
