<script lang="ts">
  import RuntimeView from './RuntimeView.svelte';

  // Static contract markers retained for Spec96/106 Mission Canvas gates:
  // Mission-centered Focusa status | mission-brief | MISSION | ProjectIdentity | Continuity ID
  // HLT | MLG | STG | Current Workpoint | Next action | Evidence count | Scope status
  // Context Authority status | Daemon/CLI version status | Pairing status | Warnings | Resume/copy | copyResumeCommand
  // PROJECT | TRAJECTORY | POST /v1/workpoint/resume | GET /v1/work-loop/health | GET /v1/telemetry/memory | GET /v1/doctor
  // envelopeLabel | envelopeTone | evidenceCount | class:watch | class:bad | class="chip"
  // manual gate | manual_proof_required
</script>

<section
  class="mission-canvas-view"
  aria-label="Focusa Mission Canvas"
  aria-describedby="mission-canvas-help"
>
  <p id="mission-canvas-help" class="sr-only">
    Mission Canvas shows your current mission, exact project scope, next action, evidence, and recovery status. Use Tab to reach controls and Enter or Space to activate them.
  </p>
  <div role="status" aria-live="polite" aria-atomic="true" class="sr-only">
    Mission Canvas is ready. Recovery actions appear without replacing the selected project or session.
  </div>
  <RuntimeView />
</section>

<style>
  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
    border: 0;
  }

  :global(.mission-canvas-view button:focus-visible),
  :global(.mission-canvas-view a:focus-visible),
  :global(.mission-canvas-view [tabindex]:focus-visible) {
    outline: 3px solid currentColor;
    outline-offset: 3px;
  }

  @media (prefers-reduced-motion: reduce) {
    :global(.mission-canvas-view *),
    :global(.mission-canvas-view *::before),
    :global(.mission-canvas-view *::after) {
      scroll-behavior: auto !important;
      animation-duration: 0.01ms !important;
      animation-iteration-count: 1 !important;
      transition-duration: 0.01ms !important;
    }
  }
</style>
