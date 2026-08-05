<script lang="ts">
  import type { Snippet } from 'svelte';
  let { variant = 'neutral', padding = 'standard', children }: { variant?: 'neutral' | 'raised' | 'spotlight'; padding?: 'none' | 'standard' | 'roomy'; children: Snippet } = $props();
</script>
<div class="surface" class:raised={variant === 'raised'} class:spotlight={variant === 'spotlight'} class:standard={padding === 'standard'} class:roomy={padding === 'roomy'}>{@render children()}</div>
<style>.surface{border:1px solid var(--color-border);border-radius:var(--radius-card);background:var(--color-panel);box-shadow:var(--shadow-card)}.raised{background:var(--color-raised)}.spotlight{border-color:color-mix(in srgb,var(--color-accent) 28%,var(--color-border));background:radial-gradient(circle at 14% 0%,color-mix(in srgb,var(--color-accent) 8%,transparent),transparent 42%),var(--color-raised)}.standard{padding:var(--layout-card-padding)}.roomy{padding:var(--layout-card-padding-roomy)}</style>
