/**
 * CONTRACT-008 — pure traversal utilities over the canonical projection.
 *
 * Helpers traverse canonical geometry (contribution graph, layout nodes, Work
 * Surfaces) and return READ-ONLY views. They NEVER choose eligibility or
 * layout: Core owns composition; these functions only navigate what Core
 * already resolved.
 */

import type {
  ResolvedWorkspaceProjection,
  ResolvedContribution,
} from '../../docs/contracts/spec135/mission-canvas-v1/typescript/mission-canvas-types.generated';

export interface ContributionRef {
  contribution_id: string;
  kind: string;
  ref: string;
}

/** Flat list of every eligible contribution ref in canonical order. */
export function collectContributionRefs(
  projection: ResolvedWorkspaceProjection
): readonly ContributionRef[] {
  return projection.eligible_contributions.map((contribution) => ({
    contribution_id: contribution.contribution_id,
    kind: contribution.kind,
    ref: contribution.data_ref.ref,
  }));
}

/** Contribution ids of one kind (e.g. 'focused_work_surface'). */
export function contributionIdsOfKind(
  projection: ResolvedWorkspaceProjection,
  kind: string
): readonly string[] {
  return projection.eligible_contributions
    .filter((contribution) => contribution.kind === kind)
    .map((contribution) => contribution.contribution_id);
}

/** Resolve a contribution by its canonical id; undefined when absent. */
export function findContribution(
  projection: ResolvedWorkspaceProjection,
  contributionId: string
): ResolvedContribution | undefined {
  return projection.eligible_contributions.find(
    (contribution) => contribution.contribution_id === contributionId
  );
}

/** Work Surface refs (data_ref.ref) present in the projection. */
export function workSurfaceRefs(projection: ResolvedWorkspaceProjection): readonly string[] {
  return projection.eligible_contributions
    .filter((contribution) => contribution.kind === 'focused_work_surface')
    .map((contribution) => contribution.data_ref.ref);
}

/** Canonical ordering: a pure view of node order for presentation iteration. */
export function contributionOrder(projection: ResolvedWorkspaceProjection): readonly string[] {
  return projection.eligible_contributions.map((contribution) => contribution.contribution_id);
}
