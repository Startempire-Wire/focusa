// Pairing store — Mac menubar OAuth-like device pairing (focusa-ui0y).
//
// State machine for the local pairing flow:
//   idle → starting → waiting_vps → completed
//   idle → starting → expired
//   completed → revoking → idle
//
// Token persistence is localStorage-only for the menubar prototype.
// Keychain integration is a follow-up (see tauri-plugin-keyring track).

import { getApiUrl } from '$lib/api';

export type PairingState =
  | { kind: 'idle' }
  | { kind: 'starting' }
  | { kind: 'waiting_vps'; code: string; deviceId: string; deviceName: string; platform: string; daemonBaseUrl: string; pairUrl: string; pairUrlQrPayload: string; scopes: string[]; onYourVpsRun: string; startedAt: number; expiresAt: number; attempt: number }
  | { kind: 'completed'; deviceId: string; deviceName: string; token: string; tokenExpiresAt: string; host: string; completedAt: number }
  | { kind: 'expired'; code: string; deviceId: string; deviceName: string; reason: string }
  | { kind: 'error'; message: string; failureClass?: string };

export interface PairedDevice {
  device_id: string;
  name: string;
  platform: string;
  host: string;
  scopes: string[];
  paired_at: string;
  last_seen_at?: string | null;
  revoked: boolean;
  revoked_at?: string | null;
}

const POLL_INTERVAL_MS = 2_000;
const CODE_TTL_MS = 5 * 60 * 1_000;
const STORAGE_KEY = 'focusa_paired_device';

function apiBase(): string {
  return getApiUrl();
}

async function fetchJson<T>(path: string, init?: RequestInit): Promise<T> {
  const resp = await fetch(`${apiBase()}${path}`, {
    ...init,
    signal: AbortSignal.timeout(8_000),
    headers: { 'Content-Type': 'application/json', ...(init?.headers || {}) },
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

function loadStoredDevice(): { deviceId: string; deviceName: string; token: string; tokenExpiresAt: string; host: string } | null {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return null;
    const parsed = JSON.parse(raw);
    if (!parsed || typeof parsed !== 'object') return null;
    if (!parsed.token || !parsed.device_id) return null;
    return {
      deviceId: String(parsed.device_id),
      deviceName: String(parsed.device_name || 'paired-device'),
      token: String(parsed.token),
      tokenExpiresAt: String(parsed.token_expires_at || ''),
      host: String(parsed.host || 'operator-vps'),
    };
  } catch {
    return null;
  }
}

function persistDevice(d: { deviceId: string; deviceName: string; token: string; tokenExpiresAt: string; host: string }): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify({
      device_id: d.deviceId,
      device_name: d.deviceName,
      token: d.token,
      token_expires_at: d.tokenExpiresAt,
      host: d.host,
    }));
  } catch {}
}

function clearStoredDevice(): void {
  try { localStorage.removeItem(STORAGE_KEY); } catch {}
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
        const status = String(result.status || 'unknown');
        if (status === 'completed' && result.token) {
          const completed: PairingState = {
            kind: 'completed',
            deviceId: String(result.device_id || deviceId),
            deviceName: String(result.device_name || 'paired-device'),
            token: String(result.token),
            tokenExpiresAt: String(result.token_expires_at || ''),
            host: String(result.host || 'operator-vps'),
            completedAt: Date.now(),
          };
          state = completed;
          persistDevice({
            deviceId: completed.deviceId,
            deviceName: completed.deviceName,
            token: completed.token,
            tokenExpiresAt: completed.tokenExpiresAt,
            host: completed.host,
          });
          stopPolling();
          // Refresh the device list in the background
          void list('operator-vps');
        } else if (status === 'expired') {
          state = {
            kind: 'expired',
            code,
            deviceId,
            deviceName: String(result.device_name || 'paired-device'),
            reason: 'Pairing code expired (5-minute TTL). Generate a new code.',
          };
          stopPolling();
        } else if (state.kind === 'waiting_vps') {
          // Bump the attempt counter for UI feedback
          state = { ...state, attempt: state.attempt + 1 };
        }
      } catch (e: any) {
        // Network blip — keep polling
        if (state.kind === 'waiting_vps') {
          state = { ...state, attempt: state.attempt + 1 };
        }
      }
    }, POLL_INTERVAL_MS);
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
      const onYourVpsRun = String(result.operator_handoff?.on_your_vps_run || '');
      // focusa-ui0y.9: pair_url + pair_url_qr_payload (from FOCUSA_PAIRING_URL or daemon_base_url)
      const pairUrl = String(result.pair_url || '');
      const pairUrlQrPayload = String(result.pair_url_qr_payload || pairUrl);
      const startedAt = Date.now();
      const expiresAt = startedAt + CODE_TTL_MS;
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
        startedAt,
        expiresAt,
        attempt: 0,
      };
      startPolling(code, deviceId);
    } catch (e: any) {
      state = {
        kind: 'error',
        message: e?.message || 'Failed to start pairing',
        failureClass: e?.failure_class,
      };
    }
  }

  async function list(host: string = 'operator-vps'): Promise<void> {
    try {
      const result = await fetchJson<any>(`/device/pair/list?host=${encodeURIComponent(host)}&limit=50`);
      const devices = Array.isArray(result.devices) ? result.devices : [];
      paired = devices.map((d: any) => ({
        device_id: String(d.device_id || ''),
        name: String(d.name || '?'),
        platform: String(d.platform || ''),
        host: String(d.host || host),
        scopes: Array.isArray(d.scopes) ? d.scopes : [],
        paired_at: String(d.paired_at || ''),
        last_seen_at: d.last_seen_at ?? null,
        revoked: d.revoked === true,
        revoked_at: d.revoked_at ?? null,
      }));
    } catch (e: any) {
      // Silent — UI will show previous list
    }
  }

  async function revoke(deviceId: string, host: string = 'operator-vps', reason?: string): Promise<void> {
    try {
      await fetchJson<any>('/device/pair/revoke', {
        method: 'POST',
        body: JSON.stringify({ device_id: deviceId, host, reason: reason ?? 'menubar-revoke' }),
      });
      // If we revoked ourselves, clear the local token
      const stored = loadStoredDevice();
      if (stored && stored.deviceId === deviceId) {
        clearStoredDevice();
        if (state.kind === 'completed' && state.deviceId === deviceId) {
          state = { kind: 'idle' };
        }
      }
      await list(host);
    } catch (e: any) {
      state = {
        kind: 'error',
        message: e?.message || 'Revoke failed',
        failureClass: e?.failure_class,
      };
    }
  }

  function reset(): void {
    stopPolling();
    state = { kind: 'idle' };
  }

  function bootstrapFromStorage(): void {
    const stored = loadStoredDevice();
    if (stored) {
      // Don't auto-complete; show as completed if token still valid
      state = {
        kind: 'completed',
        deviceId: stored.deviceId,
        deviceName: stored.deviceName,
        token: stored.token,
        tokenExpiresAt: stored.tokenExpiresAt,
        host: stored.host,
        completedAt: 0,
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
