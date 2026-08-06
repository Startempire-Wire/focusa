import { sameWorkstreamAuthority as sameScope } from './exact-scope';
import type { AttachmentKey, CanvasDraftState, WorkstreamAuthorityContext } from './types';

export type DraftBinding = Omit<WorkstreamAuthorityContext, 'attachment'> & {
  attachment: AttachmentKey;
  draftId: string;
  recipientRef: string;
};

export interface DraftSyncInput extends DraftBinding {
  baseDraft: CanvasDraftState;
  content: string;
  expectedDraftRevision: number;
  idempotencyKey: string;
  selectionStart?: number;
  selectionEnd?: number;
}

export interface DraftTransport {
  get(binding: DraftBinding): Promise<CanvasDraftState>;
  sync(input: DraftSyncInput): Promise<CanvasDraftState>;
}

export type DraftControllerState =
  | { kind: 'unbound' }
  | { kind: 'loading'; binding: DraftBinding }
  | { kind: 'ready'; binding: DraftBinding; draft: CanvasDraftState }
  | { kind: 'saving'; binding: DraftBinding; draft: CanvasDraftState; localContent: string }
  | { kind: 'conflict'; binding: DraftBinding; draft: CanvasDraftState; localContent: string; reason: string }
  | { kind: 'error'; binding?: DraftBinding; reason: string };

function matchesBinding(draft: CanvasDraftState, binding: DraftBinding): boolean {
  return sameScope(draft, binding)
    && draft.draft_id === binding.draftId
    && draft.recipient_ref === binding.recipientRef;
}

export class MissionCanvasDraftController {
  state = $state<DraftControllerState>({ kind: 'unbound' });
  #generation = 0;

  constructor(private readonly transport: DraftTransport) {}

  async load(binding: DraftBinding): Promise<void> {
    const generation = ++this.#generation;
    this.state = { kind: 'loading', binding };
    try {
      const draft = await this.transport.get(binding);
      if (generation !== this.#generation) return;
      if (!matchesBinding(draft, binding)) {
        this.state = { kind: 'error', binding, reason: 'foreign_draft_binding' };
        return;
      }
      this.state = { kind: 'ready', binding, draft };
    } catch (error) {
      if (generation === this.#generation) this.state = { kind: 'error', binding, reason: errorMessage(error) };
    }
  }

  async sync(content: string, selectionStart?: number, selectionEnd?: number): Promise<void> {
    if (this.state.kind !== 'ready' && this.state.kind !== 'conflict') return;
    const { binding, draft } = this.state;
    const generation = ++this.#generation;
    this.state = { kind: 'saving', binding, draft, localContent: content };

    try {
      const next = await this.transport.sync({
        ...binding,
        baseDraft: draft,
        content,
        expectedDraftRevision: draft.draft_revision,
        idempotencyKey: crypto.randomUUID(),
        selectionStart,
        selectionEnd
      });
      if (generation !== this.#generation) return;
      if (!matchesBinding(next, binding)) {
        this.state = { kind: 'conflict', binding, draft, localContent: content, reason: 'foreign_draft_binding' };
        return;
      }
      if (next.draft_revision < draft.draft_revision) {
        this.state = { kind: 'conflict', binding, draft, localContent: content, reason: 'draft_revision_regressed' };
        return;
      }
      if (next.sync_state === 'conflict') {
        this.state = { kind: 'conflict', binding, draft: next, localContent: content, reason: next.conflict_ref ?? 'draft_conflict' };
        return;
      }
      this.state = { kind: 'ready', binding, draft: next };
    } catch (error) {
      if (generation === this.#generation) {
        this.state = { kind: 'conflict', binding, draft, localContent: content, reason: errorMessage(error) };
      }
    }
  }

  clear(): void {
    this.#generation += 1;
    this.state = { kind: 'unbound' };
  }
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : 'draft_operation_failed';
}
