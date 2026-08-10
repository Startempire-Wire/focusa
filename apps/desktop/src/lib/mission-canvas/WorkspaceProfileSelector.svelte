<script lang="ts">
  import { validateMissionCanvasContract } from '../../../../../docs/contracts/spec135/mission-canvas-v1/typescript/mission-canvas-validators.generated';
  import type { WorkspaceProfile } from './types';

  let {
    profiles,
    activeProfileId,
    onSelect,
    enabled = true
  }: {
    profiles: readonly WorkspaceProfile[];
    activeProfileId: string;
    onSelect: (profile: WorkspaceProfile) => void;
    enabled?: boolean;
  } = $props();

  // The profile list is already Core-resolved. These checks are only a
  // renderer boundary: malformed, ambiguous, explicitly uninstalled, or
  // content-free values are withheld rather than repaired or turned into a
  // client-local eligibility decision.
  const renderableProfiles = $derived(trustedProfiles(profiles));
  const activeProfile = $derived(
    renderableProfiles.find((profile) => profile.profile_id === activeProfileId)
  );

  function selectProfile(event: Event): void {
    const profileId = (event.currentTarget as HTMLSelectElement | null)?.value;
    if (!profileId) return;
    const profile = renderableProfiles.find((candidate) => candidate.profile_id === profileId);
    if (enabled && profile) onSelect(profile);
  }

  function trustedProfiles(value: readonly WorkspaceProfile[]): readonly WorkspaceProfile[] {
    if (!Array.isArray(value)) return [];

    const seen = new Set<string>();
    const trusted: WorkspaceProfile[] = [];
    for (const profile of value) {
      const validation = validateMissionCanvasContract('WorkspaceProfile', profile);
      if (!validation.valid
        || typeof profile.profile_id !== 'string'
        || typeof profile.display_name !== 'string'
        || !Array.isArray(profile.candidate_contribution_ids)
        || profile.candidate_contribution_ids.some((contributionId: unknown) => typeof contributionId !== 'string')) {
        return [];
      }
      if (profile.installed === false) continue;
      if (profile.candidate_contribution_ids.length === 0) continue;
      if (seen.has(profile.profile_id)) return [];
      seen.add(profile.profile_id);
      trusted.push(profile);
    }
    return trusted;
  }
</script>

{#if renderableProfiles.length > 1 && activeProfile}
  <label class="profile-selector" data-profile-selector="eligible">
    <span>Workspace profile</span>
    <select
      aria-label="Workspace profile"
      value={activeProfile.profile_id}
      disabled={!enabled}
      title={!enabled ? 'Workspace switching is unavailable for this attachment' : 'Workspace profile'}
      onchange={selectProfile}
    >
      {#each renderableProfiles as profile (profile.profile_id)}
        <option value={profile.profile_id} data-profile-id={profile.profile_id}>{profile.display_name}</option>
      {/each}
    </select>
  </label>
{/if}

<style>
  .profile-selector{display:flex;align-items:center;gap:var(--space-2);min-width:0;color:var(--color-text-tertiary);font:var(--type-caption)}
  .profile-selector span{white-space:nowrap}
  select{min-width:0;max-width:18rem;border:1px solid var(--color-border);border-radius:var(--radius-control);padding:var(--space-2) var(--space-3);background:var(--color-raised);color:var(--color-text);font:inherit}
  select:disabled{cursor:not-allowed;opacity:.68}
</style>
