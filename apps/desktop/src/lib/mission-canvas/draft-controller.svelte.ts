import type { CanvasDraftState, ExactScope } from './types';

export interface DraftBinding {
  scope: ExactScope;
  attachmentId: string;
  recipientRef: string;
}

export interface DraftSyncInput extends DraftBinding {
  content: string;
  expectedDraftRevision: number;
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

function sameScope(left: ExactScope, right: ExactScope): boolean {
  return left.project_root === right.project_root
    && left.continuity_id === right.continuity_id
    && left.attachment_id === right.attachment_id
    && left.session_id === right.session_id
    && (left.instance_id ?? null) === (right.instance_id ?? null)
    && (left.working_subpath_id ?? null) === (right.working_subpath_id ?? null);
}

function matchesBinding(draft: CanvasDraftState, binding: DraftBinding): boolean {
  return sameScope(draft.scope, binding.scope)
    && draft.attachment_id === binding.attachmentId
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
        content,
        expectedDraftRevision: draft.draft_revision,
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
