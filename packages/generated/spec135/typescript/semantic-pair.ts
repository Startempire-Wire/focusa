/** Generated Spec144 semantic-pair surface contract. Do not weaken status truth. */
export const semanticPairTruthStates = [
  "supported", "schema_only",
  "pack_missing",
  "migration_required",
  "verification_required",
  "verification_blocked",
  "operator_required",
  "unsupported_future_definition",
  "writer_blocked",
  "degraded",
  "stale",
  "conflicted",
  "quarantined",
] as const;

export type SemanticPairTruthState = (typeof semanticPairTruthStates)[number];
export type SemanticPairCapability = "read" | "mutate";

export interface SemanticPairScope {
  project_root: string;
  continuity_id: string;
}

export interface SemanticPairObligation {
  obligation_id: string;
  statement: string;
  source_refs: string[];
  status: "open" | "satisfied" | "waived" | "blocked";
}

export interface SemanticPairFinding {
  finding_id: string;
  obligation_id?: string;
  severity: "info" | "warning" | "error" | "critical";
  verdict: "pass" | "fail" | "unknown" | "disputed";
  summary: string;
  evidence_refs: string[];
}

export interface SemanticPairSettlement {
  status: "unsettled" | "accepted" | "rejected" | "disputed" | "superseded";
  verdict?: "pass" | "fail" | "blocked";
  settled_at?: string;
  receipt_refs: string[];
}

export interface SemanticPairReplay {
  status: "not_requested" | "pending" | "replaying" | "complete" | "failed";
  generation: number;
  last_event_sequence?: number;
  receipt_refs: string[];
}

export interface SemanticPairRecovery {
  required: boolean;
  state?: SemanticPairTruthState;
  next_operation?: SemanticPairOperationId;
  reason?: string;
}

export interface SemanticPairPortfolioItem {
  pair_id: string;
  title: string;
  state: SemanticPairTruthState;
  obligations: SemanticPairObligation[];
  findings: SemanticPairFinding[];
  settlement: SemanticPairSettlement;
  replay: SemanticPairReplay;
  recovery: SemanticPairRecovery;
  evidence_refs: string[];
  receipt_refs: string[];
  updated_at?: string;
}

export interface SemanticPairPortfolio {
  schema: "focusa.semantic_pair.portfolio.v1";
  scope: SemanticPairScope;
  items: SemanticPairPortfolioItem[];
  state: SemanticPairTruthState;
  stale: boolean;
  conflicted: boolean;
  quarantined: boolean;
}

export const semanticPairOperationIds = [
  "semantic.integrity.status", "semantic.integrity.registry",
  "semantic.integrity.artifact.list", "semantic.integrity.artifact.get",
  "semantic.integrity.validate", "semantic.integrity.reason.preview",
  "semantic.integrity.reason.explain", "semantic.integrity.receipt.get",
  "semantic_pair.create", "semantic_pair.get", "semantic_pair.pause",
  "semantic_pair.resume", "semantic_pair.cancel", "semantic_pair.contract.preview",
  "semantic_pair.contract.commit", "semantic_pair.builder.start", "semantic_pair.builder.claim",
  "semantic_pair.builder.respond", "semantic_pair.builder.repair", "semantic_pair.snapshot.freeze",
  "semantic_pair.snapshot.get", "semantic_pair.obligations.compile",
  "semantic_pair.verification.plan.preview", "semantic_pair.verification.plan.commit",
  "semantic_pair.verify.start", "semantic_pair.verify.findings", "semantic_pair.verify.verdict",
  "semantic_pair.finding.respond", "semantic_pair.finding.resolve",
  "semantic_pair.settlement.preview", "semantic_pair.settlement.commit",
  "semantic_pair.receipt.get", "semantic_pair.replay", "semantic_pair.eval",
  "semantic_pair.migration.status", "semantic_pair.migration.run",
  "semantic_pair.rollback.preview", "semantic_pair.rollback.commit",
  "vertical.bundle.validate", "vertical.bundle.preview", "vertical.bundle.activate",
  "vertical.bundle.conformance", "semantic.reflex.visibility",
] as const;

export type SemanticPairOperationId = (typeof semanticPairOperationIds)[number];

export interface SemanticPairAction {
  operation_id: SemanticPairOperationId;
  capability: SemanticPairCapability;
  available: boolean;
  disabled_reason?: SemanticPairTruthState | "read_only_surface";
  idempotency_key?: string;
  confirmation?: string;
}

export interface SemanticPairActionRequest {
  operation_id: SemanticPairOperationId;
  scope: SemanticPairScope;
  pair_id?: string;
  idempotency_key?: string;
  confirmation?: string;
  payload?: Record<string, unknown>;
}
