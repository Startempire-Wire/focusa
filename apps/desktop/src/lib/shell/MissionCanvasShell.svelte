<script lang="ts">
  import Icon, { type IconName } from '$lib/ui/Icon.svelte';
  const surfaceTabs = ['Overview', 'Pi', 'UIAI Browser', 'Silent Session', 'Docs', 'Research', 'Evidence', 'Providers', 'Custom'];
  const activityModes = ['Overview', 'Context', 'Role', 'Interview', 'Spec', 'Tasks / Work', 'Sessions', 'Documents', 'Research', 'Evidence', 'History', 'Controls'];
  const activityIcons: Record<string, IconName> = { Overview: 'deck', Context: 'context', Role: 'scope', Interview: 'sessions', Spec: 'documents', 'Tasks / Work': 'target', Sessions: 'sessions', Documents: 'documents', Research: 'research', Evidence: 'evidence', History: 'route', Controls: 'settings' };
  let activeSurface = $state('Overview');
  let activeMode = $state('Overview');
</script>

<section class="canvas-frame" aria-label="Focusa Mission Canvas workspace">
  <div class="context-bar">
    <span class="f-mark">F</span>
    <span class="context-field"><small>Project</small><strong>Focusa Desktop</strong></span>
    <span class="context-field"><small>Workstream</small><strong>Unbound</strong></span>
    <span class="context-field"><small>Workspace</small><strong>Software Engineering</strong></span>
    <span class="session-state"><i></i> No Attachment</span>
    <span class="context-field pi-context"><small>Pi</small><strong>Not attached</strong></span>
    <span class="mode-token">{activeMode}</span>
  </div>

  <div class="surface-tabs" aria-label="Work Surface tabs">
    {#each surfaceTabs as tab}
      <button type="button" class:active={activeSurface === tab} onclick={() => (activeSurface = tab)}>{tab}</button>
    {/each}
  </div>

  <div class="canvas-body">
    <aside class="activity-rail" aria-label="Mission Canvas activity modes">
      {#each activityModes as mode}
        <button type="button" class:active={activeMode === mode} onclick={() => (activeMode = mode)}>
          <span aria-hidden="true"><Icon name={activityIcons[mode] ?? 'sparkles'} size={14}/></span>{mode}
        </button>
      {/each}
    </aside>

    <div class="composition">
      <header>
        <div>
          <span class="canvas-eyebrow">{activeSurface} surface</span>
          <h2>{activeMode} · {activeMode === 'Overview' ? 'Project Home' : 'Semantic Workspace'}</h2>
        </div>
        <span class="truth-chip">Unbound · presentation only</span>
      </header>

      <div class="status-grid">
        <article><i class="purple"></i><small>Mission status</small><strong>Awaiting Workstream</strong><span>No implicit current state</span></article>
        <article><i class="green"></i><small>Today's focus</small><strong>Not attached</strong><span>Exact Attachment required</span></article>
        <article><i class="blue"></i><small>Active work</small><strong>No Workpoint</strong><span>Projection unavailable</span></article>
        <article><i class="amber"></i><small>Evidence posture</small><strong>Unscoped</strong><span>No Evidence requested</span></article>
      </div>

      <div class="work-grid">
        <article class="primary-panel">
          <div class="panel-title"><span>{activeMode} workspace</span><small>read-only shell</small></div>
          <div class="code-lines" aria-label="Unbound semantic workspace">
            <span><em>01</em> ScopeRef <b><Icon name="chevron-right" size={14}/></b> WorkstreamId</span>
            <span><em>02</em> WorkstreamId <b><Icon name="chevron-right" size={14}/></b> ContinuityId</span>
            <span><em>03</em> ContinuityId <b><Icon name="chevron-right" size={14}/></b> AttachmentKey</span>
            <span><em>04</em> AttachmentKey <b><Icon name="chevron-right" size={14}/></b> Session / Instance</span>
            <span><em>05</em> Runtime object <b><Icon name="chevron-right" size={14}/></b> WorkSurfaceId</span>
          </div>
        </article>
        <article class="transcript-panel">
          <div class="panel-title"><span>Pi transcript</span><small>not attached</small></div>
          <p>Agent TUI remains a separate complete inner surface. An exact Attachment will connect its live transcript here without duplicating cognition.</p>
        </article>
      </div>

      <div class="queue-grid">
        <article><small>Steering queue</small><strong>No scoped steering</strong><span>Attach a Workstream before sending.</span></article>
        <article><small>Follow-up queue</small><strong>No scoped follow-up</strong><span>Nothing is inferred from UI selection.</span></article>
      </div>

      <div class="prompt-editor">
        <div><strong>Prompt Editor</strong><span>To: no Pi Attachment</span></div>
        <textarea aria-label="Prompt editor" placeholder="Attach an exact Workstream and Pi runtime to steer…" disabled></textarea>
        <button type="button" disabled>Send</button>
      </div>
    </div>
  </div>
</section>

<style>
  .canvas-frame{display:grid;grid-template-rows:38px 38px minmax(0,1fr);height:calc(100vh - 190px);min-height:610px;border:1px solid rgba(142,166,207,.2);border-radius:12px;overflow:hidden;background:var(--color-bg);color:var(--color-text-soft);box-shadow:0 20px 60px rgba(0,0,0,.28)}
  .context-bar,.surface-tabs{display:flex;align-items:center;gap:6px;padding:0 9px;border-bottom:1px solid rgba(130,154,194,.14);background:var(--color-panel);overflow-x:auto;white-space:nowrap}
  .f-mark{display:grid;place-items:center;width:22px;height:22px;border-radius:6px;background:var(--color-violet);color:white;font-size:11px;font-weight:800}
  .context-field{display:flex;align-items:center;gap:5px;height:24px;padding:0 8px;border:1px solid rgba(131,153,190,.12);border-radius:5px;background:var(--color-raised);font-size:9px}.context-field small{color:var(--color-text-tertiary)}.context-field strong{font-weight:560;color:var(--color-text-soft)}.session-state{display:flex;align-items:center;gap:5px;color:var(--color-text-tertiary);font-size:9px}.session-state i{width:7px;height:7px;border-radius:50%;background:var(--color-warning)}.mode-token{margin-left:auto;padding:3px 8px;border-radius:99px;background:color-mix(in srgb,var(--color-violet) 35%,var(--color-raised));color:var(--color-text-soft);font-size:8px}
  .surface-tabs button{padding:5px 12px;border:1px solid rgba(125,148,187,.12);border-radius:5px;color:var(--color-text-tertiary);background:var(--color-raised);font-size:9px;cursor:pointer}.surface-tabs button.active{color:var(--color-text);border-color:color-mix(in srgb,var(--color-violet) 42%,var(--color-raised));background:color-mix(in srgb,var(--color-violet) 24%,var(--color-raised))}
  .canvas-body{display:grid;grid-template-columns:142px minmax(0,1fr);min-height:0}.activity-rail{display:flex;flex-direction:column;gap:2px;padding:10px 8px;border-right:1px solid rgba(130,154,194,.14);background:var(--color-panel)}.activity-rail button{display:flex;gap:8px;align-items:center;padding:7px 9px;border:0;border-radius:5px;color:var(--color-text-tertiary);background:transparent;text-align:left;font-size:9px;cursor:pointer}.activity-rail button span{font-size:7px}.activity-rail button.active{color:var(--color-text);background:color-mix(in srgb,var(--color-violet) 28%,var(--color-raised));box-shadow:inset 0 0 0 1px rgba(157,119,228,.32)}
  .composition{display:grid;grid-template-rows:auto auto minmax(150px,1fr) auto 142px;gap:8px;min-width:0;padding:12px;overflow:auto}.composition header{display:flex;justify-content:space-between;align-items:center}.canvas-eyebrow{color:var(--color-text-tertiary);font-size:8px;text-transform:uppercase;letter-spacing:.12em}.composition h2{margin:3px 0 0;font-size:18px}.truth-chip{padding:5px 8px;border:1px solid rgba(204,161,80,.2);border-radius:99px;color:var(--color-warning);font-size:8px}
  .status-grid{display:grid;grid-template-columns:repeat(4,1fr);gap:7px}.status-grid article,.work-grid article,.queue-grid article{position:relative;padding:10px;border:1px solid rgba(128,151,188,.14);border-radius:7px;background:var(--color-panel)}.status-grid i{display:inline-block;width:10px;height:10px;margin-right:5px;border-radius:50%}.purple{background:var(--color-violet)}.green{background:var(--color-success)}.blue{background:var(--color-accent)}.amber{background:var(--color-warning)}.status-grid small,.status-grid strong,.status-grid span{display:block}.status-grid small{display:inline;color:var(--color-text-secondary);font-size:8px}.status-grid strong{margin:5px 0 2px;font-size:10px}.status-grid span{color:var(--color-text-tertiary);font-size:8px}
  .work-grid{display:grid;grid-template-columns:1.25fr .75fr;gap:8px}.panel-title{display:flex;justify-content:space-between;color:var(--color-text-secondary);font-size:9px}.panel-title small{color:var(--color-text-tertiary)}.code-lines{display:grid;gap:8px;margin-top:16px;font:10px ui-monospace,SFMono-Regular,Menlo,monospace;color:var(--color-success)}.code-lines em{display:inline-block;width:25px;color:var(--color-text-tertiary);font-style:normal}.code-lines b{display:inline-flex;vertical-align:middle;color:var(--color-violet)}.transcript-panel p{margin-top:18px;color:var(--color-text-tertiary);font-size:10px;line-height:1.6}
  .queue-grid{display:grid;grid-template-columns:1fr 1fr;gap:8px}.queue-grid small,.queue-grid strong,.queue-grid span{display:block}.queue-grid small{color:var(--color-text-secondary);font-size:8px}.queue-grid strong{margin:5px 0 2px;font-size:9px}.queue-grid span{color:var(--color-text-tertiary);font-size:8px}
  .prompt-editor{position:relative;padding:9px;border:1px solid rgba(130,154,194,.15);border-radius:var(--radius-control);background:var(--color-panel)}.prompt-editor>div{display:flex;align-items:center;gap:8px;font-size:9px}.prompt-editor>div span{padding:2px 6px;border-radius:99px;background:var(--color-raised);color:var(--color-text-tertiary);font-size:8px}.prompt-editor textarea{width:100%;height:86px;margin-top:8px;padding:9px;resize:none;border:1px solid rgba(135,91,211,.48);border-radius:5px;color:var(--color-text-tertiary);background:var(--color-bg);font:9px inherit}.prompt-editor button{position:absolute;right:15px;bottom:15px;border:0;border-radius:99px;padding:4px 9px;color:var(--color-text-soft);background:color-mix(in srgb,var(--color-violet) 52%,var(--color-raised));font-size:8px;opacity:.45}
  @media(max-width:900px){.canvas-frame{height:auto;min-height:720px}.canvas-body{grid-template-columns:105px minmax(0,1fr)}.status-grid{grid-template-columns:1fr 1fr}.work-grid{grid-template-columns:1fr}.composition{grid-template-rows:auto auto auto auto 142px}.pi-context{display:none}}
</style>
