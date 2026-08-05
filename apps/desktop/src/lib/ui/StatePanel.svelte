<script lang="ts">
  import Icon from './Icon.svelte';
  import type { IconName } from './Icon.svelte';
  let { state, title, description }: { state: 'loading' | 'empty' | 'ready' | 'stale' | 'blocked' | 'error'; title: string; description: string } = $props();
  let icon = $derived<IconName>(state === 'ready' ? 'check' : state === 'blocked' || state === 'error' ? 'blocked' : state === 'stale' ? 'warning' : state === 'loading' ? 'sparkles' : 'scope');
</script>
<section class={state} aria-live={state === 'loading' ? 'polite' : undefined}><span class="icon"><Icon name={icon} size={18}/></span><div><strong>{title}</strong><p>{description}</p></div></section>
<style>section{display:flex;align-items:flex-start;gap:var(--space-3);min-height:84px;padding:var(--layout-card-padding);border:1px solid var(--color-border);border-radius:var(--radius-card);background:var(--color-panel)}.icon{display:grid;place-items:center;width:30px;height:30px;flex:0 0 30px;border-radius:var(--radius-control);color:var(--color-text-tertiary);background:var(--color-raised)}strong{color:var(--color-text-soft);font:var(--type-title)}p{max-width:68ch;margin:var(--space-1) 0 0;color:var(--color-text-tertiary);font:var(--type-body)}.ready .icon{color:var(--color-success)}.stale .icon{color:var(--color-warning)}.blocked .icon,.error .icon{color:var(--color-error)}.loading .icon{color:var(--color-accent)}</style>
