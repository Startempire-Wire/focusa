import type { ExactScope } from './types';

export function sameExactScope(left: ExactScope, right: ExactScope): boolean {
  return left.project_root === right.project_root
    && left.continuity_id === right.continuity_id
    && left.attachment_id === right.attachment_id
    && left.session_id === right.session_id
    && (left.instance_id ?? null) === (right.instance_id ?? null)
    && (left.working_subpath_id ?? null) === (right.working_subpath_id ?? null);
}

export function exactScopeStorageKey(scope: ExactScope): string {
  return encodeURIComponent(JSON.stringify([
    scope.project_root,
    scope.continuity_id,
    scope.attachment_id,
    scope.session_id,
    scope.instance_id ?? null,
    scope.working_subpath_id ?? null
  ]));
}
