<script lang="ts">
  import type { OperationBinding } from './types';

  let {
    binding,
    subjectLabel,
    onConfirm,
    onCancel
  }: {
    binding: OperationBinding;
    subjectLabel: string;
    onConfirm: () => void;
    onCancel: () => void;
  } = $props();
</script>

<div class="backdrop" role="presentation" onclick={(event) => event.currentTarget === event.target && onCancel()}>
  <div class="confirmation" role="alertdialog" aria-modal="true" aria-labelledby="operation-confirmation-title">
    <span class="requirement">{binding.confirmation === 'preview' ? 'Preview required' : 'Confirmation required'}</span>
    <h2 id="operation-confirmation-title">{subjectLabel}</h2>
    <dl>
      <div><dt>Operation</dt><dd>{binding.operation_id}</dd></div>
      <div><dt>Authority</dt><dd>{binding.authority_ref}</dd></div>
    </dl>
    <div class="actions">
      <button type="button" class="secondary" onclick={onCancel}>{binding.confirmation === 'preview' ? 'Close' : 'Cancel'}</button>
      {#if binding.confirmation === 'explicit'}
        <button type="button" class="primary" onclick={onConfirm}>Confirm</button>
      {/if}
    </div>
  </div>
</div>

<style>
  .backdrop{position:absolute;z-index:10;inset:0;display:grid;place-items:center;padding:var(--space-5);background:color-mix(in srgb,var(--color-bg) 72%,transparent)}
  .confirmation{display:grid;gap:var(--space-4);width:min(32rem,100%);padding:var(--layout-card-padding-roomy);border:1px solid var(--color-border-strong);border-radius:var(--radius-panel);background:var(--color-elevated);box-shadow:var(--shadow-popover);color:var(--color-text)}
  .requirement{color:var(--color-warning);font:var(--type-eyebrow)}
  h2{margin:0;font:var(--type-title)}
  dl{display:grid;gap:var(--space-2);margin:0}
  dl div{display:grid;grid-template-columns:6rem minmax(0,1fr);gap:var(--space-2)}
  dt{color:var(--color-text-tertiary)}dd{margin:0;overflow-wrap:anywhere;color:var(--color-text-secondary)}
  .actions{display:flex;justify-content:flex-end;gap:var(--space-2)}
  button{border-radius:var(--radius-control);padding:var(--space-2) var(--space-4);font:inherit;cursor:pointer}
  .secondary{border:1px solid var(--color-border);background:transparent;color:var(--color-text)}
  .primary{border:0;background:var(--color-accent);color:var(--color-bg);font-weight:700}
</style>
