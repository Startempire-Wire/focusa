<script lang="ts">
  import { runtimeStore } from '$lib/stores/runtime.svelte';

  const data = $derived(runtimeStore.snapshot.silentSessionDashboard?.data ?? runtimeStore.snapshot.silentSessionDashboard ?? null);
  const sessions = $derived(Array.isArray(data?.sessions) ? data.sessions.slice(0, 20) : []);
  const attentionCount = $derived(sessions.filter((session: any) => Boolean(session?.attention)).length);

  function short(value: unknown, max = 42): string {
    const text = String(value ?? 'unknown');
    return text.length <= max ? text : `${text.slice(0, max - 1)}…`;
  }
</script>

<section class="silent-card" aria-label="Daemon Silent Sessions">
  <header>
    <div>
      <h2>Silent Sessions</h2>
      <p>Daemon-backed · {sessions.length} visible · {attentionCount} need attention</p>
    </div>
    <span class:ok={data?.restart_safe} class="source">{data?.source ?? 'unavailable'}</span>
  </header>

  {#if sessions.length === 0}
    <p class="empty">No durable Silent Sessions are visible.</p>
  {:else}
    <div class="session-list">
      {#each sessions as session (session.session_id)}
        <article class:attention={Boolean(session.attention)}>
          <div class="title-row">
            <strong>{session.display_name ?? short(session.session_id)}</strong>
            <span>{session.lifecycle_state} · {session.health}</span>
          </div>
          <p>{short(session.work_item_ref ?? session.project_identity_ref, 64)}</p>
          <dl>
            <div><dt>Model</dt><dd>{short(session.model?.provider)}/{short(session.model?.model)}</dd></div>
            <div><dt>Activity</dt><dd>{short(session.current_activity?.activity ?? 'idle')}</dd></div>
            <div><dt>Elapsed</dt><dd>{session.elapsed_seconds ?? 0}s</dd></div>
            <div><dt>Checkpoint</dt><dd>{short(session.last_checkpoint_ref)}</dd></div>
          </dl>
          {#if session.attention}
            <p class="attention-line">Why: {session.attention}</p>
          {/if}
          <div class="handles">
            <span>Worktree: {short(session.workspace_root, 56)}</span>
            <span>Evidence: {session.evidence_refs?.length ?? 0}</span>
            <span>Completion: {session.completion_status}</span>
          </div>
          <details>
            <summary>Daemon controls and rehydrate handles</summary>
            <p>Controls: {(session.controls ?? []).join(', ') || 'none'}</p>
            <p>Run: {session.run_id ?? 'not started'} · generation {session.generation ?? 'n/a'}</p>
            <p>Recent events: {session.recent_events?.length ?? 0}. Open full output by daemon cursor/artifact; it is not inlined here.</p>
          </details>
        </article>
      {/each}
    </div>
  {/if}
</section>

<style>
  .silent-card { border: 1px solid var(--border, #30343b); border-radius: 12px; padding: 14px; background: var(--surface, #15171b); }
  header, .title-row { display: flex; align-items: center; justify-content: space-between; gap: 12px; }
  h2 { margin: 0; font-size: 15px; }
  header p, article p { margin: 3px 0; color: var(--muted, #9aa1aa); font-size: 12px; }
  .source { font-size: 11px; color: #d9a441; }
  .source.ok { color: #54c987; }
  .session-list { display: grid; gap: 9px; margin-top: 12px; }
  article { border: 1px solid var(--border, #30343b); border-radius: 9px; padding: 10px; }
  article.attention { border-color: #d9a441; }
  .title-row span, dt, dd, .handles, details { font-size: 11px; }
  dl { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 5px 12px; margin: 8px 0; }
  dl div { min-width: 0; }
  dt { color: var(--muted, #9aa1aa); }
  dd { margin: 1px 0 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .attention-line { color: #e7b652; }
  .handles { display: flex; flex-wrap: wrap; gap: 8px; color: var(--muted, #9aa1aa); }
  details { margin-top: 8px; }
  summary { cursor: pointer; }
  .empty { margin: 12px 0 0; color: var(--muted, #9aa1aa); }
</style>
