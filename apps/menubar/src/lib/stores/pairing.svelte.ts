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

export type PairingState =
  | { kind: 'idle' }
  | { kind: 'starting' }
  | { kind: 'waiting_vps'; code: string; deviceId: string; deviceName: string; platform: string; daemonBaseUrl: string; pairUrl: string; pairUrlQrPayload: string; scopes: string[]; onYourVpsRun: string; startedAt: number; expiresAt: number; attempt: number }
  | { kind: 'completed'; deviceId: string; deviceName: string; tokenPreview: string; tokenExpiresAt: string; host: string; completedAt: number }
  | { kind: 'expired'; code: string; deviceId: string; deviceName: string; reason: string }
  | { kind: 'error'; message: string; recoverable: boolean; failureClass?: string };

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

function apiBase(): string {
  return `${getApiUrl().replace(/\/$/, '')}/v1`;
}

async function fetchJson<T>(path: string, init?: RequestInit): Promise<T> {
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
    ...((init?.headers as Record<string, string> | undefined) || {}),
  };
  if (currentAuthToken && !headers.Authorization) {
    headers.Authorization = `Bearer ${currentAuthToken}`;
  }
  const resp = await fetch(`${apiBase()}${path}`, {
    ...init,
    signal: AbortSignal.timeout(8_000),
    headers,
  });
  const text = await resp.text();
  let json: any = null;
  if (text) {
    try { json = JSON.parse(text); } catch { json = { raw: text }; }
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
        // Keep polling on transient network errors until expiry timer flips UI.
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
          daemon_base_url: args.daemonBaseUrl ?? apiBase(),
          scopes: args.scopes ?? ['read', 'write'],
        }),
      });
      const code = String(result.code || '');
      const deviceId = String(result.device_id || '');
      const deviceName = String(result.device_name || args.deviceName);
      const platform = String(result.platform || args.platform || 'macos');
      const daemonBaseUrl = String(result.daemon_base_url || args.daemonBaseUrl || apiBase());
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
        attempt: Date.now(),
      };
      startPolling(code, deviceId);
    } catch (err) {
      state = { kind: 'error', message: err instanceof Error ? err.message : 'Pairing failed', recoverable: true };
    }
  }

  async function list(host: string = 'operator-vps'): Promise<void> {
    try {
      const result = await fetchJson<any>(`/device/pair/list?host=${encodeURIComponent(host)}&limit=50`);
      const devices = Array.isArray(result.devices) ? result.devices : [];
      paired = devices.map((d: any) => ({
        device_id: String(d.device_id || ''),
        name: String(d.name || d.device_name || 'device'),
        platform: String(d.platform || 'unknown'),
        host: String(d.host || host),
        scopes: Array.isArray(d.scopes) ? d.scopes : [],
        paired_at: String(d.paired_at || ''),
        last_seen_at: String(d.last_seen_at || ''),
        revoked: d.revoked === true,
        revoked_at: d.revoked_at ?? null,
      })).filter((d: PairedDevice) => d.device_id);
    } catch (err) {
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
        await clearPairingToken(deviceId).catch((err) => console.debug('focusa keychain clear failed', err));
        currentAuthToken = null;
        clearStoredDeviceMeta();
        state = { kind: 'idle' };
      }
      await list(host);
    } catch (err) {
      state = { kind: 'error', message: err instanceof Error ? err.message : 'Revoke failed', recoverable: true };
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
      state = {
        kind: 'error',
        message: `Stored pairing metadata exists, but Keychain token could not be loaded: ${err instanceof Error ? err.message : String(err)}`,
        recoverable: true,
      };
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
