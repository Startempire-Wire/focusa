const TOKEN_BUDGETS = {
    normal: 900,
    pressure: 600,
    critical: 400,
    blocked: 250,
};
function text(value, fallback, max) {
    const candidate = String(value ?? "").trim();
    return (candidate || fallback).slice(0, max).replace(/\s+/g, " ");
}
function appendWithin(lines, line, budgetChars) {
    const used = lines.reduce((total, value) => total + value.length + 1, 0);
    if (used + line.length + 1 <= budgetChars)
        lines.push(line);
}
export function projectionPressure(packet, runtimeTier) {
    if (String(packet?.status) === "blocked")
        return "blocked";
    if (runtimeTier === "hard" || runtimeTier === "critical")
        return "critical";
    if (runtimeTier === "warn" || runtimeTier === "pressure")
        return "pressure";
    return "normal";
}
/** Mandatory authority fields reserve budget first; optional material is added
 * only when it fits. No final/bottom truncation can remove a required field. */
export function renderCompactionResumeProjection(packet, pressure) {
    const budgetTokens = TOKEN_BUDGETS[pressure];
    const budgetChars = budgetTokens * 4;
    const trajectory = packet?.trajectory ?? {};
    const workpoint = packet?.workpoint ?? {};
    const scope = packet?.scope ?? {};
    const next = packet?.next ?? {};
    const temporal = packet?.temporal ?? {};
    const fieldMax = pressure === "blocked" ? 60 : pressure === "critical" ? 140 : 300;
    const scopeMax = pressure === "blocked" ? 80 : pressure === "critical" ? 120 : 180;
    const idMax = pressure === "blocked" ? 64 : pressure === "critical" ? 100 : 160;
    const lines = [
        "## CompactionResumeProjectionV1",
        `BUDGET: ${pressure}:${budgetTokens}`,
        `STATUS: ${text(packet?.status, "blocked", 32)}`,
        `SCOPE_STATUS: ${text(scope?.scope_status, "missing", 32)}`,
        `PROJECT_ROOT: ${text(scope?.project_root, "missing", scopeMax)}`,
        `CONTINUITY_ID: ${text(scope?.continuity_id, "missing", idMax)}`,
        `HLT_STATUS: ${text(trajectory?.hlt_status, "missing_required", 48)}`,
        `WORKPOINT_STATUS: ${text(workpoint?.status, "missing", 32)}`,
        `TEMPORAL_STATUS: ${text(temporal?.status, "unavailable", 32)}`,
        `DEADLINE_STATUS: ${text(temporal?.deadline_status, "none", 32)}`,
        `TEMPORAL_REFS: ${text([temporal?.calendar_context_ref, temporal?.priority_frame_ref, temporal?.execution_guard_ref]
            .filter(Boolean)
            .join(","), "none", idMax)}`,
        `HLT: ${text(trajectory?.hlt, "missing", fieldMax)}`,
        `MISSION: ${text(workpoint?.mission, "missing", fieldMax)}`,
        `NEXT_SLICE: ${text(workpoint?.next_slice, "missing", fieldMax)}`,
        `EXACT_NEXT_TOOL: ${text(next?.exact_next_tool, "focusa_workpoint_resume", 80)}`,
        `PACKET_ID: ${text(packet?.packet_id, "missing", idMax)}`,
        "AUTHORITY: advisory projection only; canonical Trajectory, Workpoint, Focus State, and evidence prevail.",
    ];
    const evidenceRefs = Array.isArray(packet?.evidence?.evidence_refs)
        ? packet.evidence.evidence_refs.slice(0, 12).map(String)
        : [];
    const rehydrateRefs = Array.isArray(packet?.bloatgaurd?.rehydrate_refs)
        ? packet.bloatgaurd.rehydrate_refs.slice(0, 12).map(String)
        : [];
    appendWithin(lines, `EVIDENCE_REFS: ${evidenceRefs.join(",") || "none"}`, budgetChars);
    appendWithin(lines, `REHYDRATE_REFS: ${rehydrateRefs.join(",") || "focusa_workpoint_resume"}`, budgetChars);
    const warnings = Array.isArray(trajectory?.warnings)
        ? trajectory.warnings
            .slice(0, 4)
            .map((value) => text(value, "", 120))
            .filter(Boolean)
        : [];
    appendWithin(lines, `WARNINGS: ${warnings.join(" | ") || "none"}`, budgetChars);
    const rendered = lines.join("\n");
    if (rendered.length > budgetChars) {
        // Mandatory fields are deliberately sized to fit even the blocked budget.
        throw new Error("compaction_resume_projection_mandatory_budget_exceeded");
    }
    return rendered;
}
export function compactionProjectionBudgetTokens(pressure) {
    return TOKEN_BUDGETS[pressure];
}
