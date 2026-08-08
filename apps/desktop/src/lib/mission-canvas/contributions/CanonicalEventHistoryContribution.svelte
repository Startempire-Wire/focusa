<script lang="ts">
  import { authorityFromProjection, sameWorkstreamAuthority } from '../exact-scope';
  import { MissionCanvasEventClient } from '../event-client';
  import { CanonicalEventHistory, MAX_CANONICAL_HISTORY_ROWS } from '../canonical-event-history';
  import type { MissionCanvasClient } from '../../../../../../docs/contracts/spec135/mission-canvas-v1/typescript/mission-canvas-client.generated';
  import type { ResolvedContribution, ResolvedWorkspaceProjection } from '../types';

  type HistoryLoadState = 'loading' | 'ready' | 'blocked' | 'error';

  interface Props {
    contribution: ResolvedContribution;
    projection: ResolvedWorkspaceProjection;
    client?: MissionCanvasClient;
    historyEvents?: readonly unknown[];
  }

  let {
    contribution,
    projection,
    client,
    historyEvents
  }: Props = $props();

  const authority = $derived(authorityFromProjection(projection));
  const seededRender = historyEvents !== undefined
    ? CanonicalEventHistory.render({
      projection,
      events: historyEvents,
      authority,
      maxRows: MAX_CANONICAL_HISTORY_ROWS
    })
    : undefined;

  let loadState = $state<HistoryLoadState>(
    seededRender
      ? (seededRender.rejected.length > 0 ? 'error' : 'ready')
      : client
        ? 'ready'
        : 'blocked'
  );
  let renderError = $state<string | undefined>(
    seededRender?.rejected[0]?.reason ??
      (client ? undefined : 'operation_unavailable:focusa.mission_canvas.events.stream')
  );
  let rows = $state(seededRender?.rows ?? CanonicalEventHistory.render({
    projection,
    events: [],
    maxRows: MAX_CANONICAL_HISTORY_ROWS
  }).rows);

  function applyEvents(events: readonly unknown[]): void {
    const result = CanonicalEventHistory.render({
      projection,
      events,
      authority,
      maxRows: MAX_CANONICAL_HISTORY_ROWS
    });
    rows = result.rows;
    renderError = result.rejected[0]?.reason;
    loadState = result.rejected.length > 0 ? 'error' : 'ready';
  }

  $effect(() => {
    if (!sameWorkstreamAuthority(authority, authorityFromProjection(projection))) {
      loadState = 'error';
      renderError = 'invalid_authority';
      rows = [];
      return;
    }

    if (historyEvents !== undefined) {
      if (!Array.isArray(historyEvents)) {
        loadState = 'error';
        renderError = 'invalid_history_events';
        rows = [];
        return;
      }
      applyEvents(historyEvents);
      return;
    }

    if (!client) {
      loadState = 'blocked';
      renderError = 'operation_unavailable:focusa.mission_canvas.events.stream';
      rows = [];
      return;
    }

    const eventScopeClient = new MissionCanvasEventClient(
      client,
      authority,
      {
        load: () => projection.durable_event_cursor,
        persist: () => undefined
      }
    );

    loadState = 'loading';
    renderError = undefined;
    rows = [];
    void (async () => {
      try {
        const events = await eventScopeClient.poll();
        applyEvents(events.accepted);
      } catch (error) {
        loadState = 'error';
        renderError = error instanceof Error ? error.message : 'failed_to_load_history';
      }
    })();
  });
</script>

<section
  class="history"
  aria-label={contribution.accessibility.label}
  data-contribution-id={contribution.contribution_id}
  data-history-status={loadState}
  data-history-count={rows.length}
>
  <header class="history-header">
    <strong>{contribution.accessibility.label}</strong>
    <span>{contribution.accessibility.description}</span>
    <small data-durable-cursor={projection.durable_event_cursor}>cursor: {projection.durable_event_cursor}</small>
  </header>

  {#if loadState === 'loading'}
    <p>Loading cursor-scoped history…</p>
  {:else if loadState === 'blocked'}
    <p data-history-fail-state="blocked">History stream is not available.</p>
  {:else if rows.length === 0}
    <p data-history-empty>No cursor-scoped history is available for this Workstream.</p>
  {:else}
    <ol class="history-events">
      {#each rows as row (row.event_id)}
        <li data-history-event={row.event_id} data-event-kind={row.event_kind} data-event-cursor={row.event_cursor}>
          <strong>{row.event_kind}</strong>
          <code>{row.event_cursor}</code>
          <time datetime={row.occurred_at}>{row.occurred_at}</time>
          <span>projection {row.projection_revision}</span>
        </li>
      {/each}
    </ol>
  {/if}

  {#if loadState === 'error' && renderError}
    <p data-history-error={renderError}>{renderError}</p>
  {/if}
</section>

<style>
  .history{display:grid;gap:var(--space-2);padding:var(--space-2);min-width:0}
  .history-header{display:grid;gap:2px}
  .history-header strong{color:var(--color-text);font:var(--type-label)}
  .history-header span,.history-header small{color:var(--color-text-tertiary);font:var(--type-caption)}
  .history-events{margin:0;padding-left:var(--space-4);display:grid;gap:var(--space-1)}
  .history-events li{display:grid;gap:3px;color:var(--color-text-secondary);font:var(--type-caption)}
</style>
