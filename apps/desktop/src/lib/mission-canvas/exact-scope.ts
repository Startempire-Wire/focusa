import { sameWorkstreamAuthorityContext } from '../../../../../docs/contracts/spec135/mission-canvas-v1/typescript/mission-canvas-validators.generated';
import { exactWorkSurfaceIdentityKey, isExactWorkSurfaceIdentity } from './types';
import type {
  ExactWorkSurfaceIdentity,
  ProjectionLifecycleEvent,
  ResolvedWorkspaceProjection,
  WorkSurfaceProjection,
  WorkstreamAuthorityContext
} from './types';

/**
 * Compare the generated canonical identity context, not a tab, CWD, or latest
 * record.  The generated validator owns the identity equality semantics.
 */
export function sameWorkstreamAuthority(left: WorkstreamAuthorityContext, right: WorkstreamAuthorityContext): boolean {
  return sameWorkstreamAuthorityContext(left, right);
}

export function authorityFromProjection(projection: ResolvedWorkspaceProjection): WorkstreamAuthorityContext {
  return {
    workstream: projection.workstream,
    continuity_id: projection.continuity_id ?? null,
    attachment: projection.attachment ?? null,
    workspace_binding_id: projection.workspace_binding_id ?? null,
    runtime_object: projection.runtime_object ?? null,
    work_surface_id: projection.work_surface_id ?? projection.focused_work_surface_id ?? null
  };
}

export function authorityFromEvent(event: ProjectionLifecycleEvent): WorkstreamAuthorityContext {
  return {
    workstream: event.workstream,
    continuity_id: event.continuity_id ?? null,
    attachment: event.attachment ?? null,
    workspace_binding_id: event.workspace_binding_id ?? null,
    runtime_object: event.runtime_object ?? null,
    work_surface_id: event.work_surface_id ?? null
  };
}

export function workstreamAuthorityStorageKey(authority: WorkstreamAuthorityContext): string {
  return encodeURIComponent(JSON.stringify([
    authority.workstream,
    authority.continuity_id ?? null,
    authority.attachment ?? null,
    authority.workspace_binding_id ?? null,
    authority.runtime_object ?? null,
    authority.work_surface_id ?? null
  ]));
}

/**
 * Return a stable storage key only for a complete generated Work Surface
 * identity.  A legacy row, project/continuity pair, or aggregate identity has
 * no key because it cannot be an authority-bearing Desktop target.
 */
export function exactScopeKey(
  value: WorkSurfaceProjection | ExactWorkSurfaceIdentity
): string | undefined {
  const identity = isProjection(value) ? value.identity : value;
  if (!isExactWorkSurfaceIdentity(identity)) return undefined;
  return exactWorkSurfaceIdentityKey(identity);
}

function isProjection(value: WorkSurfaceProjection | ExactWorkSurfaceIdentity): value is WorkSurfaceProjection {
  return typeof value === 'object'
    && value !== null
    && 'identity' in value;
}
