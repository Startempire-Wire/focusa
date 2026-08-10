const QUALITY_REGRESSION_TOLERANCE = 0.05;
function finiteScore(value) {
    if (value === null || !Number.isFinite(value))
        return null;
    return Math.min(1, Math.max(0, value));
}
function boundedRefs(refs) {
    return [...new Set(refs.map((ref) => ref.trim()).filter(Boolean))].sort().slice(0, 128);
}
export function rollbackRouteFor(route) {
    switch (route) {
        case "native_compact":
        case "summarize":
            return "checkpoint";
        case "curate_context":
        case "rollover":
            return "no_op";
        default:
            return route;
    }
}
/**
 * Compare explicit pre/post authority snapshots. Unknown measurements stay
 * unknown; they never become a success or failure claim. Any authority drift,
 * missing prior evidence, provider failure, or measured quality regression
 * deterministically quarantines the attempted policy.
 */
export function evaluateCompactionOutcome(baseline, outcome, regressionTolerance = QUALITY_REGRESSION_TOLERANCE) {
    const reasons = [];
    if (outcome.providerOutcome === "failed")
        reasons.push("provider_failure");
    if (baseline.snapshot.projectRoot !== outcome.projectRoot ||
        baseline.snapshot.sessionId !== outcome.sessionId) {
        reasons.push("scope_drift");
    }
    if (baseline.snapshot.continuityRef !== null && baseline.snapshot.continuityRef !== outcome.continuityRef) {
        reasons.push("hallucinated_continuity");
    }
    if (baseline.snapshot.workpointRef !== null && baseline.snapshot.workpointRef !== outcome.workpointRef) {
        reasons.push("workpoint_drift");
    }
    const outcomeEvidence = new Set(boundedRefs(outcome.evidenceRefs));
    const missingEvidenceRefs = boundedRefs(baseline.snapshot.evidenceRefs).filter((ref) => !outcomeEvidence.has(ref));
    if (missingEvidenceRefs.length > 0)
        reasons.push("missing_evidence");
    const baselineQuality = finiteScore(baseline.snapshot.qualityScore);
    const outcomeQuality = finiteScore(outcome.qualityScore);
    const qualityDelta = baselineQuality === null || outcomeQuality === null
        ? null
        : Math.round((outcomeQuality - baselineQuality) * 1_000_000) / 1_000_000;
    if (qualityDelta !== null && qualityDelta < -Math.abs(regressionTolerance)) {
        reasons.push("quality_regression");
    }
    const tokenDelta = baseline.snapshot.contextTokens === null || outcome.contextTokens === null
        ? null
        : outcome.contextTokens - baseline.snapshot.contextTokens;
    const rollbackRequired = reasons.length > 0;
    const fullyMeasured = baselineQuality !== null && outcomeQuality !== null;
    return {
        schema: "focusa.compaction_outcome_evaluation.v1",
        policyVersion: baseline.policyVersion,
        policyKey: baseline.policyKey,
        attemptedRoute: baseline.route,
        disposition: rollbackRequired ? "quarantine" : fullyMeasured ? "promote" : "retain",
        rollbackRequired,
        rollbackRoute: rollbackRequired ? rollbackRouteFor(baseline.route) : baseline.route,
        reasons,
        missingEvidenceRefs,
        qualityDelta,
        tokenDelta,
        shadowComparison: {
            baselineQuality,
            outcomeQuality,
            regressionTolerance: Math.abs(regressionTolerance),
        },
    };
}
