<!-- SyncPanel.svelte — Multi-device sync status and controls (docs/43) -->
<script lang="ts">
  import { onMount } from 'svelte';
  
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
  
  const API_BASE = 'http://127.0.0.1:8787';
  
  async function fetchPeers() {
    try {
      const res = await fetch(`${API_BASE}/v1/sync/peers`);
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const data = await res.json();
      peers = data.peers || [];
      
      // Fetch status for each peer
      for (const peer of peers) {
        fetchPeerStatus(peer.peer_id);
      }
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to fetch peers';
    } finally {
      loading = false;
    }
  }
  
  async function fetchPeerStatus(peerId: string) {
    try {
      const res = await fetch(`${API_BASE}/v1/sync/status/${peerId}`);
      if (!res.ok) return;
      const data = await res.json();
      peerStatuses.set(peerId, data);
      peerStatuses = peerStatuses; // Trigger reactivity
    } catch (e) {
      console.error(`Failed to fetch status for ${peerId}:`, e);
    }
  }
  
  async function triggerSync(peerId: string) {
    try {
      const res = await fetch(`${API_BASE}/v1/sync/pull/${peerId}`, { method: 'POST' });
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      await fetchPeerStatus(peerId);
    } catch (e) {
      alert(`Sync failed: ${e instanceof Error ? e.message : 'Unknown error'}`);
    }
  }
  
  async function registerPeer() {
    const peerId = prompt('Peer ID (e.g., macbook):');
    if (!peerId) return;
    const name = prompt('Peer name (e.g., MacBook):');
    if (!name) return;
    const endpoint = prompt('Peer endpoint (e.g., http://100.94.238.56:8787):');
    if (!endpoint) return;
    
    try {
      const res = await fetch(`${API_BASE}/v1/sync/peers`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ peer_id: peerId, name, endpoint })
      });
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      await fetchPeers();
    } catch (e) {
      alert(`Registration failed: ${e instanceof Error ? e.message : 'Unknown error'}`);
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
  
  onMount(() => {
    fetchPeers();
    const interval = setInterval(fetchPeers, 30000); // Refresh every 30s
    return () => clearInterval(interval);
  });
</script>

<div class="sync-panel">
  <header>
    <h3>Sync</h3>
    <button class="btn-add" on:click={registerPeer} title="Add peer">+</button>
  </header>
  
  {#if loading}
    <div class="loading">Loading...</div>
  {:else if error}
    <div class="error">{error}</div>
  {:else if peers.length === 0}
    <div class="empty">No peers configured</div>
  {:else}
    <ul class="peer-list">
      {#each peers as peer}
        {@const status = peerStatuses.get(peer.peer_id)}
        <li class="peer-item" class:connected={peer.status === 'connected'}>
          <div class="peer-header">
            <span class="peer-name">{peer.name}</span>
            <span class="peer-status" class:status-pending={peer.status === 'pending'} class:status-connected={peer.status === 'connected'} class:status-error={peer.status === 'error'}>
              {peer.status}
            </span>
          </div>
          <div class="peer-details">
            <span class="endpoint">{peer.endpoint}</span>
            {#if status}
              <span class="backlog" class:has-backlog={status.backlog_estimate > 0}>
                {status.backlog_estimate} pending
              </span>
            {/if}
          </div>
          <div class="peer-actions">
            <span class="last-sync">Last sync: {formatLastSync(status?.cursor?.updated_at)}</span>
            <button class="btn-sync" on:click={() => triggerSync(peer.peer_id)}>Sync now</button>
          </div>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .sync-panel {
    padding: 12px;
    background: var(--surface-1, #1a1a1a);
    border-radius: 8px;
    color: var(--text-primary, #fff);
    font-size: 13px;
  }
  
  header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 12px;
  }
  
  h3 {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
  }
  
  .btn-add {
    width: 24px;
    height: 24px;
    border-radius: 50%;
    border: none;
    background: var(--accent, #3b82f6);
    color: white;
    font-size: 16px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  
  .peer-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  
  .peer-item {
    padding: 10px;
    background: var(--surface-2, #252525);
    border-radius: 6px;
    border-left: 3px solid var(--status-pending, #6b7280);
  }
  
  .peer-item.connected {
    border-left-color: var(--status-connected, #22c55e);
  }
  
  .peer-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 4px;
  }
  
  .peer-name {
    font-weight: 500;
  }
  
  .peer-status {
    font-size: 11px;
    padding: 2px 6px;
    border-radius: 4px;
    text-transform: uppercase;
    background: var(--surface-3, #333);
  }
  
  .status-pending { background: #6b7280; }
  .status-connected { background: #22c55e; }
  .status-error { background: #ef4444; }
  
  .peer-details {
    display: flex;
    justify-content: space-between;
    font-size: 11px;
    color: var(--text-secondary, #9ca3af);
    margin-bottom: 8px;
  }
  
  .backlog.has-backlog {
    color: #f59e0b;
    font-weight: 500;
  }
  
  .peer-actions {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  
  .last-sync {
    font-size: 11px;
    color: var(--text-secondary, #9ca3af);
  }
  
  .btn-sync {
    padding: 4px 10px;
    border: none;
    border-radius: 4px;
    background: var(--surface-3, #333);
    color: var(--text-primary, #fff);
    font-size: 11px;
    cursor: pointer;
  }
  
  .btn-sync:hover {
    background: var(--accent, #3b82f6);
  }
  
  .loading, .error, .empty {
    padding: 20px;
    text-align: center;
    color: var(--text-secondary, #9ca3af);
  }
  
  .error {
    color: #ef4444;
  }
</style>
