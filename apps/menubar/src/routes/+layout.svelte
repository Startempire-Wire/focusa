<script lang="ts">
  import type { Snippet } from 'svelte';
  let { children }: { children: Snippet } = $props();
</script>

<div class="popover">
  {@render children()}
</div>

<style>
  .popover {
    width: 100vw;
    height: 100vh;
    display: flex;
    flex-direction: column;
    background: var(--bg);
    border-radius: var(--r-lg);
    border: 1px solid var(--border-heavy);
    overflow: hidden;
  }
</style>
