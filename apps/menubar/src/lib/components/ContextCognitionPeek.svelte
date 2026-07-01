<script lang="ts">
  import { onMount } from 'svelte';
  import { fetchJson } from '$lib/api';
  import { deriveTypedScopeStatus, type ScopeContext } from '$lib/projectContext.svelte';

  interface Packet {
    schema_version?: string;
    status?: string;
    advisory?: boolean;
    canonical?: boolean;
    scope_status?: string;
    scope?: {
      project_root?: string;
      continuity_id?: string | null;
      workpoint_id?: string | null;
      trajectory_id?: string | null;
    };
    authority?: { action_authority?: string; canonical_mutation_allowed?: boolean };
    reasoning_frame?: { likely_goal?: string; active_gap?: string; confidence?: number };
    route_frame?: { next_tools?: string[]; recovery_tools?: string[] };
    evidence_refs?: string[];
  }

  let packet = $state<Packet | null>(null);
  let scopeStatus = $state<string>('unknown');
  let workpointId = $state<string | null>(null);
  let trajectoryId = $state<string | null>(null);
  let actionAuthority = $state<string>('unknown');
  let evidenceCount = $state<number>(0);
  let nextTools = $state<string[]>([]);
  let likelyGoal = $state<string | null>(null);
  let activeGap = $state<string | null>(null);
  let error = $state<string | null>(null);
  let loading = $state<boolean>(true);

  function text(v: any, fallback = 'unknown') {
    if (v === null || v === undefined || v === '') return fallback;
    if (Array.isArray(v)) return v.length ? v.join(', ') : fallback;
    if (typeof v === 'string') return v;
    return String(v);
  }

  onMount(async () => {
    try {
      const resp = await fetchJson<{ status: string; scope_status: string; packet: Packet; next_tools: string[] }>(
        '/v1/context-cognition?project_root=/home/wirebot/focusa',
        3000,
      );
      packet = resp.packet ?? null;
      scopeStatus = resp.scope_status ?? 'unknown';
      workpointId = resp.packet?.scope?.workpoint_id ?? null;
      trajectoryId = resp.packet?.scope?.trajectory_id ?? null;
      actionAuthority = resp.packet?.authority?.action_authority ?? 'unknown';
      evidenceCount = Array.isArray(resp.packet?.evidence_refs) ? resp.packet!.evidence_refs!.length : 0;
      nextTools = Array.isArray(resp.next_tools) ? resp.next_tools : [];
      likelyGoal = resp.packet?.reasoning_frame?.likely_goal ?? null;
      activeGap = resp.packet?.reasoning_frame?.active_gap ?? null;
    } catch (e) {
      error = e instanceof Error ? e.message : 'fetch failed';
    } finally {
      loading = false;
    }
  });
</script>

<section class="context-cognition-peek" aria-label="Spec 100 Context Cognition">
  <header class="peek-header">
    <div>
      <div class="eyebrow">Spec 100 · Context Cognition</div>
      <div class="value">
        {loading ? 'loading…' : error ? 'unavailable' : `scope=${scopeStatus}`}
      </div>
      <div class="meta">
        {text(packet?.schema_version, 'focusa.context_cognition_packet.v1')} · advisory · never mutates
      </div>
    </div>
    <span class="chip" class:ok={!error && !loading} class:bad={!!error}>
      {error ? 'err' : loading ? '...' : 'advisory'}
    </span>
  </header>

  <div class="proof-row">
    <section>
      <div class="label">Authority</div>
      <p>
        <strong>{text(actionAuthority, 'unknown')}</strong> ·
        {packet?.authority?.canonical_mutation_allowed === false ? 'read-only' : 'mutation allowed'}
      </p>
    </section>
    <section>
      <div class="label">Workpoint / Trajectory</div>
      <p>
        wp: <code>{text(workpointId, 'none')}</code><br />
        traj: <code>{text(trajectoryId, 'none')}</code>
      </p>
    </section>
  </div>

  <div class="proof-row">
    <section>
      <div class="label">Reasoning</div>
      <p>likely_goal: <code>{text(likelyGoal, 'none')}</code></p>
      <p>active_gap: <code>{text(activeGap, 'none')}</code></p>
    </section>
    <section>
      <div class="label">Evidence</div>
      <p>{evidenceCount} ref(s) attached.</p>
    </section>
  </div>

  {#if nextTools.length > 0}
    <div class="proof-row">
      <section>
        <div class="label">Next tools</div>
        <ul>{#each nextTools.slice(0, 3) as tool}<li><code>{tool}</code></li>{/each}</ul>
      </section>
    </div>
  {/if}

  {#if error}
    <p class="muted">error: {error}</p>
  {/if}
</section>

<style>
  .context-cognition-peek {
    padding: var(--sp-3);
    display: flex;
    flex-direction: column;
    gap: var(--sp-3);
  }
  .peek-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--sp-3);
  }
  .eyebrow,
  .label {
    font-size: 0.7rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--fg-muted);
  }
  .value {
    font-size: 1.05rem;
    font-weight: 600;
  }
  .meta {
    font-size: 0.8rem;
    color: var(--fg-muted);
  }
  .chip {
    padding: 2px 8px;
    border-radius: 999px;
    font-size: 0.75rem;
    background: var(--bg-2);
    color: var(--fg-muted);
  }
  .chip.ok { background: var(--ok); color: var(--bg-0); }
  .chip.bad { background: var(--bad); color: var(--bg-0); }
  .proof-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--sp-3);
  }
  .proof-row section p,
  .proof-row section ul {
    font-size: 0.85rem;
  }
  .muted { color: var(--fg-muted); font-size: 0.8rem; }
  code {
    font-family: var(--mono);
    font-size: 0.78rem;
  }
</style>
