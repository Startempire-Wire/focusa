<script lang="ts">
  import type { DraftControllerState } from '../draft-controller.svelte';
  import type { ResolvedContribution } from '../types';

  let {
    contribution,
    draftState,
    sendAuthorized,
    onEdit,
    onSend
  }: {
    contribution: ResolvedContribution;
    draftState: DraftControllerState;
    sendAuthorized: boolean;
    onEdit: (content: string, selectionStart?: number, selectionEnd?: number) => void;
    onSend: (content: string, recipientRef: string) => void;
  } = $props();

  const bound = $derived(
    draftState.kind === 'ready' || draftState.kind === 'saving' || draftState.kind === 'conflict'
      ? draftState
      : undefined
  );
  const content = $derived(
    bound?.kind === 'saving' || bound?.kind === 'conflict' ? bound.localContent : bound?.draft.content ?? ''
  );
  const recipientRef = $derived(bound?.binding.recipientRef);
  const editable = $derived(bound?.kind === 'ready' || bound?.kind === 'conflict');

  function edit(event: Event): void {
    const input = event.currentTarget as HTMLTextAreaElement;
    onEdit(input.value, input.selectionStart, input.selectionEnd);
  }
</script>

{#if bound}
  <section class="prompt-editor" aria-label={contribution.accessibility.label}>
    <header>
      <strong>{contribution.accessibility.label}</strong>
      <span>{recipientRef}</span>
    </header>
    <textarea
      aria-label={contribution.accessibility.description ?? contribution.accessibility.label}
      value={content}
      disabled={!editable}
      oninput={edit}
    ></textarea>
    {#if sendAuthorized && recipientRef}
      <button type="button" disabled={!editable || content.trim().length === 0} onclick={() => onSend(content, recipientRef)}>
        Send
      </button>
    {/if}
    {#if bound.kind === 'conflict'}
      <p role="alert">{bound.reason}</p>
    {/if}
  </section>
{/if}

<style>
  .prompt-editor{display:grid;grid-template-columns:minmax(0,1fr) auto;gap:var(--space-2);align-items:end;min-width:0;padding:var(--layout-card-padding);border:1px solid var(--color-border);border-radius:var(--radius-card);background:var(--color-panel)}
  header{grid-column:1/-1;display:flex;justify-content:space-between;gap:var(--space-3);min-width:0;color:var(--color-text)}
  header span{overflow:hidden;color:var(--color-text-tertiary);font:var(--type-caption);text-overflow:ellipsis;white-space:nowrap}
  textarea{min-height:5rem;resize:vertical;border:1px solid var(--color-border);border-radius:var(--radius-control);padding:var(--space-3);background:var(--color-bg);color:var(--color-text);font:var(--type-body)}
  button{border:0;border-radius:var(--radius-control);padding:var(--space-2) var(--space-4);background:var(--color-accent);color:var(--color-bg);font:inherit;font-weight:700;cursor:pointer}
  button:disabled{opacity:.45;cursor:not-allowed}
  p{grid-column:1/-1;margin:0;color:var(--color-warning);font:var(--type-caption)}
</style>
