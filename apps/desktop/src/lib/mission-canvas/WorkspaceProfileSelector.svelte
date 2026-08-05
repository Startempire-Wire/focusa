<script lang="ts">
  import type { WorkspaceProfile } from './types';

  let {
    profiles,
    activeProfileId,
    onSelect
  }: {
    profiles: readonly WorkspaceProfile[];
    activeProfileId: string;
    onSelect: (profile: WorkspaceProfile) => void;
  } = $props();

  function selectProfile(event: Event): void {
    const profileId = (event.currentTarget as HTMLSelectElement).value;
    const profile = profiles.find((candidate) => candidate.profile_id === profileId);
    if (profile) onSelect(profile);
  }
</script>

{#if profiles.length > 0}
  <label class="profile-selector">
    <span>Workspace profile</span>
    <select value={activeProfileId} onchange={selectProfile}>
      {#each profiles as profile (profile.profile_id)}
        <option value={profile.profile_id}>{profile.display_name}</option>
      {/each}
    </select>
  </label>
{/if}

<style>
  .profile-selector{display:flex;align-items:center;gap:var(--space-2);min-width:0;color:var(--color-text-tertiary);font:var(--type-caption)}
  .profile-selector span{white-space:nowrap}
  select{min-width:0;max-width:18rem;border:1px solid var(--color-border);border-radius:var(--radius-control);padding:var(--space-2) var(--space-3);background:var(--color-raised);color:var(--color-text);font:inherit}
</style>
