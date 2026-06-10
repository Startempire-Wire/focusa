// Workpoint actions — write-side control surface for the menubar (focusa-ui0y).
//
// Three actions that let the operator drive the active Workpoint on the VPS
// daemon from the Mac menubar:
//   - checkpoint:    persist the current Workpoint state to the ledger
//   - resume:        re-render the active Workpoint packet from the daemon
//   - linkEvidence:  attach a proof to the active Workpoint
//
// Each action accepts the runtime snapshot (for scope) and the active
// workpoint fields (for the request payload). On success, callers should
// refresh the runtime store. On failure, callers should surface a toast.

import { postJson, fetchJson, summarizeError } from '$lib/api';
import { toastStore } from '$lib/stores/toast.svelte';

export interface WorkpointScope {
  projectRoot: string;
  continuityId: string;
  sessionId?: string;
  workItemId?: string;
}

export interface WorkpointSnapshot {
  workpoint_id?: string;
  work_item_id?: string;
  mission?: string;
  current_action?: string;
  next_action?: string;
  next_slice?: string;
  source_turn_id?: string;
  verified_evidence?: string[];
  blockers?: string[];
  do_not_drift?: string[];
}

export interface CheckpointArgs {
  scope: WorkpointScope;
  workpoint: WorkpointSnapshot;
  reason?: string;
  canonical?: boolean;
}

export interface ResumeArgs {
  scope: WorkpointScope;
  workpoint_id?: string;
  mode?: 'compact_prompt' | 'summary' | 'operator_summary' | 'full_json';
}

export interface LinkEvidenceArgs {
  scope: WorkpointScope;
  workpoint_id: string;
  target_ref: string;
  result: string;
  evidence_ref: string;
}

interface ActionOk {
  ok: true;
  status: string;
  workpoint_id?: string;
  next_tools?: string[];
  tool_result_v1?: any;
}

interface ActionErr {
  ok: false;
  failure_class: string;
  message: string;
  status?: number;
}

function asError(e: unknown): ActionErr {
  const anyErr = e as any;
  return {
    ok: false,
    failure_class: anyErr?.failure_class || 'request_failed',
    message: summarizeError(e),
    status: anyErr?.status,
  };
}

function createWorkpointActions() {
  let busy = $state<string | null>(null);
  let lastError = $state<ActionErr | null>(null);

  async function checkpoint(args: CheckpointArgs): Promise<ActionOk | ActionErr> {
    busy = 'checkpoint';
    lastError = null;
    try {
      const body: Record<string, unknown> = {
        project_root: args.scope.projectRoot,
        continuity_id: args.scope.continuityId,
        session_id: args.scope.sessionId,
        work_item_id: args.scope.workItemId || args.workpoint.work_item_id,
        workpoint_id: args.workpoint.workpoint_id,
        mission: args.workpoint.mission || args.workpoint.current_action || 'menubar-checkpoint',
        next_action: args.workpoint.next_action || args.workpoint.next_slice,
        next_slice: args.workpoint.next_slice || args.workpoint.next_action,
        source_turn_id: args.workpoint.source_turn_id,
        canonical: args.canonical ?? true,
        promote: true,
        checkpoint_reason: args.reason || 'menubar_checkpoint',
      };
      const result = await postJson<any>('/v1/workpoint/checkpoint', body, 8_000);
      toastStore.ok('Workpoint checkpointed', result?.workpoint_id || args.workpoint.workpoint_id);
      return {
        ok: true,
        status: result?.status || 'completed',
        workpoint_id: result?.workpoint_id || args.workpoint.workpoint_id,
        next_tools: result?.next_tools,
        tool_result_v1: result?.details?.tool_result_v1 ?? result?.tool_result_v1,
      };
    } catch (e) {
      const err = asError(e);
      lastError = err;
      toastStore.err('Checkpoint failed', `${err.failure_class}: ${err.message}`);
      return err;
    } finally {
      busy = null;
    }
  }

  async function resume(args: ResumeArgs): Promise<ActionOk | ActionErr> {
    busy = 'resume';
    lastError = null;
    try {
      const body: Record<string, unknown> = {
        project_root: args.scope.projectRoot,
        continuity_id: args.scope.continuityId,
        session_id: args.scope.sessionId,
        workpoint_id: args.workpoint_id,
        mode: args.mode || 'compact_prompt',
      };
      const result = await postJson<any>('/v1/workpoint/resume', body, 8_000);
      toastStore.ok('Workpoint re-rendered', args.workpoint_id || '(no-id)');
      return {
        ok: true,
        status: result?.status || 'completed',
        workpoint_id: result?.details?.workpoint_id || result?.workpoint_id || args.workpoint_id,
        next_tools: result?.next_tools,
        tool_result_v1: result?.details?.tool_result_v1 ?? result?.tool_result_v1,
      };
    } catch (e) {
      const err = asError(e);
      lastError = err;
      toastStore.err('Resume failed', `${err.failure_class}: ${err.message}`);
      return err;
    } finally {
      busy = null;
    }
  }

  async function linkEvidence(args: LinkEvidenceArgs): Promise<ActionOk | ActionErr> {
    busy = 'linkEvidence';
    lastError = null;
    try {
      const body: Record<string, unknown> = {
        project_root: args.scope.projectRoot,
        continuity_id: args.scope.continuityId,
        session_id: args.scope.sessionId,
        workpoint_id: args.workpoint_id,
        target_ref: args.target_ref,
        result: args.result,
        evidence_ref: args.evidence_ref,
      };
      const result = await postJson<any>('/v1/workpoint/evidence/link', body, 8_000);
      toastStore.ok('Evidence linked', `${args.evidence_ref.slice(0, 16)}…`);
      return {
        ok: true,
        status: result?.status || 'completed',
        workpoint_id: args.workpoint_id,
        next_tools: result?.next_tools,
        tool_result_v1: result?.details?.tool_result_v1 ?? result?.tool_result_v1,
      };
    } catch (e) {
      const err = asError(e);
      lastError = err;
      toastStore.err('Link evidence failed', `${err.failure_class}: ${err.message}`);
      return err;
    } finally {
      busy = null;
    }
  }

  return {
    get busy() { return busy; },
    get lastError() { return lastError; },
    checkpoint,
    resume,
    linkEvidence,
  };
}

export const workpointActions = createWorkpointActions();
