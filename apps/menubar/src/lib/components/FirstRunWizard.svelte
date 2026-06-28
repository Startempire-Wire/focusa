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
  const BRIDGE_POLL_INTERVAL_MS = 1500;

  type WizardStep =
    | 'welcome'
    | 'vps_install'
    | 'vps_discover'
    | 'idle'
    | 'waiting_phone'
    | 'connected'
    | 'connected_degraded';

  const STEP_ORDER: WizardStep[] = [
    'welcome',
    'vps_install',
    'vps_discover',
    'idle',
    'waiting_phone',
    'connected',
    'connected_degraded',
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
      if (!r.ok) return false;
      // Strengthen: confirm this is a Focusa daemon, not just any 200 OK.
      const body = await r.json().catch(() => null);
      if (!body || typeof body !== 'object') return false;
      if (body.status !== 'ok') return false;
      if (typeof body.version !== 'string' || !body.version.startsWith('0.9.')) return false;
      return true;
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
      // Tailscale MagicDNS discovery targets the daemon API URL
      // (HTTP on port 8787), NOT an HTTPS public origin.
      const url = `http://${host}:8787`;
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
  // ---------- Step: vps_discover -> idle (static mac_offer QR) ----------
  // Canonical v0.9.35 flow: the Mac does NOT create the room.
  // The VPS-side `focusa pairing wizard` creates the room and prints a QR
  // for the phone to scan. The Mac idles showing a STATIC mac_offer QR
  // (mac_name + nonce + pubkey + callback). The phone's PWA scans the
  // Mac's mac_offer QR, POSTs it to /join, then operator taps Approve.
  // The Mac polls /v1/connect/rooms?status=waiting_for_mac to discover
  // rooms and POSTs its mac_offer to /join, then polls /status for token.
  let idleStartInFlight = $state(false);
  let macNonce = $state('');
  let macCallback = $state('');

  async function startIdleQr(): Promise<void> {
    if (!discoveredUrl) {
      error = 'No VPS URL discovered — go back to vps_discover step';
      return;
    }
    if (idleStartInFlight) return;
    idleStartInFlight = true;
    error = '';
    try {
      macName = macDeviceName();
      macNonce = generateNonce();
      // Try to bind a Tauri-side TCP bridge for low-latency completion delivery.
      try {
        const cb = await invoke<string | null>('focusa_start_bridge_callback', { nonce: macNonce });
        if (cb) macCallback = cb;
      } catch {
        // Bridge is optional; the Mac polls /status as fallback.
        macCallback = '';
      }
      // Build the canonical mac_offer payload. The phone PWA parses this from
      // the QR and POSTs it to /join. mac_callback is optional per spec but
      // included when the bridge is available.
      macOffer = JSON.stringify({
        protocol: 'focusa-connect-v1',
        role: 'mac_handoff_offer',
        mac_name: macName,
        nonce: macNonce,
        mac_callback: macCallback || undefined,
      });
      diagnosticsStore.record({
        area: 'first_run_wizard',
        phase: 'idle_qr_ready',
        error_class: 'network',
        message: `static mac_offer QR ready for mac=${macName}`,
        url: discoveredUrl,
        context: { mac_nonce: macNonce.slice(0, 8), has_callback: !!macCallback },
      });
      advanceTo('idle');
      startRoomDiscovery();
    } finally {
      idleStartInFlight = false;
    }
  }

  // Poll /v1/connect/rooms?status=waiting_for_mac every 1.5s.
  // When a room appears, POST our mac_offer to /join, then transition to
  // waiting_phone and start polling that room's /status.
  async function pollRoomsList(): Promise<void> {
    if (!discoveredUrl || !macOffer) return;
    try {
      const resp = await fetch(
        new URL('/v1/connect/rooms?status=waiting_for_mac', discoveredUrl),
        { headers: { accept: 'application/json' } },
      );
      if (!resp.ok) return;
      const body = (await resp.json()) as { rooms?: Array<{ room_id: string }> };
      const rooms = body.rooms || [];
      if (rooms.length === 0) return;
      const candidate = rooms[0];
      // POST our mac_offer to /join on the first waiting room.
      const joinResp = await fetch(
        new URL(`/v1/connect/room/${encodeURIComponent(candidate.room_id)}/join`, discoveredUrl),
        {
          method: 'POST',
          headers: { 'content-type': 'application/json' },
          body: JSON.stringify(JSON.parse(macOffer)),
        },
      );
      if (!joinResp.ok) return;
      roomId = candidate.room_id;
      diagnosticsStore.record({
        area: 'first_run_wizard',
        phase: 'mac_offer_posted',
        error_class: 'network',
        message: `mac_offer posted to room ${roomId}`,
        url: discoveredUrl,
        context: { room_id: roomId.slice(0, 8) },
      });
      stopRoomDiscovery();
      advanceTo('waiting_phone');
      startPolling();
    } catch (err) {
      diagnosticsStore.record({
        area: 'first_run_wizard',
        phase: 'room_discovery',
        error_class: 'network',
        error: err,
        message: err instanceof Error ? err.message : String(err),
        url: discoveredUrl,
      });
    }
  }

  function startRoomDiscovery() {
    stopRoomDiscovery();
    pollHandle = setInterval(pollRoomsList, 1500);
    pollRoomsList();
  }
  function stopRoomDiscovery() {
    if (pollHandle) {
      clearInterval(pollHandle);
      pollHandle = null;
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
          // Clear stale state so the next createRoomAndShowQr starts fresh.
          roomId = '';
          pairUrl = '';
          macOffer = '';
          completionPayload = '';
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
    // Also start the bridge callback TCP fast-path (low-latency alternative
    // to room-status polling). When the phone PWA approves via the TCP
    // callback, the bridge delivers the completion payload within ~1.5s
    // instead of waiting for the next room-status poll.
    void startBridgeCallback();
  }

  let bridgePollHandle: ReturnType<typeof setInterval> | null = null;

  async function startBridgeCallback(): Promise<void> {
    if (!macNonce) return;
    try {
      // The bridge listener is already started by startIdleQr (with the
      // canonical nonce from macOffer). This second call would create a
      // duplicate listener for the same nonce; instead, just poll the
      // completion queue keyed on the nonce.
    } catch (_) {
      // unreachable
    }
    bridgePollHandle = setInterval(async () => {
      try {
        const payload = await invoke<string | null>('focusa_take_bridge_completion', { nonce: macNonce });
        if (payload) {
          if (bridgePollHandle) clearInterval(bridgePollHandle);
          bridgePollHandle = null;
          completionPayload = payload;
          const parsed = JSON.parse(payload) as { device_id?: string; token?: string; server_url?: string };
          if (parsed.token) {
            await completePairing(parsed.token, parsed.device_id || roomId, parsed.server_url || discoveredUrl);
          }
        }
      } catch (err) {
        diagnosticsStore.record({
          area: 'first_run_wizard',
          phase: 'take_bridge_completion',
          error_class: 'pairing_bootstrap',
          error: err,
          message: err instanceof Error ? err.message : String(err),
          context: { mac_name: macName },
        });
      }
    }, 1500);
  }

  function stopPolling() {
    if (pollHandle) {
      clearInterval(pollHandle);
      pollHandle = null;
    }
    if (bridgePollHandle) {
      clearInterval(bridgePollHandle);
      bridgePollHandle = null;
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
      try {
        // V2: mirror the device token to localStorage so api.requestJson()
        // can attach `Authorization: Bearer <token>` to subsequent calls.
        // The Keychain copy is the durable store; this is a hot-path cache.
        localStorage.setItem('focusa_device_token', token);
      } catch {
        /* ignore */
      }
      localStorage.setItem('focusa_has_connected_successfully', 'true');
      advanceTo('connected');
    } catch (err) {
      // Keychain unavailable — surface as a DEGRADED state so the operator
      // knows the daemon does NOT durably trust this Mac (token not in
      // Keychain, only in process memory). They can re-pair or repair
      // Keychain to reach the normal 'connected' state.
      diagnosticsStore.record({
        area: 'first_run_wizard',
        phase: 'save_pairing_token',
        error_class: 'keychain',
        error: err,
        message: err instanceof Error ? err.message : String(err),
        context: { device_id: deviceId.slice(0, 8) },
      });
      localStorage.setItem('focusa_pairing_token_preview', String(token).slice(0, 6) + '…');
      localStorage.setItem('focusa_keychain_failed', 'true');
      completionPayload = JSON.stringify({ protocol: 'focusa-connect-v1', server_url: server, device_id: deviceId, token });
      advanceTo('connected_degraded');
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
      <p>Focusa connects this Mac — <strong>{macDeviceName()}</strong> — to a Focusa daemon running on your VPS.</p>
      <ol class="how-it-works">
        <li>Install Focusa on your VPS (<code>curl install.focusa.dev/focusa | bash</code>).</li>
        <li>On the VPS, run <code>focusa pairing wizard</code> — it prints a QR for your phone.</li>
        <li>This Mac auto-discovers the VPS via Tailscale, Bonjour, or your saved URL.</li>
        <li>Scan the Mac's static QR with the phone's Focusa Connect page, then tap Approve.</li>
        <li>Token lands in macOS Keychain. You're paired.</li>
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
        <input id="paste-url" bind:value={pasteUrl} placeholder="http://focusa-vps.tail-net.ts.net:8787" />
        <button class="utility" onclick={usePastedUrl}>Use this URL</button>
        <p class="dim">Save location: <code>~/.config/focusa/public-url</code> on macOS.</p>
      </details>
      <div class="row">
        <button class="primary" disabled={!discoveredUrl} onclick={startIdleQr}>Continue</button>
        <button class="utility" onclick={() => advanceTo('vps_install')}>Back</button>
      </div>
    </div>
  {:else if step === 'idle'}
    <div class="card">
      <h3>Scan this Mac with your phone</h3>
      <p>On your VPS run <code>focusa pairing wizard</code>, then scan the
      <strong>VPS terminal QR</strong> with your phone camera. After the Focusa
      Connect page loads, point the phone at <strong>this QR</strong> below.</p>
      <div class="qr-card">
        <QRCode payload={macOffer} size={260} />
      </div>
      <p class="dim">Mac: <code>{macName}</code> · Nonce: <code>{macNonce.slice(0, 8)}…</code></p>
      {#if macCallback}
        <p class="dim">Bridge: <code>{macCallback}</code></p>
      {/if}
      <p class="dim">Mac is watching for new pairing rooms every 1.5s.</p>
      <button class="utility" onclick={() => { stopRoomDiscovery(); advanceTo('vps_discover'); }}>Cancel</button>
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
  {:else if step === 'connected_degraded'}
    <div class="card" style="border-color:#a0651f;background:#2a1f10;">
      <h3 style="color:#f0a050;">Paired (degraded)</h3>
      <p>macOS Keychain refused to save the token. The Focusa daemon has not
      been told this Mac is trusted beyond this process.</p>
      <p>Repair options:</p>
      <ol class="how-it-works">
        <li>Open <strong>Keychain Access</strong> → unlock the login keychain.</li>
        <li>Click <strong>Restart</strong> below to retry pairing.</li>
        <li>If this keeps happening, your keychain may be corrupted; reset with <code>security delete-generic-password -s focusa</code>.</li>
      </ol>
      <p class="dim">token preview: <code>{localStorage.getItem('focusa_pairing_token_preview') || '(unset)'}</code></p>
      <details>
        <summary>Connection details</summary>
        <p>device_id: <code>{(localStorage.getItem('focusa_device_id') || '(unset)').slice(0, 8)}…</code></p>
        <p>server: <code>{discoveredUrl}</code></p>
      </details>
      <div class="row">
        <button class="primary" onclick={() => {
          localStorage.removeItem('focusa_keychain_failed');
          localStorage.removeItem('focusa_has_connected_successfully');
          advanceTo('welcome');
        }}>Restart pairing</button>
        <button class="utility" onclick={copyDebugBundle}>
          {copiedDebugBundle ? 'Copied bundle' : 'Copy debug bundle'}
        </button>
      </div>
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