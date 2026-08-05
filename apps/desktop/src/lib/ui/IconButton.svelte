<script lang="ts">
  import Icon, { type IconName } from './Icon.svelte';
  let { icon, label, pressed = false, disabled = false, onclick }: { icon: IconName; label: string; pressed?: boolean; disabled?: boolean; onclick?: () => void } = $props();
</script>
<button type="button" aria-label={label} title={label} aria-pressed={pressed} {disabled} {onclick}><Icon name={icon} /></button>
<style>button{width:30px;height:30px;display:grid;place-items:center;padding:0;border:1px solid transparent;border-radius:var(--radius-control);color:var(--color-text-tertiary);background:transparent;cursor:pointer;transition:color var(--motion-fast) var(--ease-out-quart),background var(--motion-fast) var(--ease-out-quart),border-color var(--motion-fast) var(--ease-out-quart),transform var(--motion-micro) var(--ease-out-quart)}button:hover:not(:disabled),button[aria-pressed='true']{color:var(--color-text);background:color-mix(in srgb,var(--color-accent) 9%,transparent)}button:active:not(:disabled){transform:scale(.98)}button:disabled{cursor:default;opacity:.45}button:focus-visible{outline:2px solid var(--color-accent-bright);outline-offset:2px}</style>
