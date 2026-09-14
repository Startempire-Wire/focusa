<script lang="ts">
  import { runtimeStore } from '$lib/stores/runtime.svelte';

  let snapshot = $derived(runtimeStore.snapshot);
  let workpoint = $derived(snapshot.workpointResume ?? snapshot.workpoint ?? {});
  let temporalSource = $derived(
    workpoint.temporal_context ?? workpoint.resume_packet_v2?.temporal_context ?? snapshot.temporal ?? {}
  );
  let temporal = $derived(temporalSource.projection ?? temporalSource);
  let conformance = $derived(temporalSource.conformance ?? temporal.conformance ?? {});

  function value(input: unknown, fallback = 'none'): string {
    if (input === null || input === undefined || input === '') return fallback;
    return String(input);
  }

  let status = $derived(value(temporal.status, temporal.deadline_status ? 'projected' : 'unavailable'));
  let deadline = $derived(value(temporal.deadline_status));
  let slack = $derived(value(temporal.slack_ms, 'unknown'));
  let criticalPath = $derived(value(temporal.critical_path_ms, 'unknown'));
  let observations = $derived(value(temporal.observed_duration_count, '0'));
  let approaching = $derived(Array.isArray(temporal.approaching_deadlines) ? temporal.approaching_deadlines.length : 0);
  let conflict = $derived(value(temporal.deadline_conflict_state, 'unknown'));
  let calendar = $derived(value(temporal.human_calendar_context?.context_id, 'missing'));
  let forecast = $derived(temporal.authorized_forecast_range);
  let forecastLabel = $derived(
    forecast ? `${value(forecast.p50_ms)}–${value(forecast.p95_ms)} ms (${value(forecast.confidence)})` : 'none'
  );
  let urgency = $derived(temporal.urgency?.subject_ref ?? temporal.urgency?.reason_code ?? 'none');
  let lastProgress = $derived(value(temporal.last_material_progress_at, 'unknown'));
  let noProgressAge = $derived(value(temporal.no_progress_age_ms, 'unknown'));
  let lostTime = $derived(value(temporal.lost_time_incident_count, '0'));
  let opportunity = $derived(value(temporal.opportunity_posture, 'unknown'));
  let cancellation = $derived(value(temporal.cancellation_state, 'none'));
  let conformanceStatus = $derived(value(conformance.full_conformance_status, 'unknown'));
  let warnings = $derived([
    ...(Array.isArray(temporal.warnings) ? temporal.warnings : []),
    ...(Array.isArray(conformance.warnings) ? conformance.warnings : []),
  ]);
</script>

<section class="temporal" aria-labelledby="temporal-heading" aria-live="polite">
  <div class="heading">
    <h2 id="temporal-heading">Temporal authority</h2>
    <span class:verified={status === 'completed' || status === 'projected'}>{status}</span>
  </div>
  <dl>
    <div><dt>Calendar</dt><dd>{calendar}</dd></div>
    <div><dt>Deadline</dt><dd>{deadline}</dd></div>
    <div><dt>Slack</dt><dd>{slack} ms</dd></div>
    <div><dt>Critical path</dt><dd>{criticalPath} ms</dd></div>
    <div><dt>Observations</dt><dd>{observations}</dd></div>
    <div><dt>Approaching</dt><dd>{approaching}</dd></div>
    <div><dt>Conflict</dt><dd>{conflict}</dd></div>
    <div><dt>Forecast</dt><dd>{forecastLabel}</dd></div>
    <div><dt>Urgency</dt><dd>{value(urgency)}</dd></div>
    <div><dt>Last progress</dt><dd>{lastProgress}</dd></div>
    <div><dt>No-progress age</dt><dd>{noProgressAge} ms</dd></div>
    <div><dt>Lost-time incidents</dt><dd>{lostTime}</dd></div>
    <div><dt>Opportunity</dt><dd>{opportunity}</dd></div>
    <div><dt>Cancellation</dt><dd>{cancellation}</dd></div>
    <div><dt>Conformance</dt><dd>{conformanceStatus}</dd></div>
  </dl>
  {#if warnings.length}
    <ul aria-label="Temporal warnings">
      {#each warnings as warning}<li>{value(warning)}</li>{/each}
    </ul>
  {/if}
  {#if status === 'unavailable' || status === 'degraded'}
    <p class="warning">No exact temporal authority. Estimates and deadline arithmetic remain blocked or bounded.</p>
  {/if}
</section>

<style>
  .temporal { margin: .75rem 0; padding: .75rem; border: 1px solid var(--border, #3a3a3a); border-radius: .6rem; }
  .heading { display: flex; align-items: center; justify-content: space-between; gap: .5rem; }
  h2 { margin: 0; font-size: .95rem; }
  span { font-size: .72rem; color: var(--muted, #aaa); }
  span.verified { color: var(--success, #68d391); }
  dl { display: grid; grid-template-columns: 1fr 1fr; gap: .4rem .75rem; margin: .65rem 0 0; }
  dl div { min-width: 0; }
  dt { color: var(--muted, #aaa); font-size: .68rem; }
  dd { margin: .1rem 0 0; overflow-wrap: anywhere; font-size: .78rem; }
  ul { margin: .5rem 0 0; padding-left: 1.1rem; font-size: .74rem; }
  .warning { margin: .55rem 0 0; color: var(--warning, #f6ad55); font-size: .74rem; }
</style>
