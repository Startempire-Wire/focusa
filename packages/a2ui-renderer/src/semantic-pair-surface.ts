import {
  semanticPairOperationIds,
  type SemanticPairAction,
  type SemanticPairOperationId,
  type SemanticPairPortfolio,
  type SemanticPairTruthState,
} from "@focusa/spec135-client";

export type SemanticPairSurfaceMode = "interactive" | "read_only";

const READ_OPERATIONS = new Set<SemanticPairOperationId>([
  "semantic_pair.get", "semantic_pair.contract.preview", "semantic_pair.snapshot.get",
  "semantic_pair.verification.plan.preview", "semantic_pair.verify.findings",
  "semantic_pair.verify.verdict", "semantic_pair.settlement.preview",
  "semantic_pair.receipt.get", "semantic_pair.eval", "semantic_pair.migration.status",
  "semantic_pair.rollback.preview",
]);

const BLOCKING_STATES = new Set<SemanticPairTruthState>([
  "unsupported_future_definition", "writer_blocked", "conflicted", "quarantined",
]);

/** Every registered pair operation remains visible; unsupported mutations are disabled, never hidden. */
export function semanticPairActions(
  mode: SemanticPairSurfaceMode,
  state: SemanticPairTruthState,
): SemanticPairAction[] {
  return semanticPairOperationIds.map((operation_id) => {
    const read = READ_OPERATIONS.has(operation_id);
    const surfaceBlocked = mode === "read_only" && !read;
    const stateBlocked = !read && BLOCKING_STATES.has(state);
    return {
      operation_id,
      capability: read ? "read" : "mutate",
      available: !surfaceBlocked && !stateBlocked,
      disabled_reason: surfaceBlocked ? "read_only_surface" : stateBlocked ? state : undefined,
    };
  });
}

export interface SemanticPairSurfaceModel {
  portfolio: SemanticPairPortfolio;
  actions: SemanticPairAction[];
  announcement: string;
}

export function semanticPairSurfaceModel(
  portfolio: SemanticPairPortfolio,
  mode: SemanticPairSurfaceMode,
): SemanticPairSurfaceModel {
  return {
    portfolio,
    actions: semanticPairActions(mode, portfolio.state),
    announcement: `Semantic Pair ${portfolio.state}; ${portfolio.items.length} portfolio items`,
  };
}
