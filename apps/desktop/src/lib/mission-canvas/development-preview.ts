import layoutVariants from '../../../tests/fixtures/mission-canvas/layout-variants.json';
import oneQueueProjection from '../../../tests/fixtures/mission-canvas/one-queue-projection.json';
import type { ResolvedWorkspaceProjection } from './types';

const PROJECTIONS: Readonly<Record<string, ResolvedWorkspaceProjection>> = {
  'mission-canvas': oneQueueProjection as ResolvedWorkspaceProjection,
  'pi-work-surface': layoutVariants.tabs as ResolvedWorkspaceProjection
};

export function developmentProjection(workspaceId: string): ResolvedWorkspaceProjection | undefined {
  const projection = PROJECTIONS[workspaceId];
  return projection ? structuredClone(projection) : undefined;
}
