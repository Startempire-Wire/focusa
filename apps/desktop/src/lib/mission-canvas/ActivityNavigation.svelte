<script lang="ts">
  import type { ActivityMode } from './types';

  let {
    activities,
    activeActivityModeId,
    onSelect
  }: {
    activities: readonly ActivityMode[];
    activeActivityModeId: string;
    onSelect: (activity: ActivityMode) => void;
  } = $props();
</script>

{#if activities.length > 1}
  <nav class="activity-navigation" aria-label="Activities">
    {#each activities as activity (activity.activity_mode_id)}
      <button
        type="button"
        aria-current={activity.activity_mode_id === activeActivityModeId ? 'page' : undefined}
        data-activity-mode-id={activity.activity_mode_id}
        onclick={() => onSelect(activity)}
      >
        {activity.display_name}
      </button>
    {/each}
  </nav>
{/if}

<style>
  .activity-navigation{display:flex;align-items:center;gap:var(--space-1);min-width:0;overflow:auto;padding:var(--space-1)}
  button{appearance:none;border:0;border-radius:var(--radius-control);padding:var(--space-2) var(--space-3);background:transparent;color:var(--color-text-tertiary);font:inherit;white-space:nowrap;cursor:pointer}
  button:hover{background:var(--color-raised);color:var(--color-text)}
  button[aria-current='page']{background:var(--color-elevated);color:var(--color-accent);font-weight:650}
</style>
