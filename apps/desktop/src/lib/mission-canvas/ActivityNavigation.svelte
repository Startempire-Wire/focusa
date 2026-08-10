<script lang="ts">
  import type { ActivityMode } from './types';

  let {
    activities,
    activeActivityModeId,
    onSelect,
    enabled = true
  }: {
    activities: readonly ActivityMode[];
    activeActivityModeId: string;
    onSelect: (activity: ActivityMode) => void;
    enabled?: boolean;
  } = $props();
</script>

{#if activities.length > 1}
  <nav class="activity-navigation" aria-label="Activities">
    {#each activities as activity (activity.activity_mode_id)}
      <button
        type="button"
        aria-current={activity.activity_mode_id === activeActivityModeId ? 'page' : undefined}
        data-activity-mode-id={activity.activity_mode_id}
        aria-disabled={!enabled}
        disabled={!enabled}
        title={!enabled ? 'Activity switching is unavailable for this attachment' : activity.display_name}
        onclick={() => enabled && onSelect(activity)}
      >
        {activity.display_name}
      </button>
    {/each}
  </nav>
{/if}

<style>
  .activity-navigation{display:flex;flex-direction:column;align-items:stretch;gap:var(--space-1);min-width:0;height:100%;overflow:auto;padding:var(--space-2);border-inline-end:1px solid var(--color-border);background:var(--color-panel)}
  button{appearance:none;border:0;border-radius:var(--radius-control);min-height:34px;padding:var(--space-2) var(--space-3);background:transparent;color:var(--color-text-tertiary);font:inherit;text-align:start;white-space:nowrap;cursor:pointer}
  button:hover:not(:disabled){background:var(--color-raised);color:var(--color-text)}
  button[aria-current='page']{background:color-mix(in srgb,var(--color-accent) 22%,var(--color-elevated));color:var(--color-text);font-weight:650}
  button:disabled{cursor:not-allowed;opacity:.62}
  @container mission-canvas (max-width:820px){
    .activity-navigation{flex-direction:row;height:auto;padding:var(--space-1);border-inline-end:0;border-block-end:1px solid var(--color-border)}
    button{min-height:30px;text-align:center}
  }
</style>
