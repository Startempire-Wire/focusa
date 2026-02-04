<!--
  FocusBubble — represents the current Focus Frame.
  Cloud-like shape, slight inner glow, title on hover.
  Per docs/11-menubar-ui-spec.md: always centered, calm.
-->
<script lang="ts">
  import { focusStore } from '$lib/stores/focus.svelte';

  let hovered = $state(false);
  let frame = $derived(focusStore.activeFrame);
</script>

<div
  class="bubble"
  class:active={!!frame}
  class:idle={!frame}
  onmouseenter={() => hovered = true}
  onmouseleave={() => hovered = false}
  role="status"
  aria-label={frame ? `Focus: ${frame.intent}` : 'No active focus'}
>
  <div class="glow"></div>

  {#if hovered && frame}
    <div class="tooltip">
      <div class="tooltip-intent">{frame.intent}</div>
      {#if frame.checkpoint?.current_state}
        <div class="tooltip-state">{frame.checkpoint.current_state}</div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .bubble {
    position: relative;
    width: 64px;
    height: 64px;
    border-radius: 50%;
    border: 2px solid var(--border);
    transition: all var(--transition-gentle);
    cursor: default;
  }

  .bubble.active {
    background: var(--active);
    border-color: var(--accent-soft);
  }

  .bubble.idle {
    background: transparent;
    border-color: var(--fg-muted);
  }

  .glow {
    position: absolute;
    inset: -8px;
    border-radius: 50%;
    background: var(--active-glow);
    opacity: 0;
    transition: opacity var(--transition-gentle);
  }

  .bubble.active .glow {
    opacity: 1;
  }

  .tooltip {
    position: absolute;
    top: calc(100% + 12px);
    left: 50%;
    transform: translateX(-50%);
    background: var(--bg-panel);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    padding: var(--space-sm) var(--space-md);
    box-shadow: var(--shadow-panel);
    white-space: nowrap;
    z-index: 10;
    animation: fadeIn var(--transition-fade) forwards;
  }

  .tooltip-intent {
    font-weight: 600;
    color: var(--fg);
    font-size: var(--font-size-base);
  }

  .tooltip-state {
    color: var(--fg-dim);
    font-size: var(--font-size-sm);
    margin-top: 2px;
  }

  @keyframes fadeIn {
    from { opacity: 0; transform: translateX(-50%) translateY(4px); }
    to { opacity: 1; transform: translateX(-50%) translateY(0); }
  }
</style>
