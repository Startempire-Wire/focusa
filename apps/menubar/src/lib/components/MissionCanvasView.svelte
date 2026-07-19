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

<section class="mission-canvas-view" aria-label="Focusa Mission Canvas">
  <RuntimeView />
</section>
