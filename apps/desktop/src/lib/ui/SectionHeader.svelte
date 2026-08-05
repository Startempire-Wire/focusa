<script lang="ts">
  import type { Snippet } from 'svelte';
  let { eyebrow, title, description, action }: { eyebrow?: string; title: string; description?: string; action?: Snippet } = $props();
</script>
<header class="section-header"><div>{#if eyebrow}<span>{eyebrow}</span>{/if}<h2>{title}</h2>{#if description}<p>{description}</p>{/if}</div>{#if action}<aside>{@render action()}</aside>{/if}</header>
<style>.section-header{display:flex;align-items:flex-start;justify-content:space-between;gap:var(--space-4)}.section-header>div{min-width:0}.section-header span{display:block;color:var(--color-text-tertiary);font:var(--type-eyebrow);letter-spacing:.07em;text-transform:uppercase}.section-header h2{margin:var(--space-1) 0 0;color:var(--color-text);font:var(--type-heading);letter-spacing:-.015em}.section-header p{max-width:68ch;margin:var(--space-2) 0 0;color:var(--color-text-tertiary);font:var(--type-body)}aside{flex:0 0 auto}</style>
