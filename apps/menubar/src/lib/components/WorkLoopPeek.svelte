<script lang="ts">
  import { runtimeStore } from '$lib/stores/runtime.svelte';
  import { formatScopeForDisplay, type ScopeContext } from '$lib/projectContext.svelte';

  let s = $derived(runtimeStore.snapshot);
  let health = $derived(s.workLoopHealth ?? {});
  let status = $derived(s.workLoop ?? {});
  let checkpointsPayload = $derived(s.workLoopCheckpoints ?? {});

  function text(v: any, fallback = 'unknown') {
    if (v === null || v === undefined || v === '') return fallback;
    if (Array.isArray(v)) return v.length ? v.join(', ') : fallback;
    if (typeof v === 'string') return v;
    return String(v);
  }

  function records(payload: any, keys: string[]): any[] {
    if (!payload) return [];
    if (Array.isArray(payload)) return payload;
    for (const key of keys) {
      if (Array.isArray(payload[key])) return payload[key];
    }
    return [];
  }

  let dispatchReady = $derived(health.dispatch_ready ?? health.ready ?? health.dispatch_readiness?.ready);
  let transport = $derived(status.transport_health ?? health.transport_health ?? {});
  let degraded = $derived(health.degraded ?? status.degraded ?? transport.status === 'degraded');
  let boundary = $derived(health.boundary_reason ?? status.boundary_reason ?? transport.last_reason);
  let pauseFlags = $derived(records(health.pause_flags ?? status.pause_flags, ['flags', 'items']));
  let checkpoints = $derived(records(checkpointsPayload, ['checkpoints', 'items', 'records']));
  let activeTask = $derived(status.current_task ?? status.work_loop?.current_task ?? {});
  let partition = $derived(status.execution_partition ?? health.execution_partition ?? {});
  let budget = $derived(status.budget_remaining ?? health.budget_remaining ?? {});
  let typedState = $derived(
    status.schema === 'focusa.work_loop_status.v3' &&
    ['absent', 'unavailable', 'stale', 'unsupported', 'blocked', 'zero', 'healthy'].includes(status.state)
      ? status.state
      : 'unsupported'
  );
  let writer = $derived(partition.writer_key ?? health.writer_owner ?? status.writer_owner ?? status.active_writer ?? health.active_writer ?? status.writer?.owner);
  let leaseFreshness = $derived(partition.lease_freshness ?? 'unclaimed');
  let projectRoot = $derived(s.projectIdentity?.project_root ?? s.workpointResume?.project_root ?? s.session?.project_root);
  let loopWorkpoint = $derived(status.active_workpoint?.active ?? status.active_workpoint ?? {});
  let loopProjectRoot = $derived(loopWorkpoint.project_root);
  let loopTaskStale = $derived(Boolean(status.authority?.canonical === false || (loopProjectRoot && projectRoot && loopProjectRoot !== projectRoot)));
  let currentWorkpoint = $derived(s.workpointResume ?? {});
</script>

<section class="workloop-peek" aria-label="Work Loop peek">
  <header class="peek-header">
    <div>
      <div class="eyebrow">WORK LOOP</div>
      <h2>{dispatchReady === true ? 'Ready to dispatch' : dispatchReady === false ? 'Boundary active' : 'Readiness unknown'}</h2>
    </div>
    <span class="status-chip" class:ok={dispatchReady === true} class:watch={dispatchReady === false} class:bad={degraded === true}>{degraded === true ? 'degraded' : dispatchReady === true ? 'ready' : 'hold'}</span>
  </header>

  <div class="summary-card" class:ok={dispatchReady === true} class:watch={dispatchReady === false} class:bad={degraded === true}>
    <div class="label">Dispatch posture</div>
    <p>{text(boundary, dispatchReady === true ? 'No active boundary' : 'No boundary reason surfaced')}</p>
    <div class="chips">
      <span class="chip" class:stale-chip={typedState === 'unsupported' || typedState === 'unavailable'}>typed {typedState}</span>
      <span class="chip">writer {text(writer, 'unknown')}</span>
      <span class="chip" class:stale-chip={leaseFreshness !== 'current'}>lease {text(leaseFreshness, 'unclaimed')}</span>
      <span class="chip">status {text(status.status ?? status.work_loop?.status, 'unknown')}</span>
      <span class="chip">transport {text(transport.status, 'unknown')}</span>
    </div>
  </div>

  <div class="grid">
    <article class="panel">
      <div class="label">Execution partition</div>
      <p>{text(partition.project_root_key, 'unbound project')}</p>
      <p class="muted">continuity {text(partition.workstream_key, 'unbound')} · work item {text(partition.work_item_key, 'unbound')}</p>
      <p class="muted">provider {text(partition.work_item_provider, 'unknown')} · workpoint {text(partition.workpoint_id, 'unbound')}</p>
      <p class="muted">transport {text(partition.transport_session_id, 'detached')} · item {text(partition.transport_work_item_id, 'unbound')}</p>
      <p class="muted">fence {text(partition.fencing_token, 'none')} · expires {text(partition.lease_expires_at, 'not leased')}</p>
      <p class="muted">budget {text(budget.state, 'unknown')} · wall clock {text(budget.remaining_wall_clock_ms, 'unbounded')} ms</p>
      {#if budget.exhaustion}
        <p class="warn">{text(budget.exhaustion.dimension)} exhausted · approved renew_budget resume required</p>
      {/if}
    </article>

    <article class="panel">
      <div class="label">Active task</div>
      {#if loopTaskStale}
        <p><span class="stale-chip">stale/unscoped</span> {text(activeTask.work_item_id ?? activeTask.id ?? status.current_work_item_id, 'no active task')}</p>
        <p class="muted">Hidden from current scope: {text(activeTask.title ?? activeTask.summary ?? status.current_task?.summary, 'no summary')}</p>
        <p class="muted">Loop authority is advisory or out-of-scope; current project root {text(projectRoot, 'not surfaced')}.</p>
      {:else}
        <p>{text(activeTask.work_item_id ?? activeTask.id ?? status.current_work_item_id, 'no active task')}</p>
        <p class="muted">{text(activeTask.title ?? activeTask.summary ?? status.current_task?.summary, 'no summary')}</p>
      {/if}
    </article>

    <article class="panel">
      <div class="label">Current Workpoint</div>
      <p>{text(currentWorkpoint.workpoint_id, 'no scoped Workpoint')}</p>
      <p class="muted">{text(currentWorkpoint.mission ?? currentWorkpoint.resume_packet?.mission, 'no scoped mission')}</p>
    </article>

    <article class="panel">
      <div class="label">Pause flags</div>
      {#if pauseFlags.length > 0}
        <ul>{#each pauseFlags.slice(0, 5) as item}<li>{text(item.flag ?? item.id ?? item)}</li>{/each}</ul>
      {:else}
        <p class="muted">No pause flags surfaced.</p>
      {/if}
    </article>

    <article class="panel wide">
      <div class="label">Recent checkpoints</div>
      {#if checkpoints.length > 0}
        <ul>{#each checkpoints.slice(0, 5) as item}<li>{text(item.summary ?? item.reason ?? item.id ?? item.checkpoint_id)}</li>{/each}</ul>
      {:else}
        <p class="muted">No checkpoints surfaced.</p>
      {/if}
    </article>
  </div>
</section>

<style>
  .workloop-peek { padding: var(--sp-3); display: flex; flex-direction: column; gap: var(--sp-3); }
  .peek-header { display: flex; align-items: flex-start; justify-content: space-between; gap: var(--sp-3); }
  .eyebrow, .label { font-size: 10px; font-weight: 700; color: var(--fg-tertiary); letter-spacing: 0.8px; text-transform: uppercase; }
  h2 { margin: 2px 0 0; color: var(--fg); font-size: var(--text-lg); line-height: 1.25; }
  .status-chip, .chip { border: 1px solid var(--border); border-radius: var(--r-full); color: var(--fg-secondary); background: var(--bg-elevated); font-family: var(--font-mono); }
  .status-chip { flex-shrink: 0; padding: 2px 7px; font-size: 10px; }
  .chip { display: inline-flex; align-items: center; min-height: 16px; padding: 1px 6px; font-size: 9px; }
  .stale-chip { display: inline-flex; align-items: center; border: 1px solid color-mix(in srgb, var(--orange) 45%, var(--border)); border-radius: var(--r-full); color: var(--orange); font-family: var(--font-mono); font-size: 9px; padding: 1px 6px; }
  .status-chip.ok, .summary-card.ok { border-color: color-mix(in srgb, var(--green) 40%, var(--border)); }
  .status-chip.ok { color: var(--green); }
  .status-chip.watch, .summary-card.watch { border-color: color-mix(in srgb, var(--orange) 45%, var(--border)); }
  .status-chip.watch { color: var(--orange); }
  .status-chip.bad, .summary-card.bad { border-color: color-mix(in srgb, var(--red) 45%, var(--border)); }
  .status-chip.bad { color: var(--red); }
  .summary-card, .panel { min-width: 0; padding: var(--sp-3); border: 1px solid var(--border); border-radius: var(--r-lg); background: var(--bg-panel); }
  .grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: var(--sp-2); }
  .wide { grid-column: 1 / -1; }
  .chips { display: flex; flex-wrap: wrap; gap: 4px; margin-top: var(--sp-2); }
  p { margin: var(--sp-1) 0 0; color: var(--fg-secondary); font-size: var(--text-sm); line-height: 1.45; }
  ul { margin: var(--sp-2) 0 0; padding-left: var(--sp-3); color: var(--fg-secondary); font-size: var(--text-xs); line-height: 1.45; }
  .muted { color: var(--fg-tertiary); font-size: var(--text-xs); }
</style>
