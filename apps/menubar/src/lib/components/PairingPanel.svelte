<!--
  PairingPanel — Mac menubar OAuth-like device pairing UI (focusa-ui0y).

  Three views, switched by the store state:
    - idle        → "Pair this Mac" button
    - waiting_vps → Big code display + "on your VPS run" command + countdown + cancel
    - completed   → "Paired" success card with device name + token expiry + revoke
    - expired     → "Code expired" card with "Generate a new code" button
    - error       → "Something went wrong" card with retry

  Below the active view, always shows the paired device list (with revoke buttons).
  macOS HIG-style: clean, calm, ambient, informative.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { PUBLIC_PAIRING_URL_KEY } from '$lib/api';
  import { pairingStore } from '$lib/stores/pairing.svelte';
  import QRCode from './QRCode.svelte';

  let { host = 'operator-vps' }: { host?: string } = $props();

  let deviceNameInput = $state(localStorage.getItem('focusa_device_name') || 'operator-mac');
  let copied = $state(false);
  let copiedErrorLog = $state(false);
  let now = $state(Date.now());
  // Apple-like default: QR scan first; manual CLI/code is fallback only.
  let handoffMode = $state<'A' | 'B' | 'C'>('B');

  // Tick once per second for the countdown
  let tickHandle: ReturnType<typeof setInterval> | null = null;
  onMount(() => {
    void pairingStore.bootstrapFromStorage();
    void pairingStore.list(host);
    tickHandle = setInterval(() => { now = Date.now(); }, 1_000);
    return () => {
      if (tickHandle) clearInterval(tickHandle);
    };
  });

  let activeDevices = $derived(pairingStore.paired.filter((d) => !d.revoked));
  let revokedDevices = $derived(pairingStore.paired.filter((d) => d.revoked));

  let remainingMs = $derived.by(() => {
    const s = pairingStore.state;
    if (s.kind !== 'waiting_vps') return 0;
    return Math.max(0, s.expiresAt - now);
  });
  let remainingLabel = $derived.by(() => {
    const s = Math.floor(remainingMs / 1_000);
    const mm = Math.floor(s / 60);
    const ss = s % 60;
    return `${mm}:${ss.toString().padStart(2, '0')}`;
  });

  async function copyToClipboard(text: string) {
    try {
      await navigator.clipboard.writeText(text);
      copied = true;
      setTimeout(() => (copied = false), 1_500);
    } catch {
      // Fallback: select-and-prompt
      window.prompt('Copy this code:', text);
    }
  }

  async function copyErrorLog(text?: string) {
    await copyToClipboard(text || 'No pairing diagnostic log available');
    copiedErrorLog = true;
    setTimeout(() => (copiedErrorLog = false), 1_500);
  }

  function saveDeviceName() {
    try { localStorage.setItem('focusa_device_name', deviceNameInput); } catch {}
  }

  function publicPairingUrl(): string | undefined {
    const value = localStorage.getItem(PUBLIC_PAIRING_URL_KEY)?.trim().replace(/\/$/, '');
    return value || undefined;
  }

  async function startPairing() {
    saveDeviceName();
    handoffMode = 'B';
    await pairingStore.start({ deviceName: deviceNameInput, daemonBaseUrl: publicPairingUrl() });
  }

  async function revokeAndClear(deviceId: string) {
    if (!confirm(`Revoke pairing for ${deviceId}? You'll need to re-pair to reconnect.`)) return;
    await pairingStore.revoke(deviceId, host, 'menubar-revoke');
  }
</script>

<div class="pairing">
  <header>
    <h2>Device Pairing</h2>
    <p class="sub">Connect this Mac to the Focusa daemon on the VPS.</p>
  </header>

  {#if pairingStore.state.kind === 'idle'}
    <section class="card hero-card">
      <p class="lead">Pair this Mac with a QR scan.</p>
      <p class="hint">Click once, scan the QR with your phone, then approve pairing on your Focusa server. No manual numbers unless QR is unavailable.</p>
      <button class="primary big" onclick={startPairing}>Show QR code</button>
      <details class="advanced">
        <summary>Advanced / fallback</summary>
        <label class="row">
          <span>Device name</span>
          <input
            type="text"
            bind:value={deviceNameInput}
            onblur={saveDeviceName}
            placeholder="operator-mac"
            maxlength="120"
          />
        </label>
        <p class="alt-hint">If QR opens the wrong host, set <strong>Public pairing URL for QR scans</strong> in Connect settings.</p>
      </details>
    </section>

  {:else if pairingStore.state.kind === 'starting'}
    <section class="card">
      <p class="lead">Generating pairing code…</p>
    </section>

  {:else if pairingStore.state.kind === 'waiting_vps'}
    {@const s = pairingStore.state}
    <section class="card code-card">
      <p class="lead">Complete pairing within <strong>{remainingLabel}</strong>:</p>
      {#if remainingMs <= 0}
        <p class="warn">Code expired. Generating a new one…</p>
      {/if}

      <div class="tabs" role="tablist" aria-label="Pairing options">
        <button
          role="tab"
          aria-selected={handoffMode === 'B'}
          class:active={handoffMode === 'B'}
          onclick={() => (handoffMode = 'B')}
          disabled={!s.pairUrl}>Scan QR</button>
        <button
          role="tab"
          aria-selected={handoffMode === 'C'}
          class:active={handoffMode === 'C'}
          onclick={() => (handoffMode = 'C')}
          disabled={!s.pairUrl}>Open URL</button>
        <button
          role="tab"
          aria-selected={handoffMode === 'A'}
          class:active={handoffMode === 'A'}
          onclick={() => (handoffMode = 'A')}>Manual CLI</button>
      </div>

      {#if handoffMode === 'A'}
        <p class="vps-label">On your VPS, run:</p>
        <div class="code-block">
          <code class="cmd">{s.onYourVpsRun}</code>
          <button class="ghost small" onclick={() => copyToClipboard(s.onYourVpsRun)}>
            {copied ? 'Copied' : 'Copy cmd'}
          </button>
        </div>
        <p class="alt-hint">Or just paste this code: <code class="code-inline">{s.code}</code></p>
      {:else}
        <div class="qr-wrap">
          <QRCode payload={s.pairUrlQrPayload || s.pairUrl} size={240} />
        </div>
        {#if handoffMode === 'B'}
          <p class="alt-hint">Scan with your phone. Opens a focusa-pairing page where you tap <em>Complete on this VPS</em> to finish.</p>
        {:else}
          <p class="alt-hint">Open this URL in a VPS browser or kiosk: <code class="code-inline">{s.pairUrl}</code></p>
        {/if}
        <details class="raw-url">
          <summary>Or paste the URL</summary>
          <code class="code-inline">{s.pairUrl}</code>
        </details>
      {/if}

      <p class="poll">Polling for completion (attempt {s.attempt + 1})…</p>
      <button class="ghost" onclick={() => pairingStore.reset()}>Cancel</button>
    </section>

  {:else if pairingStore.state.kind === 'completed'}
    {@const s = pairingStore.state}
    <section class="card ok-card">
      <p class="ok-head">Paired: {s.deviceName}</p>
      <dl class="meta">
        <dt>Device ID</dt><dd class="mono">{s.deviceId}</dd>
        <dt>Host</dt><dd>{s.host}</dd>
        <dt>Token expires</dt><dd>{s.tokenExpiresAt || 'unknown'}</dd>
        <dt>Token preview</dt><dd class="mono">{s.tokenPreview || 'stored in Keychain'}…</dd>
      </dl>
      <p class="hint">This Mac is now connected to the daemon. All Focusa calls will use the stored token.</p>
      <div class="row gap">
        <button class="ghost" onclick={() => pairingStore.reset()}>Pair a different Mac</button>
        <button class="danger" onclick={() => revokeAndClear(s.deviceId)}>Revoke</button>
      </div>
    </section>

  {:else if pairingStore.state.kind === 'expired'}
    {@const s = pairingStore.state}
    <section class="card warn-card">
      <p class="warn-head">Code expired</p>
      <p class="hint">{s.reason}</p>
      <button class="primary" onclick={() => pairingStore.reset()}>Generate a new code</button>
    </section>

  {:else if pairingStore.state.kind === 'error'}
    {@const s = pairingStore.state}
    <section class="card err-card">
      <p class="err-head">Pairing failed</p>
      <p class="hint">{s.message}</p>
      {#if s.failureClass}
        <p class="meta-line">failure_class: <code>{s.failureClass}</code></p>
      {/if}
      {#if s.diagnostic}
        <p class="meta-line">time: <code>{s.diagnostic.ts}</code></p>
        <p class="meta-line">class: <code>{s.diagnostic.error_class}</code> · phase: <code>{s.diagnostic.phase}</code></p>
      {/if}
      <div class="row-actions">
        <button class="secondary" onclick={() => copyErrorLog(s.diagnosticText)}>Copy error log</button>
        <button class="primary" onclick={() => pairingStore.reset()}>Try again</button>
      </div>
      {#if copiedErrorLog}<p class="copied">Error log copied.</p>{/if}
    </section>
  {/if}

  <section class="devices">
    <h3>Paired devices on {host}</h3>
    {#if pairingStore.paired.length === 0}
      <p class="empty">No paired devices yet.</p>
    {:else}
      <ul class="dev-list">
        {#if activeDevices.length > 0}
          <li class="dev-group">Active devices ({activeDevices.length})</li>
          {#each activeDevices as d (d.device_id + '-active')}
            <li class="dev active">
              <div class="dev-main">
                <div class="dev-name">{d.name}</div>
                <div class="dev-meta">{d.device_id.slice(0, 8)}… · paired {d.paired_at}</div>
              </div>
              <button class="ghost small" onclick={() => revokeAndClear(d.device_id)}>Revoke</button>
            </li>
          {/each}
        {/if}
        {#if revokedDevices.length > 0}
          <li class="dev-group muted">Revoked history ({revokedDevices.length})</li>
          {#each revokedDevices as d (d.device_id + '-revoked')}
            <li class="dev revoked">
              <div class="dev-main">
                <div class="dev-name">{d.name} <span class="badge">revoked</span></div>
                <div class="dev-meta">{d.device_id.slice(0, 8)}… · revoked {d.revoked_at}</div>
              </div>
            </li>
          {/each}
        {/if}
      </ul>
    {/if}
    <button class="ghost small" onclick={() => pairingStore.list(host)}>Refresh list</button>
  </section>
</div>

<style>
  .pairing {
    display: flex;
    flex-direction: column;
    gap: var(--sp-3);
    padding: var(--sp-3);
    color: var(--fg);
    font-family: var(--font);
  }
  header h2 {
    margin: 0;
    font-size: var(--text-lg);
    font-weight: 600;
  }
  .sub {
    margin: 0;
    font-size: var(--text-sm);
    color: var(--fg-secondary);
  }
  .card {
    background: var(--bg-panel);
    border: 1px solid var(--border);
    border-radius: var(--r-sm, 6px);
    padding: var(--sp-3);
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
  }
  .lead {
    margin: 0;
    font-size: var(--text-base);
    font-weight: 500;
  }
  .hint {
    margin: 0;
    font-size: var(--text-sm);
    color: var(--fg-secondary);
    line-height: 1.45;
  }
  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--sp-2);
  }
  .row.gap {
    gap: var(--sp-2);
  }
  .row > span {
    font-size: var(--text-sm);
    color: var(--fg-secondary);
  }
  input[type="text"] {
    flex: 1;
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 4px 8px;
    color: var(--fg);
    font-size: var(--text-base);
    font-family: var(--font);
  }
  input[type="text"]:focus {
    outline: 1px solid var(--accent);
  }
  button {
    font-family: var(--font);
    font-size: var(--text-base);
    border-radius: 4px;
    padding: 6px 12px;
    cursor: pointer;
    border: 1px solid var(--border);
    background: var(--bg-elevated);
    color: var(--fg);
  }
  button:hover { background: var(--bg-hover); }
  button.primary {
    background: var(--accent);
    color: white;
    border-color: var(--accent);
  }
  button.primary:hover { filter: brightness(1.05); }
  button.danger {
    background: var(--red);
    color: white;
    border-color: var(--red);
  }
  button.ghost {
    background: transparent;
  }
  button.small {
    font-size: var(--text-xs);
    padding: 2px 8px;
  }
  .code-card {
    border-color: var(--accent);
  }
  .code-block {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: var(--sp-2) var(--sp-3);
  }
  .code, .cmd {
    flex: 1;
    font-family: var(--font-mono);
    font-size: var(--text-base);
    color: var(--fg);
    user-select: all;
    word-break: break-all;
  }
  .code {
    font-size: var(--text-lg);
    font-weight: 600;
    letter-spacing: 0.5px;
  }
  .vps-label {
    margin: var(--sp-1) 0 0;
    font-size: var(--text-xs);
    color: var(--fg-secondary);
    text-transform: uppercase;
    letter-spacing: 0.4px;
  }
  .poll {
    margin: 0;
    font-size: var(--text-xs);
    color: var(--fg-tertiary);
  }
  .warn {
    margin: 0;
    color: var(--orange);
    font-size: var(--text-sm);
  }
  .ok-card {
    border-color: var(--green);
  }
  .ok-head {
    margin: 0;
    color: var(--green);
    font-weight: 600;
  }
  .err-card {
    border-color: var(--red);
  }
  .err-head {
    margin: 0;
    color: var(--red);
    font-weight: 600;
  }
  .warn-card {
    border-color: var(--orange);
  }
  .warn-head {
    margin: 0;
    color: var(--orange);
    font-weight: 600;
  }
  .meta-line {
    margin: 0;
    font-size: var(--text-xs);
    color: var(--fg-tertiary);
  }
  .meta-line code {
    font-family: var(--font-mono);
  }
  dl.meta {
    display: grid;
    grid-template-columns: max-content 1fr;
    gap: 4px var(--sp-2);
    margin: 0;
    font-size: var(--text-sm);
  }
  dl.meta dt { color: var(--fg-secondary); }
  dl.meta dd { margin: 0; }
  .mono { font-family: var(--font-mono); }
  .devices h3 {
    margin: 0;
    font-size: var(--text-sm);
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.4px;
    color: var(--fg-secondary);
  }
  .empty {
    margin: 0;
    font-size: var(--text-sm);
    color: var(--fg-tertiary);
  }
  .dev-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .dev-group {
    margin: var(--sp-2) 0 2px;
    color: var(--fg-secondary);
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 0.4px;
    text-transform: uppercase;
  }
  .dev-group.muted { color: var(--fg-tertiary); }
  .dev {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--sp-2);
    padding: var(--sp-2);
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: 4px;
  }
  .dev.revoked {
    opacity: 0.65;
    background: transparent;
    border-style: dashed;
  }
  .dev-name { font-size: var(--text-sm); font-weight: 500; }
  .dev-meta { font-size: var(--text-xs); color: var(--fg-secondary); font-family: var(--font-mono); }
  .badge {
    display: inline-block;
    background: var(--bg-hover);
    color: var(--fg-secondary);
    border: 1px solid var(--border);
    border-radius: 999px;
    padding: 0 6px;
    font-size: 10px;
    margin-left: 4px;
  }
  /* focusa-ui0y.10: tabs + QR display */
  .tabs {
    display: flex;
    gap: 4px;
    margin: 12px 0 16px;
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 2px;
  }
  .tabs button {
    flex: 1;
    background: transparent;
    border: 0;
    color: var(--fg-secondary);
    font-size: var(--text-xs);
    font-weight: 500;
    padding: 6px 8px;
    border-radius: 4px;
    cursor: pointer;
    transition: background 0.15s, color 0.15s;
  }
  .tabs button:hover:not(:disabled) { color: var(--fg-primary); }
  .tabs button.active {
    background: var(--bg-hover);
    color: var(--fg-primary);
  }
  .tabs button:disabled { opacity: 0.4; cursor: not-allowed; }
  .qr-wrap {
    display: flex;
    justify-content: center;
    margin: 16px 0;
    padding: 16px;
    background: white;
    border-radius: 8px;
  }
  /* QR itself renders dark on light; don't theme it */
  .qr-wrap :global(svg) { color: black; }
  .alt-hint {
    font-size: var(--text-xs);
    color: var(--fg-secondary);
    margin: 8px 0;
    line-height: 1.4;
  }
  .code-inline {
    font-family: var(--font-mono);
    font-size: 11px;
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: 3px;
    padding: 1px 4px;
    word-break: break-all;
  }
  .raw-url {
    margin-top: 8px;
    font-size: var(--text-xs);
    color: var(--fg-tertiary);
  }
  .raw-url summary { cursor: pointer; }
  .row-actions {
    display: flex;
    gap: 8px;
    align-items: center;
    margin-top: 12px;
  }
  button.secondary {
    background: var(--bg-elevated);
    color: var(--fg-primary);
    border: 1px solid var(--border);
  }
  .copied {
    font-size: var(--text-xs);
    color: var(--success, #22c55e);
    margin-top: 6px;
  }
</style>
