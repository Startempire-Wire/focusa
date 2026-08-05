import { MissionCanvasClient, type MissionCanvasTransport } from '../../../../../docs/contracts/spec135/mission-canvas-v1/typescript/mission-canvas-client.generated';
import type { CanvasDraftState, OperationBinding, ResolvedWorkspaceProjection } from './types';

function clone<T>(value: T): T {
  return structuredClone(value);
}

export function createBrowserPreviewClient(projection: ResolvedWorkspaceProjection): MissionCanvasClient {
  const prompt = projection.eligible_contributions.find((contribution) => contribution.kind === 'prompt_editor');
  const draftId = prompt?.data_ref.kind === 'canvas_draft' ? prompt.data_ref.ref : 'draft:prompt-preview';
  let draft: CanvasDraftState = {
    attachment_id: projection.scope.attachment_id,
    content: '',
    content_sha256: '0'.repeat(64),
    draft_id: draftId,
    draft_revision: 1,
    idempotency_key: 'preview:draft:initial',
    owner: 'canvas_prompt_editor',
    recipient_ref: projection.focused_work_surface_id ?? 'surface:pi',
    scope: clone(projection.scope),
    sync_state: 'synchronized',
    updated_at: new Date(0).toISOString()
  };

  const transport: MissionCanvasTransport = {
    async request<T>(operationId: string, input?: unknown): Promise<T> {
      if (operationId === 'focusa.mission_canvas.draft.get') return clone(draft) as T;
      if (operationId === 'focusa.mission_canvas.draft.sync') {
        const body = (input as { body?: CanvasDraftState } | undefined)?.body;
        if (!body || body.draft_id !== draft.draft_id || body.scope.attachment_id !== draft.scope.attachment_id) {
          throw new Error('Preview draft binding mismatch.');
        }
        draft = { ...clone(body), draft_revision: draft.draft_revision + 1, sync_state: 'synchronized' };
        return clone(draft) as T;
      }
      if (operationId === 'focusa.mission_canvas.recipient.resolve') {
        const body = (input as { body?: { scope?: unknown; recipient_ref?: string } } | undefined)?.body;
        return {
          schema: 'focusa.mission_canvas.recipient_resolution.v1',
          scope: body?.scope,
          recipient_ref: body?.recipient_ref,
          routable: Boolean(body?.recipient_ref)
        } as T;
      }
      throw new Error(`Operation ${operationId} is unavailable in the browser preview runtime.`);
    }
  };

  return new MissionCanvasClient(transport);
}

export async function executeBrowserPreviewOperation(
  binding: OperationBinding,
  projection: ResolvedWorkspaceProjection
): Promise<void> {
  if (binding.operation_id !== 'focusa.agent_execution.prompt'
    || binding.target_contribution_id !== 'contribution:prompt-editor'
    || !binding.enabled
    || !binding.authority_ref) {
    throw new Error('Preview operation binding rejected.');
  }
  window.dispatchEvent(new CustomEvent('focusa-preview-operation', {
    detail: {
      operation_id: binding.operation_id,
      target_contribution_id: binding.target_contribution_id,
      projection_revision: projection.projection_revision
    }
  }));
}
