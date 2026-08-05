import type { ResolvedContribution, ResolvedWorkspaceProjection } from './types';

export interface GeneratedSurfaceSource {
  snapshot: readonly unknown[];
  subscribeDelta?: (
    listener: (messages: readonly unknown[]) => void
  ) => (() => void) | Promise<() => void>;
}

export type GeneratedSurfaceSnapshotResolver = (
  contribution: ResolvedContribution,
  projection: ResolvedWorkspaceProjection
) => Promise<readonly unknown[] | GeneratedSurfaceSource>;
