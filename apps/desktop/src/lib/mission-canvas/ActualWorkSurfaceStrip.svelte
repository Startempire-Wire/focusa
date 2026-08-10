<script lang="ts">
  import type { ResolvedContribution, ResolvedWorkspaceProjection } from './types';

  let {
    projection,
    onSelect
  }: {
    projection: ResolvedWorkspaceProjection;
    onSelect?: (contributionId: string) => void;
  } = $props();

  const surfaces = $derived(
    projection.eligible_contributions.filter((contribution) => contribution.kind === 'focused_work_surface')
  );

  function selected(surface: ResolvedContribution): boolean {
    return surface.data_ref.ref === projection.focused_work_surface_id
      || surface.contribution_id === projection.focused_work_surface_id;
  }

  function displayLabel(surface: ResolvedContribution): string {
    const label = surface.accessibility.label;
    const source = label.startsWith('contribution:') ? surface.contribution_id : label;
    return source
      .replace(/^contribution:/, '')
      .replace(/^surface:/, '')
      .replace(/[-_]+/g, ' ')
      .replace(/\b\w/g, (letter) => letter.toUpperCase());
  }
</script>

{#if surfaces.length > 0}
  <nav class="work-surface-strip" aria-label="Work Surfaces">
    <span class="strip-label">Work Surfaces</span>
    <div role="tablist" aria-label="Actual Work Surfaces">
      {#each surfaces as surface (surface.contribution_id)}
        <button
          type="button"
          role="tab"
          aria-selected={selected(surface)}
          aria-controls={`work-surface-${surface.contribution_id}`}
          data-work-surface-id={surface.data_ref.ref}
          disabled={!onSelect}
          onclick={() => onSelect?.(surface.contribution_id)}
        >
          {displayLabel(surface)}
        </button>
      {/each}
    </div>
  </nav>
{/if}

<style>
  .work-surface-strip{display:flex;align-items:center;gap:var(--space-2);min-width:0;min-height:34px;padding:3px 7px;border:1px solid var(--color-border);border-radius:7px;background:var(--color-panel)}
  .strip-label{flex-shrink:0;color:var(--color-text-tertiary);font:var(--type-overline);letter-spacing:.08em;text-transform:uppercase}
  [role='tablist']{display:flex;align-items:center;gap:var(--space-1);min-width:0;overflow-x:auto}
  button{min-height:28px;padding:0 var(--space-2);border:1px solid transparent;border-radius:var(--radius-control);background:transparent;color:var(--color-text-tertiary);font:var(--type-caption);white-space:nowrap;cursor:pointer}
  button[aria-selected='true']{border-color:var(--color-border-strong);background:var(--color-elevated);color:var(--color-text)}
  button:disabled{cursor:default;opacity:1}
</style>
