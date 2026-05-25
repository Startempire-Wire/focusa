<script lang="ts">
  import { runtimeStore } from '$lib/stores/runtime.svelte';

  let s = $derived(runtimeStore.snapshot);
  let workpoint = $derived(s.workpointResume ?? s.workpoint ?? {});
  let packet = $derived(workpoint.resume_packet ?? workpoint.packet ?? workpoint);

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

  function records(payload: any, keys: string[]): any[] {
    if (!payload) return [];
    if (Array.isArray(payload)) return payload;
    for (const key of keys) {
      if (Array.isArray(payload[key])) return payload[key];
    }
    return [];
  }

  let evidence = $derived(list(packet.verified_evidence ?? packet.evidence_refs ?? workpoint.evidence_refs));
  let predictions = $derived(records(s.predictionsRecent, ['predictions', 'items', 'records']));
  let evaluations = $derived(records(s.metacogEvaluations, ['evaluations', 'items', 'records']));
  let snapshots = $derived(records(s.snapshotsRecent, ['snapshots', 'items', 'records']));
  let predictionStats = $derived(s.predictionsStats ?? {});
  let metacogStatus = $derived(s.metacogStatus ?? {});
  let lineage = $derived(s.lineageHead ?? {});
</script>

<section class="proof-peek" aria-label="Proof peek">
  <header class="peek-header">
    <div>
      <div class="eyebrow">PROOF</div>
      <h2>Evidence, learning, prediction, recovery</h2>
    </div>
    <span class="status-chip">read-only</span>
  </header>

  <div class="proof-grid">
    <article class="panel primary">
      <div class="label">Workpoint evidence</div>
      <div class="metric">{evidence.length}</div>
      {#if evidence.length > 0}
        <ul>{#each evidence.slice(0, 5) as item}<li>{item}</li>{/each}</ul>
      {:else}
        <p class="muted">No evidence refs linked to current Workpoint packet.</p>
      {/if}
    </article>

    <article class="panel">
      <div class="label">Predictions</div>
      <div class="metric">{predictions.length}</div>
      <p>accuracy {text(predictionStats.accuracy ?? predictionStats.average_score, 'n/a')} · total {text(predictionStats.total ?? predictionStats.count, 'n/a')}</p>
      {#if predictions.length > 0}
        <ul>{#each predictions.slice(0, 3) as item}<li>{text(item.predicted_outcome ?? item.outcome ?? item.prediction_type ?? item.id)}</li>{/each}</ul>
      {/if}
    </article>

    <article class="panel">
      <div class="label">Metacognition</div>
      <div class="metric">{evaluations.length}</div>
      <p>{text(metacogStatus.status ?? metacogStatus.summary, 'status unknown')}</p>
      {#if evaluations.length > 0}
        <ul>{#each evaluations.slice(0, 3) as item}<li>{text(item.outcome ?? item.actual_outcome ?? item.adjustment_id ?? item.id)}</li>{/each}</ul>
      {:else}
        <p class="muted">No recent evaluation records surfaced.</p>
      {/if}
    </article>

    <article class="panel">
      <div class="label">Snapshots</div>
      <div class="metric">{snapshots.length}</div>
      {#if snapshots.length > 0}
        <ul>{#each snapshots.slice(0, 4) as item}<li>{text(item.snapshot_id ?? item.id ?? item.reason)}</li>{/each}</ul>
      {:else}
        <p class="muted">No recent snapshots surfaced.</p>
      {/if}
    </article>
  </div>

  <section class="lineage-card">
    <div class="label">Lineage head</div>
    <p>{text(lineage.head?.clt_node_id ?? lineage.clt_node_id ?? lineage.id, 'no lineage head surfaced')}</p>
    <div class="chips">
      <span class="chip">GET /v1/lineage/head</span>
      <span class="chip">GET /v1/focus/snapshots/recent</span>
    </div>
  </section>
</section>

<style>
  .proof-peek {
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
    min-height: 16px;
    align-items: center;
    padding: 1px 6px;
    font-size: 9px;
  }
  .proof-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: var(--sp-2);
  }
  .panel,
  .lineage-card {
    min-width: 0;
    padding: var(--sp-3);
    border: 1px solid var(--border);
    border-radius: var(--r-lg);
    background: var(--bg-panel);
  }
  .panel.primary { border-color: color-mix(in srgb, var(--accent) 35%, var(--border)); }
  .metric {
    margin-top: var(--sp-1);
    color: var(--fg);
    font-size: 26px;
    font-weight: 700;
    line-height: 1;
  }
  p {
    margin: var(--sp-1) 0 0;
    color: var(--fg-secondary);
    font-size: var(--text-sm);
    line-height: 1.45;
  }
  ul {
    margin: var(--sp-2) 0 0;
    padding-left: var(--sp-3);
    color: var(--fg-secondary);
    font-size: var(--text-xs);
    line-height: 1.45;
  }
  .muted { color: var(--fg-tertiary); font-size: var(--text-xs); }
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    margin-top: var(--sp-2);
  }
</style>
