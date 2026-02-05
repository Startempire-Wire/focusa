<!--
  Settings — full-tab settings view.
  Connection configuration, test, status display.
-->
<script lang="ts">
  import { focusStore } from '$lib/stores/focus.svelte';

  // Initialize directly — ssr=false so localStorage is always available
  let url = $state(localStorage.getItem('focusa_api_url') || 'http://127.0.0.1:8787');
  let saved = $state(false);
  let testing = $state(false);
  let testResult = $state<{ ok: boolean; msg: string } | null>(null);
  let showRemoteInput = $state(false);
  let remoteIp = $state('');

  function save() {
    try {
      localStorage.setItem('focusa_api_url', url);
    } catch {}
    saved = true;
    setTimeout(() => saved = false, 2000);
  }

  async function testConnection() {
    testing = true;
    testResult = null;
    try {
      const resp = await fetch(`${url}/v1/health`, {
        signal: AbortSignal.timeout(5000),
      });
      if (resp.ok) {
        const data = await resp.json();
        testResult = { ok: true, msg: `Connected — daemon v${data.version ?? '?'}` };
      } else {
        testResult = { ok: false, msg: `HTTP ${resp.status} ${resp.statusText}` };
      }
    } catch (e) {
      const msg = e instanceof Error ? e.message : 'Cannot reach server';
      testResult = { ok: false, msg };
    }
    testing = false;
  }

  function setLocal() {
    url = 'http://127.0.0.1:8787';
    save();
  }

  function setRemote() {
    showRemoteInput = true;
    remoteIp = '';
  }

  function confirmRemote() {
    if (remoteIp.trim()) {
      url = `http://${remoteIp.trim()}:8787`;
      showRemoteInput = false;
      save();
    }
  }
</script>

<div class="settings-view">
  <!-- Connection section -->
  <section class="section">
    <div class="section-label">CONNECTION</div>

    <label class="field">
      <span class="field-label">API URL</span>
      <input
        type="text"
        bind:value={url}
        placeholder="http://127.0.0.1:8787"
        class="input"
        onkeydown={(e) => { if (e.key === 'Enter') save(); }}
      />
    </label>

    <div class="preset-row">
      <button class="preset-btn" onclick={setLocal}>
        Local (127.0.0.1)
      </button>
      <button class="preset-btn" onclick={setRemote}>
        Remote…
      </button>
    </div>

    {#if showRemoteInput}
      <div class="remote-input-row">
        <input
          type="text"
          bind:value={remoteIp}
          placeholder="IP or hostname"
          class="input remote-ip"
          onkeydown={(e) => { if (e.key === 'Enter') confirmRemote(); if (e.key === 'Escape') showRemoteInput = false; }}
        />
        <button class="btn primary small" onclick={confirmRemote}>Connect</button>
      </div>
    {/if}

    <div class="action-row">
      <button class="btn secondary" onclick={testConnection} disabled={testing}>
        {testing ? 'Testing…' : 'Test Connection'}
      </button>
      <button class="btn primary" onclick={save}>
        {saved ? '✓ Saved' : 'Save'}
      </button>
    </div>

    {#if testResult}
      <div class="test-result" class:ok={testResult.ok} class:err={!testResult.ok}>
        {testResult.ok ? '✓' : '✗'} {testResult.msg}
      </div>
    {/if}
  </section>

  <!-- Status section -->
  <section class="section">
    <div class="section-label">STATUS</div>
    <div class="status-grid">
      <div class="status-row">
        <span class="status-key">Connection</span>
        <span class="status-val" class:green={focusStore.connected === 'connected'} class:red={focusStore.connected === 'error' || focusStore.connected === 'disconnected'}>
          {focusStore.connected === 'connected' ? 'Connected' : focusStore.connected === 'error' ? 'Error' : 'Disconnected'}
        </span>
      </div>
      <div class="status-row">
        <span class="status-key">Frames</span>
        <span class="status-val">{focusStore.frameCount}</span>
      </div>
      <div class="status-row">
        <span class="status-key">State version</span>
        <span class="status-val mono">{focusStore.version}</span>
      </div>
    </div>
  </section>

  <!-- Help section -->
  <section class="section">
    <div class="section-label">SETUP GUIDE</div>
    <div class="help-list">
      <div class="help-item">
        <div class="help-num">1</div>
        <div class="help-text">
          <strong>Local daemon</strong> — run <code>focusa-daemon</code> on this machine. It binds to <code>127.0.0.1:8787</code> by default.
        </div>
      </div>
      <div class="help-item">
        <div class="help-num">2</div>
        <div class="help-text">
          <strong>Remote daemon</strong> — on your server, run:
          <code>FOCUSA_BIND=0.0.0.0:8787 focusa-daemon</code>
          Then click "Remote…" above and enter the server IP.
        </div>
      </div>
      <div class="help-item">
        <div class="help-num">3</div>
        <div class="help-text">
          <strong>SSH tunnel</strong> (safest) — keep daemon on localhost, tunnel the port:
          <code>ssh -L 8787:127.0.0.1:8787 user@server</code>
          Then use Local (127.0.0.1).
        </div>
      </div>
    </div>
  </section>

  <!-- About -->
  <section class="section about">
    <span>Focusa v0.2.5</span>
    <span>·</span>
    <span>Cognitive Governance</span>
  </section>
</div>

<style>
  .settings-view {
    padding: var(--sp-3);
    display: flex;
    flex-direction: column;
    gap: var(--sp-3);
  }

  .section {
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
  }

  .section-label {
    font-size: 10px;
    font-weight: 700;
    color: var(--fg-tertiary);
    letter-spacing: 0.8px;
  }

  /* Fields */
  .field {
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .field-label {
    font-size: var(--text-xs);
    color: var(--fg-secondary);
    font-weight: 500;
  }

  .input {
    width: 100%;
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    padding: var(--sp-2);
    border: 1px solid var(--border);
    border-radius: var(--r-sm);
    background: var(--bg-panel);
    color: var(--fg);
    outline: none;
    transition: border-color var(--dur-fast) var(--ease);
  }

  .input:focus { border-color: var(--accent); }

  /* Preset buttons */
  .preset-row {
    display: flex;
    gap: var(--sp-1);
  }

  .preset-btn {
    flex: 1;
    font-family: var(--font);
    font-size: var(--text-xs);
    padding: var(--sp-1) var(--sp-2);
    border: 1px solid var(--border);
    border-radius: var(--r-sm);
    background: var(--bg-panel);
    color: var(--fg-secondary);
    cursor: pointer;
    transition: all var(--dur-fast) var(--ease);
  }

  .preset-btn:hover {
    border-color: var(--accent);
    color: var(--accent);
  }

  .remote-input-row {
    display: flex;
    gap: var(--sp-1);
  }

  .remote-ip {
    flex: 1;
  }

  .btn.small {
    flex: none;
    padding: var(--sp-1) var(--sp-2);
    font-size: var(--text-xs);
  }

  /* Action buttons */
  .action-row {
    display: flex;
    gap: var(--sp-2);
  }

  .btn {
    flex: 1;
    font-family: var(--font);
    font-size: var(--text-sm);
    font-weight: 500;
    padding: var(--sp-2);
    border-radius: var(--r-sm);
    border: 1px solid var(--border);
    cursor: pointer;
    transition: all var(--dur-fast) var(--ease);
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn.secondary {
    background: var(--bg-panel);
    color: var(--fg-secondary);
  }

  .btn.secondary:hover:not(:disabled) {
    background: var(--bg-hover);
  }

  .btn.primary {
    background: var(--accent);
    color: white;
    border-color: var(--accent);
  }

  .btn.primary:hover:not(:disabled) {
    filter: brightness(1.1);
  }

  /* Test result */
  .test-result {
    font-size: var(--text-sm);
    font-weight: 600;
    padding: var(--sp-2) var(--sp-3);
    border-radius: var(--r-sm);
    text-align: center;
  }

  .test-result.ok {
    color: var(--green);
    background: rgba(52, 199, 89, 0.12);
    border: 1px solid rgba(52, 199, 89, 0.3);
  }

  .test-result.err {
    color: var(--red);
    background: rgba(255, 59, 48, 0.1);
    border: 1px solid rgba(255, 59, 48, 0.25);
  }

  /* Status grid */
  .status-grid {
    background: var(--bg-panel);
    border: 1px solid var(--border);
    border-radius: var(--r-md);
    overflow: hidden;
  }

  .status-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--sp-2) var(--sp-3);
    font-size: var(--text-xs);
  }

  .status-row + .status-row {
    border-top: 1px solid var(--border);
  }

  .status-key { color: var(--fg-secondary); }
  .status-val { color: var(--fg); font-weight: 500; }
  .status-val.green { color: var(--green); }
  .status-val.red { color: var(--red); }
  .mono { font-family: var(--font-mono); }

  /* Help */
  .help-list {
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
  }

  .help-item {
    display: flex;
    gap: var(--sp-2);
  }

  .help-num {
    width: 20px;
    height: 20px;
    border-radius: 50%;
    background: var(--bg-elevated);
    color: var(--fg-secondary);
    font-size: var(--text-xs);
    font-weight: 600;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    margin-top: 1px;
  }

  .help-text {
    font-size: var(--text-xs);
    color: var(--fg-secondary);
    line-height: 1.5;
  }

  .help-text strong { color: var(--fg); }

  .help-text code {
    font-family: var(--font-mono);
    font-size: 10px;
    background: var(--bg-elevated);
    padding: 1px 4px;
    border-radius: 3px;
    border: 1px solid var(--border);
    display: inline;
  }

  /* About */
  .about {
    flex-direction: row;
    align-items: center;
    justify-content: center;
    gap: var(--sp-1);
    font-size: 10px;
    color: var(--fg-tertiary);
    padding-top: var(--sp-2);
    border-top: 1px solid var(--border);
  }

</style>
