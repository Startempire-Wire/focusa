<script lang="ts">
  import type { ResolvedWorkspaceProjection } from './types';

  let { projection }: { projection: ResolvedWorkspaceProjection } = $props();

  function plainLabel(value: unknown, fallback: string): string {
    if (typeof value !== 'string' || !value.trim()) return fallback;
    const tail = value.split(':').at(-1) ?? value;
    return tail
      .replace(/^contribution-/, '')
      .replace(/[-_]+/g, ' ')
      .replace(/\b\w/g, (letter) => letter.toUpperCase());
  }

  const projectName = $derived(projection.workstream.scope.scope_key.canonical_name
    ?? plainLabel(projection.workstream.scope.scope_key.root_path, 'Project'));
  const contextItems = $derived([
    { label: 'Project', value: projectName, raw: projection.workstream.scope.scope_key.root_path },
    { label: 'Workstream', value: plainLabel(projection.workstream.workstream_id, 'Workstream'), raw: projection.workstream.workstream_id },
    { label: 'Workspace', value: plainLabel(projection.workspace_profile_id, 'General'), raw: projection.workspace_profile_id },
    { label: 'Session', value: projection.attachment ? 'Session active' : 'No session', raw: projection.attachment?.session_id },
    { label: 'Pi', value: projection.attachment ? plainLabel(projection.attachment.session_id, 'Attached') : 'Not attached', raw: projection.attachment?.attachment_id },
    { label: 'Activity', value: plainLabel(projection.activity_mode_id, 'Overview'), raw: projection.activity_mode_id }
  ]);
</script>

<header class="context-bar" aria-label="Mission Canvas context">
  <span class="brand" aria-hidden="true">F</span>
  {#each contextItems as item (item.label)}
    <div class="context-item" class:session={item.label === 'Session'}>
      <span>{item.label}</span>
      <strong title={item.raw ?? item.value}>{item.value}</strong>
    </div>
  {/each}
  <div class="projection-status" title={`Projection revision ${projection.projection_revision}`}>
    <i></i><span>Live</span>
  </div>
</header>

<style>
  .context-bar{display:flex;align-items:center;gap:4px;min-width:0;min-height:38px;overflow-x:auto;padding:4px 7px;border:1px solid var(--color-border);border-radius:8px;background:var(--color-panel)}
  .brand{flex:0 0 22px;display:grid;place-items:center;width:22px;height:22px;border-radius:6px;background:linear-gradient(145deg,var(--color-accent-bright),var(--color-accent));color:white;font-size:11px;font-weight:750}
  .context-item{display:flex;align-items:center;gap:5px;min-width:0;max-width:190px;padding:4px 8px;border:1px solid var(--color-border);border-radius:6px;background:var(--color-raised)}
  .context-item>span{color:var(--color-text-tertiary);font:var(--type-overline);text-transform:none;letter-spacing:0}
  .context-item strong{overflow:hidden;color:var(--color-text-secondary);font:var(--type-caption);font-weight:600;text-overflow:ellipsis;white-space:nowrap}
  .context-item.session strong{color:var(--color-success)}
  .projection-status{display:flex;align-items:center;gap:5px;margin-inline-start:auto;padding:4px 8px;color:var(--color-text-tertiary);font:var(--type-caption)}
  .projection-status i{width:6px;height:6px;border-radius:50%;background:var(--color-success);box-shadow:0 0 8px color-mix(in srgb,var(--color-success) 60%,transparent)}
  @container mission-canvas (max-width:980px){.context-item:nth-of-type(3),.context-item:nth-of-type(5){display:none}}
  @container mission-canvas (max-width:720px){.context-item{max-width:140px}.context-item:nth-of-type(2),.context-item:nth-of-type(4){display:none}}
</style>
