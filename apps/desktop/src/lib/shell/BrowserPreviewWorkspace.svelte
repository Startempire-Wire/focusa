<script lang="ts">
  import StatusBadge from '$lib/ui/StatusBadge.svelte';
  import type { WorkspaceManifestItem } from './workspace-manifest';

  type PreviewTone = 'ready' | 'watch' | 'neutral' | 'blocked';
  interface PreviewItem { label: string; value: string; detail: string; tone?: PreviewTone }
  interface PreviewSection { title: string; description: string; items: readonly PreviewItem[] }

  let { workspace }: { workspace: WorkspaceManifestItem } = $props();

  const SCREEN_SECTIONS: Readonly<Record<string, readonly PreviewSection[]>> = {
    crist: [
      { title: 'Reasoning frame', description: 'Structured C.R.I.S.T. stages for the selected Workstream.', items: [
        { label: 'Context', value: 'Bound', detail: 'Exact project and Workstream context', tone: 'ready' },
        { label: 'Role', value: 'Software implementation', detail: 'Current professional operating role' },
        { label: 'Intent', value: 'Mission Canvas completion', detail: 'Operator-steered implementation intent', tone: 'ready' },
        { label: 'Strategy', value: 'Functionality before polish', detail: 'Current bounded execution strategy' },
        { label: 'Task', value: 'Desktop workspace surfaces', detail: 'Active implementation frontier', tone: 'watch' }
      ]}
    ],
    'context-role': [
      { title: 'Exact context binding', description: 'Identity layers remain separate and inspectable.', items: [
        { label: 'ScopeRef', value: 'project:focusa-mission-canvas', detail: 'Project authority boundary', tone: 'ready' },
        { label: 'WorkstreamId', value: 'desktop-pivot', detail: 'Current logical workstream' },
        { label: 'ContinuityId', value: 'mission-canvas', detail: 'Cross-session continuity' },
        { label: 'AttachmentKey', value: 'Preview unavailable', detail: 'Native attachment is never inferred', tone: 'blocked' }
      ]}
    ],
    workpoints: [
      { title: 'Current Workpoint', description: 'Execution state, evidence posture, and bounded next action.', items: [
        { label: 'Mission', value: 'Desktop workspace implementation', detail: 'Build functional Workstream-aware surfaces', tone: 'ready' },
        { label: 'Current action', value: 'Complete workspace screens', detail: 'Identity-independent Svelte implementation', tone: 'watch' },
        { label: 'Verification', value: 'Desktop checks passing', detail: 'TypeScript and runtime contract coverage', tone: 'ready' },
        { label: 'Next action', value: 'Bind canonical runtime data', detail: 'Requires exact Workstream attachment' }
      ]}
    ],
    trajectory: [
      { title: 'Tactical trajectory', description: 'Goal hierarchy and progress markers without fabricated deadlines.', items: [
        { label: 'HLT', value: 'Primary Workstream-aware Focusa Desktop', detail: 'Long-term product trajectory', tone: 'ready' },
        { label: 'MLG', value: 'Complete functional workspace parity', detail: 'Current mid-level goal' },
        { label: 'STG', value: 'Activate all Desktop workspace surfaces', detail: 'Current short-term goal', tone: 'watch' },
        { label: 'Waypoint', value: 'Canonical runtime binding', detail: 'Next dependency-sensitive progress marker' }
      ]}
    ],
    sessions: [
      { title: 'Runtime sessions', description: 'Temporal sessions remain subordinate to exact Workstream attachments.', items: [
        { label: 'Browser preview', value: 'Active', detail: 'Development presentation host', tone: 'ready' },
        { label: 'Native Desktop', value: 'Not attached', detail: 'No native AttachmentKey selected', tone: 'neutral' },
        { label: 'Pi PTY', value: 'Awaiting attachment', detail: 'Real PTY activation requires exact identity', tone: 'blocked' }
      ]}
    ],
    contention: [
      { title: 'Contention and approvals', description: 'Mutation authority, writer posture, and explicit confirmation boundaries.', items: [
        { label: 'Active writer', value: 'None in preview', detail: 'Browser preview does not claim canonical writer authority', tone: 'neutral' },
        { label: 'Pending approvals', value: '0', detail: 'No preview mutations require operator approval', tone: 'ready' },
        { label: 'Explicit confirmations', value: 'Enforced', detail: 'Close and destructive operations remain confirmation-gated', tone: 'ready' }
      ]}
    ],
    evidence: [
      { title: 'Evidence and receipts', description: 'Stable proof references replace transcript-only claims.', items: [
        { label: 'Desktop contract', value: 'Passing', detail: 'Shell and authority contract verification', tone: 'ready' },
        { label: 'Projection runtime', value: 'Passing', detail: 'Layout, renderer, transport, draft, and event coverage', tone: 'ready' },
        { label: 'Browser proof', value: 'UIAI Engine', detail: 'Exclusive visual evaluation authority' }
      ]}
    ],
    documents: [
      { title: 'Workstream documents', description: 'Documents remain bound to explicit scope and shared-context policy.', items: [
        { label: 'Transition handoff', value: 'Available', detail: 'Desktop transition implementation authority', tone: 'ready' },
        { label: 'Executable callgraph', value: 'Available', detail: 'Atomic implementation graph and dependencies', tone: 'ready' },
        { label: 'Generated contracts', value: 'Available', detail: 'OpenAPI, clients, validators, and registries', tone: 'ready' }
      ]}
    ],
    research: [
      { title: 'Governed research', description: 'Browser execution remains isolated in UIAI Engine.', items: [
        { label: 'Browser authority', value: 'UIAI Engine', detail: 'Desktop renders exact artifacts but performs no browser actions', tone: 'ready' },
        { label: 'Artifact isolation', value: 'Enforced', detail: 'Session and artifact references remain explicit', tone: 'ready' },
        { label: 'Current research', value: 'None', detail: 'No unscoped browser task is running', tone: 'neutral' }
      ]}
    ],
    'agent-runtime': [
      { title: 'Runtime infrastructure', description: 'Infrastructure health is separate from cognitive authority.', items: [
        { label: 'Daemon', value: 'Connected · read-only', detail: 'Infrastructure read path is available', tone: 'ready' },
        { label: 'Desktop host', value: 'Browser preview', detail: 'Native Tauri commands are unavailable in this host', tone: 'watch' },
        { label: 'Pi PTY bridge', value: 'Not attached', detail: 'No ordinary child-process fallback is permitted', tone: 'blocked' }
      ]}
    ]
  };

  const sections = $derived(SCREEN_SECTIONS[workspace.id] ?? []);
</script>

<section class="preview-workspace" aria-label={workspace.label}>
  <header class="workspace-header">
    <div>
      <span class="eyebrow">Browser preview</span>
      <h1>{workspace.label}</h1>
      <p>{workspace.description}</p>
    </div>
    <StatusBadge tone="watch" label="preview data"/>
  </header>

  <div class="section-grid">
    {#each sections as section (section.title)}
      <section class="workspace-section">
        <header>
          <h2>{section.title}</h2>
          <p>{section.description}</p>
        </header>
        <div class="item-list">
          {#each section.items as item (item.label)}
            <article>
              <div class="item-heading">
                <strong>{item.label}</strong>
                {#if item.tone}<StatusBadge tone={item.tone} label={item.value}/>{:else}<span>{item.value}</span>{/if}
              </div>
              <p>{item.detail}</p>
            </article>
          {/each}
        </div>
      </section>
    {/each}
  </div>
</section>

<style>
  .preview-workspace{height:100%;min-height:0;overflow:auto;padding:clamp(var(--space-4),3vw,var(--space-7));background:var(--color-bg)}
  .workspace-header{display:flex;align-items:flex-start;justify-content:space-between;gap:var(--space-5);max-width:76rem;margin:0 auto var(--space-5)}
  .workspace-header>div{display:grid;gap:var(--space-2)}
  .eyebrow{color:var(--color-accent-bright);font:var(--type-caption);letter-spacing:.1em;text-transform:uppercase}
  h1,h2,p{margin:0}h1{color:var(--color-text);font:var(--type-title)}h2{color:var(--color-text);font:var(--type-heading)}
  .workspace-header p,.workspace-section header p,article p{color:var(--color-text-tertiary);font:var(--type-body)}
  .section-grid{display:grid;grid-template-columns:repeat(auto-fit,minmax(min(100%,28rem),1fr));gap:var(--layout-cluster-gap);max-width:76rem;margin:0 auto}
  .workspace-section{display:grid;align-content:start;gap:var(--space-4);border:1px solid var(--color-border);border-radius:var(--radius-panel);padding:var(--layout-card-padding-roomy);background:var(--color-panel)}
  .workspace-section>header{display:grid;gap:var(--space-2)}
  .item-list{display:grid;border-top:1px solid var(--color-border)}
  article{display:grid;gap:var(--space-1);padding:var(--space-3) 0;border-bottom:1px solid var(--color-border)}article:last-child{border-bottom:0}
  .item-heading{display:flex;align-items:center;justify-content:space-between;gap:var(--space-3)}
  .item-heading strong{color:var(--color-text);font:var(--type-label)}.item-heading>span{color:var(--color-text-secondary);font:var(--type-caption)}
  @media(max-width:640px){.workspace-header{display:grid}.preview-workspace{padding:var(--space-3)}}
</style>
