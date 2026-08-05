<script lang="ts">
  import { onMount } from 'svelte';
  import Icon from './Icon.svelte';
  import { readMotionPreference, setMotionPreference, type MotionMode } from './motion';
  const modes: MotionMode[] = ['system', 'full', 'reduced'];
  let mode = $state<MotionMode>('system');
  onMount(() => { mode = readMotionPreference(); });
  function select(next: MotionMode) { mode = next; setMotionPreference(next); }
</script>
<div class="motion-control">
  <span class="motion-label"><Icon name="settings" size={14}/><span>Motion</span></span>
  <div role="group" aria-label="Motion preference">
    {#each modes as option}<button type="button" class:active={mode === option} aria-pressed={mode === option} onclick={() => select(option)}>{option}</button>{/each}
  </div>
</div>
<style>.motion-control{display:flex;align-items:center;justify-content:space-between;gap:var(--space-2);padding:var(--space-2);border-top:1px solid var(--color-border)}.motion-label{display:flex;align-items:center;gap:var(--space-2);color:var(--color-text-tertiary);font:var(--type-caption)}.motion-control>div{display:flex;padding:2px;border:1px solid var(--color-border);border-radius:var(--radius-pill);background:var(--color-panel)}button{min-height:24px;padding:0 7px;border:0;border-radius:var(--radius-pill);color:var(--color-text-tertiary);background:transparent;font-size:9px;text-transform:capitalize;cursor:pointer;transition:color var(--motion-fast) var(--ease-out-quart),background var(--motion-fast) var(--ease-out-quart),transform var(--motion-micro) var(--ease-out-quart)}button:hover{color:var(--color-text-soft)}button.active{color:var(--color-text);background:color-mix(in srgb,var(--color-accent) 14%,var(--color-raised))}button:active{transform:scale(.98)}</style>
