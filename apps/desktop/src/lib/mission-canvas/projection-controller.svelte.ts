import {
  sameWorkstreamAuthorityContext,
  validateMissionCanvasContract
} from '../../../../../docs/contracts/spec135/mission-canvas-v1/typescript/mission-canvas-validators.generated';
import type {
  MissionCanvasClient,
  MissionCanvasOperationInput
} from '../../../../../docs/contracts/spec135/mission-canvas-v1/typescript/mission-canvas-client.generated';
import { authorityFromProjection, sameWorkstreamAuthority as sameScope } from './exact-scope';
import { collectLayoutContributionIds, validateLayoutIntegrity } from './layout-references';
import type { ResolvedWorkspaceProjection, WorkstreamAuthorityContext } from './types';

export type ProjectionState =
  | { kind: 'unbound' }
  | { kind: 'loading'; scope: WorkstreamAuthorityContext }
  | { kind: 'refreshing'; scope: WorkstreamAuthorityContext; projection: ResolvedWorkspaceProjection }
  | { kind: 'ready'; scope: WorkstreamAuthorityContext; projection: ResolvedWorkspaceProjection }
  | { kind: 'stale'; scope: WorkstreamAuthorityContext; projection: ResolvedWorkspaceProjection; reason: string }
  | { kind: 'blocked'; scope?: WorkstreamAuthorityContext; reason: string }
  | { kind: 'error'; scope?: WorkstreamAuthorityContext; reason: string };

/**
 * The generated client is the normal source.  The function form remains a
 * narrow test/integration seam for callers that already bind projectionGet;
 * neither form gives Desktop authority to resolve or repair a Workstream.
 */
export type ProjectionLoader = (scope: WorkstreamAuthorityContext) => Promise<unknown>;
export type ProjectionClient = Pick<MissionCanvasClient, 'projectionGet'>;
export type ProjectionSource = ProjectionLoader | ProjectionClient;

export type ProjectionValidation =
  | { valid: true; errors: string[]; projection: ResolvedWorkspaceProjection }
  | { valid: false; errors: string[]; reason: string; failure: 'invalid' | 'scope' };

type LayoutValidation =
  | { valid: true; errors: string[] }
  | Extract<ProjectionValidation, { valid: false }>;

type RevisionedProjection = Pick<ResolvedWorkspaceProjection, 'projection_revision' | 'layout_revision' | 'durable_event_cursor'>;

/**
 * Validate a generated projection at the renderer boundary.
 *
 * Generated validators own the DTO shape.  This boundary only adds the
 * checks needed to keep a malformed or foreign response out of Svelte: exact
 * Workstream authority, nested contribution authority, projection watermarks,
 * and layout references.  It never computes eligibility or changes layout.
 */
export function validateProjection(
  value: unknown,
  expectedScope?: WorkstreamAuthorityContext
): ProjectionValidation {
  const structural = validateMissionCanvasContract('ResolvedWorkspaceProjection', value);
  if (!structural.valid) return invalidProjection(structural.errors);

  const projection = value as ResolvedWorkspaceProjection;
  if (projection.schema !== 'focusa.resolved_workspace_projection.v1') {
    return invalidProjection(['invalid:schema']);
  }
  if (!Array.isArray(projection.eligible_contributions)) {
    return invalidProjection(['invalid:eligible_contributions']);
  }
  if (!Array.isArray(projection.candidate_contribution_ids)) {
    return invalidProjection(['invalid:candidate_contribution_ids']);
  }
  if (!Array.isArray(projection.omission_diagnostics)) {
    return invalidProjection(['invalid:omission_diagnostics']);
  }
  if (!Array.isArray(projection.operation_bindings)) {
    return invalidProjection(['invalid:operation_bindings']);
  }
  if (typeof projection.projection_digest !== 'string' || projection.projection_digest.trim().length === 0) {
    return invalidProjection(['invalid:projection_digest']);
  }
  if (!isNonNegativeSafeInteger(projection.projection_revision)) {
    return invalidProjection(['invalid:projection_revision']);
  }
  if (!isNonNegativeSafeInteger(projection.layout_revision)) {
    return invalidProjection(['invalid:layout_revision']);
  }
  if (projection.focused_work_surface_id !== null
    && (typeof projection.focused_work_surface_id !== 'string'
      || projection.focused_work_surface_id.trim().length === 0)) {
    return invalidProjection(['invalid:focused_work_surface_id']);
  }
  if (projection.focused_work_surface_id !== null
    && (!projection.attachment
      || projection.work_surface_id !== projection.focused_work_surface_id)) {
    return invalidProjection(['invalid:focused_work_surface_authority']);
  }
  if (!parseProjectionCursor(projection.durable_event_cursor)) {
    return invalidProjection(['invalid:projection_cursor']);
  }

  const responseAuthority = authorityFromProjection(projection);
  const responseAuthorityValidation = validateMissionCanvasContract(
    'WorkstreamAuthorityContext',
    responseAuthority
  );
  if (!responseAuthorityValidation.valid) {
    return invalidProjection(responseAuthorityValidation.errors);
  }

  if (expectedScope !== undefined) {
    const expectedValidation = validateMissionCanvasContract(
      'WorkstreamAuthorityContext',
      expectedScope
    );
    if (!expectedValidation.valid) {
      return invalidProjection(
        [`invalid_workstream_scope:${expectedValidation.errors.join(',')}`],
        'scope'
      );
    }
    if (!sameWorkstreamAuthorityContext(responseAuthority, expectedScope)) {
      return invalidProjection(['projection_scope_mismatch'], 'scope');
    }
  }

  const eligibleIds = new Set<string>();
  for (const [index, candidate] of projection.eligible_contributions.entries()) {
    const contributionValidation = validateMissionCanvasContract('ResolvedContribution', candidate);
    if (!contributionValidation.valid) {
      return invalidProjection(
        contributionValidation.errors.map((error) => `contribution:${index}:${error}`)
      );
    }
    const contribution = candidate as ResolvedWorkspaceProjection['eligible_contributions'][number];
    if (eligibleIds.has(contribution.contribution_id)) {
      return invalidProjection([`duplicate:eligible_contributions:${contribution.contribution_id}`]);
    }
    eligibleIds.add(contribution.contribution_id);

    const contributionAuthority = authorityFromContribution(contribution);
    if (!contributionAuthority) {
      return invalidProjection([`contribution:${index}:invalid:authority`]);
    }
    const contributionAuthorityValidation = validateMissionCanvasContract(
      'WorkstreamAuthorityContext',
      contributionAuthority
    );
    if (!contributionAuthorityValidation.valid) {
      return invalidProjection(
        contributionAuthorityValidation.errors.map((error) => `contribution:${index}:${error}`)
      );
    }
    if (!sameWorkstreamAuthorityContext(contributionAuthority, responseAuthority)) {
      return invalidProjection(['foreign_contribution_scope'], 'scope');
    }
  }

  const layoutValidation = validateLayout(projection.layout_tree, eligibleIds);
  if (!layoutValidation.valid) return layoutValidation;

  return { valid: true, errors: [], projection };
}

/**
 * Reject a response that moves any canonical projection watermark backwards.
 * Cursor namespaces are not interchangeable: a client must not compare an
 * event cursor to an unrelated cursor namespace and guess ordering.
 */
export function rejectStaleRevision(
  previous: RevisionedProjection | undefined,
  next: RevisionedProjection
): string | undefined {
  if (!isNonNegativeSafeInteger(next.projection_revision)) return 'projection_revision_invalid';
  if (!isNonNegativeSafeInteger(next.layout_revision)) return 'projection_layout_revision_invalid';
  const nextCursor = parseProjectionCursor(next.durable_event_cursor);
  if (!nextCursor) return 'projection_cursor_invalid';
  if (!previous) return undefined;
  if (!isNonNegativeSafeInteger(previous.projection_revision)) return 'prior_projection_revision_invalid';
  if (!isNonNegativeSafeInteger(previous.layout_revision)) return 'prior_projection_layout_revision_invalid';
  if (next.projection_revision < previous.projection_revision) return 'projection_revision_regressed';
  if (next.layout_revision < previous.layout_revision) return 'projection_layout_revision_regressed';

  const priorCursor = parseProjectionCursor(previous.durable_event_cursor);
  if (!priorCursor) return 'prior_projection_cursor_invalid';
  if (priorCursor.kind !== nextCursor.kind) return 'projection_cursor_namespace_mismatch';
  if (nextCursor.value < priorCursor.value) return 'projection_cursor_regressed';
  return undefined;
}

export class MissionCanvasProjectionController {
  state = $state<ProjectionState>({ kind: 'unbound' });
  #requestGeneration = 0;
  readonly #loader: ProjectionLoader;

  constructor(source: ProjectionSource) {
    this.#loader = typeof source === 'function'
      ? source
      : (scope) => source.projectionGet({ ...scope } as MissionCanvasOperationInput);
  }

  async refresh(inputScope: WorkstreamAuthorityContext): Promise<void> {
    await this.load(inputScope);
  }

  async load(inputScope: WorkstreamAuthorityContext): Promise<void> {
    const generation = ++this.#requestGeneration;
    const scope = cloneAndFreezeScope(inputScope);
    if (!scope) {
      this.state = {
        kind: 'blocked',
        reason: `invalid_workstream_scope:${scopeValidationErrors(inputScope).join(',') || 'uncloneable'}`
      };
      return;
    }

    const prior = this.projectionForScope(scope);
    this.state = prior
      ? { kind: 'refreshing', scope, projection: prior }
      : { kind: 'loading', scope };

    try {
      // Keep the controller's binding immutable, while giving generated
      // transport a detached request object that cannot mutate state.
      const value = await this.#loader(cloneJson(scope));
      if (generation !== this.#requestGeneration) return;

      const validation = validateProjection(value, scope);
      if (!validation.valid) {
        this.applyValidationFailure(scope, prior, validation);
        return;
      }

      const staleReason = rejectStaleRevision(prior, validation.projection);
      if (staleReason) {
        this.applyStale(scope, prior, staleReason);
        return;
      }

      this.state = { kind: 'ready', scope, projection: freezeProjection(validation.projection) };
    } catch (error) {
      if (generation !== this.#requestGeneration) return;
      const reason = error instanceof Error ? error.message : 'projection_load_failed';
      if (isScopeFailure(reason)) {
        this.state = { kind: 'blocked', scope, reason };
      } else if (isStaleFailure(reason) && prior) {
        this.applyStale(scope, prior, reason);
      } else {
        this.state = prior ? { kind: 'stale', scope, projection: prior, reason } : { kind: 'error', scope, reason };
      }
    }
  }

  accept(inputScope: WorkstreamAuthorityContext, value: unknown): boolean {
    this.#requestGeneration += 1;
    const scope = cloneAndFreezeScope(inputScope);
    if (!scope) {
      this.state = {
        kind: 'blocked',
        reason: `invalid_workstream_scope:${scopeValidationErrors(inputScope).join(',') || 'uncloneable'}`
      };
      return false;
    }

    const prior = this.projectionForScope(scope);
    const validation = validateProjection(value, scope);
    if (!validation.valid) {
      this.applyValidationFailure(scope, prior, validation);
      return false;
    }

    const staleReason = rejectStaleRevision(prior, validation.projection);
    if (staleReason) {
      this.applyStale(scope, prior, staleReason);
      return false;
    }

    this.state = { kind: 'ready', scope, projection: freezeProjection(validation.projection) };
    return true;
  }

  markStale(reason: string): void {
    const projection = this.currentProjection();
    if (projection && (this.state.kind === 'ready' || this.state.kind === 'refreshing' || this.state.kind === 'stale')) {
      this.state = {
        kind: 'stale',
        scope: this.state.scope,
        projection,
        reason: reason.trim() || 'projection_stale'
      };
    }
  }

  clear(): void {
    this.#requestGeneration += 1;
    this.state = { kind: 'unbound' };
  }

  private applyValidationFailure(
    scope: WorkstreamAuthorityContext,
    prior: ResolvedWorkspaceProjection | undefined,
    validation: Extract<ProjectionValidation, { valid: false }>
  ): void {
    if (validation.failure === 'scope') {
      this.state = { kind: 'blocked', scope, reason: validation.reason ?? 'projection_scope_mismatch' };
    } else if (prior) {
      this.state = { kind: 'stale', scope, projection: prior, reason: validation.reason ?? 'invalid_projection' };
    } else {
      this.state = { kind: 'error', scope, reason: validation.reason ?? 'invalid_projection' };
    }
  }

  private applyStale(
    scope: WorkstreamAuthorityContext,
    prior: ResolvedWorkspaceProjection | undefined,
    reason: string
  ): void {
    if (prior) {
      this.state = { kind: 'stale', scope, projection: prior, reason };
    } else {
      this.state = { kind: 'error', scope, reason };
    }
  }

  private projectionForScope(scope: WorkstreamAuthorityContext): ResolvedWorkspaceProjection | undefined {
    const projection = this.currentProjection();
    return projection && sameScope(authorityFromProjection(projection), scope) ? projection : undefined;
  }

  private currentProjection(): ResolvedWorkspaceProjection | undefined {
    return this.state.kind === 'ready' || this.state.kind === 'refreshing' || this.state.kind === 'stale'
      ? this.state.projection
      : undefined;
  }
}

function invalidProjection(
  errors: string[],
  failure: 'invalid' | 'scope' = 'invalid'
): Extract<ProjectionValidation, { valid: false }> {
  return {
    valid: false,
    errors,
    reason: errors.join(',') || 'invalid_projection',
    failure
  };
}

function authorityFromContribution(
  contribution: unknown
): WorkstreamAuthorityContext | undefined {
  if (!contribution || typeof contribution !== 'object' || Array.isArray(contribution)) return undefined;
  const authority = (contribution as Record<string, unknown>).authority;
  if (!authority || typeof authority !== 'object' || Array.isArray(authority)) return undefined;
  const value = authority as Record<string, unknown>;
  return {
    workstream: value.workstream as WorkstreamAuthorityContext['workstream'],
    continuity_id: value.continuity_id as WorkstreamAuthorityContext['continuity_id'] ?? null,
    attachment: value.attachment as WorkstreamAuthorityContext['attachment'] ?? null,
    workspace_binding_id: value.workspace_binding_id as WorkstreamAuthorityContext['workspace_binding_id'] ?? null,
    runtime_object: value.runtime_object as WorkstreamAuthorityContext['runtime_object'] ?? null,
    work_surface_id: value.work_surface_id as WorkstreamAuthorityContext['work_surface_id'] ?? null
  };
}

function validateLayout(
  layout: unknown,
  eligibleIds: ReadonlySet<string>
): LayoutValidation {
  if (!layout || typeof layout !== 'object' || Array.isArray(layout)) {
    return invalidProjection(['invalid:layout_tree']);
  }

  const shapeError = validateLayoutNodeShape(layout);
  if (shapeError) return invalidProjection([`invalid_layout:${shapeError}`]);

  try {
    const issues = validateLayoutIntegrity(layout as ResolvedWorkspaceProjection['layout_tree']);
    if (issues.length > 0) {
      return invalidProjection(issues.map((issue) => `invalid_layout:${issue.code}:${issue.nodeId}`));
    }
    const layoutIds = collectLayoutContributionIds(layout as ResolvedWorkspaceProjection['layout_tree']);
    for (const contributionId of layoutIds) {
      if (!eligibleIds.has(contributionId)) {
        return invalidProjection([`invalid_layout:unknown_contribution:${contributionId}`]);
      }
    }
  } catch {
    return invalidProjection(['invalid:layout_tree']);
  }
  return { valid: true, errors: [] };
}

function validateLayoutNodeShape(value: unknown, seen = new WeakSet<object>(), path = 'root'): string | undefined {
  if (!value || typeof value !== 'object' || Array.isArray(value)) return `${path}:node`;
  if (seen.has(value)) return `${path}:cycle`;
  seen.add(value);

  const node = value as Record<string, unknown>;
  if (typeof node.node_id !== 'string' || node.node_id.trim().length === 0) return `${path}:node_id`;
  if (typeof node.kind !== 'string') return `${path}:kind`;

  const children = (kind: string): string | undefined => {
    if (!Array.isArray(node.children) || node.children.length === 0) return `${path}:${kind}:children`;
    for (const [index, child] of node.children.entries()) {
      const error = validateLayoutNodeShape(child, seen, `${path}.children[${index}]`);
      if (error) return error;
    }
    return undefined;
  };

  switch (node.kind) {
    case 'single':
      return typeof node.contribution_id === 'string' && node.contribution_id.trim().length > 0
        ? undefined
        : `${path}:contribution_id`;
    case 'split':
      if (node.orientation !== 'horizontal' && node.orientation !== 'vertical') return `${path}:orientation`;
      if (typeof node.ratio !== 'number' || !Number.isFinite(node.ratio) || node.ratio < 0 || node.ratio > 1) return `${path}:ratio`;
      return children('split');
    case 'stack':
      return children('stack');
    case 'grid':
      if (!isNonNegativeSafeInteger(node.columns) || node.columns < 1) return `${path}:columns`;
      return children('grid');
    case 'tabs':
      if (!Array.isArray(node.contribution_ids) || node.contribution_ids.length === 0) return `${path}:contribution_ids`;
      if (node.contribution_ids.some((id) => typeof id !== 'string' || id.trim().length === 0)) return `${path}:contribution_ids`;
      if (typeof node.active_contribution_id !== 'string' || !node.contribution_ids.includes(node.active_contribution_id)) {
        return `${path}:active_contribution_id`;
      }
      return undefined;
    case 'inspector':
      if (node.side !== 'start' && node.side !== 'end') return `${path}:side`;
      if (!Array.isArray(node.inspector_contribution_ids) || node.inspector_contribution_ids.length === 0) {
        return `${path}:inspector_contribution_ids`;
      }
      if (node.inspector_contribution_ids.some((id) => typeof id !== 'string' || id.trim().length === 0)) {
        return `${path}:inspector_contribution_ids`;
      }
      return validateLayoutNodeShape(node.primary, seen, `${path}.primary`);
    default:
      return `${path}:kind`;
  }
}

function isNonNegativeSafeInteger(value: unknown): value is number {
  return typeof value === 'number' && Number.isSafeInteger(value) && value >= 0;
}

function parseProjectionCursor(cursor: unknown): { kind: string; value: number } | undefined {
  if (typeof cursor !== 'string') return undefined;
  const normalized = cursor.trim();
  const prefixed = /^(event|cursor|mission-canvas):([0-9]+)$/.exec(normalized);
  const match = prefixed ?? /^([0-9]+)$/.exec(normalized);
  if (!match) return undefined;
  const kind = prefixed ? prefixed[1] : 'opaque-numeric';
  const value = Number(prefixed ? prefixed[2] : match[1]);
  return Number.isSafeInteger(value) ? { kind, value } : undefined;
}

function scopeValidationErrors(scope: unknown): string[] {
  return validateMissionCanvasContract('WorkstreamAuthorityContext', scope).errors;
}

function cloneAndFreezeScope(scope: unknown): WorkstreamAuthorityContext | undefined {
  const validation = validateMissionCanvasContract('WorkstreamAuthorityContext', scope);
  if (!validation.valid) return undefined;
  try {
    return deepFreeze(cloneJson(scope as WorkstreamAuthorityContext));
  } catch {
    return undefined;
  }
}

function freezeProjection(projection: ResolvedWorkspaceProjection): ResolvedWorkspaceProjection {
  return deepFreeze(cloneJson(projection));
}

function cloneJson<T>(value: T): T {
  if (typeof globalThis.structuredClone === 'function') {
    try {
      return globalThis.structuredClone(value);
    } catch {
      // Svelte 5 $state wraps values in reactive proxies that structuredClone
      // cannot serialize (DataCloneError on internal functions). Fall back to
      // JSON round-trip which strips the proxy and yields plain data.
      return JSON.parse(JSON.stringify(value)) as T;
    }
  }
  return JSON.parse(JSON.stringify(value)) as T;
}

function deepFreeze<T>(value: T, seen = new WeakSet<object>()): T {
  if (!value || typeof value !== 'object' || seen.has(value as object)) return value;
  seen.add(value as object);
  for (const child of Object.values(value as Record<string, unknown>)) deepFreeze(child, seen);
  return Object.freeze(value);
}

function isScopeFailure(reason: string): boolean {
  return reason === 'projection_scope_mismatch'
    || reason === 'foreign_projection_scope'
    || reason === 'foreign_contribution_scope'
    || reason === 'foreign_response_scope'
    || reason.startsWith('foreign_')
    || reason.startsWith('invalid_workstream_scope:');
}

function isStaleFailure(reason: string): boolean {
  return reason.startsWith('stale_projection_')
    || reason === 'projection_revision_regressed'
    || reason === 'projection_layout_revision_regressed'
    || reason === 'projection_cursor_regressed'
    || reason === 'projection_cursor_namespace_mismatch';
}
