import type { CompactionPolicyRoute } from "./compaction-policy-selector.js";

export type CompactionProviderOutcome = "succeeded" | "failed" | "unknown";

export interface CompactionContinuationSnapshot {
  projectRoot: string;
  sessionId: string;
  continuityRef: string | null;
  workpointRef: string | null;
  evidenceRefs: string[];
  providerOutcome: CompactionProviderOutcome;
  qualityScore: number | null;
  contextTokens: number | null;
}

export interface CompactionOutcomeBaseline {
  schema: "focusa.compaction_outcome_baseline.v1";
  policyVersion: string;
  policyKey: string;
  route: CompactionPolicyRoute;
  snapshot: CompactionContinuationSnapshot;
}

export type CompactionOutcomeFailureReason =
  | "provider_failure"
  | "scope_drift"
  | "hallucinated_continuity"
  | "workpoint_drift"
  | "missing_evidence"
  | "quality_regression";

export interface CompactionOutcomeEvaluation {
  schema: "focusa.compaction_outcome_evaluation.v1";
  policyVersion: string;
  policyKey: string;
  attemptedRoute: CompactionPolicyRoute;
  disposition: "promote" | "retain" | "quarantine";
  rollbackRequired: boolean;
  rollbackRoute: CompactionPolicyRoute;
  reasons: CompactionOutcomeFailureReason[];
  missingEvidenceRefs: string[];
  qualityDelta: number | null;
  tokenDelta: number | null;
  shadowComparison: {
    baselineQuality: number | null;
    outcomeQuality: number | null;
    regressionTolerance: number;
  };
}

const QUALITY_REGRESSION_TOLERANCE = 0.05;

function finiteScore(value: number | null): number | null {
  if (value === null || !Number.isFinite(value)) return null;
  return Math.min(1, Math.max(0, value));
}

function boundedRefs(refs: string[]): string[] {
  return [...new Set(refs.map((ref) => ref.trim()).filter(Boolean))].sort().slice(0, 128);
}

export function rollbackRouteFor(route: CompactionPolicyRoute): CompactionPolicyRoute {
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
export function evaluateCompactionOutcome(
  baseline: CompactionOutcomeBaseline,
  outcome: CompactionContinuationSnapshot,
  regressionTolerance = QUALITY_REGRESSION_TOLERANCE
): CompactionOutcomeEvaluation {
  const reasons: CompactionOutcomeFailureReason[] = [];
  if (outcome.providerOutcome === "failed") reasons.push("provider_failure");
  if (
    baseline.snapshot.projectRoot !== outcome.projectRoot ||
    baseline.snapshot.sessionId !== outcome.sessionId
  ) {
    reasons.push("scope_drift");
  }
  if (baseline.snapshot.continuityRef !== null && baseline.snapshot.continuityRef !== outcome.continuityRef) {
    reasons.push("hallucinated_continuity");
  }
  if (baseline.snapshot.workpointRef !== null && baseline.snapshot.workpointRef !== outcome.workpointRef) {
    reasons.push("workpoint_drift");
  }

  const outcomeEvidence = new Set(boundedRefs(outcome.evidenceRefs));
  const missingEvidenceRefs = boundedRefs(baseline.snapshot.evidenceRefs).filter(
    (ref) => !outcomeEvidence.has(ref)
  );
  if (missingEvidenceRefs.length > 0) reasons.push("missing_evidence");

  const baselineQuality = finiteScore(baseline.snapshot.qualityScore);
  const outcomeQuality = finiteScore(outcome.qualityScore);
  const qualityDelta =
    baselineQuality === null || outcomeQuality === null
      ? null
      : Math.round((outcomeQuality - baselineQuality) * 1_000_000) / 1_000_000;
  if (qualityDelta !== null && qualityDelta < -Math.abs(regressionTolerance)) {
    reasons.push("quality_regression");
  }

  const tokenDelta =
    baseline.snapshot.contextTokens === null || outcome.contextTokens === null
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
