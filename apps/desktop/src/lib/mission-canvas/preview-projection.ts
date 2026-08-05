import layoutVariants from '../../../tests/fixtures/mission-canvas/layout-variants.json';
import oneQueueProjection from '../../../tests/fixtures/mission-canvas/one-queue-projection.json';
import type { ResolvedWorkspaceProjection } from './types';

/** Development-only schema-valid projections used by the browser preview host. */
export const DESKTOP_BROWSER_PREVIEW_PROJECTIONS: Readonly<Record<string, ResolvedWorkspaceProjection>> = {
  'mission-canvas': oneQueueProjection as ResolvedWorkspaceProjection,
  'pi-work-surface': layoutVariants.tabs as ResolvedWorkspaceProjection
};
