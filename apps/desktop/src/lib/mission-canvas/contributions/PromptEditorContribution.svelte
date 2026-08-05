<script lang="ts">
  import type { MissionCanvasClient } from '../../../../../../docs/contracts/spec135/mission-canvas-v1/typescript/mission-canvas-client.generated';
  import { MissionCanvasDraftController } from '../draft-controller.svelte';
  import { GeneratedDraftTransport, resolveRecipient } from '../generated-draft-transport';
  import type { OperationBinding, ResolvedContribution, ResolvedWorkspaceProjection } from '../types';

  let {
    contribution,
    projection,
    client,
    onOperation
  }: {
    contribution: ResolvedContribution;
    projection: ResolvedWorkspaceProjection;
    client?: MissionCanvasClient;
    onOperation?: (binding: OperationBinding) => void | Promise<void>;
  } = $props();

  let localContent = $state('');
  let loadedRevision = $state<number>();
  let sending = $state(false);
  let sendError = $state('');
  const controller = $derived(client ? new MissionCanvasDraftController(new GeneratedDraftTransport(client)) : undefined);
  const draftId = $derived(contribution.data_ref.kind === 'canvas_draft' ? contribution.data_ref.ref : undefined);
  const recipientRef = $derived(projection.focused_work_surface_id ?? undefined);
  const bindings = $derived(projection.operation_bindings.filter((binding) =>
    binding.target_contribution_id === contribution.contribution_id
    && contribution.operation_ids.includes(binding.operation_id)
  ));
  const sendBinding = $derived(bindings.find((binding) =>
    binding.enabled && Boolean(binding.authority_ref) && !binding.disabled_reason_ref
  ));
  const draftReady = $derived(controller?.state.kind === 'ready' || controller?.state.kind === 'conflict');
  const sendEnabled = $derived(Boolean(client && controller && draftId && recipientRef && draftReady && sendBinding && onOperation && localContent.trim() && !sending));

  $effect(() => {
    if (!controller || !draftId || !recipientRef) return;
    void controller.load({
      scope: projection.scope,
      attachmentId: projection.scope.attachment_id,
      draftId,
      recipientRef
    });
  });

  $effect(() => {
    const state = controller?.state;
    if (!state || (state.kind !== 'ready' && state.kind !== 'conflict')) return;
    if (loadedRevision === state.draft.draft_revision) return;
    loadedRevision = state.draft.draft_revision;
    localContent = state.kind === 'conflict' ? state.localContent : state.draft.content;
  });

  async function submit(): Promise<void> {
    if (!sendEnabled || !client || !controller || !recipientRef || !sendBinding) return;
    sending = true;
    sendError = '';
    try {
      await resolveRecipient(client, projection.scope, recipientRef);
      await controller.sync(localContent);
      if (controller.state.kind !== 'ready') {
        throw new Error(controller.state.kind === 'conflict' ? controller.state.reason : 'Draft synchronization did not complete.');
      }
      await onOperation?.(sendBinding);
    } catch (error) {
      sendError = error instanceof Error ? error.message : 'Prompt routing failed.';
    } finally {
      sending = false;
    }
  }
</script>

<section class="prompt-editor" aria-label={contribution.accessibility.label}>
  <label for={`prompt-${contribution.contribution_id}`}>{contribution.accessibility.label}</label>
  <textarea
    id={`prompt-${contribution.contribution_id}`}
    value={localContent}
    oninput={(event) => localContent = event.currentTarget.value}
    placeholder={draftId ? 'Steer the focused Work Surface' : 'Canonical draft unavailable'}
    disabled={!draftReady || contribution.authority.read_only || sending}
    aria-describedby={`prompt-status-${contribution.contribution_id}`}
  ></textarea>
  <div class="prompt-actions">
    <span id={`prompt-status-${contribution.contribution_id}`} class:error={Boolean(sendError)}>
      {sendError || (controller?.state.kind === 'conflict' ? 'Draft conflict requires reconciliation.' : recipientRef ? `Recipient: ${recipientRef}` : 'No exact recipient resolved.')}
    </span>
    <button type="button" disabled={!sendEnabled} onclick={() => void submit()}>
      {sending ? 'Routing…' : 'Send'}
    </button>
  </div>
</section>

<style>
  .prompt-editor{display:grid;gap:var(--space-2);height:100%;min-height:0;padding:var(--layout-card-padding);background:var(--color-panel)}
  label{color:var(--color-text);font:var(--type-label)}
  textarea{min-height:6rem;resize:none;border:1px solid var(--color-border);border-radius:var(--radius-control);padding:var(--space-3);background:var(--color-bg);color:var(--color-text);font:var(--type-body)}
  textarea:focus{outline:2px solid var(--color-focus);outline-offset:1px}
  textarea:disabled{opacity:.64}
  .prompt-actions{display:flex;align-items:center;justify-content:space-between;gap:var(--space-3)}
  .prompt-actions span{min-width:0;overflow:hidden;color:var(--color-text-tertiary);font:var(--type-caption);text-overflow:ellipsis;white-space:nowrap}
  .prompt-actions span.error{color:var(--color-error)}
  button{border:0;border-radius:var(--radius-control);padding:var(--space-2) var(--space-4);background:var(--color-accent);color:var(--color-bg);font:var(--type-label);cursor:pointer}
  button:disabled{cursor:not-allowed;opacity:.45}
</style>
