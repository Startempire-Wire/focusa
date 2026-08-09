import type { OperationBinding } from './types';

/**
 * GEN-003 — five resumable generated C.R.I.S.T. stage surfaces.
 *
 * context → role → interview → spec → tasks. Every stage is a real generated
 * surface (never CLI-only, static-form-only, transcript-only, or mock-only):
 * each has a primary canonical action, autosave/resume via the draft path,
 * explicit recovery states, and a terminal fallback truth — the canonical
 * projection. UXP/UFI may alter presentation only; the surface never owns
 * authority, required proof, or safety.
 */

export type CristStageId = 'context' | 'role' | 'interview' | 'spec' | 'tasks';

export const CRIST_STAGE_ORDER: readonly CristStageId[] = ['context', 'role', 'interview', 'spec', 'tasks'];

export type RecoveryState = 'error' | 'retry' | 'resume';

export interface CristStageSurface {
  stage_id: CristStageId;
  /** Primary action = a canonical operation binding id, resolved by the projection. */
  primary_action_operation: string;
  /** Autosave/resume target: the draft id for this stage's working content. */
  autosave_draft_id: string;
  /** Recovery states a surface can enter without inventing authority. */
  recovery_states: readonly RecoveryState[];
  /** Terminal fallback truth: the canonical projection revision remains the
   *  authoritative source; the surface is presentation-only. */
  terminal_fallback_truth: 'canonical_projection';
  /** Resume support: a resumable stage restores from its draft, never from a
   *  local snapshot that could become canonical. */
  resumable: boolean;
  /** Stages that may run after this one (canonical ordering, not enforcement). */
  next_stages: readonly CristStageId[];
}

export const CRIST_STAGE_SURFACES: readonly CristStageSurface[] = [
  {
    stage_id: 'context',
    primary_action_operation: 'operation:genesis.context.advance',
    autosave_draft_id: 'draft:genesis:context',
    recovery_states: ['error', 'retry', 'resume'],
    terminal_fallback_truth: 'canonical_projection',
    resumable: true,
    next_stages: ['role']
  },
  {
    stage_id: 'role',
    primary_action_operation: 'operation:genesis.role.advance',
    autosave_draft_id: 'draft:genesis:role',
    recovery_states: ['error', 'retry', 'resume'],
    terminal_fallback_truth: 'canonical_projection',
    resumable: true,
    next_stages: ['interview']
  },
  {
    stage_id: 'interview',
    primary_action_operation: 'operation:genesis.interview.advance',
    autosave_draft_id: 'draft:genesis:interview',
    recovery_states: ['error', 'retry', 'resume'],
    terminal_fallback_truth: 'canonical_projection',
    resumable: true,
    next_stages: ['spec']
  },
  {
    stage_id: 'spec',
    primary_action_operation: 'operation:genesis.spec.advance',
    autosave_draft_id: 'draft:genesis:spec',
    recovery_states: ['error', 'retry', 'resume'],
    terminal_fallback_truth: 'canonical_projection',
    resumable: true,
    next_stages: ['tasks']
  },
  {
    stage_id: 'tasks',
    primary_action_operation: 'operation:genesis.tasks.commit',
    autosave_draft_id: 'draft:genesis:tasks',
    recovery_states: ['error', 'retry', 'resume'],
    terminal_fallback_truth: 'canonical_projection',
    resumable: true,
    next_stages: []
  }
];

export function cristStageSurface(stageId: CristStageId): CristStageSurface {
  const surface = CRIST_STAGE_SURFACES.find((candidate) => candidate.stage_id === stageId);
  if (!surface) throw new Error(`unknown C.R.I.S.T. stage: ${stageId}`);
  return surface;
}

/**
 * Resolve a stage's primary action through the canonical permission
 * projection (GEN-002): the operation must be registered, enabled, and
 * authority-bound on the stage's contribution.
 */
export function stagePrimaryAction(
  stage: CristStageSurface,
  bindings: readonly OperationBinding[],
  stageContributionId: string
): OperationBinding | undefined {
  return bindings.find((binding) =>
    binding.operation_id === stage.primary_action_operation
    && binding.target_contribution_id === stageContributionId
    && binding.enabled
    && Boolean(binding.authority_ref)
    && !binding.disabled_reason_ref
  );
}
