<script lang="ts">
  import type { Snippet } from 'svelte';
  let { gap = 'section', horizontal = false, children }: { gap?: 'section' | 'cluster' | 'control'; horizontal?: boolean; children: Snippet } = $props();
</script>
<div class="stack" class:horizontal style={`--stack-gap:var(--layout-${gap}-gap)`}>{@render children()}</div>
<style>.stack{display:flex;flex-direction:column;gap:var(--stack-gap)}.horizontal{align-items:center;flex-direction:row}</style>
