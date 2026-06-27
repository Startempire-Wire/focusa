<script lang="ts">
  // Focusa first-run wizard (focusa-ui0y v0.9.35-dev).
  //
  // Replaces FirstRunConnect.svelte for the VPS-initiated pairing model.
  // State machine:
  //   welcome -> vps_install -> vps_discover -> show_qr -> waiting_phone -> connected
  //
  // The Mac app does NOT need to know the VPS URL until the vps_discover step.
  // The mac_offer QR shown in show_qr contains ONLY mac identity (no server URL).
  // The VPS URL is discovered separately via Tailscale MagicDNS, Bonjour, or
  // a one-shot CLI paste (Advanced fallback only).
  //
  // Discovery priority:
  //   1. Tailscale MagicDNS (recommended self-host topology)
  //   2. Bonjour / mDNS (`_focusa._tcp.local` on LAN)
  //   3. FOCUSA_DAEMON_URL env / localStorage hint
  //   4. one-shot CLI paste (Advanced only — not in primary flow)
  //
  // Spec: docs/55-focusa-self-host-architecture.md §6.2, doc 53 §2.0.

  import { onMount } from 'svelte';
  import { getApiUrl } from '$lib/api';
  import QRCode from './QRCode.svelte';
  import Settings from './Settings.svelte';
  import {
    diagnosticsStore,
    installGlobalDiagnostics,
    renderRedactedDebugBundle,
  } from '$lib/stores/diagnostics.svelte';

  type InvokeFn = <T>(cmd: string, args?: Record<string, unknown>) => Promise<T>;
  // Headless / non-Tauri stub so the Svelte app boots in a plain browser
  // (used by the menubar_headless_e2e test).
  const headlessStub: InvokeFn = async <T>(_cmd: string, _args?: Record<string, unknown>): Promise<T> => {
    console.warn('[focusa-headless] invoke stub:', _cmd, _args);
    return undefined as unknown as T;
  };
  let invoke: InvokeFn = headlessStub;
  let invokeLoadError: unknown = null;

  const PUBLIC_PAIRING_URL_KEY = 'focusa_public_pairing_url';
  const WIZARD_STATE_KEY = 'focusa_wizard_state_v1';

  type WizardStep =
    | 'welcome'
    | 'vps_install'
    | 'vps_discover'
    | 'show_qr'
    | 'waiting_phone'
    | 'connected';

  const STEP_ORDER: WizardStep[] = [
    'welcome',
    'vps_install',
    'vps_discover',
    'show_qr',
    'waiting_phone',
    'connected',
  ];

  function loadPersistedState(): WizardStep | null {
    try {
      const v = localStorage.getItem(WIZARD_STATE_KEY);
      if (v && STEP_ORDER.includes(v as WizardStep)) return v as WizardStep;
    } catch {
      /* ignore */
    }
    return null;
  }
  function persistState(step: WizardStep) {
    try {
      localStorage.setItem(WIZARD_STATE_KEY, step);
    } catch {
      /* ignore */
    }
  }

  let step = $state<WizardStep>(loadPersistedState() ?? 'welcome');
  let daemonUrl = $state('');
  let discoveredUrl = $state('');
  let discoverySource = $state('');
  let discoveryAttempts = $state<string[]>([]);
  let roomId = $state('');
  let pairUrl = $state('');
  let macOffer = $state('');
  let macName = $state('');
  let showAdvanced = $state(false);
  let pasteUrl = $state('');
  let error = $state('');
  let completionPayload = $state('');
  let copiedDebugBundle = $state(false);
  let pollHandle: ReturnType<typeof setInterval> | null = null;
  let tickHandle: ReturnType<typeof setInterval> | null = null;
  let now = $state(Date.now());

  function advanceTo(next: WizardStep) {
    step = next;
    persistState(next);
  }

  function macDeviceName(): string {
    try {
      return localStorage.getItem('focusa_device_name') || 'operator-mac';
    } catch {
      return 'operator-mac';
    }
  }

  function generateNonce(): string {
    const bytes = new Uint8Array(16);
    crypto.getRandomValues(bytes);
    return Array.from(bytes, (b) => b.toString(16).padStart(2, '0')).join('');
  }

  // ---------- Step: vps_discover ----------
  async function probeUrl(url: string): Promise<boolean> {
    try {
      const r = await fetch(new URL('/v1/health', url), {
        method: 'GET',
        // mode 'no-cors' would hide status; use default so we see real responses
        signal: AbortSignal.timeout(2000),
      });
      return r.ok;
    } catch {
      return false;
    }
  }

  async function discoverVps(): Promise<void> {
    error = '';
    discoveredUrl = '';
    discoverySource = '';
    discoveryAttempts = [];

    // 1. Tailscale MagicDNS — probe common hostnames
    const tailscaleHosts = [
      'focusa-vps',
      'focusa',
      'focusa-daemon',
      // operator-hostname is set during install if Tailscale is present
      (localStorage.getItem('focusa_tailscale_host') || '').trim(),
    ].filter((h) => h.length > 0);
    for (const host of tailscaleHosts) {
      const url = `https://${host}`;
      discoveryAttempts.push(`tailscale: ${url}`);
      if (await probeUrl(url)) {
        discoveredUrl = url;
        discoverySource = `Tailscale MagicDNS (${host})`;
        return;
      }
    }

    // 2. Bonjour / mDNS via Tauri command (best-effort; no-op in headless)
    try {
      const mdns = await invoke<{ url?: string } | null>(
        'focusa_discover_via_bonjour',
        {},
      );
      if (mdns?.url) {
        discoveryAttempts.push(`bonjour: ${mdns.url}`);
        if (await probeUrl(mdns.url)) {
          discoveredUrl = mdns.url;
          discoverySource = `Bonjour / mDNS (${mdns.url})`;
          return;
        }
      }
    } catch (err) {
      diagnosticsStore.record({
        area: 'first_run_wizard',
        phase: 'bonjour_discovery',
        error_class: 'network',
        error: err,
        message: err instanceof Error ? err.message : String(err),
      });
    }

    // 3. env / localStorage hint
    const stored = localStorage.getItem(PUBLIC_PAIRING_URL_KEY);
    if (stored && stored.trim().length > 0) {
      discoveryAttempts.push(`localStorage: ${stored}`);
      if (await probeUrl(stored)) {
        discoveredUrl = stored;
        discoverySource = 'saved pairing URL';
        return;
      }
    }

    error =
      'Could not auto-discover your Focusa VPS. Set one of: ' +
      '(a) run `focusa pairing transport-setup` on your VPS, ' +
      '(b) install Tailscale on both machines, ' +
      '(c) paste the URL below (Advanced).';
  }

  async function usePastedUrl(): Promise<void> {
    if (!pasteUrl) return;
    error = '';
    if (!(await probeUrl(pasteUrl))) {
      error = `Couldn't reach Focusa daemon at ${pasteUrl}. Verify the daemon is running.`;
      return;
    }
    discoveredUrl = pasteUrl;
    discoverySource = 'manual paste (Advanced)';
    try {
      localStorage.setItem(PUBLIC_PAIRING_URL_KEY, pasteUrl);
    } catch {
      /* ignore */
    }
  }

  // ---------- Step: show_qr ----------
  // The Mac generates a mac_offer (name + nonce + pubkey) and POSTs to the
  // VPS /v1/connect/room/{id}/join endpoint. The VPS already created the
  // room via `focusa pairing wizard` on the VPS terminal. The Mac discovers
  // the room by polling /v1/connect/rooms?status=waiting_for_mac OR by
  // the operator telling it the room_id via the wizard output.
  //
  // For v0.9.35-dev the simplest flow is: the Mac creates a fresh room
  // via the daemon's /v1/connect/room/create endpoint (this is acceptable
  // because the VPS daemon owns the room state, just like the wizard does
  // in terminal mode). The phone's PWA is the bridge.
  async function createRoomAndShowQr(): Promise<void> {
    if (!discoveredUrl) {
      error = 'No VPS URL discovered — go back to vps_discover step';
      return;
    }
    error = '';
    try {
      macName = macDeviceName();
      macOffer = JSON.stringify({
        protocol: 'focusa-connect-v1',
        role: 'mac_handoff_offer',
        mac_name: macName,
        nonce: generateNonce(),
      });
      // The Mac creates a room via the daemon (single round-trip).
      const resp = await fetch(new URL('/v1/connect/room/create', discoveredUrl), {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({}),
      });
      if (!resp.ok) throw new Error(`HTTP ${resp.status}`);
      const body = await resp.json();
      roomId = body.room_id || body.connect_id || '';
      pairUrl = body.pair_url || body.pair_url_qr_payload || '';
      if (!roomId || !pairUrl) throw new Error('server returned no room_id / pair_url');
      diagnosticsStore.record({
        area: 'first_run_wizard',
        phase: 'room_created',
        error_class: 'network',
        message: `room created: ${roomId}`,
        url: discoveredUrl,
        method: 'POST',
        context: { room_id: roomId.slice(0, 8) },
      });
      advanceTo('show_qr');
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      error = `Could not create room: ${msg}`;
      diagnosticsStore.record({
        area: 'first_run_wizard',
        phase: 'room_create',
        error_class: 'http',
        error: err,
        message: msg,
        url: discoveredUrl,
        method: 'POST',
      });
    }
  }

  // ---------- Step: show_qr -> waiting_phone -> connected ----------
  // Poll /v1/connect/room/{room_id}/status every 1.5s.
  // Handle 401 token_expired / pairing_revoked by jumping back to welcome.
  async function pollRoomStatus(): Promise<void> {
    if (!roomId || !discoveredUrl) return;
    try {
      const resp = await fetch(
        new URL(`/v1/connect/room/${encodeURIComponent(roomId)}/status`, discoveredUrl),
        { headers: { accept: 'application/json' } },
      );
      if (resp.status === 401) {
        const body = await resp.json().catch(() => ({}));
        const errorCode = body?.error || 'unknown';
        diagnosticsStore.record({
          area: 'first_run_wizard',
          phase: 'room_status_401',
          error_class: 'http',
          error: new Error(`401 ${errorCode}`),
          message: `Room status returned 401 (${errorCode}); re-pair required`,
          url: resp.url,
          status: 401,
        });
        if (errorCode === 'token_expired' || errorCode === 'pairing_revoked') {
          // Reset wizard to vps_discover so operator can re-pair
          stopPolling();
          advanceTo('vps_discover');
          error =
            'Pairing expired or revoked. Re-discover your VPS and tap Approve again.';
          return;
        }
      }
      if (!resp.ok) return;
      const body = await resp.json();
      if (body.status === 'completed' && body.token) {
        stopPolling();
        await completePairing(body.token, body.device_id || roomId, body.server_url || discoveredUrl);
      }
    } catch (err) {
      diagnosticsStore.record({
        area: 'first_run_wizard',
        phase: 'room_status_network',
        error_class: 'network',
        error: err,
        message: err instanceof Error ? err.message : String(err),
        url: discoveredUrl,
        method: 'GET',
        context: { room_id: roomId.slice(0, 8) },
      });
    }
  }

  function startPolling() {
    stopPolling();
    pollHandle = setInterval(pollRoomStatus, 1500);
    pollRoomStatus();
  }

  function stopPolling() {
    if (pollHandle) {
      clearInterval(pollHandle);
      pollHandle = null;
    }
  }

  async function completePairing(token: string, deviceId: string, server: string) {
    try {
      await invoke('focusa_save_pairing_token', { deviceId, token });
      completionPayload = JSON.stringify({ protocol: 'focusa-connect-v1', server_url: server, device_id: deviceId, token });
      try {
        localStorage.setItem('focusa_api_url', server);
      } catch {
        /* ignore */
      }
      try {
        localStorage.setItem('focusa_device_id', deviceId);
      } catch {
        /* ignore */
      }
      localStorage.setItem('focusa_has_connected_successfully', 'true');
      advanceTo('connected');
    } catch (err) {
      // Keychain unavailable — still mark connected so the operator can use
      // the daemon via the token in the debug bundle.
      diagnosticsStore.record({
        area: 'first_run_wizard',
        phase: 'save_pairing_token',
        error_class: 'keychain',
        error: err,
        message: err instanceof Error ? err.message : String(err),
        context: { device_id: deviceId.slice(0, 8) },
      });
      localStorage.setItem('focusa_pairing_token_preview', String(token).slice(0, 6) + '…');
      completionPayload = JSON.stringify({ protocol: 'focusa-connect-v1', server_url: server, device_id: deviceId, token });
      advanceTo('connected');
    }
  }

  async function copyDebugBundle() {
    const payload = renderRedactedDebugBundle({
      surface: 'first_run_wizard',
      step,
      daemon_url: getApiUrl(),
      discovered_url: discoveredUrl || '(unset)',
      discovery_source: discoverySource || '(unset)',
      discovery_attempts: discoveryAttempts,
      pair_url: pairUrl ? `${pairUrl.slice(0, 64)}...` : '(unset)',
      connect_id: roomId ? `${roomId.slice(0, 8)}…` : '(unset)',
      mac_name: macName || '(unset)',
      mac_offer_preview: macOffer ? `${macOffer.slice(0, 64)}…` : '(unset)',
      completion_status: completionPayload ? 'received' : 'pending',
      error: error || '(none)',
      extra: {
        diagnostics_entry_count: diagnosticsStore.entries.length,
        latest_failure_class: diagnosticsStore.latest()?.error_class || '(none)',
      },
    });
    try {
      await navigator.clipboard.writeText(payload);
      copiedDebugBundle = true;
      setTimeout(() => (copiedDebugBundle = false), 1500);
    } catch {
      window.prompt('Copy Focusa debug bundle:', payload);
    }
  }

  onMount(() => {
    installGlobalDiagnostics();
    if (typeof window !== 'undefined' && !(window as { __FOCUSA_HEADLESS__?: boolean }).__FOCUSA_HEADLESS__) {
      void import('@tauri-apps/api/core')
        .then((mod) => {
          invoke = mod.invoke as InvokeFn;
        })
        .catch((e) => {
          invokeLoadError = e;
          console.warn('[focusa] Tauri runtime not available; using stub.', e);
        });
    }
    tickHandle = setInterval(() => (now = Date.now()), 1000);
    return () => {
      if (tickHandle) clearInterval(tickHandle);
      stopPolling();
    };
  });

  // ---------- UI ----------
  const stepIndex = $derived(STEP_ORDER.indexOf(step) + 1);
  const stepTotal = STEP_ORDER.length;
</script>

<section class="first-run-wizard" aria-label="Focusa first-run wizard">
  <header>
    <h2>Focusa</h2>
    <p class="stepper">Step {stepIndex} of {stepTotal}</p>
  </header>

  {#if step === 'welcome'}
    <div class="card">
      <h3>Welcome</h3>
      <p>Focusa connects this Mac to a Focusa daemon running on your VPS.</p>
      <ol class="how-it-works">
        <li>Install Focusa on your VPS (<code>curl install.focusa.dev/focusa | bash</code>).</li>
        <li>Run <code>focusa pairing wizard</code> on the VPS to start a pairing room.</li>
        <li>This app will auto-discover the VPS and show a QR.</li>
        <li>Scan with your phone camera; tap Approve in the browser.</li>
      </ol>
      <button class="primary" onclick={() => advanceTo('vps_install')}>Get started</button>
      <details bind:open={showAdvanced}>
        <summary>Advanced</summary>
        <p>If you've already installed Focusa on your VPS:</p>
        <button class="utility" onclick={() => advanceTo('vps_discover')}>Skip to discovery</button>
      </details>
    </div>
  {:else if step === 'vps_install'}
    <div class="card">
      <h3>Install on your VPS</h3>
      <p>SSH into your VPS and run:</p>
      <pre class="code">{`curl install.focusa.dev/focusa | bash`}</pre>
      <p>When the installer finishes, it prints a pairing URL. Continue when ready.</p>
      <div class="row">
        <button class="primary" onclick={() => advanceTo('vps_discover')}>Continue</button>
        <button class="utility" onclick={() => advanceTo('welcome')}>Back</button>
      </div>
    </div>
  {:else if step === 'vps_discover'}
    <div class="card">
      <h3>Discover your VPS</h3>
      <p>Looking for your Focusa daemon via Tailscale MagicDNS, Bonjour, or saved pairing URL.</p>
      {#if !discoveredUrl}
        <button class="primary" onclick={discoverVps}>Discover</button>
      {:else}
        <p class="ok">Found: <code>{discoveredUrl}</code> <span class="dim">({discoverySource})</span></p>
      {/if}
      {#if error}
        <p class="err">{error}</p>
      {/if}
      {#if discoveryAttempts.length > 0}
        <details>
          <summary>Tried {discoveryAttempts.length} address(es)</summary>
          <ul>
            {#each discoveryAttempts as a}
              <li><code>{a}</code></li>
            {/each}
          </ul>
        </details>
      {/if}
      <details bind:open={showAdvanced}>
        <summary>Advanced — paste URL manually</summary>
        <label for="paste-url">Focusa daemon URL</label>
        <input id="paste-url" bind:value={pasteUrl} placeholder="https://focusa-vps.tail-net.ts.net" />
        <button class="utility" onclick={usePastedUrl}>Use this URL</button>
        <p class="dim">Save location: <code>~/.config/focusa/public-url</code> on macOS.</p>
      </details>
      <div class="row">
        <button class="primary" disabled={!discoveredUrl} onclick={createRoomAndShowQr}>Continue</button>
        <button class="utility" onclick={() => advanceTo('vps_install')}>Back</button>
      </div>
    </div>
  {:else if step === 'show_qr'}
    <div class="card">
      <h3>Scan with your phone</h3>
      <p>Open your iPhone or Android camera and point it at this QR.</p>
      <div class="qr-card">
        <QRCode payload={pairUrl} size={260} />
      </div>
      <p class="dim">URL: <code>{pairUrl}</code></p>
      <p class="dim">Mac: <code>{macName}</code> · Room: <code>{roomId.slice(0, 8)}…</code></p>
      <button class="primary" onclick={() => { advanceTo('waiting_phone'); startPolling(); }}>I've scanned — start polling</button>
      <button class="utility" onclick={() => { stopPolling(); createRoomAndShowQr(); }}>Re-create room</button>
      <button class="utility" onclick={() => advanceTo('vps_discover')}>Back</button>
    </div>
  {:else if step === 'waiting_phone'}
    <div class="card">
      <h3>Waiting for phone approval</h3>
      <p>In your phone browser, tap <strong>Approve</strong> on the Focusa Connect page.</p>
      <p class="dim">Polling every 1.5s. Room expires in 5 minutes.</p>
      <button class="utility" onclick={() => { stopPolling(); advanceTo('vps_discover'); }}>Cancel</button>
    </div>
  {:else if step === 'connected'}
    <div class="card ok-card">
      <h3>Paired</h3>
      <p>Your Mac is connected to <code>{discoveredUrl}</code>.</p>
      <p>Token stored in macOS Keychain. The Focusa daemon now trusts this Mac.</p>
      <details>
        <summary>Connection details</summary>
        <p>device_id: <code>{(localStorage.getItem('focusa_device_id') || '(unset)').slice(0, 8)}…</code></p>
        <p>server: <code>{discoveredUrl}</code></p>
      </details>
      <button class="primary" onclick={copyDebugBundle}>
        {copiedDebugBundle ? 'Copied bundle' : 'Copy debug bundle'}
      </button>
    </div>
  {/if}

  <footer class="utility-row">
    <button class="utility" onclick={copyDebugBundle}>{copiedDebugBundle ? 'Copied bundle' : 'Copy debug bundle'}</button>
    {#if step !== 'welcome' && step !== 'connected'}
      <button class="utility" onclick={() => advanceTo('welcome')}>Restart</button>
    {/if}
  </footer>
</section>

<style>
  .first-run-wizard {
    min-height: 100%;
    padding: var(--sp-4);
    display: flex;
    flex-direction: column;
    gap: var(--sp-3);
    background: var(--bg);
    color: var(--fg);
  }
  header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding-bottom: var(--sp-2);
    border-bottom: 1px solid var(--border);
  }
  h2 { margin: 0; font-size: var(--text-lg); }
  h3 { margin: 0 0 var(--sp-2); font-size: var(--text-md); }
  .stepper {
    margin: 0;
    color: var(--fg-secondary);
    font-size: var(--text-xs);
  }
  .card {
    background: var(--bg-panel);
    border: 1px solid var(--border);
    border-radius: var(--r-md);
    padding: var(--sp-4);
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
  }
  .ok-card { border-color: #1e6f3a; background: #1a2a20; }
  .qr-card {
    width: 308px;
    height: 308px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: #fff;
    border-radius: var(--r-md);
    align-self: center;
  }
  .how-it-works {
    margin: 0;
    padding-left: var(--sp-4);
    color: var(--fg-secondary);
    line-height: 1.5;
  }
  .code {
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: var(--r-sm);
    padding: var(--sp-2);
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    font-size: var(--text-xs);
    overflow-x: auto;
    white-space: pre;
  }
  .row {
    display: flex;
    gap: var(--sp-2);
  }
  .primary, .utility {
    padding: var(--sp-2) var(--sp-3);
    border-radius: var(--r-sm);
    border: 0;
    font: inherit;
    font-weight: 700;
    cursor: pointer;
  }
  .primary { background: #5b8cff; color: #0f1115; }
  .primary[disabled] { opacity: .5; cursor: default; }
  .utility { background: transparent; color: var(--fg-tertiary); font-size: var(--text-xs); }
  input {
    padding: var(--sp-2);
    border: 1px solid var(--border);
    border-radius: var(--r-sm);
    background: var(--bg);
    color: var(--fg);
    font: inherit;
  }
  .ok { color: #6ec47b; }
  .err { color: #ff8a7c; }
  .dim { color: var(--fg-secondary); font-size: var(--text-xs); margin: 0; }
  details { border-top: 1px solid var(--border); padding-top: var(--sp-2); }
  details summary {
    cursor: pointer;
    font-size: var(--text-xs);
    color: var(--fg-secondary);
  }
  .utility-row {
    display: flex;
    justify-content: space-between;
    gap: var(--sp-2);
    padding-top: var(--sp-2);
    border-top: 1px solid var(--border);
  }
  code {
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    font-size: 0.85em;
  }
</style>