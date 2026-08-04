export type DaemonReadStatus =
  | { kind: 'checking'; label: 'Checking daemon'; detail: string }
  | { kind: 'unavailable'; label: 'Daemon unavailable'; detail: string }
  | { kind: 'read-only'; label: 'Daemon connected · read-only'; detail: string };

const DEFAULT_DAEMON_URL = 'http://127.0.0.1:8787';

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
    return {
      kind: 'read-only',
      label: 'Daemon connected · read-only',
      detail: 'Infrastructure health is reachable. No Workstream is attached and no canonical mutation is enabled.'
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
