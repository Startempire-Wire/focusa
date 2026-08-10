<script lang="ts">
  import type { ResolvedContribution, ResolvedWorkspaceProjection } from '../types';
  let { contribution, projection }: { contribution: ResolvedContribution; projection: ResolvedWorkspaceProjection } = $props();
  const profile = $derived(projection.workspace_profile_id.replace(/[-_]+/g, ' ').replace(/\b\w/g, (c) => c.toUpperCase()));
  const activity = $derived(projection.activity_mode_id.replace(/[-_]+/g, ' ').replace(/\b\w/g, (c) => c.toUpperCase()));
</script>
<section class="projection-controls" aria-label={contribution.accessibility.label}>
  <strong>{profile} · {activity}</strong>
  <span><i></i> Canonical projection r{projection.projection_revision}</span>
</section>
<style>
  .projection-controls{display:flex;align-items:center;justify-content:space-between;gap:12px;min-height:38px;padding:7px 11px;border:1px solid var(--color-border);border-radius:8px;background:var(--color-panel)}
  strong{font:var(--type-label);color:var(--color-text)}span{display:flex;align-items:center;gap:6px;color:var(--color-text-tertiary);font:var(--type-caption)}i{width:7px;height:7px;border-radius:50%;background:var(--color-success)}
</style>
