<script lang="ts">
  import type { ResolvedWorkspaceProjection } from './types';

  let { projection }: { projection: ResolvedWorkspaceProjection } = $props();

  const contextItems = $derived([
    { label: 'Project', value: projection.scope.project_root },
    { label: 'Profile', value: projection.workspace_profile_id },
    { label: 'Session', value: projection.scope.session_id },
    { label: 'Attachment', value: projection.scope.attachment_id },
    { label: 'Focus', value: projection.focused_semantic_target },
    { label: 'Activity', value: projection.activity_mode_id }
  ].filter(({ value }) => Boolean(value)));
</script>

<header class="context-bar" aria-label="Mission Canvas context">
  {#each contextItems as item (item.label)}
    <div class="context-item">
      <span>{item.label}</span>
      <strong title={item.value}>{item.value}</strong>
    </div>
  {/each}
  <div class="context-revision" aria-label={`Projection revision ${projection.projection_revision}`}>
    <span>Projection</span>
    <strong>r{projection.projection_revision}</strong>
  </div>
</header>

<style>
  .context-bar{display:flex;align-items:center;gap:var(--space-1);min-width:0;overflow-x:auto;padding:var(--space-1) var(--space-2);border-block-end:1px solid var(--color-border);background:var(--color-panel)}
  .context-item,.context-revision{display:grid;gap:1px;min-width:88px;max-width:190px;padding:var(--space-1) var(--space-2);border-inline-end:1px solid var(--color-border)}
  .context-revision{min-width:auto;margin-inline-start:auto;border-inline-end:0}
  span{color:var(--color-text-tertiary);font:var(--type-overline);text-transform:uppercase;letter-spacing:.08em}
  strong{overflow:hidden;color:var(--color-text-secondary);font:var(--type-caption);font-weight:600;text-overflow:ellipsis;white-space:nowrap}
  @container mission-canvas (max-width:820px){.context-item{min-width:78px}.context-item:nth-of-type(1),.context-item:nth-of-type(4){display:none}}
</style>
