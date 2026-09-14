<script lang="ts">
  import { runtimeStore } from '../stores/runtime.svelte';
  import { SPEC138_OPERATIONS } from '../generated/spec138-operations';

  const value = (input: any, fallback = 'unknown') =>
    input === null || input === undefined || input === '' ? fallback : String(input);
  let source = $derived(runtimeStore.snapshot.predictionAuthority ?? {});
  let projection = $derived(source.projection ?? {});
  let profile = $derived(source.profile_conformance ?? {});
  let warnings = $derived([
    ...(Array.isArray(source.warnings) ? source.warnings : []),
    ...(Array.isArray(profile.warnings) ? profile.warnings : []),
  ]);
  let counts = $derived({
    questions: Object.keys(projection.questions ?? {}).length,
    commitments: Object.keys(projection.commitments ?? {}).length,
    outcomes: Object.keys(projection.outcome_authority_events ?? {}).length,
    evaluations: Object.keys(projection.evaluations ?? {}).length,
    learning: Object.keys(projection.learning ?? {}).length,
    transfers: Object.keys(projection.transfer_evaluations ?? projection.transfers ?? {}).length,
  });
</script>

<section class="epistemic" aria-labelledby="epistemic-heading" aria-live="polite">
  <div class="heading">
    <h2 id="epistemic-heading">Prediction & learning authority</h2>
    <span class:verified={source.status === 'completed'}>{value(source.status, 'unavailable')}</span>
  </div>
  <dl>
    <div><dt>Conformance</dt><dd>{value(profile.full_conformance_status, 'unknown')}</dd></div>
    <div><dt>Durability</dt><dd>{value(source.durability, 'unknown')}</dd></div>
    <div><dt>Events</dt><dd>{value(source.event_count, '0')}</dd></div>
    <div><dt>Questions</dt><dd>{counts.questions}</dd></div>
    <div><dt>Commitments</dt><dd>{counts.commitments}</dd></div>
    <div><dt>Outcomes</dt><dd>{counts.outcomes}</dd></div>
    <div><dt>Evaluations</dt><dd>{counts.evaluations}</dd></div>
    <div><dt>Learning</dt><dd>{counts.learning}</dd></div>
    <div><dt>Transfers</dt><dd>{counts.transfers}</dd></div>
    <div><dt>Legacy</dt><dd>{value(source.legacy_event_count, '0')}</dd></div>
  </dl>
  {#if warnings.length}
    <ul aria-label="Prediction authority warnings">
      {#each warnings as warning}<li>{value(warning)}</li>{/each}
    </ul>
  {/if}
  <details>
    <summary>{SPEC138_OPERATIONS.length} canonical operations · daemon authority</summary>
    <ul class="operations" aria-label="Canonical epistemic operations">
      {#each SPEC138_OPERATIONS as operation}
        <li><code>{operation.operation_id}</code> · {operation.method} {operation.path}</li>
      {/each}
    </ul>
  </details>
  {#if source.status !== 'completed'}
    <p class="recovery">Verify project scope, then use <code>focusa_prediction_authority</code> projection.</p>
  {/if}
</section>

<style>
  .epistemic { margin-top: .8rem; border: 1px solid var(--border, #334155); border-radius: .65rem; padding: .75rem; }
  .heading { display: flex; align-items: center; justify-content: space-between; gap: .75rem; }
  h2 { margin: 0; font-size: .95rem; }
  span { font-size: .72rem; color: #fbbf24; } span.verified { color: #86efac; }
  dl { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: .35rem .8rem; margin: .65rem 0 0; }
  dl div { min-width: 0; } dt { color: #94a3b8; font-size: .7rem; } dd { margin: .1rem 0 0; overflow-wrap: anywhere; }
  ul, .recovery { margin: .55rem 0 0; color: #fbbf24; font-size: .75rem; }
  details { margin-top: .65rem; font-size: .75rem; color: #94a3b8; }
  .operations { max-height: 10rem; overflow: auto; padding-left: 1.1rem; }
  code { color: #c4b5fd; }
</style>
