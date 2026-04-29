<script lang="ts">
  import { runtimeStore } from '$lib/stores/runtime.svelte';

  let s = $derived(runtimeStore.snapshot);
  let daemonOk = $derived(s.health?.ok === true);
  let workpoint = $derived(s.workpoint ?? {});
  let workLoop = $derived(s.workLoop ?? {});
  let token = $derived(s.tokenBudget ?? {});
  let cache = $derived(s.cacheMetadata ?? {});
  let release = $derived(s.releaseProof ?? {});

  function text(v: any, fallback = 'unknown') {
    if (v === null || v === undefined || v === '') return fallback;
    if (typeof v === 'string') return v;
    return String(v);
  }
</script>

<section class="mission-grid" aria-label="Focusa mission control">
  <article class="card" class:ok={daemonOk} class:bad={!daemonOk}>
    <div class="label">DAEMON</div>
    <div class="value">{daemonOk ? 'Live' : 'Unavailable'}</div>
    <div class="meta">v{text(s.health?.version, 'n/a')} · {text(s.health?.uptime_ms, '0')}ms</div>
    <code>curl /v1/health</code>
  </article>

  <article class="card">
    <div class="label">WORKPOINT</div>
    <div class="value">{text(workpoint.status ?? (workpoint.canonical ? 'canonical' : 'unknown'))}</div>
    <div class="meta">{text(workpoint.mission ?? workpoint.resume_packet?.mission, 'no mission')}</div>
    <code>focusa_workpoint_resume</code>
  </article>

  <article class="card">
    <div class="label">WORK LOOP</div>
    <div class="value">{text(workLoop.status ?? workLoop.work_loop?.status)}</div>
    <div class="meta">{text(workLoop.current_task?.id ?? workLoop.current_work_item_id, 'no active task')}</div>
    <code>focusa status --agent</code>
  </article>

  <article class="card">
    <div class="label">TOOL CONTRACTS</div>
    <div class="value">{s.ontologyContractsCount}</div>
    <div class="meta">{text(s.ontologyContractsVersion, 'no version')}</div>
    <code>node scripts/validate-focusa-tool-contracts.mjs</code>
  </article>

  <article class="card" class:watch={token.status === 'watch' || token.status === 'high' || token.status === 'critical'}>
    <div class="label">TOKENS</div>
    <div class="value">{text(token.status, 'pending')}</div>
    <div class="meta">{text(token.summary, 'no token records yet')}</div>
    <code>focusa tokens doctor</code>
  </article>

  <article class="card">
    <div class="label">CACHE</div>
    <div class="value">{text(cache.status, 'pending')}</div>
    <div class="meta">{text(cache.summary, 'no cache metadata yet')}</div>
    <code>focusa cache doctor</code>
  </article>

  <article class="card">
    <div class="label">RELEASE</div>
    <div class="value">{text(release.status, 'ready')}</div>
    <div class="meta">{text(release.summary, 'run proof before publish')}</div>
    <code>focusa release prove --tag &lt;tag&gt;</code>
  </article>

  <article class="card" class:bad={!!runtimeStore.errorMsg}>
    <div class="label">RECOVERY</div>
    <div class="value">{runtimeStore.errorMsg ? 'Holdover' : 'Ready'}</div>
    <div class="meta">{runtimeStore.errorMsg ?? 'daemon reachable'}</div>
    <code>systemctl restart focusa-daemon</code>
  </article>
</section>

<style>
  .mission-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: var(--sp-2);
    padding: var(--sp-3);
  }
  .card {
    background: var(--bg-panel);
    border: 1px solid var(--border);
    border-radius: var(--r-md);
    padding: var(--sp-3);
    min-width: 0;
  }
  .card.ok { border-color: color-mix(in srgb, var(--green) 45%, var(--border)); }
  .card.bad { border-color: color-mix(in srgb, var(--red) 45%, var(--border)); }
  .card.watch { border-color: color-mix(in srgb, var(--orange) 55%, var(--border)); }
  .label {
    font-size: 10px;
    font-weight: 700;
    color: var(--fg-tertiary);
    letter-spacing: 0.8px;
    margin-bottom: var(--sp-1);
  }
  .value {
    font-size: var(--text-lg);
    font-weight: 700;
    color: var(--fg);
    margin-bottom: var(--sp-1);
  }
  .meta {
    min-height: 32px;
    font-size: var(--text-xs);
    color: var(--fg-secondary);
    line-height: 1.35;
    overflow: hidden;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
  }
  code {
    display: block;
    margin-top: var(--sp-2);
    padding: var(--sp-1) var(--sp-2);
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--fg-secondary);
    background: var(--bg-elevated);
    border-radius: var(--r-sm);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
