<!--
  FocusStackView — background thought clouds.
  Inactive frames drift slowly, fade with distance.
  Never overlap focused bubble.
-->
<script lang="ts">
  import { focusStore } from '$lib/stores/focus';

  let frames = $derived(focusStore.inactiveFrames);
</script>

<div class="clouds">
  {#each frames as frame, i}
    <div
      class="cloud"
      class:paused={frame.status === 'Paused' || frame.status === 'Suspended'}
      class:completed={frame.status === 'Completed' || frame.status === 'Archived'}
      style="
        --delay: {i * 0.5}s;
        --x: {20 + (i * 30) % 80}%;
        --y: {15 + (i * 25) % 70}%;
        --opacity: {Math.max(0.15, 0.5 - i * 0.1)};
      "
      title={frame.intent}
    >
      <span class="cloud-label">{frame.intent.slice(0, 20)}</span>
    </div>
  {/each}
</div>

<style>
  .clouds {
    position: absolute;
    inset: 0;
  }

  .cloud {
    position: absolute;
    left: var(--x);
    top: var(--y);
    padding: var(--space-xs) var(--space-sm);
    background: var(--bg-panel);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    opacity: var(--opacity);
    font-size: var(--font-size-sm);
    color: var(--fg-dim);
    animation: drift var(--transition-drift) infinite alternate;
    animation-delay: var(--delay);
    pointer-events: auto;
    cursor: default;
    transition: opacity var(--transition-gentle);
  }

  .cloud:hover {
    opacity: 0.8;
  }

  .cloud.completed {
    opacity: calc(var(--opacity) * 0.5);
    border-style: dashed;
  }

  .cloud-label {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 120px;
    display: inline-block;
  }

  @keyframes drift {
    from { transform: translate(0, 0); }
    to { transform: translate(4px, -6px); }
  }
</style>
