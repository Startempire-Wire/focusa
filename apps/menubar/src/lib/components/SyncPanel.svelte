<!-- SyncPanel.svelte — Multi-device sync status and controls (docs/43) -->
<script lang="ts">
  import { onMount } from 'svelte';
  import AddPeerModal from './AddPeerModal.svelte';

  interface Peer {
    peer_id: string;
    name: string;
    endpoint: string;
    status: 'pending' | 'connected' | 'error';
    last_seen_at?: string;
    created_at: string;
  }

  interface PeerStatus {
    peer_id: string;
    cursor?: {
      last_event_id?: string;
      last_event_ts?: string;
      updated_at: string;
    };
    backlog_estimate: number;
  }

  let peers: Peer[] = [];
  let peerStatuses: Map<string, PeerStatus> = new Map();
  let loading = true;
  let error: string | null = null;
  let showAddModal = false;
  let syncInProgress: Set<string> = new Set();
  let peerErrors: Map<string, string> = new Map();

  const API_BASE = 'http://127.0.0.1:8787';

  async function fetchPeers() {
    try {
      const res = await fetch(`${API_BASE}/v1/sync/peers`);
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const data = await res.json();
      peers = data.peers || [];
      peerErrors.clear();
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to fetch peers';
    } finally {
      loading = false;
    }
  }

  async function fetchPeerStatus(peerId: string) {
    try {
      const res = await fetch(`${API_BASE}/v1/sync/status/${peerId}`);
      if (!res.ok) {
        peerErrors.set(peerId, `HTTP ${res.status}`);
        peerStatuses.delete(peerId);
        return;
      }
      const data = await res.json();
      peerStatuses.set(peerId, data);
      peerErrors.delete(peerId);
    } catch (e) {
      peerErrors.set(peerId, e instanceof Error ? e.message : 'Network error');
      peerStatuses.delete(peerId);
    }
  }

  async function triggerSync(peerId: string) {
    syncInProgress.add(peerId);
    syncInProgress = syncInProgress;

    try {
      const res = await fetch(`${API_BASE}/v1/sync/pull/${peerId}`, { method: 'POST' });
      if (!res.ok) {
        peerErrors.set(peerId, `Sync failed: HTTP ${res.status}`);
      } else {
        peerErrors.delete(peerId);
      }
      await fetchPeerStatus(peerId);
    } catch (e) {
      peerErrors.set(peerId, e instanceof Error ? e.message : 'Network error');
    } finally {
      syncInProgress.delete(peerId);
      syncInProgress = syncInProgress;
    }
  }

  function formatLastSync(ts?: string): string {
    if (!ts) return 'Never';
    const date = new Date(ts);
    const now = new Date();
    const diff = now.getTime() - date.getTime();
    const minutes = Math.floor(diff / 60000);
    const hours = Math.floor(diff / 3600000);
    const days = Math.floor(diff / 86400000);

    if (minutes < 1) return 'Just now';
    if (minutes < 60) return `${minutes}m ago`;
    if (hours < 24) return `${hours}h ago`;
    return `${days}d ago`;
  }

  function formatTimestamp(ts: string): string {
    try {
      return new Date(ts).toLocaleString();
    } catch {
      return ts;
    }
  }

  function handlePeerAdded(e: CustomEvent) {
    peers = [...peers, {
      peer_id: e.detail.peerId,
      name: e.detail.name,
      endpoint: e.detail.endpoint,
      status: 'pending' as const,
      created_at: new Date().toISOString(),
    }];
    fetchPeerStatus(e.detail.peerId);
  }

  onMount(() => {
    fetchPeers();
    const interval = setInterval(() => {
      peers.forEach(p => fetchPeerStatus(p.peer_id));
    }, 30000); // Refresh every 30s
    return () => clearInterval(interval);
  });
</script>

<div class="sync-panel">
  <header>
    <h3>Sync</h3>
    <button class="btn-add" on:click={() => showAddModal = true} title="Add peer">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <line x1="12" y1="5" x2="12" y2="19"></line>
        <line x1="5" y1="12" x2="19" y2="12"></line>
      </svg>
    </button>
  </header>

  {#if loading}
    <div class="loading">
      <div class="spinner"></div>
      <span>Loading peers...</span>
    </div>
  {:else if error}
    <div class="error">
      <span>{error}</span>
      <button on:click={fetchPeers}>Retry</button>
    </div>
  {:else if peers.length === 0}
    <div class="empty">
      <p>No peers configured</p>
      <button class="btn-primary" on:click={() => showAddModal = true}>
        Add Your First Peer
      </button>
    </div>
  {:else}
    <ul class="peer-list">
      {#each peers as peer}
        {@const status = peerStatuses.get(peer.peer_id)}
        {@const syncBusy = syncInProgress.has(peer.peer_id)}
        {@const peerErr = peerErrors.get(peer.peer_id)}
        <li class="peer-item" class:connected={peer.status === 'connected'} class:error={peerErr}>
          <div class="peer-header">
            <span class="peer-name">{peer.name}</span>
            <span class="peer-status" class:status-pending={peer.status === 'pending'} class:status-connected={peer.status === 'connected'} class:status-error={peer.status === 'error'}>
              {peer.status}
            </span>
          </div>

          {#if peerErr}
            <div class="peer-error">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="12" cy="12" r="10"></circle>
                <line x1="12" y1="8" x2="12" y2="12"></line>
                <line x1="12" y1="16" x2="12.01" y2="16"></line>
              </svg>
              <span>{peerErr}</span>
              <button class="btn-retry" on:click={() => fetchPeerStatus(peer.peer_id)}>Retry</button>
            </div>
          {/if}

          <div class="peer-details">
            <span class="endpoint">{peer.endpoint}</span>
            {#if status}
              <span class="backlog" class:has-backlog={status.backlog_estimate > 0}>
                {#if status.backlog_estimate > 0}
                  <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M22 12h-4l-3 9L9 3l-3 9H2"></path>
                  </svg>
                {/if}
                {status.backlog_estimate} pending
              </span>
            {/if}
          </div>

          <div class="peer-meta">
            {#if status?.cursor?.updated_at}
              <span class="last-sync" title={formatTimestamp(status.cursor.updated_at)}>
                Last sync: {formatLastSync(status.cursor.updated_at)}
              </span>
            {:else}
              <span class="last-sync">Never synced</span>
            {/if}
          </div>

          <div class="peer-actions">
            <button
              class="btn-sync"
              class:syncing={syncBusy}
              on:click={() => triggerSync(peer.peer_id)}
              disabled={syncBusy}
            >
              {#if syncBusy}
                <span class="spinner-small"></span>
                Syncing...
              {:else}
                Sync now
              {/if}
            </button>
          </div>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<AddPeerModal bind:isOpen={showAddModal} on:success={handlePeerAdded} on:close={() => showAddModal = false} />

<style>
  .sync-panel {
    padding: 16px;
    background: var(--surface-1, #1a1a1a);
    border-radius: 12px;
    color: var(--text-primary, #fff);
    font-size: 13px;
  }

  header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 16px;
  }

  h3 {
    margin: 0;
    font-size: 16px;
    font-weight: 600;
  }

  .btn-add {
    width: 32px;
    height: 32px;
    border-radius: 8px;
    border: none;
    background: var(--accent, #3b82f6);
    color: white;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.15s;
  }

  .btn-add:hover {
    background: var(--accent-hover, #2563eb);
    transform: scale(1.05);
  }

  .loading, .error, .empty {
    padding: 32px;
    text-align: center;
    color: var(--text-secondary, #9ca3af);
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
  }

  .spinner, .spinner-small {
    width: 24px;
    height: 24px;
    border: 2px solid var(--surface-3, #333);
    border-top-color: var(--accent, #3b82f6);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  .spinner-small {
    width: 14px;
    height: 14px;
    border-width: 2px;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .error {
    color: #ef4444;
  }

  .error button, .empty button {
    margin-top: 8px;
    padding: 8px 16px;
    background: var(--surface-3, #333);
    border: none;
    border-radius: 6px;
    color: var(--text-primary, #fff);
    cursor: pointer;
  }

  .btn-primary {
    background: var(--accent, #3b82f6) !important;
    color: white !important;
  }

  .peer-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .peer-item {
    padding: 14px;
    background: var(--surface-2, #252525);
    border-radius: 10px;
    border-left: 3px solid var(--status-pending, #6b7280);
    transition: all 0.15s;
  }

  .peer-item:hover {
    background: var(--surface-2-hover, #2a2a2a);
  }

  .peer-item.connected {
    border-left-color: var(--status-connected, #22c55e);
  }

  .peer-item.error {
    border-left-color: var(--status-error, #ef4444);
  }

  .peer-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 8px;
  }

  .peer-name {
    font-weight: 600;
    font-size: 14px;
  }

  .peer-status {
    font-size: 10px;
    padding: 3px 8px;
    border-radius: 4px;
    text-transform: uppercase;
    font-weight: 600;
    letter-spacing: 0.5px;
  }

  .status-pending { background: rgba(107, 114, 128, 0.2); color: #9ca3af; }
  .status-connected { background: rgba(34, 197, 94, 0.2); color: #22c55e; }
  .status-error { background: rgba(239, 68, 68, 0.2); color: #ef4444; }

  .peer-error {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 8px 10px;
    background: rgba(239, 68, 68, 0.1);
    border-radius: 6px;
    margin-bottom: 8px;
    font-size: 11px;
    color: #ef4444;
  }

  .btn-retry {
    margin-left: auto;
    padding: 2px 8px;
    background: rgba(239, 68, 68, 0.2);
    border: none;
    border-radius: 4px;
    color: #ef4444;
    font-size: 10px;
    cursor: pointer;
  }

  .peer-details {
    display: flex;
    justify-content: space-between;
    font-size: 12px;
    color: var(--text-secondary, #9ca3af);
    margin-bottom: 6px;
  }

  .endpoint {
    font-family: 'SF Mono', Monaco, monospace;
    font-size: 11px;
    opacity: 0.8;
    max-width: 60%;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .backlog {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .backlog.has-backlog {
    color: #f59e0b;
    font-weight: 500;
  }

  .peer-meta {
    font-size: 11px;
    color: var(--text-muted, #6b7280);
    margin-bottom: 10px;
  }

  .peer-actions {
    display: flex;
    justify-content: flex-end;
  }

  .btn-sync {
    padding: 6px 14px;
    border: none;
    border-radius: 6px;
    background: var(--surface-3, #333);
    color: var(--text-primary, #fff);
    font-size: 12px;
    font-weight: 500;
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 6px;
    transition: all 0.15s;
  }

  .btn-sync:hover:not(:disabled) {
    background: var(--accent, #3b82f6);
  }

  .btn-sync.syncing {
    background: var(--surface-4, #444);
    cursor: wait;
  }

  .btn-sync:disabled {
    opacity: 0.7;
    cursor: not-allowed;
  }
</style>
