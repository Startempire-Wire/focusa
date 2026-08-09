import type { ResolvedWorkspaceProjection } from './types';

/**
 * DOMAIN-001..012 — domain surface contributions (135f).
 *
 * Domain surfaces are contribution definitions, NOT fixed screens: a surface
 * renders ONLY when Core's resolver emits it in the canonical projection's
 * eligible_contributions. No fixed mission-status dashboard is encoded; the
 * desktop never auto-shows or reorders domain surfaces. The management flow
 * (DOMAIN-012) is a separate surface family, never mixed into the daily
 * workspace composition.
 */

export type DomainSurfaceId =
  | 'overview'
  | 'context'
  | 'role'
  | 'interview'
  | 'spec'
  | 'tasks'
  | 'runtime_inventory'
  | 'document'
  | 'research'
  | 'proof'
  | 'history'
  | 'management';

/** The daily workspace family (DOMAIN-001..011). */
export const DAILY_DOMAIN_SURFACES: readonly DomainSurfaceId[] = [
  'overview', 'context', 'role', 'interview', 'spec', 'tasks',
  'runtime_inventory', 'document', 'research', 'proof', 'history'
];

/** The explicit management flow (DOMAIN-012) — separate from the daily set. */
export const MANAGEMENT_DOMAIN_SURFACES: readonly DomainSurfaceId[] = ['management'];

export interface DomainSurfaceDescriptor {
  domain_surface_id: DomainSurfaceId;
  /** data_ref prefix emitted by Core for this surface family. */
  ref_prefix: string;
  /** The management flow is never part of the daily workspace composition. */
  management_flow: boolean;
}

export const DOMAIN_SURFACE_DESCRIPTORS: readonly DomainSurfaceDescriptor[] = [
  { domain_surface_id: 'overview', ref_prefix: 'surface:overview:', management_flow: false },
  { domain_surface_id: 'context', ref_prefix: 'surface:context:', management_flow: false },
  { domain_surface_id: 'role', ref_prefix: 'surface:role:', management_flow: false },
  { domain_surface_id: 'interview', ref_prefix: 'surface:interview:', management_flow: false },
  { domain_surface_id: 'spec', ref_prefix: 'surface:spec:', management_flow: false },
  { domain_surface_id: 'tasks', ref_prefix: 'surface:tasks:', management_flow: false },
  { domain_surface_id: 'runtime_inventory', ref_prefix: 'surface:runtime-inventory:', management_flow: false },
  { domain_surface_id: 'document', ref_prefix: 'surface:document:', management_flow: false },
  { domain_surface_id: 'research', ref_prefix: 'surface:research:', management_flow: false },
  { domain_surface_id: 'proof', ref_prefix: 'surface:proof:', management_flow: false },
  { domain_surface_id: 'history', ref_prefix: 'surface:history:', management_flow: false },
  { domain_surface_id: 'management', ref_prefix: 'surface:management:', management_flow: true }
];

export function domainSurfaceDescriptor(surfaceId: DomainSurfaceId): DomainSurfaceDescriptor {
  const descriptor = DOMAIN_SURFACE_DESCRIPTORS.find((candidate) => candidate.domain_surface_id === surfaceId);
  if (!descriptor) throw new Error(`unknown domain surface: ${surfaceId}`);
  return descriptor;
}

/**
 * DOMAIN-001..012 fail-closed eligibility: a domain surface renders ONLY
 * when Core's resolver emitted a contribution whose data_ref.ref starts with
 * the family prefix. Never inferred from CWD, tabs, or local state.
 */
export function domainSurfaceEligible(
  projection: ResolvedWorkspaceProjection,
  surfaceId: DomainSurfaceId
): boolean {
  const descriptor = domainSurfaceDescriptor(surfaceId);
  return projection.eligible_contributions.some((contribution) =>
    contribution.data_ref.ref.startsWith(descriptor.ref_prefix)
  );
}

/**
 * DOMAIN-007: a multiplexed runtime surface renders one contribution per
 * runtime ref Core emitted (each ref is its own eligible contribution).
 */
export function runtimeInventoryRefs(projection: ResolvedWorkspaceProjection): readonly string[] {
  return projection.eligible_contributions
    .filter((contribution) => contribution.data_ref.ref.startsWith('surface:runtime-inventory:'))
    .map((contribution) => contribution.data_ref.ref);
}

/**
 * DOMAIN-012: the explicit management flow is separate — management
 * contributions never appear inside the daily workspace composition.
 */
export function managementFlowEligible(projection: ResolvedWorkspaceProjection): boolean {
  const hasManagement = domainSurfaceEligible(projection, 'management');
  const hasDaily = DAILY_DOMAIN_SURFACES.some((surfaceId) => domainSurfaceEligible(projection, surfaceId));
  // Both may be present in the projection (Core emits both), but the desktop
  // never merges them into one composition — presentation separates the flow.
  return hasManagement && !hasDaily;
}
