<script lang="ts">
  let { tone = 'neutral', label }: { tone?: 'neutral' | 'ready' | 'watch' | 'blocked' | 'error'; label: string } = $props();
</script>
<span class={tone}><i aria-hidden="true"></i>{label}</span>
<style>span{display:inline-flex;align-items:center;gap:6px;width:max-content;padding:4px 8px;border:1px solid var(--color-border);border-radius:var(--radius-pill);color:var(--color-text-tertiary);background:color-mix(in srgb,var(--color-raised) 82%,transparent);font:var(--type-caption)}i{width:6px;height:6px;border-radius:50%;background:currentColor}.ready{color:var(--color-success)}.watch{color:var(--color-warning)}.blocked,.error{color:var(--color-error)}</style>
