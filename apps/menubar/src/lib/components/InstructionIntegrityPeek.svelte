<script lang="ts">
  import { runtimeStore } from '../stores/runtime.svelte';
  let value = $derived(runtimeStore.snapshot.instructionIntegrity ?? {});
  let ready = $derived(value.status === 'available' && value.canonical === true);
</script>

<section class="integrity" aria-labelledby="instruction-integrity-heading" aria-live="polite">
  <div class="heading">
    <h2 id="instruction-integrity-heading">Instruction integrity</h2>
    <span class:verified={ready}>{ready ? 'available' : 'unavailable'}</span>
  </div>
  <dl>
    <div><dt>Mission Canvas authority</dt><dd>{value.mission_canvas_required === false ? 'independent' : 'unknown'}</dd></div>
    <div><dt>Authority outage</dt><dd>{value.dynamic_authority_outage_posture ?? 'unknown'}</dd></div>
    <div><dt>Amendment activation</dt><dd>{value.amendment_activation_requires ?? 'unknown'}</dd></div>
  </dl>
  {#if !ready}
    <p>Durable or consequential action blocked. Recover with <code>focusa_instruction_integrity_status</code>.</p>
  {/if}
</section>

<style>
  .integrity { margin-top: .8rem; border: 1px solid var(--border, #334155); border-radius: .65rem; padding: .75rem; }
  .heading { display: flex; justify-content: space-between; gap: .75rem; align-items: center; }
  h2 { margin: 0; font-size: .95rem; }
  span { color: #fbbf24; font-size: .72rem; } span.verified { color: #86efac; }
  dl { margin: .65rem 0 0; display: grid; gap: .45rem; }
  dt { color: #94a3b8; font-size: .7rem; } dd { margin: .1rem 0 0; overflow-wrap: anywhere; }
  p { color: #fbbf24; font-size: .75rem; margin: .55rem 0 0; }
  code { color: #c4b5fd; }
</style>
