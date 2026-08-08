<script lang="ts">
  import Icon from '$lib/ui/Icon.svelte';
  import StatusBadge from '$lib/ui/StatusBadge.svelte';
  import StatePanel from '$lib/ui/StatePanel.svelte';
  import type { DaemonReadStatus } from './daemon-health';

  let { open = $bindable(false), daemon }: { open?: boolean; daemon: DaemonReadStatus } = $props();
  const identitySteps = [
    ['ScopeRef / ProjectRootKey', 'No verified project scope'],
    ['WorkstreamId', 'No canonical Workstream selected'],
    ['ContinuityId (optional)', 'Lineage inside the Workstream'],
    ['AttachmentKey', 'No exact runtime Attachment'],
    ['SessionId / InstanceId', 'No temporal runtime identity'],
    ['WorkspaceBindingId', 'No Desktop workspace binding'],
    ['RuntimeObject', 'No bound runtime object'],
    ['WorkSurfaceId', 'No presentation surface binding']
  ] as const;
</script>

{#if open}
  <div class="context-panel" role="dialog" aria-label="Context Control">
    <header><div><span>Context Control</span><strong>Unbound authority</strong></div><button type="button" aria-label="Close Context Control" onclick={() => (open = false)}><Icon name="x" size={16}/></button></header>
    <div class="node-status"><span><Icon name="runtime" size={16}/><span><strong>Focusa daemon</strong><small>{daemon.detail}</small></span></span><StatusBadge tone={daemon.kind === 'read-only' ? 'ready' : daemon.kind === 'checking' ? 'watch' : 'error'} label={daemon.kind === 'read-only' ? 'Connected · read-only' : daemon.kind === 'checking' ? 'Checking' : 'Unavailable'}/></div>
    <div class="identity-ladder" aria-label="Required canonical identity">
      {#each identitySteps as step, index}
        <div><span class="step-icon">{index + 1}</span><span><strong>{step[0]}</strong><small>{step[1]}</small></span>{#if index < identitySteps.length - 1}<i aria-hidden="true"></i>{/if}</div>
      {/each}
    </div>
    <StatePanel state="blocked" title="Exact authority required" description="Desktop cannot infer authority from the current tab, CWD, latest record, remembered selection, or daemon-global state."/>
    <footer><button type="button" disabled><Icon name="scope" size={16}/>Connect verified Workstream</button><small>Unavailable until an exact generated identity binding is supplied.</small></footer>
  </div>
{/if}

<style>
  .context-panel{position:fixed;top:116px;right:calc(var(--sidebar-width,248px) + 16px);z-index:45;width:min(390px,calc(100vw - var(--sidebar-width,248px) - 40px));max-height:calc(100vh - 140px);display:grid;gap:var(--space-3);overflow-y:auto;padding:var(--space-3);border:1px solid var(--color-border-strong);border-radius:var(--radius-card);color:var(--color-text);background:var(--color-elevated);box-shadow:var(--shadow-popover)}
  header{display:flex;align-items:flex-start;justify-content:space-between;gap:var(--space-3)}header>div{display:grid;gap:2px}header span{color:var(--color-text-tertiary);font:var(--type-eyebrow);letter-spacing:.07em;text-transform:uppercase}header strong{font:var(--type-title)}header button{width:30px;height:30px;border:0;border-radius:var(--radius-control);color:var(--color-text-tertiary);background:transparent;font-size:18px;cursor:pointer}header button:hover{color:var(--color-text);background:color-mix(in srgb,var(--color-accent) 8%,transparent)}
  .node-status{display:flex;align-items:center;justify-content:space-between;gap:var(--space-3);padding:var(--space-3);border:1px solid var(--color-border);border-radius:var(--radius-card);background:var(--color-panel)}.node-status>span{min-width:0;display:flex;align-items:center;gap:var(--space-2);color:var(--color-accent-bright)}.node-status>span>span{min-width:0;display:grid}.node-status strong{color:var(--color-text-soft);font:var(--type-caption)}.node-status small{overflow:hidden;color:var(--color-text-tertiary);font-size:9px;text-overflow:ellipsis;white-space:nowrap}
  .identity-ladder{display:grid;padding:var(--space-2);border:1px solid var(--color-border);border-radius:var(--radius-card);background:var(--color-panel)}.identity-ladder>div{position:relative;min-height:42px;display:grid;grid-template-columns:24px 1fr;align-items:center;gap:var(--space-2)}.step-icon{z-index:1;width:20px;height:20px;display:grid;place-items:center;border:1px solid var(--color-border-strong);border-radius:50%;color:var(--color-text-tertiary);background:var(--color-raised);font-size:9px;font-variant-numeric:tabular-nums}.identity-ladder>div>span:nth-child(2){display:grid;gap:1px}.identity-ladder strong{color:var(--color-text-soft);font:var(--type-caption)}.identity-ladder small{color:var(--color-text-tertiary);font-size:9px}.identity-ladder i{position:absolute;top:31px;bottom:-11px;left:9px;width:1px;background:var(--color-border)}
  footer{display:grid;gap:var(--space-2);padding-top:var(--space-1);border-top:1px solid var(--color-border)}footer button{min-height:34px;display:flex;align-items:center;justify-content:center;gap:var(--space-2);border:0;border-radius:var(--radius-control);color:var(--color-text-secondary);background:var(--color-raised);font:var(--type-caption)}footer button:disabled{cursor:default;opacity:.48}footer small{color:var(--color-text-tertiary);font-size:9px;text-align:center}
  @media(max-width:820px){.context-panel{position:fixed;top:70px;left:76px;width:min(390px,calc(100vw - 92px));max-height:calc(100vh - 116px);right:auto}}
</style>
