import type { MissionCanvasClient } from '../../../../../docs/contracts/spec135/mission-canvas-v1/typescript/mission-canvas-client.generated';
import type { CanvasDraftState, ExactScope } from './types';
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
    return requireDraft(await this.client.draftGet({ params: { path: { draft_id: binding.draftId } } }));
  }

  async sync(input: DraftSyncInput): Promise<CanvasDraftState> {
    if (input.baseDraft.draft_revision !== input.expectedDraftRevision) {
      throw new Error('Draft revision changed before synchronization.');
    }
    const body: CanvasDraftState = {
      ...input.baseDraft,
      scope: input.scope,
      attachment_id: input.attachmentId,
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
    return requireDraft(await this.client.draftSync({ body }));
  }
}

export interface RecipientResolution {
  schema: 'focusa.mission_canvas.recipient_resolution.v1';
  scope: ExactScope;
  recipient_ref: string;
  routable: boolean;
}

export async function resolveRecipient(
  client: MissionCanvasClient,
  scope: ExactScope,
  recipientRef: string
): Promise<RecipientResolution> {
  const value = await client.recipientResolve({ body: { scope, recipient_ref: recipientRef } });
  if (!value || typeof value !== 'object') throw new Error('Invalid recipient resolution response.');
  const candidate = value as Partial<RecipientResolution>;
  if (candidate.schema !== 'focusa.mission_canvas.recipient_resolution.v1'
    || candidate.recipient_ref !== recipientRef
    || candidate.routable !== true) {
    throw new Error('Recipient is not routable for this exact scope.');
  }
  return candidate as RecipientResolution;
}
