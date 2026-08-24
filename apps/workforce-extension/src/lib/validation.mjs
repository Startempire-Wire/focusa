const LOOPBACK_HOSTS = new Set(['localhost', '127.0.0.1', '[::1]']);

export function normalizeDaemonOrigin(value) {
  if (typeof value !== 'string' || !value.trim()) throw new TypeError('daemon URL is required');
  const parsed = new URL(value.trim());
  if (parsed.username || parsed.password) throw new TypeError('daemon URL must not contain credentials');
  if (parsed.search || parsed.hash) throw new TypeError('daemon URL must not contain query or fragment');
  if (parsed.pathname !== '/' && parsed.pathname !== '') throw new TypeError('daemon URL must be an origin without a path');
  const loopback = LOOPBACK_HOSTS.has(parsed.hostname);
  if (parsed.protocol !== 'https:' && !(parsed.protocol === 'http:' && loopback)) {
    throw new TypeError('remote daemon URL must use HTTPS; HTTP is loopback-only');
  }
  return parsed.origin;
}

export function originPermission(origin) {
  return `${normalizeDaemonOrigin(origin)}/*`;
}

export async function requestDaemonOriginPermission(origin, chromeApi = globalThis.chrome) {
  if (!chromeApi?.permissions?.request) throw new Error('Chrome permissions API is unavailable');
  return chromeApi.permissions.request({ origins: [originPermission(origin)] });
}
