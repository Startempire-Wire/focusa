<script lang="ts">
  import { normalizeToolResult } from '$lib/api';
  import { runtimeStore } from '$lib/stores/runtime.svelte';

  let s = $derived(runtimeStore.snapshot);
  let workpoint = $derived(s.workpointResume ?? s.workpoint ?? {});
  let packet = $derived(workpoint.resume_packet ?? workpoint.packet ?? workpoint);
  let result = $derived(normalizeToolResult(workpoint));

  function text(v: any, fallback = 'unknown') {
    if (v === null || v === undefined || v === '') return fallback;
    if (Array.isArray(v)) return v.length ? v.join(', ') : fallback;
    if (typeof v === 'string') return v;
    return String(v);
  }

  function list(v: any): string[] {
    if (!v) return [];
    if (Array.isArray(v)) return v.map((item) => typeof item === 'string' ? item : JSON.stringify(item));
    return [text(v)];
  }

  let canonical = $derived(workpoint.canonical ?? packet.canonical ?? result.canonical);
  let advisory = $derived(workpoint.advisory ?? workpoint.advisory_only ?? packet.advisory ?? packet.advisory_only ?? result.advisory);
  let degraded = $derived(workpoint.degraded ?? packet.degraded ?? result.degraded);
  let stale = $derived(workpoint.stale ?? packet.stale ?? packet.freshness?.stale ?? result.stale);
  let scopeStatus = $derived(packet.scope?.scope_status ?? workpoint.scope?.scope_status ?? result.scope_status ?? packet.current_ask_scope?.scope_status);
  let scopeSource = $derived(packet.scope?.scope_source ?? workpoint.scope?.scope_source ?? result.scope_source);
  let status = $derived(workpoint.status ?? packet.status ?? result.status ?? (canonical === true ? 'canonical' : 'unknown'));
  let mission = $derived(packet.mission ?? workpoint.mission ?? packet.objective);
  let currentAction = $derived(packet.current_action ?? workpoint.current_action);
  let nextAction = $derived(packet.next_action ?? workpoint.next_action ?? workpoint.next);
  let workpointId = $derived(workpoint.id ?? packet.workpoint_id ?? packet.id);
  let projectRoot = $derived(packet.project_root ?? workpoint.project_root);
  let continuityId = $derived(packet.continuity_id ?? workpoint.continuity_id);
  let blockers = $derived(list(packet.blockers ?? workpoint.blockers));
  let targets = $derived(list(packet.target_objects ?? packet.active_object_refs ?? workpoint.target_objects));
  let evidence = $derived(list(packet.verified_evidence ?? packet.evidence_refs ?? result.evidence_refs ?? workpoint.evidence_refs));
  let doNotDrift = $derived(list(packet.do_not_drift ?? workpoint.do_not_drift));
  let warnings = $derived(list(workpoint.warnings ?? packet.warnings ?? workpoint.scope_warnings));
</script>

<section class="workpoint-peek" aria-label="Workpoint peek">
  <header class="peek-header">
    <div>
      <div class="eyebrow">WORKPOINT</div>
      <h2>{text(nextAction ?? mission, 'No canonical continuation')}</h2>
    </div>
    <span class="status-chip" class:ok={canonical === true} class:watch={status === 'pending'} class:bad={canonical === false || degraded === true}>{text(status)}</span>
  </header>

  <div class="identity-line">
    <span>{text(workpointId, 'no id')}</span>
    <span>·</span>
    <span>{text(projectRoot, 'no project root')}</span>
  </div>

  <div class="summary-card" class:ok={canonical === true} class:bad={canonical === false || degraded === true}>
    <div class="label">Continuation contract</div>
    <p>{text(mission, 'No mission recorded')}</p>
    <div class="chips">
      <span class="chip" class:ok={canonical === true} class:bad={canonical === false}>{canonical === true ? 'canonical' : canonical === false ? 'non-canonical' : 'unknown'}</span>
      {#if advisory === true}<span class="chip watch">advisory</span>{/if}
      {#if degraded === true}<span class="chip bad">degraded</span>{/if}
      {#if stale === true}<span class="chip bad">stale</span>{/if}
      {#if scopeStatus}<span class="chip" class:ok={scopeStatus === 'verified'} class:bad={scopeStatus !== 'verified'}>scope:{scopeStatus}</span>{/if}
      {#if scopeSource}<span class="chip">source:{scopeSource}</span>{/if}
      {#if continuityId}<span class="chip">{continuityId}</span>{/if}
    </div>
  </div>

  <div class="action-grid">
    <article class="panel">
      <div class="label">Current action</div>
      <p>{text(currentAction, 'not recorded')}</p>
    </article>
    <article class="panel primary">
      <div class="label">Next action</div>
      <p>{text(nextAction, 'not recorded')}</p>
    </article>
  </div>

  <div class="detail-grid">
    <section class="panel">
      <div class="label">Target objects</div>
      {#if targets.length > 0}
        <ul>{#each targets.slice(0, 5) as item}<li>{item}</li>{/each}</ul>
      {:else}
        <p class="muted">No target objects surfaced.</p>
      {/if}
    </section>

    <section class="panel">
      <div class="label">Evidence refs</div>
      {#if evidence.length > 0}
        <ul>{#each evidence.slice(0, 5) as item}<li>{item}</li>{/each}</ul>
      {:else}
        <p class="muted">No linked evidence yet.</p>
      {/if}
    </section>

    <section class="panel">
      <div class="label">Blockers</div>
      {#if blockers.length > 0}
        <ul>{#each blockers.slice(0, 4) as item}<li class="blocker">{item}</li>{/each}</ul>
      {:else}
        <p class="muted">No blockers surfaced.</p>
      {/if}
    </section>

    <section class="panel">
      <div class="label">Do not drift</div>
      {#if doNotDrift.length > 0}
        <ul>{#each doNotDrift.slice(0, 4) as item}<li>{item}</li>{/each}</ul>
      {:else}
        <p class="muted">No drift boundaries surfaced.</p>
      {/if}
    </section>
  </div>

  {#if warnings.length > 0}
    <section class="warning-card">
      <div class="label">Warnings</div>
      <ul>{#each warnings.slice(0, 4) as item}<li>{item}</li>{/each}</ul>
    </section>
  {/if}
</section>

<style>
  .workpoint-peek {
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
    font-size: 10px;
    font-weight: 700;
    color: var(--fg-tertiary);
    letter-spacing: 0.8px;
    text-transform: uppercase;
  }
  h2 {
    margin: 2px 0 0;
    color: var(--fg);
    font-size: var(--text-lg);
    line-height: 1.25;
  }
  .identity-line {
    display: flex;
    gap: 6px;
    color: var(--fg-tertiary);
    font-size: var(--text-xs);
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
  }
  .status-chip,
  .chip {
    border: 1px solid var(--border);
    border-radius: var(--r-full);
    color: var(--fg-secondary);
    background: var(--bg-elevated);
    font-family: var(--font-mono);
  }
  .status-chip {
    flex-shrink: 0;
    padding: 2px 7px;
    font-size: 10px;
  }
  .chip {
    display: inline-flex;
    align-items: center;
    min-height: 16px;
    padding: 1px 6px;
    font-size: 9px;
    line-height: 1.3;
  }
  .status-chip.ok,
  .chip.ok {
    color: var(--green);
    border-color: color-mix(in srgb, var(--green) 45%, var(--border));
  }
  .status-chip.watch {
    color: var(--orange);
    border-color: color-mix(in srgb, var(--orange) 50%, var(--border));
  }
  .status-chip.bad,
  .chip.bad {
    color: var(--red);
    border-color: color-mix(in srgb, var(--red) 50%, var(--border));
  }
  .summary-card,
  .panel,
  .warning-card {
    min-width: 0;
    padding: var(--sp-3);
    border: 1px solid var(--border);
    border-radius: var(--r-lg);
    background: var(--bg-panel);
  }
  .summary-card.ok { border-color: color-mix(in srgb, var(--green) 35%, var(--border)); }
  .summary-card.bad,
  .warning-card { border-color: color-mix(in srgb, var(--red) 35%, var(--border)); }
  .panel.primary { border-color: color-mix(in srgb, var(--accent) 35%, var(--border)); }
  .action-grid,
  .detail-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: var(--sp-2);
  }
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    margin-top: var(--sp-2);
  }
  p {
    margin: var(--sp-1) 0 0;
    color: var(--fg-secondary);
    font-size: var(--text-sm);
    line-height: 1.45;
  }
  ul {
    margin: var(--sp-1) 0 0;
    padding-left: var(--sp-3);
    color: var(--fg-secondary);
    font-size: var(--text-xs);
    line-height: 1.45;
  }
  .blocker { color: var(--orange); }
  .muted { color: var(--fg-tertiary); font-size: var(--text-xs); }
</style>
