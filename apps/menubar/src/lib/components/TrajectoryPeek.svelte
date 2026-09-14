<script lang="ts">
  import { runtimeStore } from '$lib/stores/runtime.svelte';
  import { formatScopeForDisplay, type ScopeContext } from '$lib/projectContext.svelte';

  let s = $derived(runtimeStore.snapshot);
  let trajectory = $derived(s.trajectory ?? {});
  let project = $derived(s.projectIdentity ?? {});
  let workpoint = $derived(s.workpointResume ?? s.workpoint ?? {});

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

  let longTerm = $derived(trajectory.long_term_goal ?? trajectory.long_term ?? trajectory.goal?.long_term_goal);
  let desired = $derived(trajectory.desired_end_state ?? trajectory.desired ?? trajectory.goal?.desired_end_state);
  let midLevel = $derived(trajectory.mid_level_goal ?? trajectory.mid_level_goals ?? trajectory.mlg ?? trajectory.goal?.mid_level_goal);
  let shortTerm = $derived(trajectory.short_term_goal ?? trajectory.short_term ?? trajectory.stg ?? trajectory.goal?.short_term_goal);
  let waypoints = $derived(list(trajectory.waypoints ?? trajectory.waypoint ?? trajectory.progress_markers));
  let current = $derived(trajectory.current_state ?? trajectory.current ?? trajectory.observed_state);
  let gap = $derived(trajectory.gap ?? trajectory.active_gap ?? trajectory.recommended_action);
  let posture = $derived(trajectory.posture ?? trajectory.status ?? 'summary');
  let risks = $derived(list(trajectory.acceptance_risks ?? trajectory.risks));
  let checks = $derived(list(trajectory.required_checks ?? trajectory.checks));
  let evidence = $derived(list(trajectory.required_evidence_refs ?? trajectory.evidence_refs));
</script>

<section class="trajectory-peek" aria-label="Trajectory peek">
  <header class="peek-header">
    <div>
      <div class="eyebrow">HLT → MLG → STG → Waypoints</div>
      <h2>{text(shortTerm ?? midLevel ?? longTerm, 'No trajectory yet')}</h2>
      <p class="definition">HLT = High-Level Trajectory. MLG = Mid-Level Goal. STG = Short-Term Goal. Defer to the operator while actively offering HLT-aligned MLGs, STGs, and Waypoints.</p>
    </div>
    <span class="status-chip" class:watch={posture === 'verify_first'}>{text(posture, 'summary')}</span>
  </header>

  <div class="context-line">
    <span>{text(project.project_id ?? project.canonical_name, 'unknown project')}</span>
    <span>·</span>
    <span>{text(project.project_root ?? project.root, 'no verified root')}</span>
  </div>

  <div class="bubble-card primary">
    <div class="label">Active gap</div>
    <p>{text(gap, 'No active gap reported')}</p>
  </div>

  <div class="bubble-grid">
    <article class="bubble-card">
      <div class="label">HLT / High-Level Trajectory</div>
      <p>{text(longTerm, 'not defined')}</p>
    </article>
    <article class="bubble-card">
      <div class="label">MLG / Mid-Level Goal</div>
      <p>{text(midLevel, 'derived from HLT')}</p>
    </article>
    <article class="bubble-card">
      <div class="label">STG / Short-Term Goal</div>
      <p>{text(shortTerm, 'derived from HLT + MLG after assessment')}</p>
    </article>
    <article class="bubble-card">
      <div class="label">Waypoints</div>
      <p>{waypoints.length ? waypoints.slice(0, 3).join(' → ') : 'concrete progress markers derive from STG'}</p>
    </article>
    <article class="bubble-card">
      <div class="label">Desired end state</div>
      <p>{text(desired, 'not defined')}</p>
    </article>
    <article class="bubble-card">
      <div class="label">Current state</div>
      <p>{text(current, 'not assessed')}</p>
    </article>
    <article class="bubble-card">
      <div class="label">Next Workpoint</div>
      <p>{text(workpoint.next_action ?? workpoint.next ?? trajectory.proposed_workpoint?.next_action, 'no proposed next action')}</p>
    </article>
  </div>

  <div class="proof-row">
    <section>
      <div class="label">Evidence refs</div>
      {#if evidence.length > 0}
        <ul>{#each evidence.slice(0, 4) as item}<li>{item}</li>{/each}</ul>
      {:else}
        <p class="muted">No required evidence refs surfaced.</p>
      {/if}
    </section>
    <section>
      <div class="label">Checks / risks</div>
      {#if checks.length > 0 || risks.length > 0}
        <ul>
          {#each checks.slice(0, 3) as item}<li>{item}</li>{/each}
          {#each risks.slice(0, 2) as item}<li class="risk">{item}</li>{/each}
        </ul>
      {:else}
        <p class="muted">No checks or risks surfaced.</p>
      {/if}
    </section>
  </div>
</section>

<style>
  .trajectory-peek {
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
  .context-line {
    display: flex;
    gap: 6px;
    color: var(--fg-tertiary);
    font-size: var(--text-xs);
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
  }
  .status-chip {
    flex-shrink: 0;
    padding: 2px 7px;
    border: 1px solid var(--border);
    border-radius: var(--r-full);
    color: var(--fg-secondary);
    background: var(--bg-elevated);
    font-family: var(--font-mono);
    font-size: 10px;
  }
  .status-chip.watch {
    color: var(--orange);
    border-color: color-mix(in srgb, var(--orange) 50%, var(--border));
  }
  .bubble-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: var(--sp-2);
  }
  .bubble-card {
    min-width: 0;
    padding: var(--sp-3);
    border: 1px solid var(--border);
    border-radius: var(--r-lg);
    background: var(--bg-panel);
  }
  .bubble-card.primary {
    border-color: color-mix(in srgb, var(--accent) 35%, var(--border));
  }
  p {
    margin: var(--sp-1) 0 0;
    color: var(--fg-secondary);
    font-size: var(--text-sm);
    line-height: 1.45;
  }
  .definition {
    max-width: 56ch;
    color: var(--fg-tertiary);
    font-size: var(--text-xs);
  }
  .proof-row {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: var(--sp-2);
  }
  .proof-row section {
    padding: var(--sp-3);
    border-radius: var(--r-md);
    background: var(--bg-panel);
    border: 1px solid var(--border);
  }
  ul {
    margin: var(--sp-1) 0 0;
    padding-left: var(--sp-3);
    color: var(--fg-secondary);
    font-size: var(--text-xs);
    line-height: 1.45;
  }
  .risk { color: var(--orange); }
  .muted { color: var(--fg-tertiary); font-size: var(--text-xs); }
</style>
