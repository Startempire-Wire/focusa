export interface CompactionAuthorityEvent {
  schema: "focusa.auto_compaction_event.v1";
  kind: string;
  epoch_id?: string;
  coordinator_state?: string;
  policy_selection?: unknown;
  pressure_telemetry?: unknown;
  [key: string]: unknown;
}

export interface CompactionAuthorityProjection {
  schema: "focusa.compaction_authority_projection.v1";
  eventCount: number;
  lastKind: string | null;
  lastEpochId: string | null;
  coordinatorState: string;
  lastPolicySelection: unknown | null;
  lastPressureTelemetry: unknown | null;
  lastOutcomeEvaluation: unknown | null;
  quarantinedPolicyKeys: string[];
  rollbackRoute: string | null;
  recoveryRequired: boolean;
}

export function emptyCompactionAuthorityProjection(): CompactionAuthorityProjection {
  return {
    schema: "focusa.compaction_authority_projection.v1",
    eventCount: 0,
    lastKind: null,
    lastEpochId: null,
    coordinatorState: "idle",
    lastPolicySelection: null,
    lastPressureTelemetry: null,
    lastOutcomeEvaluation: null,
    quarantinedPolicyKeys: [],
    rollbackRoute: null,
    recoveryRequired: false,
  };
}

/** Deterministic ordered fold; malformed custom entries have no authority. */
export function reduceCompactionAuthorityEvents(
  events: unknown[],
  initial = emptyCompactionAuthorityProjection()
): CompactionAuthorityProjection {
  return events.reduce<CompactionAuthorityProjection>((projection, candidate) => {
    if (!candidate || typeof candidate !== "object") return projection;
    const event = candidate as Record<string, any>;
    if (event.schema !== "focusa.auto_compaction_event.v1") return projection;
    const kind = String(event.kind || "").trim();
    if (!kind) return projection;
    const coordinatorState = String(event.coordinator_state || projection.coordinatorState);
    const evaluation =
      event.outcome_evaluation?.schema === "focusa.compaction_outcome_evaluation.v1"
        ? event.outcome_evaluation
        : null;
    const policyKey = String(evaluation?.policyKey || "").trim();
    const rollbackRequired = evaluation?.rollbackRequired === true;
    const priorQuarantinedPolicyKeys = projection.quarantinedPolicyKeys ?? [];
    const quarantinedPolicyKeys =
      rollbackRequired && policyKey
        ? [...new Set([...priorQuarantinedPolicyKeys, policyKey])].sort()
        : priorQuarantinedPolicyKeys;
    const failed =
      rollbackRequired ||
      /failed|blocked/.test(coordinatorState) ||
      /failed|rejected|rollback_required/.test(kind);
    const recovered = /verified|complete|projection_rehydrated|policy_promoted/.test(kind);
    return {
      schema: "focusa.compaction_authority_projection.v1",
      eventCount: projection.eventCount + 1,
      lastKind: kind,
      lastEpochId: String(event.epoch_id || projection.lastEpochId || "").trim() || null,
      coordinatorState,
      lastPolicySelection: event.policy_selection ?? projection.lastPolicySelection,
      lastPressureTelemetry: event.pressure_telemetry ?? projection.lastPressureTelemetry,
      lastOutcomeEvaluation: evaluation ?? projection.lastOutcomeEvaluation,
      quarantinedPolicyKeys,
      rollbackRoute: rollbackRequired
        ? String(evaluation?.rollbackRoute || "").trim() || projection.rollbackRoute
        : projection.rollbackRoute,
      recoveryRequired: failed ? true : recovered ? false : projection.recoveryRequired,
    };
  }, initial);
}
