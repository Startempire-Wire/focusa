import { workstreamAuthorityStorageKey } from './exact-scope';
import type { PresentationStateSnapshot } from './presentation-state';
import type { ResolvedWorkspaceProjection, WorkstreamAuthorityContext } from './types';

/**
 * LIVE-004 — advisory restoration of Canvas and Work Surfaces.
 *
 * A locally persisted presentation snapshot is NEVER canonical: it only
 * re-applies presentation (focus/scroll/active tab) for surfaces that still
 * exist in the fresh canonical projection. Missing surfaces are reconciled
 * truthfully (dropped and reported); a stale projection revision or a foreign
 * Workstream never restores anything.
 */

export interface RestoredCanvasState {
  schema: 'focusa.desktop.restoration_state.v1';
  authorityKey: string;
  projectionRevision: number;
  capturedAt: string;
  snapshot: PresentationStateSnapshot;
}

export interface RestorationCandidate {
  snapshot: PresentationStateSnapshot;
  reconciledSurfaces: readonly string[];
}

const STORAGE_KEY = 'focusa.desktop.restoration_state.v1';

export function readRestorationStorage(storage: Storage): RestoredCanvasState | undefined {
  try {
    const raw = storage.getItem(STORAGE_KEY);
    if (!raw) return undefined;
    const parsed = JSON.parse(raw) as RestoredCanvasState;
    if (parsed.schema !== 'focusa.desktop.restoration_state.v1') return undefined;
    if (typeof parsed.authorityKey !== 'string' || typeof parsed.projectionRevision !== 'number') {
      return undefined;
    }
    return parsed;
  } catch {
    return undefined;
  }
}

export function writeRestorationStorage(storage: Storage, state: RestoredCanvasState): void {
  try {
    storage.setItem(STORAGE_KEY, JSON.stringify(state));
  } catch {
    // Advisory presentation only; a full or blocked storage must never break
    // the canonical projection path.
  }
}

export function clearRestorationStorage(storage: Storage): void {
  try {
    storage.removeItem(STORAGE_KEY);
  } catch {
    // Advisory only.
  }
}

function contributionRefs(projection: ResolvedWorkspaceProjection): Set<string> {
  const refs = new Set<string>();
  for (const contribution of projection.eligible_contributions) {
    refs.add(contribution.contribution_id);
    refs.add(contribution.data_ref.ref);
  }
  return refs;
}

/**
 * Truthful reconciliation: a captured surface locator that no longer exists
 * in the canonical projection is dropped and reported. Non-surface locators
 * are kept — restoreIfStillPresent no-ops if the element is gone. A local
 * snapshot never overrides canonical projection content.
 */
export function reconcileSnapshot(
  snapshot: PresentationStateSnapshot,
  projection: ResolvedWorkspaceProjection
): RestorationCandidate {
  const live = contributionRefs(projection);
  const reconciledSurfaces: string[] = [];
  const surfaceRefOf = (locator: { attribute: string; value: string }): string | undefined => {
    if (locator.attribute === 'data-work-surface-id' || locator.attribute === 'data-contribution-id') {
      return locator.value;
    }
    return undefined;
  };
  const present = (value: string | undefined): boolean => value === undefined || live.has(value);
  const scroll = snapshot.scroll.filter((capture) => {
    const surfaceRef = surfaceRefOf(capture.locator);
    if (present(surfaceRef)) return true;
    reconciledSurfaces.push(surfaceRef as string);
    return false;
  });
  const activeTab = snapshot.activeTab;
  let activeTabAfter: PresentationStateSnapshot['activeTab'];
  if (activeTab) {
    const surfaceRef = surfaceRefOf(activeTab);
    if (present(surfaceRef)) {
      activeTabAfter = activeTab;
    } else {
      reconciledSurfaces.push(surfaceRef as string);
    }
  }
  return {
    snapshot: {
      ...snapshot,
      scroll,
      activeTab: activeTabAfter
    },
    reconciledSurfaces: [...new Set(reconciledSurfaces)]
  };
}

export class MissionCanvasRestorationController {
  constructor(private readonly storage: Storage) {}

  persist(authority: WorkstreamAuthorityContext, projectionRevision: number, snapshot: PresentationStateSnapshot): void {
    writeRestorationStorage(this.storage, {
      schema: 'focusa.desktop.restoration_state.v1',
      authorityKey: workstreamAuthorityStorageKey(authority),
      projectionRevision,
      capturedAt: new Date().toISOString(),
      snapshot
    });
  }

  /**
   * Return an advisory candidate only when the exact Workstream matches, the
   * stored projection revision is not stale (never newer than the canonical
   * revision just resolved), and at least one surface is still present.
   */
  candidate(authority: WorkstreamAuthorityContext, projectionRevision: number): RestoredCanvasState | undefined {
    const stored = readRestorationStorage(this.storage);
    if (!stored) return undefined;
    if (stored.authorityKey !== workstreamAuthorityStorageKey(authority)) return undefined;
    if (stored.projectionRevision > projectionRevision) return undefined;
    return stored;
  }

  apply(
    authority: WorkstreamAuthorityContext,
    projection: ResolvedWorkspaceProjection,
    restore: (snapshot: PresentationStateSnapshot) => void
  ): readonly string[] {
    const candidate = this.candidate(authority, projection.projection_revision);
    if (!candidate) return [];
    const reconciled = reconcileSnapshot(candidate.snapshot, projection);
    if (reconciled.snapshot.scroll.length === 0 && !reconciled.snapshot.activeTab && !reconciled.snapshot.focus) {
      return reconciled.reconciledSurfaces;
    }
    restore(reconciled.snapshot);
    return reconciled.reconciledSurfaces;
  }

  clear(): void {
    clearRestorationStorage(this.storage);
  }
}
