<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { saveConnection, setApiUrl } from '$lib/api';
  import QRCode from './QRCode.svelte';
  import Settings from './Settings.svelte';

  const OFFER_TTL_MS = 5 * 60_000;

  let nonce = $state('');
  let createdAt = $state(Date.now());
  let now = $state(Date.now());
  let showAdvanced = $state(false);
  let copiedErrors = $state(false);
  let completionPayload = $state('');
  let completionStatus = $state('');
  let tickHandle: ReturnType<typeof setInterval> | null = null;

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

  function refreshOffer() {
    nonce = randomNonce();
    createdAt = Date.now();
    now = createdAt;
  }

  const remainingLabel = $derived.by(() => {
    const remaining = Math.max(0, OFFER_TTL_MS - (now - createdAt));
    const seconds = Math.floor(remaining / 1000);
    const mm = Math.floor(seconds / 60);
    const ss = seconds % 60;
    return `${mm}:${ss.toString().padStart(2, '0')}`;
  });

  const offerPayload = $derived.by(() => JSON.stringify({
    protocol: 'focusa-connect-v1',
    role: 'mac_handoff_offer',
    mac_name: deviceName(),
    nonce,
    created_at: new Date(createdAt).toISOString(),
    expires_in_secs: Math.floor(OFFER_TTL_MS / 1000),
  }));

  async function applyCompletionPayload() {
    try {
      const payload = JSON.parse(completionPayload.trim());
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

  async function copyErrors() {
    const payload = [
      'Focusa first-run connect diagnostics',
      'note=Run focusa pair on the server first; scan this Mac code inside the Focusa Connect phone page.',
      `offer_nonce=${nonce || '(none)'}`,
      `offer_age_ms=${Date.now() - createdAt}`,
      `stored_server=${localStorage.getItem('focusa_api_url') || '(unset)'}`,
      `stored_public_pairing_url=${localStorage.getItem('focusa_public_pairing_url') || '(unset)'}`,
    ].join('\n');
    try {
      await navigator.clipboard.writeText(payload);
      copiedErrors = true;
      setTimeout(() => copiedErrors = false, 1500);
    } catch {
      window.prompt('Copy Focusa diagnostics:', payload);
    }
  }

  onMount(() => {
    refreshOffer();
    tickHandle = setInterval(() => {
      now = Date.now();
      if (now - createdAt > OFFER_TTL_MS) refreshOffer();
    }, 1000);
    return () => {
      if (tickHandle) clearInterval(tickHandle);
    };
  });
</script>

<section class="first-run-connect" aria-label="Connect to Focusa">
  <h2>Connect to Focusa</h2>
  <div class="qr-card">
    {#if nonce}
      <QRCode payload={offerPayload} size={260} />
    {/if}
  </div>
  <p class="primary-copy">Scan from Focusa Connect.</p>
  <p class="secondary-copy">Run focusa pair on the server, then scan here from the phone page · {remainingLabel}</p>

  <div class="utility-row">
    <button class="utility" onclick={copyErrors}>{copiedErrors ? 'Copied errors' : 'Copy errors'}</button>
    <details bind:open={showAdvanced}>
      <summary>Advanced</summary>
      {#if showAdvanced}
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
</style>
