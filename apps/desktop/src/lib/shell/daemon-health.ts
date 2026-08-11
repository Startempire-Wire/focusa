export type DaemonReadStatus =
  | { kind: 'checking'; label: 'Checking daemon'; detail: string }
  | { kind: 'unavailable'; label: 'Daemon unavailable'; detail: string; version?: string }
  | { kind: 'read-only'; label: 'Daemon connected · read-only'; detail: string; version?: string; uptimeMs?: number; snapshotMb?: number; failuresTotal?: number };

const DEFAULT_DAEMON_URL = 'http://127.0.0.1:8787';

/** Numeric semver compare on [major, minor, patch]; malformed versions compare equal. */
export function semverCompare(left: string | undefined, right: string | undefined): number {
  const parse = (value: string | undefined): number[] => {
    const match = String(value ?? '').trim().match(/^(\d+)\.(\d+)\.(\d+)/);
    return match ? match.slice(1).map((part) => Number(part)) : [0, 0, 0];
  };
  const a = parse(left);
  const b = parse(right);
  for (let index = 0; index < 3; index += 1) {
    if (a[index] !== b[index]) return a[index] < b[index] ? -1 : 1;
  }
  return 0;
}

/** The Mission Canvas HTTP API ships with the daemon starting at 0.9.143. */
export function supportsMissionCanvasApi(version: string | undefined): boolean {
  return semverCompare(version, '0.9.143') >= 0;
}

export async function readDaemonHealth(baseUrl = DEFAULT_DAEMON_URL): Promise<DaemonReadStatus> {
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), 1800);
  try {
    const response = await fetch(`${baseUrl.replace(/\/$/, '')}/v1/health`, {
      method: 'GET',
      headers: { accept: 'application/json' },
      signal: controller.signal
    });
    if (!response.ok) {
      return {
        kind: 'unavailable',
        label: 'Daemon unavailable',
        detail: `Health returned HTTP ${response.status}. No cognitive state was requested.`
      };
    }
    const body = await response.json() as { version?: unknown; uptime_ms?: unknown; persistence?: { snapshot_bytes?: unknown; failures_total?: unknown } };
    const version = typeof body.version === 'string' ? body.version : undefined;
    const uptimeMs = typeof body.uptime_ms === 'number' ? body.uptime_ms : undefined;
    const snapshotMb = typeof body.persistence?.snapshot_bytes === 'number'
      ? Math.round(body.persistence.snapshot_bytes / 1024 / 1024) : undefined;
    const failuresTotal = typeof body.persistence?.failures_total === 'number'
      ? body.persistence.failures_total : undefined;
    return {
      kind: 'read-only',
      label: 'Daemon connected · read-only',
      detail: 'Infrastructure health is reachable. No Workstream is attached and no canonical mutation is enabled.',
      version,
      uptimeMs,
      snapshotMb,
      failuresTotal
    };
  } catch {
    return {
      kind: 'unavailable',
      label: 'Daemon unavailable',
      detail: 'The local health endpoint is not reachable. Desktop remains unbound and does not invent state.'
    };
  } finally {
    clearTimeout(timer);
  }
}
