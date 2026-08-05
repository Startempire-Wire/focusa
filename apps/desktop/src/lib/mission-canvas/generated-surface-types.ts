import type { ResolvedContribution, ResolvedWorkspaceProjection } from './types';

export type GeneratedSurfaceSnapshotResolver = (
  contribution: ResolvedContribution,
  projection: ResolvedWorkspaceProjection
) => Promise<readonly unknown[]>;
