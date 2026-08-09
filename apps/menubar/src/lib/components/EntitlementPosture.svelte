<script lang="ts">
  import { onMount } from 'svelte';
  import { fetchJson } from '$lib/api';
  import {
    projectEntitlementPosture,
    type EntitlementPosture,
    type LicenseStatusPayload,
  } from '$lib/entitlementPosture';

  let posture = $state<EntitlementPosture | null>(null);
  let error = $state('');
  let loading = $state(true);

  async function refresh() {
    loading = true;
    error = '';
    try {
      posture = projectEntitlementPosture(await fetchJson<LicenseStatusPayload>('/v1/license/status', 5000));
    } catch {
      posture = projectEntitlementPosture({ status: 'recovery_only', authority: { recovery_reason: 'status_unavailable' } });
      error = 'Entitlement status unavailable. Execution remains locked.';
    } finally {
      loading = false;
    }
  }

  onMount(() => { void refresh(); });
</script>

<section class="entitlement-posture" aria-label="Entitlement posture">
  <div class="heading">
    <h3>Entitlement</h3>
    <button class="refresh" disabled={loading} onclick={refresh}>Refresh</button>
  </div>
  {#if posture}
    <p><strong>{posture.state.replace('_', ' ')}</strong>{#if posture.masked_identity} · {posture.masked_identity}{/if}</p>
    <p class="dim">Shared presenter state: <strong>{posture.presenter_state}</strong> · next: {posture.next_action} · allowed: {posture.allowed_actions.join(', ')}</p>
    {#if posture.expires_at}<p>Expires: <time datetime={posture.expires_at}>{posture.expires_at}</time></p>{/if}
    {#if posture.offline_grace_until}<p>Offline recovery window: <time datetime={posture.offline_grace_until}>{posture.offline_grace_until}</time></p>{/if}
    {#if posture.limits.length > 0}
      <h4>Remaining signed limits</h4>
      <ul>{#each posture.limits as limit}<li>{limit.name}: {limit.remaining}</li>{/each}</ul>
    {/if}
    {#if posture.locked_capabilities.length > 0}
      <h4>Locked capabilities</h4>
      <ul>{#each posture.locked_capabilities as capability}<li>{capability.name} · {capability.reason}</li>{/each}</ul>
    {/if}
    <p>{posture.recovery_policy}</p>
    <p class="dim">Marketing preference: managed separately from terms and entitlement.</p>
    <p class="action">Action: <strong>{posture.action}</strong> through the authority account or <code>focusa license</code>.</p>
  {/if}
  {#if error}<p class="error">{error}</p>{/if}
</section>

<style>
  .entitlement-posture { border: 1px solid #374151; border-radius: 10px; padding: 12px; }
  .heading { display: flex; align-items: center; justify-content: space-between; }
  h3, h4 { margin: 0 0 8px; }
  p { margin: 6px 0; }
  ul { margin: 4px 0 10px; padding-left: 20px; }
  .dim { color: #9ca3af; }
  .error { color: #fca5a5; }
  .refresh { font: inherit; }
</style>
