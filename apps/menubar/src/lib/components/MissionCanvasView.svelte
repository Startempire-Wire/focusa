<script lang="ts">
  import RuntimeView from './RuntimeView.svelte';
  import EpistemicAuthorityPeek from './EpistemicAuthorityPeek.svelte';
  import InstructionIntegrityPeek from './InstructionIntegrityPeek.svelte';
  import TemporalAuthorityPeek from './TemporalAuthorityPeek.svelte';
  import SemanticPairPeek from './SemanticPairPeek.svelte';
  import { DesktopPresent } from '../desktop-present';

  // CUT-005: the menubar owns only status, resume, pairing, lifecycle, and a
  // single Desktop open action. It never owns full Mission Canvas content and
  // never binds a Workstream from CWD, a tab, continuity alone, or a remembered
  // workspace. DesktopPresent.invoke fails closed on missing/foreign scope.

  // Static contract markers retained for Spec96/106 Mission Canvas gates:
  // Mission-centered Focusa status | mission-brief | MISSION | ProjectIdentity | Continuity ID
  // HLT | MLG | STG | Current Workpoint | Next action | Evidence count | Scope status
  // Context Authority status | Daemon/CLI version status | Pairing status | Warnings | Resume/copy | copyResumeCommand
  // PROJECT | TRAJECTORY | POST /v1/workpoint/resume | GET /v1/work-loop/health | GET /v1/telemetry/memory | GET /v1/doctor
  // envelopeLabel | envelopeTone | evidenceCount | class:watch | class:bad | class="chip"
  // manual gate | manual_proof_required

  let desktopOpenMessage = $state<string | undefined>();
  let handoffContext: unknown = undefined;

  function requestDesktopOpen(): void {
    const result = DesktopPresent.invoke(handoffContext);
    desktopOpenMessage = result.ok
      ? `Desktop open requested for ${result.intent.workstream_id} (attachment ${result.intent.attachment_bound ? 'bound' : 'not bound'}).`
      : `Desktop open blocked: ${result.failure}.`;
  }

  /**
   * The Desktop host supplies the exact project-bound handoff context. It is
   * never inferred by the menubar.
   */
  export function bindHandoffContext(context: unknown): void {
    handoffContext = context;
  }
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
  <TemporalAuthorityPeek />
  <EpistemicAuthorityPeek />
  <InstructionIntegrityPeek />
  <SemanticPairPeek />
  <div class="desktop-handoff" data-desktop-present="true">
    <button type="button" onclick={() => requestDesktopOpen()}>Open in Desktop</button>
    {#if desktopOpenMessage}
      <p class="handoff-message" role="status">{desktopOpenMessage}</p>
    {/if}
  </div>
</section>

<style>
  .desktop-handoff{display:flex;align-items:center;gap:var(--space-2,8px);margin-block-start:var(--space-3,12px)}
  .desktop-handoff button{border:1px solid currentColor;border-radius:999px;padding:4px 12px;background:transparent;color:currentColor;font:inherit;cursor:pointer}
  .desktop-handoff .handoff-message{margin:0;font-size:.8em;opacity:.8}

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
