<script lang="ts">
  import Icon from '$lib/ui/Icon.svelte';
  import StatusBadge from '$lib/ui/StatusBadge.svelte';
  import ThinkingOrb from '$lib/ui/ThinkingOrb.svelte';
  import { UNBOUND_PI_ATTACHMENT, type PiAttachmentProjection } from './pi-attachment-contract';

  let { attachment = UNBOUND_PI_ATTACHMENT }: { attachment?: PiAttachmentProjection } = $props();
  const tabs = ['Transcript', 'Work Rail', 'Evidence', 'Context'] as const;
  let activeTab = $state<(typeof tabs)[number]>('Transcript');
  let prompt = $state('');
</script>

<section class="agent-surface" aria-label="Integrated Focusa Agent TUI">
  <header class="agent-header">
    <div class="agent-identity">
      <span class="terminal-mark"><Icon name="terminal" size={18}/></span>
      <div><span>Agent TUI</span><strong>Integrated Pi Work Surface</strong></div>
    </div>
    <div class="runtime-state">
      <ThinkingOrb state={attachment.state === 'attached' ? 'idle' : attachment.state === 'error' ? 'error' : 'stale'} size={24} label="Pi runtime"/>
      <StatusBadge tone={attachment.state === 'attached' ? 'ready' : attachment.state === 'error' ? 'error' : 'watch'} label={attachment.state === 'attached' ? 'Attached' : 'Awaiting Attachment'}/>
    </div>
  </header>

  <div class="authority-strip" aria-label="Pi attachment authority">
    <span><small>Workstream</small><strong>{attachment.identity?.workstreamId ?? 'Unbound'}</strong></span>
    <i><Icon name="chevron-right" size={14}/></i>
    <span><small>Continuity</small><strong>{attachment.identity?.continuityId ?? 'Not selected'}</strong></span>
    <i><Icon name="chevron-right" size={14}/></i>
    <span><small>Attachment</small><strong>{attachment.identity?.attachmentKey ?? 'Not created'}</strong></span>
    <i><Icon name="chevron-right" size={14}/></i>
    <span><small>Work Surface</small><strong>{attachment.identity?.workSurfaceId ?? 'Inactive'}</strong></span>
  </div>

  <nav class="agent-tabs" aria-label="Agent TUI integrated views">
    {#each tabs as tab}<button type="button" class:active={activeTab === tab} aria-pressed={activeTab === tab} onclick={() => (activeTab = tab)}>{tab}</button>{/each}
  </nav>

  <div class="terminal-stage">
    <div class="terminal-toolbar">
      <span><i class="status-dot"></i>{attachment.runtimeLabel}</span>
      <span class="geometry">PTY · awaiting native bridge</span>
    </div>

    {#if activeTab === 'Transcript'}
      <div class="terminal-output" role="log" aria-label="Pi terminal transcript" aria-live="polite">
        <div class="system-line"><span>focusa</span><strong>Agent TUI integration shell</strong></div>
        <p>{attachment.detail}</p>
        <dl>
          <div><dt>authority</dt><dd>exact Attachment required</dd></div>
          <div><dt>writer</dt><dd>unavailable while unbound</dd></div>
          <div><dt>resume</dt><dd>no attached Pi session</dd></div>
          <div><dt>evidence</dt><dd>no scoped terminal Evidence</dd></div>
        </dl>
        <div class="terminal-cursor"><span aria-hidden="true">$</span><em>waiting for native PTY attachment</em><i aria-hidden="true"></i></div>
      </div>
    {:else if activeTab === 'Work Rail'}
      <div class="integrated-panel"><Icon name="target" size={20}/><div><strong>No active Workpoint</strong><p>The Work Rail will follow the exact attached Workstream; UI selection cannot create one.</p></div></div>
    {:else if activeTab === 'Evidence'}
      <div class="integrated-panel"><Icon name="evidence" size={20}/><div><strong>No scoped terminal Evidence</strong><p>Command, test, and artifact handles will appear after native PTY output is attached and verified.</p></div></div>
    {:else}
      <div class="integrated-panel"><Icon name="context" size={20}/><div><strong>No canonical Context projection</strong><p>Context remains unavailable until ScopeRef, Workstream, Continuity, and Attachment identity are exact.</p></div></div>
    {/if}
  </div>

  <div class="agent-composer">
    <div class="composer-meta"><span>Steer attached Pi</span><small>{attachment.canSteer ? 'Scoped steering enabled' : 'Unavailable while unbound'}</small></div>
    <div class="composer-row">
      <textarea bind:value={prompt} aria-label="Agent TUI steering prompt" placeholder="Attach a Pi runtime to send scoped steering…" disabled={!attachment.canSteer}></textarea>
      <button type="button" disabled={!attachment.canSteer || !prompt.trim()}><Icon name="chevron-right" size={16}/><span>Send</span></button>
    </div>
    <div class="lifecycle-actions" aria-label="Pi runtime lifecycle">
      <button type="button" disabled={!attachment.canInterrupt}><Icon name="blocked" size={14}/>Interrupt</button>
      <button type="button" disabled={attachment.state !== 'attached'}><Icon name="sessions" size={14}/>Resume</button>
      <button type="button" disabled={attachment.state !== 'attached'}><Icon name="terminal" size={14}/>Detach</button>
    </div>
  </div>
</section>

<style>
  .agent-surface{width:min(1180px,100%);height:calc(100vh - 148px);min-height:620px;margin:0 auto;display:grid;grid-template-rows:54px 48px 38px minmax(0,1fr) 154px;overflow:hidden;border:1px solid var(--color-border);border-radius:var(--radius-panel);background:var(--color-bg);box-shadow:var(--shadow-card)}
  .agent-header,.authority-strip,.agent-tabs,.terminal-toolbar{display:flex;align-items:center;border-bottom:1px solid var(--color-border)}
  .agent-header{justify-content:space-between;padding:0 var(--space-4);background:var(--color-panel)}.agent-identity,.runtime-state{display:flex;align-items:center;gap:var(--space-3)}.terminal-mark{width:32px;height:32px;display:grid;place-items:center;border:1px solid var(--color-border-strong);border-radius:var(--radius-control);color:var(--color-accent-bright);background:var(--color-raised)}.agent-identity>div{display:grid}.agent-identity span{color:var(--color-text-tertiary);font:var(--type-caption)}.agent-identity strong{color:var(--color-text);font:var(--type-title)}
  .authority-strip{gap:var(--space-3);padding:0 var(--space-4);overflow-x:auto;background:color-mix(in srgb,var(--color-panel) 78%,transparent)}.authority-strip>span{min-width:0;display:grid;gap:1px}.authority-strip small{color:var(--color-text-tertiary);font-size:9px}.authority-strip strong{max-width:190px;overflow:hidden;color:var(--color-text-soft);font:var(--type-caption);text-overflow:ellipsis;white-space:nowrap}.authority-strip>i{display:flex;color:var(--color-text-tertiary)}
  .agent-tabs{gap:2px;padding:0 var(--space-3);background:var(--color-panel)}.agent-tabs button{height:28px;padding:0 var(--space-3);border:0;border-radius:var(--radius-control);color:var(--color-text-tertiary);background:transparent;font:var(--type-caption);cursor:pointer;transition:color var(--motion-fast) var(--ease-out-quart),background var(--motion-fast) var(--ease-out-quart)}.agent-tabs button:hover{color:var(--color-text-soft)}.agent-tabs button.active{color:var(--color-text);background:color-mix(in srgb,var(--color-accent) 13%,var(--color-raised))}
  .terminal-stage{min-height:0;display:grid;grid-template-rows:34px minmax(0,1fr);background:#07090d}.terminal-toolbar{justify-content:space-between;padding:0 var(--space-3);color:var(--color-text-tertiary);background:#0c0f15;font:var(--type-caption)}.terminal-toolbar>span{display:flex;align-items:center;gap:var(--space-2)}.status-dot{width:6px;height:6px;border-radius:50%;background:var(--color-warning)}.geometry{font-family:var(--font-mono);font-size:9px}
  .terminal-output{padding:var(--space-5);overflow:auto;color:#b8c2d1;font:12px/1.65 var(--font-mono)}.system-line{display:flex;gap:var(--space-3);align-items:center}.system-line span{color:var(--color-cyan)}.system-line strong{color:var(--color-text-soft)}.terminal-output>p{max-width:68ch;margin:var(--space-3) 0;color:var(--color-text-tertiary)}dl{display:grid;gap:var(--space-1);margin:0}dl div{display:grid;grid-template-columns:100px 1fr}dt{color:var(--color-violet)}dd{margin:0;color:var(--color-text-tertiary)}.terminal-cursor{display:flex;align-items:center;gap:var(--space-2);margin-top:var(--space-5)}.terminal-cursor>span{color:var(--color-success)}.terminal-cursor em{color:var(--color-text-tertiary);font-style:normal}.terminal-cursor i{width:7px;height:14px;background:var(--color-text-secondary);opacity:.55;animation:cursor 1.2s step-end infinite}@keyframes cursor{50%{opacity:.08}}
  .integrated-panel{display:flex;align-items:flex-start;gap:var(--space-3);margin:var(--space-5);padding:var(--layout-card-padding-roomy);border:1px solid var(--color-border);border-radius:var(--radius-card);color:var(--color-warning);background:var(--color-panel)}.integrated-panel strong{color:var(--color-text-soft);font:var(--type-title)}.integrated-panel p{max-width:68ch;margin:var(--space-1) 0 0;color:var(--color-text-tertiary);font:var(--type-body)}
  .agent-composer{padding:var(--space-3);border-top:1px solid var(--color-border);background:var(--color-panel)}.composer-meta{display:flex;justify-content:space-between;margin-bottom:var(--space-2)}.composer-meta span{color:var(--color-text-soft);font:var(--type-caption)}.composer-meta small{color:var(--color-text-tertiary);font-size:9px}.composer-row{display:grid;grid-template-columns:1fr auto;gap:var(--space-2)}textarea{height:58px;resize:none;padding:var(--space-3);border:1px solid var(--color-border-strong);border-radius:var(--radius-control);outline:0;color:var(--color-text);background:#090c12;font:var(--type-body)}textarea:focus{border-color:var(--color-border-luminous)}.composer-row button{width:72px;display:grid;place-items:center;border:0;border-radius:var(--radius-control);color:white;background:var(--color-accent);cursor:pointer}.composer-row button span{font-size:9px}.composer-row button:disabled,textarea:disabled{cursor:default;opacity:.42}.lifecycle-actions{display:flex;gap:var(--space-2);margin-top:var(--space-2)}.lifecycle-actions button{display:flex;align-items:center;gap:6px;padding:3px 8px;border:1px solid var(--color-border);border-radius:var(--radius-control);color:var(--color-text-tertiary);background:transparent;font-size:9px}.lifecycle-actions button:disabled{opacity:.42}
  @media(prefers-reduced-motion:reduce){.terminal-cursor i{animation:none}}:global(:root[data-motion='reduced']) .terminal-cursor i{animation:none}
  @media(max-width:760px){.agent-surface{height:calc(100vh - 132px);min-height:560px;border-radius:var(--radius-card)}.authority-strip>i,.authority-strip span:nth-of-type(n+3){display:none}.runtime-state :global(span:not(.orb)){display:none}.agent-composer{padding:var(--space-2)}}
</style>
