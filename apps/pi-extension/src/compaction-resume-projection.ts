export type CompactionProjectionPressure = "normal" | "pressure" | "critical" | "blocked";

const TOKEN_BUDGETS: Record<CompactionProjectionPressure, number> = {
  normal: 900,
  pressure: 600,
  critical: 400,
  blocked: 250,
};

function text(value: unknown, fallback: string): string {
  // Preserve required meaning, including multiline instructions. The one total
  // budget below may block a projection; individual fields never lose suffixes.
  const candidate = String(value ?? "");
  return JSON.stringify(candidate || fallback);
}

function appendWithin(lines: string[], line: string, budgetChars: number): void {
  const used = lines.reduce((total, value) => total + value.length + 1, 0);
  if (used + line.length + 1 <= budgetChars) lines.push(line);
}

export function projectionPressure(packet: any, runtimeTier: string): CompactionProjectionPressure {
  if (String(packet?.status) === "blocked") return "blocked";
  if (runtimeTier === "hard" || runtimeTier === "critical") return "critical";
  if (runtimeTier === "warn" || runtimeTier === "pressure") return "pressure";
  return "normal";
}

/** Mandatory authority fields reserve budget first; optional material is added
 * only when it fits. No final/bottom truncation can remove a required field. */
export function renderCompactionResumeProjection(
  packet: any,
  pressure: CompactionProjectionPressure
): string {
  const budgetTokens = TOKEN_BUDGETS[pressure];
  const budgetChars = budgetTokens * 4;
  const trajectory = packet?.trajectory ?? {};
  const workpoint = packet?.workpoint ?? {};
  const scope = packet?.scope ?? {};
  const next = packet?.next ?? {};
  const temporal = packet?.temporal ?? {};
  const constraints = packet?.action_critical_constraints ?? packet?.focus_state?.constraints ?? [];
  const warnings = Array.isArray(trajectory?.warnings) ? trajectory.warnings.map(String) : [];
  const scopeWarning = [scope?.warning, ...warnings].filter(Boolean).join(" | ");
  const lines = [
    "## CompactionResumeProjectionV1",
    `BUDGET: ${pressure}:${budgetTokens}`,
    `STATUS: ${text(packet?.status, "blocked")}`,
    `SCOPE_STATUS: ${text(scope?.scope_status, "missing")}`,
    `PROJECT_ROOT: ${text(scope?.project_root, "missing")}`,
    `CONTINUITY_ID: ${text(scope?.continuity_id, "missing")}`,
    `HLT_STATUS: ${text(trajectory?.hlt_status, "missing_required")}`,
    `WORKPOINT_STATUS: ${text(workpoint?.status, "missing")}`,
    `CURRENT_ASK: ${text(packet?.current_ask?.text, "missing; reverify original input")}`,
    `REQUEST_FIDELITY: ${packet?.current_ask?.text_complete === true ? "complete" : "unverified; saved-action resume blocked; rehydrate before effects"}`,
    `SCOPE_WARNING: ${text(scopeWarning, "Recheck scope and current request before effects")}`,
    `HLT: ${text(trajectory?.hlt, "missing")}`,
    `MISSION: ${text(workpoint?.mission, "missing")}`,
    `NEXT_SLICE: ${text(workpoint?.next_slice, "missing")}`,
    `ACTIVE_BLOCKER: ${text(packet?.active_blocker?.exact_blocker_excerpt, "none")}`,
    `CRITICAL_CONSTRAINTS: ${JSON.stringify(constraints)}`,
    `RECEIPT_EXPECTATION: ${JSON.stringify(packet?.receipt_expectation ?? { required_before_completion: true })}`,
    `EXACT_NEXT_TOOL: ${text(next?.exact_next_tool, "focusa_workpoint_resume")}`,
    `PACKET_ID: ${text(packet?.packet_id, "missing")}`,
    "AUTHORITY: advisory; current scope, request and effect checks govern.",
  ];
  const evidenceRefs = Array.isArray(packet?.evidence?.evidence_refs)
    ? packet.evidence.evidence_refs.map(String)
    : [];
  const rehydrateRefs = Array.isArray(packet?.bloatgaurd?.rehydrate_refs)
    ? packet.bloatgaurd.rehydrate_refs.map(String)
    : [];
  // Evidence/rehydration expectations and a supplied effect guard are required.
  if (temporal?.execution_guard_ref) {
    lines.push(`TEMPORAL_GUARD_REF: ${text(temporal.execution_guard_ref, "missing")}`);
  }
  lines.push(`EVIDENCE_REFS: ${JSON.stringify(evidenceRefs)}`);
  lines.push(`REHYDRATE_REFS: ${JSON.stringify(rehydrateRefs.length ? rehydrateRefs : ["focusa_workpoint_resume"])}`);
  appendWithin(lines, `TEMPORAL_STATUS: ${text(temporal?.status, "unavailable")}`, budgetChars);
  appendWithin(lines, `DEADLINE_STATUS: ${text(temporal?.deadline_status, "none")}`, budgetChars);
  appendWithin(lines, `TEMPORAL_REFS: ${text(
    [temporal?.calendar_context_ref, temporal?.priority_frame_ref, temporal?.execution_guard_ref]
      .filter(Boolean).join(","), "none"
  )}`, budgetChars);
  const rendered = lines.join("\n");
  if (rendered.length > budgetChars) {
    // No final truncation or stale/fallback request may replace essential meaning.
    throw new Error("compaction_resume_projection_mandatory_budget_exceeded: required meaning must be rehydrated or compaction blocked");
  }
  return rendered;
}

export function compactionProjectionBudgetTokens(pressure: CompactionProjectionPressure): number {
  return TOKEN_BUDGETS[pressure];
}
