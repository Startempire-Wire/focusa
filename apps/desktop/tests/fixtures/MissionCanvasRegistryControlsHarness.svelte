<script lang="ts">
  import ActivityNavigation from '../../src/lib/mission-canvas/ActivityNavigation.svelte';
  import WorkspaceProfileSelector from '../../src/lib/mission-canvas/WorkspaceProfileSelector.svelte';
  import type { ActivityMode, WorkspaceProfile } from '../../src/lib/mission-canvas/types';

  let { empty = false }: { empty?: boolean } = $props();

  const activities = $derived<ActivityMode[]>(empty ? [] : [
    {
      activity_mode_id: 'activity:overview',
      candidate_contribution_ids: ['contribution:pi-session'],
      display_name: 'Overview',
      revision: 1,
      viability_rule_revision: 'fixture:activity:v1'
    },
    {
      activity_mode_id: 'activity:tasks',
      candidate_contribution_ids: ['contribution:tasks'],
      display_name: 'Tasks',
      revision: 1,
      viability_rule_revision: 'fixture:activity:v1'
    }
  ]);
  const profiles = $derived<WorkspaceProfile[]>(empty ? [] : [
    {
      profile_id: 'profile:software',
      revision: 1,
      display_name: 'Software Engineering',
      candidate_contribution_ids: ['contribution:pi-session'],
      density: 'standard',
      terminology_registry_ref: 'fixture:terminology',
      renderer_registry_ref: 'fixture:renderer',
      domain_semantic_binding_registry_ref: 'fixture:semantic',
      installed: true,
      viability_rule_revision: 'fixture:profile:v1'
    },
    {
      profile_id: 'profile:general',
      revision: 1,
      display_name: 'General',
      candidate_contribution_ids: ['contribution:pi-session'],
      density: 'standard',
      terminology_registry_ref: 'fixture:terminology',
      renderer_registry_ref: 'fixture:renderer',
      domain_semantic_binding_registry_ref: 'fixture:semantic',
      installed: true,
      viability_rule_revision: 'fixture:profile:v1'
    }
  ]);
</script>

<ActivityNavigation {activities} activeActivityModeId="activity:tasks" onSelect={() => {}}/>
<WorkspaceProfileSelector {profiles} activeProfileId="profile:software" onSelect={() => {}}/>
