<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { PUBLIC_PAIRING_URL_KEY, getApiUrl, saveConnection, setApiUrl } from '$lib/api';
  import { diagnosticsStore, installGlobalDiagnostics, renderRedactedDebugBundle } from '$lib/stores/diagnostics.svelte';
  import QRCode from './QRCode.svelte';
  import Settings from './Settings.svelte';

  const OFFER_TTL_MS = 5 * 60_000;
  const POLL_INTERVAL_MS = 1500;

  let nonce = $state('');
  let createdAt = $state(Date.now());
  let now = $state(Date.now());
  let showAdvanced = $state(false);
  let copiedDebugBundle = $state(false);
  let completionPayload = $state('');
  let completionStatus = $state('');
  let callbackUrl = $state('');
  let callbackStatus = $state('');
  let pairUrl = $state('');
  let serverUrl = $state('');
  let connectId = $state('');
  let macName = $state('');
  let firstrunError = $state('');
  let tickHandle: ReturnType<typeof setInterval> | null = null;
  let callbackPollHandle: ReturnType<typeof setInterval> | null = null;

  function randomNonce(): string {
    const bytes = new Uint8Array(16);
    crypto.getRandomValues(bytes);
    return Array.from(bytes, (b) => b.toString(16).padStart(2, '0')).join('');
  }

  function deviceName(): string {
    try {
      return localStorage.getItem('focusa_device_name') || 'Focusa Mac';
    } catch {
      return 'Focusa Mac';
    }
  }

  async function refreshOffer() {
    createdAt = Date.now();
    nonce = randomNonce();
    now = createdAt;
    macName = deviceName();
    callbackUrl = '';
    pairUrl = '';
    connectId = '';
    firstrunError = '';
    callbackStatus = 'Creating room…';
    try {
      const stored = localStorage.getItem(PUBLIC_PAIRING_URL_KEY);
      const hint = stored && stored.trim().length > 0 ? stored : undefined;
      const resp = await fetch(new URL('/v1/connect/room/firstrun', getApiUrl()), {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({
          mac_name: macName,
          mac_nonce: nonce,
          server_url: hint,
        }),
      });
      if (!resp.ok) throw new Error(`firstrun HTTP ${resp.status}`);
      const body = await resp.json();
      pairUrl = body.pair_url || body.pair_url_qr_payload || '';
      connectId = body.room_id || body.connect_id || '';
      serverUrl = body.server_url || getApiUrl();
      callbackStatus = pairUrl
        ? 'Scan the QR with your phone camera.'
        : 'Server returned no pair URL.';
      if (callbackPollHandle) clearInterval(callbackPollHandle);
      callbackPollHandle = setInterval(() => pollRoomStatus(), POLL_INTERVAL_MS);
      pollRoomStatus();
    } catch (err) {
      firstrunError = err instanceof Error ? err.message : String(err);
      callbackStatus = `Could not create room: ${firstrunError}`;
    }
    void startBridgeCallback(nonce);
  }

  async function pollRoomStatus() {
    if (!connectId) return;
    const pollUrl = serverUrl
      ? new URL(`/v1/connect/room/${encodeURIComponent(connectId)}/status`, serverUrl)
      : new URL(`/v1/connect/room/${encodeURIComponent(connectId)}/status`, getApiUrl());
    try {
      const resp = await fetch(pollUrl, { headers: { accept: 'application/json' } });
      if (!resp.ok) return;
      const body = await resp.json();
      if (body.status === 'completed' && body.token) {
        if (callbackPollHandle) {
          clearInterval(callbackPollHandle);
          callbackPollHandle = null;
        }
        await completePairingFromRoom(body);
      }
    } catch {
      /* keep polling until TTL */
    }
  }

  async function completePairingFromRoom(body: {
    device_id?: string;
    device_name?: string;
    token: string;
    server_url?: string;
  }) {
    const token = body.token;
    const deviceId = body.device_id || connectId;
    const server = body.server_url || serverUrl || getApiUrl();
    setApiUrl(server);
    saveConnection(server, body.device_name || 'Focusa Mac');
    try {
      await invoke('focusa_save_pairing_token', { deviceId, token });
      completionStatus = 'Connected. Token stored in Keychain.';
    } catch (err) {
      localStorage.setItem('focusa_pairing_token_preview', String(token).slice(0, 6) + '…');
      completionStatus = `Connected locally; Keychain unavailable: ${err instanceof Error ? err.message : String(err)}`;
    }
    localStorage.setItem('focusa_device_id', deviceId);
    localStorage.setItem('focusa_has_connected_successfully', 'true');
    callbackStatus = 'Pairing complete. You can close the phone browser.';
  }

  const remainingLabel = $derived.by(() => {
    const remaining = Math.max(0, OFFER_TTL_MS - (now - createdAt));
    const seconds = Math.floor(remaining / 1000);
    const mm = Math.floor(seconds / 60);
    const ss = seconds % 60;
    return `${mm}:${ss.toString().padStart(2, '0')}`;
  });

  const offerPayload = $derived.by(() => pairUrl);

  async function applyCompletionPayloadText(raw: string) {
    try {
      const payload = JSON.parse(raw.trim());
      if (payload.protocol !== 'focusa-connect-v1' || payload.role !== 'mac_completion_payload') {
        throw new Error('Not a Focusa Mac completion payload');
      }
      if (!payload.server_url || !payload.device_id || !payload.token) {
        throw new Error('Completion payload missing server_url, device_id, or token');
      }
      setApiUrl(payload.server_url);
      saveConnection(payload.server_url, 'Focusa VPS');
      try {
        await invoke('focusa_save_pairing_token', { deviceId: payload.device_id, token: payload.token });
        completionStatus = 'Connected. Token stored in Keychain.';
      } catch (err) {
        localStorage.setItem('focusa_pairing_token_preview', String(payload.token).slice(0, 6) + '…');
        completionStatus = `Connected locally; Keychain unavailable: ${err instanceof Error ? err.message : String(err)}`;
      }
      localStorage.setItem('focusa_device_id', payload.device_id);
      localStorage.setItem('focusa_has_connected_successfully', 'true');
    } catch (err) {
      completionStatus = err instanceof Error ? err.message : String(err);
    }
  }

  async function applyCompletionPayload() {
    await applyCompletionPayloadText(completionPayload);
  }

  async function startBridgeCallback(_nextNonce: string) {
    // Fast-path callback kept as a UX nicety; the URL-QR poll loop is the
    // canonical completion path. If the bridge is unavailable we silently
    // rely on the poll loop, so this must not throw or replace callbackStatus.
    try {
      callbackUrl = await invoke<string>('focusa_start_bridge_callback', { nonce: _nextNonce });
    } catch {
      callbackUrl = '';
    }
  }

  async function copyDebugBundle() {
    const payload = renderRedactedDebugBundle({
      surface: 'first_run_connect',
      daemon_url: getApiUrl(),
      public_pairing_url: localStorage.getItem(PUBLIC_PAIRING_URL_KEY) || '(unset)',
      callback_status: callbackStatus,
      completion_status: completionStatus,
      mac_callback: callbackUrl || '(unavailable)',
      offer_nonce: nonce || '(none)',
      offer_age_ms: Date.now() - createdAt,
      // v0.9.35-dev context: VPS-initiated room, Mac joining
      connect_id: connectId || '(none)',
      pair_url: pairUrl ? `${pairUrl.slice(0, 64)}...` : '(none)',
      firstrun_error: firstrunError || '(none)',
      server_url: serverUrl || getApiUrl(),
      extra: {
        has_completion_payload: Boolean(completionPayload),
        stored_server: localStorage.getItem('focusa_api_url') || '(unset)',
        diagnostics_entry_count: diagnosticsStore.entries.length,
        latest_failure_class: diagnosticsStore.latest()?.error_class || '(none)',
      },
    });
    try {
      await navigator.clipboard.writeText(payload);
      copiedDebugBundle = true;
      setTimeout(() => copiedDebugBundle = false, 1500);
    } catch {
      window.prompt('Copy Focusa debug bundle:', payload);
    }
  }

  onMount(() => {
    installGlobalDiagnostics();
    refreshOffer();
    tickHandle = setInterval(() => {
      now = Date.now();
      if (now - createdAt > OFFER_TTL_MS) refreshOffer();
    }, 1000);
    return () => {
      if (tickHandle) clearInterval(tickHandle);
      if (callbackPollHandle) clearInterval(callbackPollHandle);
    };
  });
</script>

<section class="first-run-connect" aria-label="Connect to Focusa">
  <h2>Connect to Focusa</h2>
  <div class="qr-card">
    {#if nonce}
      <QRCode payload={pairUrl || ''} size={260} />
    {/if}
  </div>
  <p class="primary-copy">Scan this QR with your phone camera.</p>
  <p class="secondary-copy">Your browser will open a Focusa Connect page; tap Approve there. No app install required. · {remainingLabel}</p>
  <p class="advanced-copy">{callbackStatus}</p>

  <div class="utility-row">
    <button class="utility" onclick={copyDebugBundle}>{copiedDebugBundle ? 'Copied bundle' : 'Copy debug bundle'}</button>
    <details bind:open={showAdvanced}>
      <summary>Advanced</summary>
      {#if showAdvanced}
        <div class="manual-completion">
          <label for="completion-payload">Paste completion payload fallback</label>
          <textarea
            id="completion-payload"
            bind:value={completionPayload}
            placeholder='Paste mac_completion_payload JSON here if the automatic callback cannot reach this Mac'
            rows="5"
          ></textarea>
          <button class="utility" onclick={applyCompletionPayload}>Apply completion payload</button>
          {#if completionStatus}<p class="advanced-copy">{completionStatus}</p>{/if}
        </div>
        <Settings />
      {/if}
    </details>
  </div>
</section>

<style>
  .first-run-connect {
    min-height: 100%;
    padding: var(--sp-4);
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--sp-3);
    text-align: center;
  }
  h2 {
    margin: 0;
    font-size: var(--text-lg);
    font-weight: 700;
    letter-spacing: -0.02em;
  }
  .qr-card {
    width: 308px;
    height: 308px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: #fff;
    border-radius: 28px;
    box-shadow: 0 18px 44px rgba(0, 0, 0, 0.28);
  }
  .primary-copy,
  .secondary-copy {
    margin: 0;
  }
  .primary-copy {
    color: var(--fg);
    font-weight: 700;
    font-size: var(--text-base);
  }
  .secondary-copy {
    max-width: 300px;
    color: var(--fg-secondary);
    font-size: var(--text-sm);
    line-height: 1.35;
  }
  .utility-row {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--sp-2);
    flex-wrap: wrap;
  }
  .utility,
  summary {
    border: 0;
    background: transparent;
    color: var(--fg-tertiary);
    font: inherit;
    font-size: var(--text-xs);
    font-weight: 700;
    cursor: pointer;
  }
  details {
    text-align: left;
  }
  details[open] {
    width: min(360px, 100vw - 32px);
    border: 1px solid var(--border);
    border-radius: var(--r-md);
    background: var(--bg-panel);
    padding: var(--sp-2);
  }
  .manual-completion {
    display: grid;
    gap: var(--sp-2);
    margin-bottom: var(--sp-2);
  }
  .manual-completion label {
    color: var(--fg-secondary);
    font-size: var(--text-xs);
    font-weight: 700;
  }
  .manual-completion textarea {
    width: 100%;
    box-sizing: border-box;
    border: 1px solid var(--border);
    border-radius: var(--r-sm);
    background: var(--bg);
    color: var(--fg);
    font: var(--text-xs) ui-monospace, SFMono-Regular, Menlo, monospace;
    padding: var(--sp-2);
  }
</style>
