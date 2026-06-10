<!--
  ToastContainer — global transient feedback for action buttons.
  Subscribes to toastStore.items and renders a small stack in the
  bottom-right of the popover. macOS-HIG style: 8px radius, subtle shadow,
  3s auto-dismiss, click-to-dismiss.
-->
<script lang="ts">
  import { toastStore } from '$lib/stores/toast.svelte';
  let items = $derived(toastStore.items);
</script>

<div class="toast-stack" role="status" aria-live="polite" aria-atomic="false">
  {#each items as t (t.id)}
    <button
      class="toast"
      class:ok={t.kind === 'ok'}
      class:info={t.kind === 'info'}
      class:warn={t.kind === 'warn'}
      class:err={t.kind === 'err'}
      type="button"
      onclick={() => toastStore.dismiss(t.id)}
      title="click to dismiss"
    >
      <span class="toast-msg">{t.message}</span>
      {#if t.detail}
        <span class="toast-detail">{t.detail}</span>
      {/if}
    </button>
  {/each}
</div>

<style>
  .toast-stack {
    position: fixed;
    right: 8px;
    bottom: 8px;
    display: flex;
    flex-direction: column;
    gap: 4px;
    z-index: 1000;
    max-width: 280px;
    pointer-events: none;
  }
  .toast {
    pointer-events: auto;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 2px;
    padding: 6px 10px;
    border-radius: 8px;
    border: 1px solid var(--border);
    background: var(--bg-panel);
    color: var(--fg);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.18);
    font-family: var(--font);
    font-size: var(--text-sm);
    text-align: left;
    cursor: pointer;
    animation: slidein 200ms ease-out;
  }
  .toast:hover { filter: brightness(0.97); }
  .toast.ok { border-color: var(--green); }
  .toast.ok .toast-msg { color: var(--green); }
  .toast.warn { border-color: var(--orange); }
  .toast.warn .toast-msg { color: var(--orange); }
  .toast.err { border-color: var(--red); }
  .toast.err .toast-msg { color: var(--red); }
  .toast-msg { font-weight: 500; }
  .toast-detail {
    font-size: var(--text-xs);
    color: var(--fg-secondary);
    font-family: var(--font-mono);
    word-break: break-all;
  }
  @keyframes slidein {
    from { transform: translateY(8px); opacity: 0; }
    to { transform: translateY(0); opacity: 1; }
  }
</style>
