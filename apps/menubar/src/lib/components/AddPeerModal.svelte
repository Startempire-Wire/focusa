<!-- AddPeerModal.svelte — Modal dialog for adding sync peers -->
<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  export let isOpen = false;

  const dispatch = createEventDispatcher();

  let peerId = '';
  let name = '';
  let endpoint = '';
  let loading = false;
  let error: string | null = null;

  const API_BASE = 'http://127.0.0.1:8787';

  function reset() {
    peerId = '';
    name = '';
    endpoint = '';
    error = null;
  }

  function close() {
    reset();
    isOpen = false;
    dispatch('close');
  }

  async function handleSubmit() {
    if (!peerId || !name || !endpoint) {
      error = 'All fields are required';
      return;
    }

    // Validate URL
    try {
      new URL(endpoint);
    } catch {
      error = 'Invalid endpoint URL';
      return;
    }

    loading = true;
    error = null;

    try {
      const res = await fetch(`${API_BASE}/v1/sync/peers`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ peer_id: peerId, name, endpoint })
      });

      if (!res.ok) {
        const data = await res.json().catch(() => ({}));
        throw new Error(data.error || `HTTP ${res.status}`);
      }

      dispatch('success', { peerId, name, endpoint });
      close();
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to register peer';
    } finally {
      loading = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      close();
    }
  }

  $: if (isOpen) {
    peerId = '';
    name = '';
    endpoint = '';
    error = null;
  }
</script>

<svelte:window on:keydown={handleKeydown} />

{#if isOpen}
  <div class="modal-backdrop" on:click={close} role="presentation">
    <div class="modal" on:click|stopPropagation role="dialog" aria-modal="true">
      <header>
        <h3>Add Sync Peer</h3>
        <button class="close-btn" on:click={close} aria-label="Close">&times;</button>
      </header>

      <form on:submit|preventDefault={handleSubmit}>
        {#if error}
          <div class="error">{error}</div>
        {/if}

        <div class="field">
          <label for="peerId">Peer ID</label>
          <input
            id="peerId"
            type="text"
            bind:value={peerId}
            placeholder="e.g., macbook"
            disabled={loading}
            required
          />
          <span class="hint">Unique identifier for this device</span>
        </div>

        <div class="field">
          <label for="name">Display Name</label>
          <input
            id="name"
            type="text"
            bind:value={name}
            placeholder="e.g., MacBook Pro"
            disabled={loading}
            required
          />
        </div>

        <div class="field">
          <label for="endpoint">Endpoint URL</label>
          <input
            id="endpoint"
            type="url"
            bind:value={endpoint}
            placeholder="http://192.168.1.100:8787"
            disabled={loading}
            required
          />
          <span class="hint">HTTP URL of the peer's daemon</span>
        </div>

        <div class="actions">
          <button type="button" class="btn-cancel" on:click={close} disabled={loading}>
            Cancel
          </button>
          <button type="submit" class="btn-submit" disabled={loading}>
            {#if loading}
              Adding...
            {:else}
              Add Peer
            {/if}
          </button>
        </div>
      </form>
    </div>
  </div>
{/if}

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.7);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .modal {
    background: var(--surface-1, #1a1a1a);
    border-radius: 12px;
    padding: 24px;
    width: 90%;
    max-width: 400px;
    color: var(--text-primary, #fff);
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.5);
  }

  header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 20px;
  }

  h3 {
    margin: 0;
    font-size: 18px;
    font-weight: 600;
  }

  .close-btn {
    background: none;
    border: none;
    color: var(--text-secondary, #9ca3af);
    font-size: 24px;
    cursor: pointer;
    padding: 0;
    line-height: 1;
  }

  .close-btn:hover {
    color: var(--text-primary, #fff);
  }

  form {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  label {
    font-size: 13px;
    font-weight: 500;
    color: var(--text-secondary, #9ca3af);
  }

  input {
    padding: 10px 12px;
    border: 1px solid var(--surface-3, #333);
    border-radius: 6px;
    background: var(--surface-2, #252525);
    color: var(--text-primary, #fff);
    font-size: 14px;
  }

  input:focus {
    outline: none;
    border-color: var(--accent, #3b82f6);
  }

  input:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .hint {
    font-size: 11px;
    color: var(--text-muted, #6b7280);
  }

  .error {
    padding: 10px;
    background: rgba(239, 68, 68, 0.1);
    border: 1px solid rgba(239, 68, 68, 0.3);
    border-radius: 6px;
    color: #ef4444;
    font-size: 13px;
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 12px;
    margin-top: 8px;
  }

  button {
    padding: 10px 20px;
    border: none;
    border-radius: 6px;
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s;
  }

  .btn-cancel {
    background: var(--surface-3, #333);
    color: var(--text-primary, #fff);
  }

  .btn-cancel:hover:not(:disabled) {
    background: var(--surface-4, #444);
  }

  .btn-submit {
    background: var(--accent, #3b82f6);
    color: white;
  }

  .btn-submit:hover:not(:disabled) {
    background: var(--accent-hover, #2563eb);
  }

  button:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
</style>
