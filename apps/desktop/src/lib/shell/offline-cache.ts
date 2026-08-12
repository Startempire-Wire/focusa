// Offline projection cache: persists the last known valid projection
// in sessionStorage so the Desktop can show a stale-but-useful view
// when the daemon is unreachable. Follows Spec 158: clients may
// remember presentation preferences but never canonical authority.

import type { ResolvedWorkspaceProjection } from '../mission-canvas/types';

const CACHE_KEY = 'focusa-desktop-projection-cache';

export interface CachedProjection {
  projection: ResolvedWorkspaceProjection;
  cachedAt: string;
  revision: number;
}

export function cacheProjection(projection: ResolvedWorkspaceProjection): void {
  try {
    const entry: CachedProjection = {
      projection,
      cachedAt: new Date().toISOString(),
      revision: projection.projection_revision
    };
    sessionStorage.setItem(CACHE_KEY, JSON.stringify(entry));
  } catch { /* quota exceeded — silently skip */ }
}

export function getCachedProjection(): CachedProjection | null {
  try {
    const raw = sessionStorage.getItem(CACHE_KEY);
    if (!raw) return null;
    const entry = JSON.parse(raw) as CachedProjection;
    if (!entry?.projection?.projection_revision) return null;
    return entry;
  } catch { return null; }
}

export function clearCachedProjection(): void {
  try { sessionStorage.removeItem(CACHE_KEY); } catch { /* skip */ }
}
