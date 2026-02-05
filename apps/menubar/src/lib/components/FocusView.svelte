<!--
  FocusView — main content view showing focus stack.
  Three states: disconnected, empty (no frames), active.
-->
<script lang="ts">
  import { focusStore } from '$lib/stores/focus.svelte';

  let active = $derived(focusStore.activeFrame);
  let paused = $derived(focusStore.pausedFrames);
  let conn = $derived(focusStore.connected);

  function timeAgo(iso: string): string {
    try {
      const ms = Date.now() - new Date(iso).getTime();
      const mins = Math.floor(ms / 60000);
      if (mins < 1) return 'just now';
      if (mins < 60) return `${mins}m ago`;
      const hrs = Math.floor(mins / 60);
      if (hrs < 24) return `${hrs}h ago`;
      return `${Math.floor(hrs / 24)}d ago`;
    } catch { return ''; }
  }

  function statusIcon(status: string): string {
    switch (status) {
      case 'active': return '●';
      case 'paused': return '◐';
      case 'completed': return '✓';
      case 'archived': return '◌';
      default: return '·';
    }
  }
</script>

{#if conn === 'disconnected' || conn === 'connecting'}
  <!-- Disconnected empty state -->
  <div class="empty-state">
    <div class="empty-icon">⊘</div>
    <div class="empty-title">Not Connected</div>
    <div class="empty-desc">
      Focusa daemon is not reachable.
    </div>
    <div class="empty-help">
      <p>Start the daemon on this machine:</p>
      <code class="cmd">focusa-daemon</code>
      <p class="hint">Or configure a remote server in Settings (⚙).</p>
    </div>
  </div>

{:else if conn === 'error'}
  <!-- Error state -->
  <div class="empty-state">
    <div class="empty-icon error">⚠</div>
    <div class="empty-title">Connection Error</div>
    <div class="empty-desc">{focusStore.errorMsg ?? 'Unknown error'}</div>
    <div class="empty-help">
      <p class="hint">Check Settings (⚙) for the correct API URL.</p>
    </div>
  </div>

{:else if focusStore.frameCount === 0}
  <!-- Connected but no focus frames -->
  <div class="empty-state">
    <div class="empty-icon calm">◎</div>
    <div class="empty-title">No Active Focus</div>
    <div class="empty-desc">
      The daemon is running but no focus frames exist yet.
    </div>
    <div class="empty-help">
      <p>Push a focus frame via CLI:</p>
      <code class="cmd">focusa push "My task" --goal "What to do"</code>
    </div>
  </div>

{:else}
  <!-- Active focus view -->
  <div class="focus-view">
    {#if active}
      <section>
        <div class="section-label">ACTIVE FOCUS</div>
        <div class="frame-card active">
          <div class="frame-header">
            <span class="frame-status" title={active.status}>{statusIcon(active.status)}</span>
            <span class="frame-title">{active.title}</span>
          </div>
          {#if active.goal}
            <div class="frame-goal">{active.goal}</div>
          {/if}
          <div class="frame-meta">
            <span title="Turns">{active.stats.turn_count} turns</span>
            <span class="meta-sep">·</span>
            <span>{timeAgo(active.updated_at)}</span>
            {#if active.beads_issue_id}
              <span class="meta-sep">·</span>
              <span class="mono">{active.beads_issue_id}</span>
            {/if}
          </div>

          <!-- Focus state details -->
          {#if active.focus_state.current_state}
            <div class="detail-row">
              <span class="detail-label">State</span>
              <span class="detail-value">{active.focus_state.current_state}</span>
            </div>
          {/if}

          {#if active.focus_state.next_steps.length > 0}
            <div class="detail-section">
              <span class="detail-label">Next Steps</span>
              <ul class="detail-list">
                {#each active.focus_state.next_steps.slice(0, 3) as step}
                  <li>{step}</li>
                {/each}
                {#if active.focus_state.next_steps.length > 3}
                  <li class="more">+{active.focus_state.next_steps.length - 3} more</li>
                {/if}
              </ul>
            </div>
          {/if}

          {#if active.focus_state.decisions.length > 0}
            <div class="detail-section">
              <span class="detail-label">Decisions</span>
              <ul class="detail-list">
                {#each active.focus_state.decisions.slice(0, 2) as dec}
                  <li>{dec}</li>
                {/each}
              </ul>
            </div>
          {/if}

          {#if active.focus_state.open_questions.length > 0}
            <div class="detail-section">
              <span class="detail-label">Open Questions</span>
              <ul class="detail-list">
                {#each active.focus_state.open_questions.slice(0, 2) as q}
                  <li>{q}</li>
                {/each}
              </ul>
            </div>
          {/if}

          {#if active.tags.length > 0}
            <div class="tags">
              {#each active.tags as tag}
                <span class="tag">{tag}</span>
              {/each}
            </div>
          {/if}
        </div>
      </section>
    {/if}

    {#if paused.length > 0}
      <section>
        <div class="section-label">PAUSED ({paused.length})</div>
        {#each paused as frame}
          <div class="frame-card paused">
            <div class="frame-header">
              <span class="frame-status paused" title={frame.status}>{statusIcon(frame.status)}</span>
              <span class="frame-title">{frame.title}</span>
            </div>
            {#if frame.goal}
              <div class="frame-goal">{frame.goal}</div>
            {/if}
            <div class="frame-meta">
              <span>{frame.stats.turn_count} turns</span>
              <span class="meta-sep">·</span>
              <span>{timeAgo(frame.updated_at)}</span>
            </div>
          </div>
        {/each}
      </section>
    {/if}

    <!-- Stack depth indicator -->
    <div class="stack-info">
      <span>Stack depth: {focusStore.frameCount}</span>
      <span>·</span>
      <span>v{focusStore.version}</span>
    </div>
  </div>
{/if}

<style>
  /* Empty states */
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: var(--sp-6) var(--sp-4);
    text-align: center;
    min-height: 300px;
  }

  .empty-icon {
    font-size: 40px;
    color: var(--fg-tertiary);
    margin-bottom: var(--sp-3);
    line-height: 1;
  }

  .empty-icon.error { color: var(--orange); }
  .empty-icon.calm { color: var(--accent); }

  .empty-title {
    font-size: var(--text-lg);
    font-weight: 600;
    color: var(--fg);
    margin-bottom: var(--sp-1);
  }

  .empty-desc {
    font-size: var(--text-sm);
    color: var(--fg-secondary);
    margin-bottom: var(--sp-4);
    max-width: 260px;
    line-height: 1.5;
  }

  .empty-help {
    font-size: var(--text-xs);
    color: var(--fg-secondary);
    line-height: 1.6;
  }

  .empty-help p { margin-bottom: var(--sp-1); }

  .cmd {
    display: block;
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: var(--r-sm);
    padding: var(--sp-2) var(--sp-3);
    margin: var(--sp-2) 0;
    color: var(--fg);
    word-break: break-all;
  }

  .hint {
    color: var(--fg-tertiary);
    font-size: var(--text-xs);
  }

  /* Focus view */
  .focus-view {
    padding: var(--sp-3);
    display: flex;
    flex-direction: column;
    gap: var(--sp-3);
  }

  .section-label {
    font-size: 10px;
    font-weight: 700;
    color: var(--fg-tertiary);
    letter-spacing: 0.8px;
    margin-bottom: var(--sp-2);
  }

  /* Frame cards */
  .frame-card {
    background: var(--bg-panel);
    border: 1px solid var(--border);
    border-radius: var(--r-md);
    padding: var(--sp-3);
    transition: background var(--dur-fast) var(--ease);
  }

  .frame-card.active {
    border-color: var(--accent);
    border-left: 3px solid var(--accent);
  }

  .frame-card.paused {
    opacity: 0.75;
  }

  .frame-card.paused:hover { opacity: 1; }

  .frame-header {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
  }

  .frame-status {
    font-size: var(--text-sm);
    color: var(--accent);
    flex-shrink: 0;
  }

  .frame-status.paused { color: var(--fg-tertiary); }

  .frame-title {
    font-size: var(--text-base);
    font-weight: 600;
    color: var(--fg);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .frame-goal {
    font-size: var(--text-sm);
    color: var(--fg-secondary);
    margin-top: var(--sp-1);
    line-height: 1.4;
  }

  .frame-meta {
    display: flex;
    align-items: center;
    gap: 4px;
    margin-top: var(--sp-2);
    font-size: var(--text-xs);
    color: var(--fg-tertiary);
  }

  .meta-sep { color: var(--fg-tertiary); }
  .mono { font-family: var(--font-mono); }

  /* Detail rows */
  .detail-row {
    display: flex;
    gap: var(--sp-2);
    margin-top: var(--sp-2);
    padding-top: var(--sp-2);
    border-top: 1px solid var(--border);
    font-size: var(--text-xs);
  }

  .detail-label {
    font-weight: 600;
    color: var(--fg-secondary);
    font-size: var(--text-xs);
    flex-shrink: 0;
  }

  .detail-value {
    color: var(--fg);
    font-size: var(--text-xs);
  }

  .detail-section {
    margin-top: var(--sp-2);
    padding-top: var(--sp-2);
    border-top: 1px solid var(--border);
  }

  .detail-list {
    list-style: none;
    padding: 0;
    margin-top: var(--sp-1);
  }

  .detail-list li {
    font-size: var(--text-xs);
    color: var(--fg);
    padding: 2px 0;
    padding-left: var(--sp-3);
    position: relative;
    line-height: 1.4;
  }

  .detail-list li::before {
    content: '→';
    position: absolute;
    left: 0;
    color: var(--fg-tertiary);
    font-size: 10px;
  }

  .detail-list li.more {
    color: var(--fg-tertiary);
    font-style: italic;
  }

  .detail-list li.more::before { content: ''; }

  .tags {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    margin-top: var(--sp-2);
  }

  .tag {
    font-size: 10px;
    font-weight: 500;
    color: var(--accent);
    background: var(--accent-dim);
    padding: 2px 6px;
    border-radius: var(--r-full);
  }

  /* Stack info footer */
  .stack-info {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: var(--sp-2) 0;
    font-size: 10px;
    color: var(--fg-tertiary);
    border-top: 1px solid var(--border);
    justify-content: center;
  }
</style>
