<script lang="ts">
  import { normalizeToolResult } from '$lib/api';
  import { runtimeStore } from '$lib/stores/runtime.svelte';
  import { formatScopeForDisplay, type ScopeContext } from '$lib/projectContext.svelte';

  let s = $derived(runtimeStore.snapshot);
  let daemonOk = $derived(s.health?.ok === true);
  let doctor = $derived(s.doctor ?? {});
  let project = $derived(s.projectIdentity ?? {});
  let trajectory = $derived(s.trajectory ?? {});
  let workpoint = $derived(s.workpointResume ?? s.workpoint ?? {});
  let workLoop = $derived(s.workLoopHealth ?? s.workLoop ?? {});
  let memory = $derived(s.memoryTelemetry ?? {});
  let token = $derived(s.tokenBudget ?? {});
  let cache = $derived(s.cacheMetadata ?? {});
  let release = $derived(s.releaseProof ?? {});
  let pairing = $derived((s as any).pairing ?? (s as any).devicePairing ?? {});
  let contextAuthority = $derived((s as any).contextAuthority ?? (s as any).gate ?? {});

  let missionTitle = $derived(text(workpoint.mission ?? workpoint.resume_packet?.mission ?? trajectory.short_term_goal ?? trajectory.stg, 'No mission loaded'));
  let hlt = $derived(text(trajectory.hlt ?? trajectory.long_term_goal ?? trajectory.intelligence_view?.long_term_goal, 'HLT unavailable'));
  let mlg = $derived(text(trajectory.mlg ?? trajectory.mid_level_goal ?? trajectory.intelligence_view?.mid_level_goal, 'MLG unavailable'));
  let stg = $derived(text(trajectory.stg ?? trajectory.short_term_goal ?? trajectory.intelligence_view?.short_term_goal, 'STG unavailable'));
  let nextAction = $derived(text(workpoint.next_action ?? workpoint.next ?? trajectory.next_action ?? trajectory.gap, 'No next action'));
  let scopeStatus = $derived(text(project.status ?? project.scope_status ?? workpoint.scope_status, 'unknown'));
  let contextAuthorityStatus = $derived(text(contextAuthority.verdict ?? contextAuthority.status ?? contextAuthority.mode, 'unknown'));
  let pairingStatus = $derived(text(pairing.status ?? pairing.paired ?? pairing.device_id, 'unknown'));
  let daemonCliVersionStatus = $derived(`daemon=${text(s.health?.version, 'n/a')} cli=${text(release.cli_version ?? release.version, 'n/a')}`);
  let warningItems = $derived([
    !daemonOk ? 'Daemon unavailable' : null,
    project.status !== 'verified' ? 'Project identity not verified' : null,
    workpoint.canonical !== true ? 'Workpoint not canonical' : null,
    workpoint.degraded === true ? 'Workpoint degraded' : null,
    contextAuthorityStatus === 'unknown' ? 'Context Authority status unknown' : null,
  ].filter(Boolean));

  function text(v: any, fallback = 'unknown') {
    if (v === null || v === undefined || v === '') return fallback;
    if (typeof v === 'string') return v;
    return String(v);
  }

  function envelopeLabel(payload: any): string | null {
    const result = normalizeToolResult(payload);
    if (result.canonical === true) return 'canonical';
    if (result.degraded === true) return 'degraded';
    if (result.status) return result.status;
    return null;
  }

  function envelopeTone(payload: any): 'ok' | 'watch' | 'bad' | 'neutral' {
    const result = normalizeToolResult(payload);
    if (result.canonical === true || result.status === 'accepted' || result.status === 'ok') return 'ok';
    if (result.degraded === true || result.status === 'pending') return 'watch';
    if (result.canonical === false || result.status === 'blocked' || result.failure_class) return 'bad';
    return 'neutral';
  }

  function evidenceCount(payload: any): number {
    const refs = normalizeToolResult(payload).evidence_refs;
    return Array.isArray(refs) ? refs.length : 0;
  }

  async function copyResumeCommand() {
    const command = `focusa workpoint resume --project-root ${text(project.project_root ?? project.root, '/path/to/project')} --continuity-id ${text((s as any).session?.continuity_id ?? workpoint.continuity_id ?? trajectory.continuity_id, 'continuity-id')}`;
    await navigator.clipboard?.writeText(command);
  }
</script>

<section class="mission-brief" aria-label="Mission-centered Focusa status">
  <div class="mission-head">
    <div>
      <div class="label">MISSION</div>
      <h2>{missionTitle}</h2>
      <p>{nextAction}</p>
    </div>
    <button class="copy-btn" type="button" onclick={copyResumeCommand}>Resume/copy</button>
  </div>
  <div class="mission-fields">
    <div><span>ProjectIdentity</span><strong>{text(project.project_id ?? project.project?.id ?? project.canonical_name, 'unknown')}</strong></div>
    <div><span>Continuity ID</span><strong>{text((s as any).session?.continuity_id ?? workpoint.continuity_id ?? trajectory.continuity_id, 'unbound')}</strong></div>
    <div><span>HLT</span><strong>{hlt}</strong></div>
    <div><span>MLG</span><strong>{mlg}</strong></div>
    <div><span>STG</span><strong>{stg}</strong></div>
    <div><span>Current Workpoint</span><strong>{text(workpoint.workpoint_id ?? workpoint.id, 'none')}</strong></div>
    <div><span>Next action</span><strong>{nextAction}</strong></div>
    <div><span>Evidence count</span><strong>{evidenceCount(workpoint)}</strong></div>
    <div><span>Scope status</span><strong>{scopeStatus}</strong></div>
    <div><span>Context Authority status</span><strong>{contextAuthorityStatus}</strong></div>
    <div><span>Daemon/CLI version status</span><strong>{daemonCliVersionStatus}</strong></div>
    <div><span>Pairing status</span><strong>{pairingStatus}</strong></div>
  </div>
  <div class="warnings" class:clear={warningItems.length === 0}>
    <span>Warnings</span>
    <strong>{warningItems.length ? warningItems.join(' · ') : 'none'}</strong>
  </div>
</section>

<section class="mission-canvas-grid" aria-label="Focusa Mission Canvas runtime summary">
  <article class="card" class:ok={daemonOk} class:bad={!daemonOk}>
    <div class="label">DAEMON</div>
    <div class="value">{daemonOk ? 'Live' : 'Unavailable'}</div>
    <div class="meta">v{text(s.health?.version, 'n/a')} · {text(s.health?.uptime_ms, '0')}ms</div>
    <div class="chips"><span class="chip" class:ok={daemonOk}>{daemonOk ? 'ok' : 'offline'}</span></div>
    <code>curl /v1/health</code>
  </article>

  <article class="card" class:ok={project.status === 'verified'}>
    <div class="label">PROJECT</div>
    <div class="value">{text(project.project_id ?? project.project?.id ?? project.canonical_name, 'unknown')}</div>
    <div class="meta">{text(project.project_root ?? project.root ?? project.workspace_root, 'no verified root')}</div>
    <div class="chips"><span class="chip" class:ok={project.status === 'verified'}>{text(project.status, 'unknown')}</span></div>
    <code>GET /v1/project/identity</code>
  </article>

  <article class="card">
    <div class="label">TRAJECTORY</div>
    <div class="value">{text(trajectory.status ?? trajectory.posture ?? 'pending')}</div>
    <div class="meta">{text(trajectory.gap ?? trajectory.active_gap ?? trajectory.short_term_goal, 'no active gap')}</div>
    <div class="chips"><span class="chip" class:watch={trajectory.posture === 'verify_first'}>{text(trajectory.posture ?? trajectory.status, 'summary')}</span></div>
    <code>GET /v1/trajectory/view</code>
  </article>

  <article class="card" class:ok={workpoint.canonical === true} class:bad={workpoint.canonical === false || workpoint.degraded === true}>
    <div class="label">WORKPOINT</div>
    <div class="value">{text(workpoint.status ?? (workpoint.canonical ? 'canonical' : 'unknown'))}</div>
    <div class="meta">{text(workpoint.next_action ?? workpoint.next ?? workpoint.mission ?? workpoint.resume_packet?.mission, 'no mission')}</div>
    <div class="chips">
      {#if envelopeLabel(workpoint)}<span class="chip" class:ok={envelopeTone(workpoint) === 'ok'} class:watch={envelopeTone(workpoint) === 'watch'} class:bad={envelopeTone(workpoint) === 'bad'}>{envelopeLabel(workpoint)}</span>{/if}
      {#if evidenceCount(workpoint) > 0}<span class="chip">{evidenceCount(workpoint)} evidence</span>{/if}
    </div>
    <code>POST /v1/workpoint/resume</code>
  </article>

  <article class="card" class:watch={workLoop.dispatch_ready === false || workLoop.degraded === true}>
    <div class="label">WORK LOOP</div>
    <div class="value">{text(workLoop.dispatch_ready ?? workLoop.status ?? workLoop.work_loop?.status)}</div>
    <div class="meta">{text(workLoop.boundary_reason ?? workLoop.current_task?.id ?? workLoop.current_work_item_id, 'no active boundary')}</div>
    <div class="chips"><span class="chip" class:ok={workLoop.dispatch_ready === true} class:watch={workLoop.dispatch_ready === false}>{workLoop.dispatch_ready === true ? 'ready' : workLoop.dispatch_ready === false ? 'boundary' : 'unknown'}</span></div>
    <code>GET /v1/work-loop/health</code>
  </article>

  <article class="card">
    <div class="label">TOOL CONTRACTS</div>
    <div class="value">{s.ontologyContractsCount}</div>
    <div class="meta">{text(s.ontologyContractsVersion, 'no version')}</div>
    <code>node scripts/validate-focusa-tool-contracts.mjs</code>
  </article>

  <article class="card" class:watch={memory.pressure_status === 'lowmem' || memory.pressure_status === 'emergency'}>
    <div class="label">MEMORY</div>
    <div class="value">{text(memory.pressure_status ?? memory.status, 'normal')}</div>
    <div class="meta">rss {text(memory.rss_kb ?? memory.current_rss_kb, 'n/a')}kb · peak {text(memory.peak_rss_kb, 'n/a')}kb</div>
    <div class="chips"><span class="chip" class:watch={memory.pressure_status === 'lowmem'} class:bad={memory.pressure_status === 'emergency'}>{text(memory.pressure_status ?? memory.status, 'normal')}</span></div>
    <code>GET /v1/telemetry/memory</code>
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

  <article class="card" class:watch={release.status === 'manual_proof_required' || release.status === 'unavailable'}>
    <div class="label">RELEASE</div>
    <div class="value">{text(release.status, 'manual_proof_required')}</div>
    <div class="meta">{text(release.summary, 'No release-proof source wired yet; run proof before publish.')}</div>
    <div class="chips"><span class="chip" class:watch={release.status !== 'proven'}>{release.status === 'proven' ? 'proven' : 'manual gate'}</span></div>
    <code>focusa release prove --tag &lt;tag&gt;</code>
  </article>

  <article class="card">
    <div class="label">PREDICTIONS</div>
    <div class="value">{text(s.predictionsStats?.count ?? s.predictionsRecent?.length ?? s.predictionsStats?.total, 'none')}</div>
    <div class="meta">{text(s.predictionsStats?.accuracy_pct ?? s.predictionsStats?.summary, 'No prediction stats yet')}</div>
    <div class="chips"><span class="chip" class:ok={!s.predictionsStats?.degraded}>{text(s.predictionsStats?.status ?? 'ok', 'ok')}</span></div>
    <code>GET /v1/predictions/recent · /v1/predictions/stats</code>
  </article>

  <article class="card">
    <div class="label">METACOG</div>
    <div class="value">{text(s.metacogStatus?.total_evaluations ?? s.metacogStatus?.evaluation_count ?? s.metacogEvaluations?.length, 'none')}</div>
    <div class="meta">{text(s.metacogStatus?.last_status ?? s.metacogStatus?.status ?? s.metacogEvaluations?.[0]?.outcome, 'No evaluations yet')}</div>
    <div class="chips"><span class="chip" class:ok={s.metacogStatus?.status !== 'degraded' && s.metacogStatus?.status !== 'error'}>{text(s.metacogStatus?.status, 'unknown')}</span></div>
    <code>GET /v1/metacognition/status · /v1/metacognition/evaluations/recent</code>
  </article>

  <article class="card">
    <div class="label">SNAPSHOTS</div>
    <div class="value">{text(s.snapshotsRecent?.length ?? s.snapshotsRecent?.count, 'none')}</div>
    <div class="meta">{text(s.snapshotsRecent?.[0]?.created_at ?? s.snapshotsRecent?.[0]?.ts, 'No snapshots')}</div>
    <div class="chips"><span class="chip">{text(s.snapshotsRecent?.[0]?.kind ?? s.snapshotsRecent?.[0]?.type, 'none')}</span></div>
    <code>GET /v1/focus/snapshots/recent</code>
  </article>

  <article class="card">
    <div class="label">LINEAGE</div>
    <div class="value">{text(s.lineageHead?.id ?? s.lineageHead?.node_id ?? s.lineageHead?.head, 'none')}</div>
    <div class="meta">{text(s.lineageHead?.summary ?? s.lineageHead?.description ?? s.lineageHead?.updated_at, 'No lineage head')}</div>
    <div class="chips"><span class="chip">{text(s.lineageHead?.type ?? s.lineageHead?.kind, '—')}</span></div>
    <code>GET /v1/clt/nodes</code>
  </article>

  <article class="card" class:bad={!!runtimeStore.errorMsg || doctor.status === 'degraded'}>
    <div class="label">RECOVERY</div>
    <div class="value">{runtimeStore.errorMsg ? 'Holdover' : text(doctor.status, 'Ready')}</div>
    <div class="meta">{runtimeStore.errorMsg ?? text(doctor.summary ?? doctor.recommended_action, 'daemon reachable')}</div>
    <div class="chips"><span class="chip" class:ok={!runtimeStore.errorMsg && doctor.status !== 'degraded'} class:bad={!!runtimeStore.errorMsg || doctor.status === 'degraded'}>{runtimeStore.errorMsg ? 'error' : text(doctor.status, 'ready')}</span></div>
    <code>GET /v1/doctor</code>
  </article>
</section>

<style>
  .mission-brief {
    margin: var(--sp-3) var(--sp-3) 0;
    padding: var(--sp-3);
    border: 1px solid color-mix(in srgb, var(--accent) 35%, var(--border));
    border-radius: var(--r-lg);
    background: color-mix(in srgb, var(--accent) 8%, var(--bg-panel));
  }
  .mission-head {
    display: flex;
    justify-content: space-between;
    gap: var(--sp-3);
    align-items: flex-start;
    margin-bottom: var(--sp-3);
  }
  .mission-head h2 {
    margin: 0;
    color: var(--fg);
    font-size: var(--text-lg);
    line-height: 1.2;
  }
  .mission-head p {
    margin: var(--sp-1) 0 0;
    color: var(--fg-secondary);
    font-size: var(--text-xs);
    line-height: 1.35;
  }
  .copy-btn {
    flex: 0 0 auto;
    border: 1px solid var(--border);
    border-radius: var(--r-full);
    padding: 5px 9px;
    color: var(--fg);
    background: var(--bg-elevated);
    font-size: 10px;
    cursor: pointer;
  }
  .mission-fields {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: var(--sp-2);
  }
  .mission-fields div {
    min-width: 0;
    padding: var(--sp-2);
    border: 1px solid color-mix(in srgb, var(--border) 70%, transparent);
    border-radius: var(--r-sm);
    background: color-mix(in srgb, var(--bg-elevated) 65%, transparent);
  }
  .mission-fields span,
  .warnings span {
    display: block;
    margin-bottom: 3px;
    color: var(--fg-tertiary);
    font-size: 9px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }
  .mission-fields strong,
  .warnings strong {
    display: block;
    color: var(--fg);
    font-size: 11px;
    line-height: 1.25;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .warnings {
    margin-top: var(--sp-2);
    padding: var(--sp-2);
    border: 1px solid color-mix(in srgb, var(--orange) 45%, var(--border));
    border-radius: var(--r-sm);
    background: color-mix(in srgb, var(--orange) 8%, var(--bg-elevated));
  }
  .warnings.clear {
    border-color: color-mix(in srgb, var(--green) 35%, var(--border));
    background: color-mix(in srgb, var(--green) 6%, var(--bg-elevated));
  }
  .mission-canvas-grid {
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
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    margin-top: var(--sp-2);
  }
  .chip {
    display: inline-flex;
    align-items: center;
    min-height: 16px;
    padding: 1px 6px;
    border: 1px solid var(--border);
    border-radius: var(--r-full);
    color: var(--fg-tertiary);
    background: var(--bg-elevated);
    font-family: var(--font-mono);
    font-size: 9px;
    line-height: 1.3;
  }
  .chip.ok {
    color: var(--green);
    border-color: color-mix(in srgb, var(--green) 45%, var(--border));
  }
  .chip.watch {
    color: var(--orange);
    border-color: color-mix(in srgb, var(--orange) 50%, var(--border));
  }
  .chip.bad {
    color: var(--red);
    border-color: color-mix(in srgb, var(--red) 50%, var(--border));
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
