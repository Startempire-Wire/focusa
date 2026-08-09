<script lang="ts">
  import Icon from '$lib/ui/Icon.svelte';
  import StatusBadge from '$lib/ui/StatusBadge.svelte';
  import PtyTerminal, { type PiTerminalBridge } from './PtyTerminal.svelte';
  import { hasExactPiAttachment, UNBOUND_PI_ATTACHMENT, type PiAttachmentProjection } from './pi-attachment-contract';
  import { readPiAttachmentStore } from './pi-attachment-store.svelte';

  const store = readPiAttachmentStore();

  let {
    attachment,
    bridge
  }: {
    attachment?: PiAttachmentProjection;
    bridge?: PiTerminalBridge;
  } = $props();

  // The store is the single source of truth: Mission Canvas and the Agent TUI
  // always show the same Pi session and Attachment, and the attachment state
  // survives view switches (Pi remains alive while Mission Canvas is visible).
  const sharedAttachment = $derived(attachment ?? store.state);

  async function interrupt(): Promise<void> {
    const current = sharedAttachment;
    if (!bridge || !current.canInterrupt || !hasExactPiAttachment(current)) return;
    await bridge.send({ kind: 'interrupt', attachment_id: current.identity.attachment_id });
  }
</script>

<section class="agent-surface" aria-label="Integrated Focusa Agent TUI">
  <header class="agent-header">
    <div class="agent-identity">
      <Icon name="terminal" size={18}/>
      <strong>Agent TUI</strong>
      <span>{sharedAttachment.runtimeLabel}</span>
    </div>
    <div class="runtime-state">
      <StatusBadge tone={sharedAttachment.state === 'attached' ? 'ready' : sharedAttachment.state === 'error' ? 'error' : 'neutral'} label={sharedAttachment.state}/>
      {#if sharedAttachment.canInterrupt && bridge && hasExactPiAttachment(sharedAttachment)}
        <button type="button" onclick={() => void interrupt()} aria-label="Interrupt Pi session">Interrupt</button>
      {/if}
    </div>
  </header>

  {#if hasExactPiAttachment(sharedAttachment) && bridge}
    <div class="terminal-frame">
      <PtyTerminal attachment={sharedAttachment} {bridge}/>
    </div>
  {:else}
    <div class="unavailable" role={sharedAttachment.state === 'error' ? 'alert' : 'status'}>
      <Icon name="terminal" size={20}/>
      <strong>{sharedAttachment.runtimeLabel}</strong>
      <span>{sharedAttachment.detail}</span>
      {#if hasExactPiAttachment(sharedAttachment) && !bridge}
        <span>The native PTY bridge is unavailable in this host.</span>
      {/if}
    </div>
  {/if}
</section>

<style>
  .agent-surface{display:grid;grid-template-rows:auto minmax(0,1fr);height:100%;min-height:0;overflow:hidden;background:var(--color-bg);color:var(--color-text)}
  .agent-header{display:flex;align-items:center;justify-content:space-between;gap:var(--space-4);padding:var(--space-2) var(--space-4);border-bottom:1px solid var(--color-border);background:var(--color-panel)}
  .agent-identity,.runtime-state{display:flex;align-items:center;gap:var(--space-2);min-width:0}
  .agent-identity{color:var(--color-accent-bright)}
  .agent-identity strong{color:var(--color-text)}
  .agent-identity span{overflow:hidden;color:var(--color-text-tertiary);font:var(--type-caption);text-overflow:ellipsis;white-space:nowrap}
  button{border:1px solid var(--color-border);border-radius:var(--radius-control);padding:var(--space-1) var(--space-3);background:transparent;color:var(--color-text-secondary);font:var(--type-caption);cursor:pointer}
  button:hover{border-color:var(--color-border-strong);color:var(--color-text)}
  .terminal-frame{min-width:0;min-height:0;padding:var(--space-2);background:var(--color-bg)}
  .unavailable{align-self:center;justify-self:center;display:grid;justify-items:center;gap:var(--space-2);max-width:36rem;padding:var(--layout-card-padding-roomy);color:var(--color-text-secondary);text-align:center}
  .unavailable strong{color:var(--color-text)}
</style>
