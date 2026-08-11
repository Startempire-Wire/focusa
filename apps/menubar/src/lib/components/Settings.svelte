<!--
  Settings — connection-first configuration, saved remote Focusa servers, status display.
-->
<script lang="ts">
  import {
    DEFAULT_API_URL,
    PUBLIC_PAIRING_URL_KEY,
    getApiUrl,
    loadSavedConnections,
    removeSavedConnection,
    saveConnection,
    setApiUrl,
    type SavedConnection,
  } from '$lib/api';
  import { focusStore } from '$lib/stores/focus.svelte';
  import { runtimeStore } from '$lib/stores/runtime.svelte';
  import EntitlementPosture from './EntitlementPosture.svelte';
  import {
    MENUBAR_RELEASE_MODE,
    runMenubarUpdate,
    type MenubarUpdateResult,
  } from '$lib/updater';

  const initialConnections = loadSavedConnections();
  let savedConnections = $state<SavedConnection[]>(initialConnections);
  let url = $state(initialConnections[0]?.url || getApiUrl());
  let saved = $state(false);
  let testing = $state(false);
  let testResult = $state<{ ok: boolean; msg: string } | null>(null);
  let copiedError = $state(false);
  let showRemoteInput = $state(initialConnections.length === 0);
  let remoteUrl = $state('');
  let publicPairingUrl = $state(localStorage.getItem(PUBLIC_PAIRING_URL_KEY) || '');
  let updateResult = $state<MenubarUpdateResult | null>(null);
  let updateBusy = $state(false);

  function persistActive(nextUrl = url) {
    const normalized = nextUrl.trim().replace(/\/$/, '');
    if (!normalized) return;
    setApiUrl(normalized);
    url = normalized;
    saved = true;
    setTimeout(() => saved = false, 2000);
  }

  function save() {
    persistActive(url);
    savePublicPairingUrl();
  }

  function savePublicPairingUrl() {
    const normalized = publicPairingUrl.trim().replace(/\/$/, '');
    publicPairingUrl = normalized;
    try {
      if (normalized) localStorage.setItem(PUBLIC_PAIRING_URL_KEY, normalized);
      else localStorage.removeItem(PUBLIC_PAIRING_URL_KEY);
    } catch {}
  }

  async function copyConnectionError() {
    const payload = [
      'Focusa connection diagnostics',
      `active_url=${url || '(unset)'}`,
      `public_pairing_url=${publicPairingUrl || '(unset)'}`,
      `connected_state=${focusStore.connected}`,
      `result=${testResult ? `${testResult.ok ? 'ok' : 'error'}: ${testResult.msg}` : '(none)'}`,
    ].join('\n');
    try {
      await navigator.clipboard.writeText(payload);
      copiedError = true;
      setTimeout(() => copiedError = false, 1500);
    } catch {
      window.prompt('Copy Focusa connection diagnostics:', payload);
    }
  }

  async function testConnection(targetUrl = url, options: { remember?: boolean } = { remember: true }) {
    const normalized = targetUrl.trim().replace(/\/$/, '');
    if (!normalized) {
      testResult = { ok: false, msg: 'Enter your Focusa server URL first.' };
      return false;
    }
    testing = true;
    testResult = null;
    try {
      const resp = await fetch(`${normalized}/v1/health`, {
        signal: AbortSignal.timeout(5000),
      });
      if (resp.ok) {
        const data = await resp.json();
        if (options.remember !== false) {
          setApiUrl(normalized);
          url = normalized;
          savedConnections = saveConnection(normalized, normalized === DEFAULT_API_URL ? 'Local Focusa' : normalized);
          showRemoteInput = false;
        }
        testResult = { ok: true, msg: `Connected — daemon v${data.version ?? '?'}` };
        window.dispatchEvent(new CustomEvent('focusa-connection-saved'));
        return true;
      }
      testResult = { ok: false, msg: `HTTP ${resp.status} ${resp.statusText}` };
      return false;
    } catch (e) {
      const msg = e instanceof Error ? e.message : 'Cannot reach server';
      testResult = { ok: false, msg };
      return false;
    } finally {
      testing = false;
    }
  }

  function setLocal() {
    url = DEFAULT_API_URL;
    void testConnection(DEFAULT_API_URL);
  }

  function setRemote() {
    showRemoteInput = true;
    remoteUrl = '';
  }

  function confirmRemote() {
    const raw = remoteUrl.trim();
    if (!raw) return;
    const normalized = raw.startsWith('http://') || raw.startsWith('https://') ? raw : `https://${raw}`;
    url = normalized.replace(/\/$/, '');
    void testConnection(url);
  }

  function quickConnect(connection: SavedConnection) {
    url = connection.url;
    void testConnection(connection.url);
  }

  function disconnect(connection: SavedConnection) {
    if (!confirm(`Forget saved Focusa connection?\n\n${connection.url}`)) return;
    savedConnections = removeSavedConnection(connection.url);
    if (url === connection.url) {
      url = savedConnections[0]?.url || '';
      if (url) setApiUrl(url);
    }
  }

  async function updateFocusa(install: boolean) {
    updateBusy = true;
    try {
      updateResult = await runMenubarUpdate({
        install,
        reporter: (result) => { updateResult = result; },
      });
    } finally {
      updateBusy = false;
    }
  }
</script>

<div class="settings-view">
  <EntitlementPosture />
  <section class="section connect-hero">
    <div class="section-label">CONNECT TO FOCUSA</div>
    <p class="hint">Connect to your remote Focusa server. Local Focusa remains available for future local development.</p>
    <p class="hint">Direct network binding exposes Focusa; use trusted private networks or authenticated public endpoints.</p>

    {#if savedConnections.length > 0}
      <div class="saved-list">
        {#each savedConnections as connection}
          <div class="saved-connection">
            <div>
              <div class="saved-label">{connection.label}</div>
              <div class="saved-url">{connection.url}</div>
              <div class="saved-meta">last connected {connection.last_connected_at}</div>
            </div>
            <div class="saved-actions">
              <button class="btn primary" onclick={() => quickConnect(connection)} disabled={testing}>Connect</button>
              <button class="btn danger" onclick={() => disconnect(connection)}>Disconnect</button>
            </div>
          </div>
        {/each}
      </div>
    {/if}

    <label class="field">
      <span class="field-label">Server URL</span>
      <input
        type="text"
        bind:value={url}
        placeholder="https://your-focusa-server.example.com"
        class="input"
        onkeydown={(e) => { if (e.key === 'Enter') void testConnection(url); }}
      />
    </label>

    <label class="field">
      <span class="field-label">Public pairing URL for QR scans</span>
      <input
        type="text"
        bind:value={publicPairingUrl}
        placeholder="https://your-focusa-server.example.com"
        class="input"
        onblur={savePublicPairingUrl}
        onkeydown={(e) => { if (e.key === 'Enter') savePublicPairingUrl(); }}
      />
      <span class="field-help">Phone-scanned QR codes use this URL when the daemon itself does not know its public server URL.</span>
    </label>

    <div class="preset-row">
      <button class="preset-btn" onclick={setRemote}>Add remote…</button>
      <button class="preset-btn" onclick={setLocal}>Use local (127.0.0.1)</button>
    </div>

    {#if showRemoteInput}
      <div class="remote-input-row">
        <input
          type="text"
          bind:value={remoteUrl}
          placeholder="focusa.example.com or https://focusa.example.com"
          class="input remote-ip"
          onkeydown={(e) => { if (e.key === 'Enter') confirmRemote(); if (e.key === 'Escape') showRemoteInput = false; }}
        />
        <button class="btn primary small" onclick={confirmRemote}>Connect</button>
        <button class="btn ghost small" onclick={() => showRemoteInput = false}>Cancel</button>
      </div>
    {/if}

    <div class="action-row">
      <button class="btn secondary" onclick={() => testConnection(url)} disabled={testing || !url.trim()}>
        {testing ? 'Connecting…' : 'Connect / Test'}
      </button>
      <button class="btn primary" onclick={save} disabled={!url.trim()}>
        {saved ? '✓ Saved' : 'Save current URL'}
      </button>
    </div>

    {#if testResult}
      <div class="test-result" class:ok={testResult.ok} class:err={!testResult.ok}>
        <span>{testResult.ok ? '✓' : '✗'} {testResult.msg}</span>
        {#if !testResult.ok}
          <button class="btn ghost small" onclick={copyConnectionError}>{copiedError ? 'Copied errors' : 'Copy errors'}</button>
        {/if}
      </div>
    {/if}
  </section>

  <section class="section">
    <div class="section-label">STATUS</div>
    <div class="status-grid">
      <div class="status-row">
        <span class="status-key">Connection</span>
        <span class="status-val" class:green={focusStore.connected === 'connected'} class:red={focusStore.connected === 'error' || focusStore.connected === 'disconnected'}>
          {focusStore.connected === 'connected' ? 'Connected' : focusStore.connected === 'error' ? 'Error' : focusStore.connected === 'connecting' ? 'Connecting…' : 'Disconnected'}
        </span>
      </div>
      <div class="status-row"><span class="status-key">Active URL</span><span class="status-val mono">{url || 'not set'}</span></div>
      <div class="status-row"><span class="status-key">Saved connections</span><span class="status-val">{savedConnections.length}</span></div>
      <div class="status-row"><span class="status-key">Events</span><span class="status-val">{runtimeStore.snapshot.recentEventCount}</span></div>
    </div>
  </section>

  <section class="section">
    <div class="section-label">SIGNED UPDATES</div>
    <p class="hint">Focusa verifies updater signatures before replacing the app. Automatic installation follows your daemon update policy.</p>
    {#if MENUBAR_RELEASE_MODE === 'beta_ad_hoc'}
      <p class="hint">Pre-license beta: Tauri-signed OTA with ad-hoc macOS bundle integrity; this build is not Apple-notarized.</p>
    {/if}
    <div class="connection-actions">
      <button class="btn secondary" onclick={() => updateFocusa(false)} disabled={updateBusy}>
        {updateBusy ? 'Checking…' : 'Check for update'}
      </button>
      {#if updateResult?.phase === 'available'}
        <button class="btn primary" onclick={() => updateFocusa(true)} disabled={updateBusy}>Install and relaunch</button>
      {/if}
    </div>
    {#if updateResult}
      <div class:ok={updateResult.phase === 'current' || updateResult.phase === 'available'} class:error={updateResult.phase === 'error'} class="test-result">
        <span>{updateResult.message}</span>
      </div>
    {/if}
  </section>

  <section class="section help-section">
    <div class="section-label">CONNECTIVITY HELP</div>
    <div class="help-list">
      <div class="help-item"><div class="help-num">1</div><div class="help-text"><strong>Remote first</strong> — use your VPS/public Focusa URL when the Mac app is not running on the server.</div></div>
      <div class="help-item"><div class="help-num">2</div><div class="help-text"><strong>Local optional</strong> — use Local only if Focusa daemon is running on this Mac or through an SSH tunnel.</div></div>
      <div class="help-item"><div class="help-num">3</div><div class="help-text"><strong>SSH tunnel</strong> — <code>ssh -L 8787:127.0.0.1:8787 user@server</code>, then connect to Local.</div></div>
    </div>
  </section>

  <section class="section about">
    <span>Focusa v0.9.148</span><span>·</span><span>Cognitive Governance</span>
  </section>
</div>

<style>
  .settings-view { padding: var(--sp-3); display: flex; flex-direction: column; gap: var(--sp-4); }
  .section { background: var(--bg-panel); border: 1px solid var(--border); border-radius: var(--r-md); padding: var(--sp-3); }
  .connect-hero { border-color: color-mix(in srgb, var(--accent) 35%, var(--border)); }
  .section-label { font-size: 10px; font-weight: 700; letter-spacing: 0.08em; color: var(--fg-tertiary); margin-bottom: var(--sp-3); }
  .hint { color: var(--fg-secondary); font-size: var(--text-sm); line-height: 1.4; margin: 0 0 var(--sp-3); }
  .field { display: flex; flex-direction: column; gap: var(--sp-1); }
  .field-label { font-size: var(--text-xs); color: var(--fg-secondary); font-weight: 600; }
  .field-help { font-size: var(--text-xs); color: var(--fg-tertiary); line-height: 1.35; }
  .input { width: 100%; background: var(--bg-elevated); border: 1px solid var(--border); border-radius: var(--r-sm); padding: var(--sp-2); color: var(--fg); font-family: var(--font-mono); font-size: var(--text-sm); box-sizing: border-box; }
  .preset-row, .action-row, .remote-input-row, .saved-actions { display: flex; gap: var(--sp-2); margin-top: var(--sp-2); }
  .remote-input-row { align-items: center; }
  .remote-ip { flex: 1; }
  button { cursor: pointer; }
  .btn, .preset-btn { border: 1px solid var(--border); border-radius: var(--r-sm); padding: var(--sp-2) var(--sp-3); font-size: var(--text-xs); font-weight: 600; }
  .btn.primary { background: var(--accent); color: white; border-color: var(--accent); }
  .btn.secondary, .preset-btn { background: var(--bg-elevated); color: var(--fg); }
  .btn.ghost { background: transparent; color: var(--fg-secondary); }
  .btn.danger { background: transparent; color: var(--red); border-color: color-mix(in srgb, var(--red) 40%, var(--border)); }
  .btn.small { padding: var(--sp-1) var(--sp-2); }
  .saved-list { display: flex; flex-direction: column; gap: var(--sp-2); margin-bottom: var(--sp-3); }
  .saved-connection { display: flex; justify-content: space-between; gap: var(--sp-2); align-items: center; border: 1px solid var(--border); background: var(--bg-elevated); border-radius: var(--r-sm); padding: var(--sp-2); }
  .saved-label { font-weight: 700; font-size: var(--text-sm); }
  .saved-url, .saved-meta, .mono { font-family: var(--font-mono); font-size: var(--text-xs); color: var(--fg-secondary); }
  .test-result { margin-top: var(--sp-2); padding: var(--sp-2); border-radius: var(--r-sm); font-size: var(--text-sm); display: flex; align-items: center; justify-content: space-between; gap: var(--sp-2); }
  .test-result.ok { background: color-mix(in srgb, var(--green) 15%, transparent); color: var(--green); }
  .test-result.err { background: color-mix(in srgb, var(--red) 15%, transparent); color: var(--red); }
  .status-grid, .help-list { display: flex; flex-direction: column; gap: var(--sp-2); }
  .status-row { display: flex; justify-content: space-between; gap: var(--sp-2); font-size: var(--text-sm); }
  .status-key { color: var(--fg-secondary); }
  .status-val.green { color: var(--green); } .status-val.red { color: var(--red); }
  .help-item { display: flex; gap: var(--sp-2); }
  .help-num { width: 20px; height: 20px; border-radius: 50%; background: var(--bg-elevated); display: flex; align-items: center; justify-content: center; font-size: 10px; color: var(--fg-secondary); flex-shrink: 0; }
  .help-text { font-size: var(--text-sm); color: var(--fg-secondary); line-height: 1.4; }
  .about { display: flex; gap: var(--sp-2); justify-content: center; color: var(--fg-tertiary); font-size: var(--text-xs); }
</style>
