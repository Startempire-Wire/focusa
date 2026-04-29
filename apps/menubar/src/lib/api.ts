export function getApiUrl(): string {
  try {
    return localStorage.getItem('focusa_api_url') || 'http://127.0.0.1:8787';
  } catch {
    return 'http://127.0.0.1:8787';
  }
}

export async function fetchJson<T = any>(path: string, timeoutMs = 3000): Promise<T> {
  const base = getApiUrl();
  const resp = await fetch(`${base}${path}`, {
    signal: AbortSignal.timeout(timeoutMs),
  });
  if (!resp.ok) throw new Error(`${path} returned HTTP ${resp.status}`);
  return await resp.json() as T;
}
