import type { ResolvedContribution, ResolvedWorkspaceProjection, OperationBinding } from './types';

/**
 * GEN-002 — governed Result, Evidence, Receipt, and generated delta.
 *
 * Generated UI can ONLY invoke operations that are registered in the
 * canonical projection AND permission-projected (enabled, authority_ref
 * present, no disabled_reason). A component cannot call an unregistered
 * operation or bypass the permission projection — the projection is the
 * single authority for what may run. Presentation (UXP/UFI) can alter
 * presentation only, never authority, required proof, or safety.
 */

export interface GovernedResult<T = unknown> {
  schema: 'focusa.generated_surface.governed_result.v1';
  ok: boolean;
  value?: T;
  error_ref?: string;
  receipt_ref?: string;
  evidence_refs: readonly string[];
}

export interface GovernedEvidence {
  schema: 'focusa.generated_surface.governed_evidence.v1';
  evidence_ref: string;
  target_ref: string;
  result: string;
}

export interface GovernedReceipt {
  schema: 'focusa.generated_surface.governed_receipt.v1';
  receipt_id: string;
  operation_id: string;
  authority_ref: string;
  accepted: boolean;
}

export interface GeneratedDelta {
  schema: 'focusa.generated_surface.generated_delta.v1';
  contribution_id: string;
  operation_id: string;
  revision: number;
  happened_at: string;
  summary: string;
}

export type PermissionVerdict =
  | { permitted: true; binding: OperationBinding }
  | { permitted: false; reason: 'unregistered_operation' | 'wrong_contribution' | 'disabled_by_projection' | 'missing_authority_ref' | 'blocked_reason' };

/**
 * Resolve the permission projection for an operation on a contribution.
 * This is the ONLY gate a generated component may use before acting.
 */
export function resolveOperationPermission(
  projection: ResolvedWorkspaceProjection,
  contribution: ResolvedContribution,
  operationId: string
): PermissionVerdict {
  const binding = projection.operation_bindings.find(
    (candidate) => candidate.operation_id === operationId
      && candidate.target_contribution_id === contribution.contribution_id
  );
  if (!binding) return { permitted: false, reason: 'unregistered_operation' };
  if (!binding.enabled) return { permitted: false, reason: 'disabled_by_projection' };
  if (!binding.authority_ref) return { permitted: false, reason: 'missing_authority_ref' };
  if (binding.disabled_reason_ref) return { permitted: false, reason: 'blocked_reason' };
  return { permitted: true, binding };
}

/** Invoke a generated action strictly through the permission projection. */
export async function invokeRegisteredOperation(
  projection: ResolvedWorkspaceProjection,
  contribution: ResolvedContribution,
  operationId: string,
  executor: (binding: OperationBinding) => Promise<GovernedResult>
): Promise<GovernedResult> {
  const verdict = resolveOperationPermission(projection, contribution, operationId);
  if (!verdict.permitted) {
    return {
      schema: 'focusa.generated_surface.governed_result.v1',
      ok: false,
      error_ref: `permission_denied:${verdict.reason}`,
      evidence_refs: []
    };
  }
  return executor(verdict.binding);
}

/** Append a durable generated delta (canonical events, never relabeled text). */
export function emitGeneratedDelta(
  delta: GeneratedDelta,
  onDelta: (delta: GeneratedDelta) => void
): GovernedReceipt {
  onDelta(delta);
  return {
    schema: 'focusa.generated_surface.governed_receipt.v1',
    receipt_id: `receipt:${delta.contribution_id}:${delta.operation_id}:${delta.revision}`,
    operation_id: delta.operation_id,
    authority_ref: 'projection',
    accepted: true
  };
}
