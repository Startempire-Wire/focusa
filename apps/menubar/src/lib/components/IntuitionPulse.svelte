<!--
  IntuitionPulse — soft concentric ripples.
  Originate below view, drift upward, fade unless gated.
  Never interrupt. Per docs/11-menubar-ui-spec.md.
-->
<script lang="ts">
  import { intuitionStore } from '$lib/stores/intuition.svelte';

  let active = $derived(intuitionStore.hasRecent);
</script>

{#if active}
  <div class="pulse-container">
    <div class="ripple ripple-1"></div>
    <div class="ripple ripple-2"></div>
    <div class="ripple ripple-3"></div>
  </div>
{/if}

<style>
  .pulse-container {
    position: absolute;
    bottom: -20px;
    left: 50%;
    transform: translateX(-50%);
    width: 80px;
    height: 80px;
    pointer-events: none;
  }

  .ripple {
    position: absolute;
    inset: 0;
    border-radius: 50%;
    border: 1px solid var(--accent);
    opacity: 0;
    animation: ripple-expand 3s ease-out infinite;
  }

  .ripple-1 { animation-delay: 0s; }
  .ripple-2 { animation-delay: 1s; }
  .ripple-3 { animation-delay: 2s; }

  @keyframes ripple-expand {
    0% {
      transform: scale(0.3) translateY(0);
      opacity: 0.3;
    }
    50% {
      opacity: 0.15;
    }
    100% {
      transform: scale(1.5) translateY(-30px);
      opacity: 0;
    }
  }
</style>
