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
  import { pairingStore } from '$lib/stores/pairing.svelte';

  let { host = 'operator-vps' }: { host?: string } = $props();

  let deviceNameInput = $state(localStorage.getItem('focusa_device_name') || 'operator-mac');
  let copied = $state(false);
  let now = $state(Date.now());

  // Tick once per second for the countdown
  let tickHandle: ReturnType<typeof setInterval> | null = null;
  onMount(() => {
    pairingStore.bootstrapFromStorage();
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

  function saveDeviceName() {
    try { localStorage.setItem('focusa_device_name', deviceNameInput); } catch {}
  }

  async function startPairing() {
    saveDeviceName();
    await pairingStore.start({ deviceName: deviceNameInput });
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
    <section class="card">
      <p class="lead">This Mac is not paired with a daemon.</p>
      <p class="hint">Enter a name (operator-mac, laptop-2, etc.) and click <strong>Pair this Mac</strong> to generate a one-time code. Run the displayed command on your VPS to complete the pairing.</p>
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
      <button class="primary" onclick={startPairing}>Pair this Mac</button>
    </section>

  {:else if pairingStore.state.kind === 'starting'}
    <section class="card">
      <p class="lead">Generating pairing code…</p>
    </section>

  {:else if pairingStore.state.kind === 'waiting_vps'}
    {@const s = pairingStore.state}
    <section class="card code-card">
      <p class="lead">Run this on your VPS within <strong>{remainingLabel}</strong>:</p>
      {#if remainingMs <= 0}
        <p class="warn">Code expired. Generating a new one…</p>
      {/if}
      <div class="code-block">
        <code class="code">{s.code}</code>
        <button class="ghost small" onclick={() => copyToClipboard(s.code)}>
          {copied ? 'Copied' : 'Copy code'}
        </button>
      </div>
      <p class="vps-label">On your VPS, run:</p>
      <div class="code-block">
        <code class="cmd">{s.onYourVpsRun}</code>
        <button class="ghost small" onclick={() => copyToClipboard(s.onYourVpsRun)}>
          {copied ? 'Copied' : 'Copy cmd'}
        </button>
      </div>
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
        <dt>Token (first 8)</dt><dd class="mono">{s.token.slice(0, 8)}…</dd>
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
      <button class="primary" onclick={() => pairingStore.reset()}>Try again</button>
    </section>
  {/if}

  <section class="devices">
    <h3>Paired devices on {host}</h3>
    {#if pairingStore.paired.length === 0}
      <p class="empty">No paired devices yet.</p>
    {:else}
      <ul class="dev-list">
        {#each activeDevices as d (d.device_id + '-active')}
          <li class="dev active">
            <div class="dev-main">
              <div class="dev-name">{d.name}</div>
              <div class="dev-meta">{d.device_id.slice(0, 8)}… · paired {d.paired_at}</div>
            </div>
            <button class="ghost small" onclick={() => revokeAndClear(d.device_id)}>Revoke</button>
          </li>
        {/each}
        {#each revokedDevices as d (d.device_id + '-revoked')}
          <li class="dev revoked">
            <div class="dev-main">
              <div class="dev-name">{d.name} <span class="badge">revoked</span></div>
              <div class="dev-meta">{d.device_id.slice(0, 8)}… · revoked {d.revoked_at}</div>
            </div>
          </li>
        {/each}
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
</style>
