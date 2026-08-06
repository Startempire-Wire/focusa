import type { MissionCanvasClient } from '../../../../../docs/contracts/spec135/mission-canvas-v1/typescript/mission-canvas-client.generated';
import type { AttachmentKey, CanvasDraftState, RecipientResolution, WorkstreamAuthorityContext } from './types';
import { validateMissionCanvasContract } from '../../../../../docs/contracts/spec135/mission-canvas-v1/typescript/mission-canvas-validators.generated';
import type { DraftBinding, DraftSyncInput, DraftTransport } from './draft-controller.svelte';

function requireDraft(value: unknown): CanvasDraftState {
  const result = validateMissionCanvasContract('CanvasDraftState', value);
  if (!result.valid) throw new Error(`Invalid CanvasDraftState response: ${result.errors.join(', ')}`);
  return value as CanvasDraftState;
}

async function sha256(content: string): Promise<string> {
  const bytes = new TextEncoder().encode(content);
  const digest = await crypto.subtle.digest('SHA-256', bytes);
  return [...new Uint8Array(digest)].map((byte) => byte.toString(16).padStart(2, '0')).join('');
}

export class GeneratedDraftTransport implements DraftTransport {
  constructor(private readonly client: MissionCanvasClient) {}

  async get(binding: DraftBinding): Promise<CanvasDraftState> {
    return requireDraft(await this.client.draftGet({ ...binding, draft_id: binding.draftId }));
  }

  async sync(input: DraftSyncInput): Promise<CanvasDraftState> {
    if (input.baseDraft.draft_revision !== input.expectedDraftRevision) {
      throw new Error('Draft revision changed before synchronization.');
    }
    const attachment: AttachmentKey = input.attachment;
    const body: CanvasDraftState = {
      ...input.baseDraft,
      workstream: input.workstream,
      continuity_id: input.continuity_id ?? attachment.continuity_id ?? null,
      attachment,
      workspace_binding_id: input.workspace_binding_id ?? attachment.workspace_binding_id,
      runtime_object: input.runtime_object ?? null,
      work_surface_id: input.work_surface_id ?? null,
      draft_id: input.draftId,
      recipient_ref: input.recipientRef,
      owner: 'canvas_prompt_editor',
      content: input.content,
      content_sha256: await sha256(input.content),
      draft_revision: input.expectedDraftRevision,
      idempotency_key: input.idempotencyKey,
      selection_start: input.selectionStart,
      selection_end: input.selectionEnd,
      sync_state: 'canvas_newer',
      updated_at: new Date().toISOString()
    };
    return requireDraft(await this.client.draftSync(body));
  }
}

export async function resolveRecipient(
  client: MissionCanvasClient,
  authority: WorkstreamAuthorityContext,
  recipientRef: string
): Promise<RecipientResolution> {
  const value = await client.recipientResolve({ ...authority, recipient_ref: recipientRef });
  if (value.schema !== 'focusa.mission_canvas.recipient_resolution.v1'
    || value.recipient_ref !== recipientRef
    || value.routable !== true) {
    throw new Error('Recipient is not routable for this exact Workstream.');
  }
  return value;
}
