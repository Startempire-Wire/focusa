<!--
  Settings panel — configure Focusa connection.
  Persists to localStorage.
-->
<script lang="ts">
  import { focusStore } from '$lib/stores/focus';

  let { onclose }: { onclose: () => void } = $props();

  let url = $state(localStorage.getItem('focusa_api_url') || 'http://127.0.0.1:8787');
  let saved = $state(false);
  let testing = $state(false);
  let testResult = $state<string | null>(null);

  function save() {
    localStorage.setItem('focusa_api_url', url);
    saved = true;
    setTimeout(() => saved = false, 2000);
  }

  async function testConnection() {
    testing = true;
    testResult = null;
    try {
      const resp = await fetch(`${url}/v1/health`, { signal: AbortSignal.timeout(3000) });
      if (resp.ok) {
        testResult = '✓ Connected';
      } else {
        testResult = `✗ HTTP ${resp.status}`;
      }
    } catch (e) {
      testResult = `✗ ${e instanceof Error ? e.message : 'Unreachable'}`;
    }
    testing = false;
  }

  function useLocal() { url = 'http://127.0.0.1:8787'; }
  function useTailscale() { url = 'http://100.94.238.56:8787'; }
</script>

<div class="settings">
  <div class="settings-header">
    <span class="settings-title">Settings</span>
    <button class="close-btn" onclick={onclose}>×</button>
  </div>

  <div class="section">
    <label class="label">Focusa API URL</label>
    <input
      type="text"
      bind:value={url}
      placeholder="http://127.0.0.1:8787"
      class="input"
    />

    <div class="presets">
      <button class="preset" onclick={useLocal}>
        Local (127.0.0.1)
      </button>
      <button class="preset" onclick={useTailscale}>
        VPS (Tailscale)
      </button>
    </div>
  </div>

  <div class="actions">
    <button class="btn btn-test" onclick={testConnection} disabled={testing}>
      {testing ? 'Testing…' : 'Test Connection'}
    </button>
    <button class="btn btn-save" onclick={save}>
      {saved ? '✓ Saved' : 'Save'}
    </button>
  </div>

  {#if testResult}
    <div class="test-result" class:success={testResult.startsWith('✓')} class:error={testResult.startsWith('✗')}>
      {testResult}
    </div>
  {/if}

  <div class="info">
    <div class="info-header">Connection Status</div>
    <div class="info-row">
      <span class="info-label">Connected:</span>
      <span class="info-value" class:ok={focusStore.connected} class:err={!focusStore.connected}>
        {focusStore.connected ? 'Yes' : 'No'}
      </span>
    </div>
    {#if focusStore.sessionId}
      <div class="info-row">
        <span class="info-label">Session:</span>
        <span class="info-value mono">{focusStore.sessionId.slice(0, 12)}…</span>
      </div>
    {/if}
    <div class="info-row">
      <span class="info-label">Version:</span>
      <span class="info-value mono">{focusStore.version}</span>
    </div>
  </div>

  <div class="help">
    <div class="help-title">Setup Guide</div>
    <ol>
      <li>
        <strong>Local:</strong> Run <code>focusa-daemon</code> on this machine.
        Default URL: <code>http://127.0.0.1:8787</code>
      </li>
      <li>
        <strong>Remote (Tailscale):</strong> On your VPS, start the daemon with:<br/>
        <code>FOCUSA_BIND=0.0.0.0:8787 focusa-daemon</code><br/>
        Then use your VPS Tailscale IP here.
      </li>
      <li>
        <strong>Remote (SSH tunnel):</strong> Keep daemon on localhost, tunnel:<br/>
        <code>ssh -L 8787:127.0.0.1:8787 your-vps</code><br/>
        Then use <code>http://127.0.0.1:8787</code> (safest).
      </li>
    </ol>
  </div>
</div>

<style>
  .settings {
    width: 300px;
    max-height: 460px;
    overflow-y: auto;
    background: var(--bg-panel);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-panel);
    animation: slideDown var(--transition-fade) forwards;
  }

  .settings-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--space-sm) var(--space-md);
    border-bottom: 1px solid var(--border);
  }

  .settings-title {
    font-size: var(--font-size-sm);
    font-weight: 600;
    color: var(--fg-dim);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .close-btn {
    background: none;
    border: none;
    color: var(--fg-dim);
    cursor: pointer;
    font-size: 1.2rem;
  }

  .section {
    padding: var(--space-sm) var(--space-md);
  }

  .label {
    display: block;
    font-size: var(--font-size-sm);
    color: var(--fg-dim);
    margin-bottom: 4px;
  }

  .input {
    width: 100%;
    font-family: var(--font-mono);
    font-size: var(--font-size-sm);
    padding: 6px 8px;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    background: var(--bg);
    color: var(--fg);
  }

  .input:focus {
    outline: none;
    border-color: var(--accent);
  }

  .presets {
    display: flex;
    gap: 6px;
    margin-top: 6px;
  }

  .preset {
    flex: 1;
    font-size: 0.65rem;
    padding: 4px 6px;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    background: var(--bg);
    color: var(--fg-dim);
    cursor: pointer;
    transition: all var(--transition-fade);
  }

  .preset:hover {
    border-color: var(--accent);
    color: var(--accent);
  }

  .actions {
    display: flex;
    gap: 6px;
    padding: 0 var(--space-md) var(--space-sm);
  }

  .btn {
    flex: 1;
    font-size: var(--font-size-sm);
    padding: 6px;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: all var(--transition-fade);
  }

  .btn-test {
    background: var(--bg);
    color: var(--fg-dim);
  }

  .btn-save {
    background: var(--accent);
    color: white;
    border-color: var(--accent);
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .test-result {
    padding: 4px var(--space-md);
    font-size: var(--font-size-sm);
    font-family: var(--font-mono);
  }

  .test-result.success { color: var(--success); }
  .test-result.error { color: var(--error); }

  .info {
    padding: var(--space-sm) var(--space-md);
    border-top: 1px solid var(--border);
  }

  .info-header {
    font-size: 0.65rem;
    font-weight: 600;
    color: var(--fg-dim);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin-bottom: 4px;
  }

  .info-row {
    display: flex;
    justify-content: space-between;
    font-size: var(--font-size-sm);
    padding: 1px 0;
  }

  .info-label { color: var(--fg-dim); }
  .info-value { color: var(--fg); }
  .info-value.ok { color: var(--success); }
  .info-value.err { color: var(--error); }
  .mono { font-family: var(--font-mono); }

  .help {
    padding: var(--space-sm) var(--space-md);
    border-top: 1px solid var(--border);
  }

  .help-title {
    font-size: 0.65rem;
    font-weight: 600;
    color: var(--fg-dim);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin-bottom: 6px;
  }

  .help ol {
    font-size: 0.7rem;
    color: var(--fg-dim);
    padding-left: 16px;
    line-height: 1.5;
  }

  .help li { margin-bottom: 6px; }
  .help code {
    font-family: var(--font-mono);
    font-size: 0.65rem;
    background: var(--bg);
    padding: 1px 4px;
    border-radius: 3px;
    border: 1px solid var(--border);
  }

  @keyframes slideDown {
    from { opacity: 0; transform: translateY(-8px); }
    to { opacity: 1; transform: translateY(0); }
  }
</style>
