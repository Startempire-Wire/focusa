// Pairing store — Mac menubar OAuth-like device pairing (focusa-ui0y).
//
// State machine for the local pairing flow:
//   idle → starting → waiting_vps → completed
//   idle → starting → expired
//   completed → revoking → idle
//
// Secret tokens are stored in macOS Keychain via Tauri commands. localStorage
// contains only non-secret metadata and a short token preview for UI display.

import { invoke } from '@tauri-apps/api/core';
import { getApiUrl } from '$lib/api';
import { diagnosticsStore, type DiagnosticEntry } from '$lib/stores/diagnostics.svelte';

export type PairingState =
  | { kind: 'idle' }
  | { kind: 'starting' }
  | { kind: 'waiting_vps'; code: string; deviceId: string; deviceName: string; platform: string; daemonBaseUrl: string; pairUrl: string; pairUrlQrPayload: string; scopes: string[]; onYourVpsRun: string; startedAt: number; expiresAt: number; attempt: number }
  | { kind: 'completed'; deviceId: string; deviceName: string; tokenPreview: string; tokenExpiresAt: string; host: string; completedAt: number }
  | { kind: 'expired'; code: string; deviceId: string; deviceName: string; reason: string }
  | { kind: 'error'; message: string; recoverable: boolean; failureClass?: string; diagnostic?: DiagnosticEntry; diagnosticText?: string };

export interface PairedDevice {
  device_id: string;
  name: string;
  platform: string;
  host: string;
  scopes: string[];
  paired_at: string;
  last_seen_at: string;
  revoked: boolean;
  revoked_at?: string | null;
}

interface StoredDeviceMeta {
  deviceId: string;
  deviceName: string;
  tokenPreview: string;
  tokenExpiresAt: string;
  host: string;
}

const STORAGE_KEY = 'focusa_paired_device_meta';
let currentAuthToken: string | null = null;
export function getCurrentAuthToken(): string | null { return currentAuthToken; }

function daemonRoot(): string {
  return getApiUrl().replace(/\/$/, '');
}

function apiBase(): string {
  return `${daemonRoot()}/v1`;
}

async function fetchJson<T>(path: string, init?: RequestInit): Promise<T> {
  const url = `${apiBase()}${path}`;
  const method = init?.method || 'GET';
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
    ...((init?.headers as Record<string, string> | undefined) || {}),
  };
  if (currentAuthToken && !headers.Authorization) {
    headers.Authorization = `Bearer ${currentAuthToken}`;
  }
  try {
    const resp = await fetch(url, {
      ...init,
      signal: AbortSignal.timeout(8_000),
      headers,
    });
    const text = await resp.text();
    let json: any = null;
    if (text) {
      try { json = JSON.parse(text); } catch (error) {
        const err = new Error(`json_parse: ${(error as Error).message}`);
        (err as any).failure_class = 'json_parse';
        (err as any).status = resp.status;
        (err as any).body = { raw: text };
        throw err;
      }
    }
    if (!resp.ok) {
      const fc = json?.failure_class || `http_${resp.status}`;
      const msg = json?.message || json?.error || resp.statusText || 'request failed';
      const err = new Error(`${fc}: ${msg}`);
      (err as any).failure_class = fc;
      (err as any).status = resp.status;
      (err as any).body = json;
      throw err;
    }
    return json as T;
  } catch (error) {
    (error as any).url = (error as any).url || url;
    (error as any).method = (error as any).method || method;
    throw error;
  }
}

function loadStoredDevice(): StoredDeviceMeta | null {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return null;
    const parsed = JSON.parse(raw);
    if (!parsed || typeof parsed !== 'object') return null;
    if (!parsed.device_id) return null;
    return {
      deviceId: String(parsed.device_id),
      deviceName: String(parsed.device_name || 'paired-device'),
      tokenPreview: String(parsed.token_preview || ''),
      tokenExpiresAt: String(parsed.token_expires_at || ''),
      host: String(parsed.host || 'operator-vps'),
    };
  } catch {
    return null;
  }
}

function persistDeviceMeta(d: StoredDeviceMeta): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify({
      device_id: d.deviceId,
      device_name: d.deviceName,
      token_preview: d.tokenPreview,
      token_expires_at: d.tokenExpiresAt,
      host: d.host,
    }));
  } catch {}
}

function clearStoredDeviceMeta(): void {
  try { localStorage.removeItem(STORAGE_KEY); } catch {}
  // V2: scrub any pre-existing token mirror from older builds.
  try { localStorage.removeItem('focusa_device_token'); } catch {}
}

function deviceTime(device: PairedDevice): number {
  const raw = device.last_seen_at || device.paired_at || device.revoked_at || '';
  const time = raw ? Date.parse(raw) : Number.NaN;
  return Number.isFinite(time) ? time : 0;
}

function dedupeDevices(devices: PairedDevice[]): PairedDevice[] {
  const activeByIdentity = new Map<string, PairedDevice>();
  const revokedById = new Map<string, PairedDevice>();
  for (const device of devices) {
    if (device.revoked) {
      revokedById.set(device.device_id, device);
      continue;
    }
    const key = [device.name.trim().toLowerCase(), device.platform.trim().toLowerCase(), device.host.trim().toLowerCase()].join('|');
    const existing = activeByIdentity.get(key);
    if (!existing || deviceTime(device) >= deviceTime(existing)) {
      activeByIdentity.set(key, device);
    }
  }
  const active = [...activeByIdentity.values()].sort((a, b) => deviceTime(b) - deviceTime(a));
  const revoked = [...revokedById.values()].sort((a, b) => deviceTime(b) - deviceTime(a));
  return [...active, ...revoked];
}

async function savePairingToken(deviceId: string, token: string): Promise<void> {
  await invoke('focusa_save_pairing_token', { deviceId, token });
}

async function loadPairingToken(deviceId: string): Promise<string> {
  return await invoke<string>('focusa_load_pairing_token', { deviceId });
}

async function clearPairingToken(deviceId: string): Promise<void> {
  await invoke('focusa_clear_pairing_token', { deviceId });
}

function pairingErrorState(phase: string, error: unknown, context?: Record<string, unknown>): PairingState {
  const diagnostic = diagnosticsStore.record({
    area: 'pairing',
    phase,
    error,
    context: { daemon_root: daemonRoot(), api_base: apiBase(), ...context },
  });
  const message = diagnostic.error_class === 'network'
    ? `Cannot reach Focusa daemon at ${daemonRoot()}. Check Settings URL, SSH tunnel, VPN, or CORS/proxy. Original error: ${diagnostic.message}`
    : diagnostic.message;
  return {
    kind: 'error',
    message,
    recoverable: true,
    failureClass: diagnostic.failure_class || diagnostic.error_class,
    diagnostic,
    diagnosticText: diagnosticsStore.render({ area: 'pairing', limit: 30 }),
  };
}

function createPairingStore() {
  let state = $state<PairingState>({ kind: 'idle' });
  let paired = $state<PairedDevice[]>([]);
  let polling = false;
  let pollHandle: ReturnType<typeof setInterval> | null = null;

  function startPolling(code: string, deviceId: string) {
    stopPolling();
    polling = true;
    pollHandle = setInterval(async () => {
      if (!polling) return;
      if (state.kind === 'waiting_vps' && state.code === code) {
        state = { ...state, attempt: state.attempt + 1 };
      }
      try {
        const result = await fetchJson<any>(`/device/pair/status?code=${encodeURIComponent(code)}`);
        const status = String(result.status || '').toLowerCase();
        if (status === 'completed' && result.token) {
          const token = String(result.token);
          const completed = {
            kind: 'completed' as const,
            deviceId,
            deviceName: String(result.device_name || 'operator-mac'),
            tokenPreview: token.slice(0, 8),
            tokenExpiresAt: String(result.token_expires_at || ''),
            host: String(result.host || 'operator-vps'),
            completedAt: Date.now(),
          };
          await savePairingToken(deviceId, token);
          currentAuthToken = token;
          persistDeviceMeta({
            deviceId,
            deviceName: completed.deviceName,
            tokenPreview: completed.tokenPreview,
            tokenExpiresAt: completed.tokenExpiresAt,
            host: completed.host,
          });
          state = completed;
          stopPolling();
          void list('operator-vps');
        } else if (status === 'expired') {
          state = { kind: 'expired', code, deviceId, deviceName: String(result.device_name || 'operator-mac'), reason: 'Code expired' };
          stopPolling();
        }
      } catch (err) {
        // Keep polling on transient network errors until expiry timer flips UI, but keep the log.
        diagnosticsStore.record({ area: 'pairing', phase: 'poll_status', error: err, context: { code, deviceId, api_base: apiBase() } });
        console.debug('focusa pairing poll failed', err);
      }
    }, 2_000);
  }

  function stopPolling() {
    polling = false;
    if (pollHandle) {
      clearInterval(pollHandle);
      pollHandle = null;
    }
  }

  async function start(args: { deviceName: string; platform?: string; daemonBaseUrl?: string; scopes?: string[] }): Promise<void> {
    state = { kind: 'starting' };
    try {
      const result = await fetchJson<any>('/device/pair/start', {
        method: 'POST',
        body: JSON.stringify({
          device_name: args.deviceName,
          platform: args.platform ?? 'macos',
          daemon_base_url: args.daemonBaseUrl ?? daemonRoot(),
          scopes: args.scopes ?? ['read', 'write'],
        }),
      });
      const code = String(result.code || '');
      const deviceId = String(result.device_id || '');
      const deviceName = String(result.device_name || args.deviceName);
      const platform = String(result.platform || args.platform || 'macos');
      const daemonBaseUrl = String(result.daemon_base_url || args.daemonBaseUrl || daemonRoot());
      const scopes = Array.isArray(result.scopes) ? result.scopes : (args.scopes ?? ['read', 'write']);
      const onYourVpsRun = String(result.on_your_vps_run || result.operator_handoff?.on_your_vps_run || `focusa device pair-complete ${code}`);
      const pairUrl = String(result.pair_url || result.operator_handoff?.pair_url || daemonBaseUrl);
      const pairUrlQrPayload = String(result.pair_url_qr_payload || result.operator_handoff?.pair_url_qr_payload || pairUrl);
      const expiresAt = Date.parse(String(result.expires_at || '')) || (Date.now() + 5 * 60_000);
      state = {
        kind: 'waiting_vps',
        code,
        deviceId,
        deviceName,
        platform,
        daemonBaseUrl,
        pairUrl,
        pairUrlQrPayload,
        scopes,
        onYourVpsRun,
        startedAt: Date.now(),
        expiresAt,
        attempt: 0,
      };
      startPolling(code, deviceId);
    } catch (err) {
      state = pairingErrorState('start_pairing', err, { deviceName: args.deviceName });
    }
  }

  async function list(host: string = 'operator-vps'): Promise<void> {
    try {
      const result = await fetchJson<any>(`/device/pair/list?host=${encodeURIComponent(host)}&limit=50`);
      const devices = Array.isArray(result.devices) ? result.devices : [];
      paired = dedupeDevices(devices.map((d: any) => ({
        device_id: String(d.device_id || ''),
        name: String(d.name || d.device_name || 'device'),
        platform: String(d.platform || 'unknown'),
        host: String(d.host || host),
        scopes: Array.isArray(d.scopes) ? d.scopes : [],
        paired_at: String(d.paired_at || ''),
        last_seen_at: String(d.last_seen_at || ''),
        revoked: d.revoked === true,
        revoked_at: d.revoked_at ?? null,
      })).filter((d: PairedDevice) => d.device_id));
    } catch (err) {
      diagnosticsStore.record({ area: 'pairing', phase: 'list_devices', error: err, context: { host, api_base: apiBase() } });
      console.debug('focusa device list failed', err);
    }
  }

  async function revoke(deviceId: string, host: string = 'operator-vps', reason?: string): Promise<void> {
    try {
      await fetchJson<any>('/device/pair/revoke', {
        method: 'POST',
        body: JSON.stringify({ device_id: deviceId, host, reason: reason ?? 'menubar-revoke' }),
      });
      const stored = loadStoredDevice();
      if (stored?.deviceId === deviceId) {
        await clearPairingToken(deviceId).catch((err) => diagnosticsStore.record({ area: 'pairing', phase: 'keychain_clear', error: err, context: { deviceId } }));
        currentAuthToken = null;
        try { localStorage.removeItem('focusa_device_token'); } catch {}
        clearStoredDeviceMeta();
        state = { kind: 'idle' };
      }
      await list(host);
    } catch (err) {
      state = pairingErrorState('revoke_device', err, { deviceId, host });
    }
  }

  function reset(): void {
    stopPolling();
    state = { kind: 'idle' };
  }

  async function bootstrapFromStorage(): Promise<void> {
    const stored = loadStoredDevice();
    if (!stored) return;
    try {
      const token = await loadPairingToken(stored.deviceId);
      currentAuthToken = token;
      // V2: the full token NEVER enters localStorage. The api client reads
      // it from the in-memory `currentAuthToken` rune. Best-effort: scrub
      // any previously-mirrored copy from older builds.
      try {
        localStorage.removeItem('focusa_device_token');
      } catch {
        /* ignore */
      }
      state = {
        kind: 'completed',
        deviceId: stored.deviceId,
        deviceName: stored.deviceName,
        tokenPreview: stored.tokenPreview || token.slice(0, 8),
        tokenExpiresAt: stored.tokenExpiresAt,
        host: stored.host,
        completedAt: 0,
      };
    } catch (err) {
      currentAuthToken = null;
      clearStoredDeviceMeta();
      state = pairingErrorState('bootstrap_keychain_load', err, { deviceId: stored.deviceId });
    }
  }

  return {
    get state() { return state; },
    get paired() { return paired; },
    get isPolling() { return polling; },
    start,
    list,
    revoke,
    reset,
    bootstrapFromStorage,
  };
}

export const pairingStore = createPairingStore();
