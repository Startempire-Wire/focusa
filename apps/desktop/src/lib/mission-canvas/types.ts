import {
  sameWorkstreamKey,
  validateMissionCanvasContract
} from '../../../../../docs/contracts/spec135/mission-canvas-v1/typescript/mission-canvas-validators.generated';
import type {
  ActivityMode,
  AttachmentId,
  AttachmentKey,
  CanvasDraftState,
  ContinuityId,
  ContributionKind,
  DomainPackInstallReceipt,
  GridLayoutNode,
  InspectorLayoutNode,
  InstanceId,
  LayoutNode,
  OperationBinding,
  ProfileLayoutMemory,
  ProjectionLifecycleEvent,
  RecipientResolution,
  ResolvedContribution,
  ResolvedWorkspaceProjection,
  RuntimeObjectRef,
  SessionId,
  SingleLayoutNode,
  SplitLayoutNode,
  StackLayoutNode,
  TabLayoutNode,
  WorkSurfaceId,
  WorkSurfaceIdentity as GeneratedWorkSurfaceIdentity,
  WorkstreamAuthorityContext,
  WorkstreamKey,
  WorkspaceBindingId,
  WorkspaceProfile
} from '../../../../../docs/contracts/spec135/mission-canvas-v1/typescript/mission-canvas-types.generated';

export type {
  ActivityMode,
  AttachmentId,
  AttachmentKey,
  CanvasDraftState,
  ContinuityId,
  ContributionKind,
  DomainPackInstallReceipt,
  GridLayoutNode,
  InspectorLayoutNode,
  InstanceId,
  LayoutNode,
  OperationBinding,
  ProfileLayoutMemory,
  ProjectionLifecycleEvent,
  RecipientResolution,
  ResolvedContribution,
  ResolvedWorkspaceProjection,
  RuntimeObjectRef,
  SessionId,
  SingleLayoutNode,
  SplitLayoutNode,
  StackLayoutNode,
  TabLayoutNode,
  WorkSurfaceId,
  WorkSurfaceIdentity,
  WorkstreamAuthorityContext,
  WorkstreamKey,
  WorkspaceBindingId,
  WorkspaceProfile
} from '../../../../../docs/contracts/spec135/mission-canvas-v1/typescript/mission-canvas-types.generated';

export const MAX_MISSION_CANVAS_ROWS = 200;

export type WorkSurfaceKind =
  | 'project_overview'
  | 'pi_session'
  | 'uiai_browser'
  | 'silent_session'
  | 'document'
  | 'research'
  | 'provider_item'
  | 'evidence'
  | 'custom';

const WORK_SURFACE_KINDS = new Set<WorkSurfaceKind>([
  'project_overview',
  'pi_session',
  'uiai_browser',
  'silent_session',
  'document',
  'research',
  'provider_item',
  'evidence',
  'custom'
]);

/**
 * A WorkSurfaceIdentity with the attachment required for an authority-bearing
 * Desktop Work Surface.  The transport definition deliberately permits an
 * aggregate Work Surface without an Attachment; this renderer model does not,
 * because attach/focus/steer actions require the complete identity chain.
 */
export type ExactWorkSurfaceIdentity = Omit<GeneratedWorkSurfaceIdentity, 'attachment'> & {
  attachment: AttachmentKey;
};

export type WorkSurfaceQuarantineReason =
  | 'missing_exact_identity'
  | 'invalid_identity'
  | 'foreign_scope'
  | 'foreign_attachment_workstream'
  | 'identity_mismatch'
  | 'duplicate_identity';

/** Bounded, non-sensitive diagnostic for a row withheld from Desktop actions. */
export interface WorkSurfaceQuarantine {
  rowIndex: number;
  workSurfaceId?: string;
  reason: WorkSurfaceQuarantineReason;
}

export interface WorkSurfaceProjectionResult {
  surfaces: WorkSurfaceProjection[];
  quarantined: WorkSurfaceQuarantine[];
}

/**
 * Desktop's render model preserves the Pi overlay's display fields, while the
 * nested identity is the sole authority-bearing source.  The legacy scalar
 * IDs remain presentation fields for existing labels; they are copied from
 * `identity.attachment` and are never used to reconstruct authority.
 */
export interface WorkSurfaceProjection {
  identity: ExactWorkSurfaceIdentity;
  workSurfaceId: WorkSurfaceId;
  displayName: string;
  kind: WorkSurfaceKind;
  projectRoot: string;
  continuityId: string;
  workpointId: string;
  workItemRef: string;
  instanceId: InstanceId;
  sessionId: SessionId;
  attachmentId: AttachmentId;
  role: string;
  rendererId: string;
  pinned: boolean;
  groupId: string;
  splitGroupId: string;
  lifecycleState: string;
  semanticActivity: string;
  health: string;
  unreadEventCount: number;
  pendingApprovalCount: number;
  conflictCount: number;
  blockerCount: number;
  writerLeaseRef: string;
  worktreeRef: string;
  browserIsolationClass: string;
}

/**
 * Project the generated Work Surface list into Desktop rows.  Only rows with
 * an explicit generated WorkstreamKey + AttachmentKey + WorkSurfaceId survive.
 * Legacy `(project_root, continuity_id, ...)` rows are returned through the
 * diagnostics helper as quarantined and are never repaired or guessed.
 */
export function projectWorkSurfaces(
  payload: unknown,
  expectedScope?: WorkstreamAuthorityContext | null
): WorkSurfaceProjection[] {
  return projectWorkSurfacesWithDiagnostics(payload, expectedScope).surfaces;
}

export function workSurfaceDetail(surface: WorkSurfaceProjection): string[] {
  return [
    `Surface: ${surface.workSurfaceId} · ${surface.kind} · ${surface.lifecycleState} · ${surface.health}`,
    `Scope: ${surface.projectRoot || 'unknown project'} · ${surface.continuityId || 'unknown continuity'}`,
    `Attachment: ${surface.instanceId || 'unknown instance'} · ${surface.sessionId || 'unknown session'} · ${surface.attachmentId || 'unknown attachment'}`,
    `Work: ${surface.workpointId || 'no Workpoint'} · ${surface.workItemRef || 'no provider item'}`,
    `Activity: ${surface.semanticActivity || 'not reported'} · ${surface.unreadEventCount} unread · ${surface.pendingApprovalCount} approvals`,
    `Isolation: ${surface.writerLeaseRef || 'no writer lease'} · ${surface.worktreeRef || 'no worktree'} · ${surface.browserIsolationClass}`
  ];
}

export function workSurfaceLabel(surface: WorkSurfaceProjection): string {
  const markers = [
    surface.pinned ? 'pinned' : '',
    surface.unreadEventCount ? `${surface.unreadEventCount} unread` : '',
    surface.pendingApprovalCount ? `${surface.pendingApprovalCount} approvals` : '',
    surface.conflictCount ? `${surface.conflictCount} conflicts` : '',
    surface.splitGroupId ? `split:${surface.splitGroupId}` : '',
    surface.writerLeaseRef ? 'writer lease' : '',
    surface.worktreeRef ? `worktree:${surface.worktreeRef}` : '',
    surface.browserIsolationClass !== 'not-applicable' ? surface.browserIsolationClass : ''
  ].filter(Boolean);
  return `${surface.displayName} · ${surface.kind} · ${surface.lifecycleState}${markers.length ? ` · ${markers.join(' · ')}` : ''}`;
}

export function projectWorkSurfacesWithDiagnostics(
  payload: unknown,
  expectedScope?: WorkstreamAuthorityContext | null
): WorkSurfaceProjectionResult {
  const root = asRecord(payload);
  const rows = Array.isArray(root?.surfaces)
    ? root.surfaces
    : Array.isArray(root?.work_surfaces)
      ? root.work_surfaces
      : [];
  const projected: Array<{ rowIndex: number; surface: WorkSurfaceProjection; key: string }> = [];
  const quarantined: WorkSurfaceQuarantine[] = [];
  const expectedScopeValid = expectedScope === undefined
    || (expectedScope !== null
      && validateMissionCanvasContract('WorkstreamAuthorityContext', expectedScope).valid);

  rows.slice(0, MAX_MISSION_CANVAS_ROWS).forEach((rawRow, rowIndex) => {
    const row = asRecord(rawRow);
    const surfaceId = text(firstDefined(row.work_surface_id, row.surface_id));
    const identityResult = identityForRow(row);
    if (!identityResult.identity) {
      quarantined.push({
        rowIndex,
        ...(surfaceId ? { workSurfaceId: surfaceId } : {}),
        reason: identityResult.reason
      });
      return;
    }

    if (!expectedScopeValid || !identityMatchesExpectedScope(identityResult.identity, expectedScope)) {
      quarantined.push({
        rowIndex,
        workSurfaceId: identityResult.identity.work_surface_id,
        reason: 'foreign_scope'
      });
      return;
    }

    const consistency = validateRowIdentityConsistency(row, identityResult.identity);
    if (consistency) {
      quarantined.push({
        rowIndex,
        workSurfaceId: identityResult.identity.work_surface_id,
        reason: consistency
      });
      return;
    }

    const key = exactWorkSurfaceIdentityKey(identityResult.identity);
    const existing = projected.findIndex((item) => item.key === key);
    if (existing >= 0) {
      const previous = projected[existing];
      projected.splice(existing, 1);
      quarantined.push({
        rowIndex: previous.rowIndex,
        workSurfaceId: previous.surface.workSurfaceId,
        reason: 'duplicate_identity'
      });
      quarantined.push({
        rowIndex,
        workSurfaceId: identityResult.identity.work_surface_id,
        reason: 'duplicate_identity'
      });
      return;
    }

    projected.push({ rowIndex, key, surface: projectSurfaceRow(row, identityResult.identity) });
  });

  return {
    surfaces: projected.map(({ surface }) => surface),
    quarantined
  };
}

function identityMatchesExpectedScope(
  identity: ExactWorkSurfaceIdentity,
  expectedScope: WorkstreamAuthorityContext | undefined
): boolean {
  if (expectedScope === undefined) return true;
  if (expectedScope === null) return false;
  const authority: WorkstreamAuthorityContext = {
    workstream: identity.workstream,
    continuity_id: identity.continuity_id ?? identity.attachment.continuity_id ?? null,
    attachment: identity.attachment,
    workspace_binding_id: identity.attachment.workspace_binding_id,
    runtime_object: identity.runtime_object ?? null,
    work_surface_id: identity.work_surface_id
  };
  if (!validateMissionCanvasContract('WorkstreamAuthorityContext', authority).valid) return false;
  if (!sameWorkstreamKey(authority.workstream, expectedScope.workstream)) return false;
  return (expectedScope.continuity_id == null || authority.continuity_id === expectedScope.continuity_id)
    && (expectedScope.attachment == null || sameWorkstreamKey(authority.attachment, expectedScope.attachment))
    && (expectedScope.workspace_binding_id == null || authority.workspace_binding_id === expectedScope.workspace_binding_id)
    && (expectedScope.runtime_object == null || sameStable(authority.runtime_object, expectedScope.runtime_object))
    && (expectedScope.work_surface_id == null || authority.work_surface_id === expectedScope.work_surface_id);
}

/** Runtime guard for the exact generated identity required by authority-bearing UI. */
export function isExactWorkSurfaceIdentity(value: unknown): value is ExactWorkSurfaceIdentity {
  const object = asRecord(value);
  if (!object || !object.attachment) return false;
  const validation = validateMissionCanvasContract('WorkSurfaceIdentity', object);
  if (!validation.valid) return false;
  const attachment = asRecord(object.attachment);
  const workstream = asRecord(object.workstream);
  const scope = asRecord(workstream.scope);
  const scopeKey = asRecord(scope.scope_key);
  return nonEmpty(object.work_surface_id)
    && nonEmpty(workstream.workstream_id)
    && validScopeKind(scope.scope_kind)
    && scopeKey.scope_kind === scope.scope_kind
    && validScopeKind(scopeKey.scope_kind)
    && nonEmpty(scopeKey.scope_id)
    && nonEmpty(scopeKey.root_path)
    && nonEmpty(scopeKey.canonical_name)
    && nonEmpty(scopeKey.fingerprint)
    && nonEmpty(attachment.instance_id)
    && nonEmpty(attachment.session_id)
    && nonEmpty(attachment.attachment_id)
    && nonEmpty(attachment.workspace_binding_id)
    && sameWorkstreamKey(attachment.workstream, object.workstream)
    && (object.continuity_id == null
      || object.continuity_id === attachment.continuity_id);
}

function projectSurfaceRow(row: Record<string, unknown>, identity: ExactWorkSurfaceIdentity): WorkSurfaceProjection {
  const attachment = identity.attachment;
  const scope = asRecord(asRecord(identity.workstream).scope);
  const scopeKey = asRecord(scope.scope_key);
  const presentation = asRecord(row.presentation);
  const activity = asRecord(row.activity);
  const isolation = asRecord(row.isolation);
  const paneId = text(row.pane_id);
  const rawKind = text(firstDefined(row.kind, row.surface_kind, 'custom')) as WorkSurfaceKind;
  const kind = WORK_SURFACE_KINDS.has(rawKind) ? rawKind : 'custom';
  const continuityId = text(firstDefined(
    identity.continuity_id,
    attachment.continuity_id,
    asRecord(row.scope).continuity_id,
    row.continuity_id
  ));

  return {
    identity,
    workSurfaceId: identity.work_surface_id,
    displayName: text(firstDefined(row.display_name, presentation.title, row.title, identity.work_surface_id)),
    kind,
    projectRoot: text(firstDefined(asRecord(row.scope).project_root, row.project_root, scopeKey.root_path)),
    continuityId,
    workpointId: text(firstDefined(asRecord(row.scope).workpoint_id, row.workpoint_id)),
    workItemRef: text(firstDefined(asRecord(row.scope).work_item_ref, row.work_item_ref)),
    instanceId: attachment.instance_id,
    sessionId: attachment.session_id,
    attachmentId: attachment.attachment_id,
    role: text(firstDefined(asRecord(row.primary_attachment).role, asRecord(row.attachment).role, row.role)),
    rendererId: text(firstDefined(presentation.renderer_id, row.renderer_id)),
    pinned: Boolean(presentation.pinned ?? row.pinned),
    groupId: text(firstDefined(presentation.group_id, row.group_id, continuityId, identity.workstream.workstream_id)),
    splitGroupId: text(firstDefined(
      presentation.split_group_id,
      row.split_group_id,
      paneId && paneId !== 'primary' ? paneId : ''
    )),
    lifecycleState: text(firstDefined(activity.lifecycle_state, row.lifecycle_state, row.status, 'unknown')),
    semanticActivity: text(firstDefined(activity.semantic_activity, row.semantic_activity)),
    health: text(firstDefined(activity.health, row.health, 'unknown')),
    unreadEventCount: count(activity.unread_event_count ?? row.unread_event_count ?? (row.unread ? 1 : 0)),
    pendingApprovalCount: count(activity.pending_approval_count ?? row.pending_approval_count),
    conflictCount: count(activity.conflict_count ?? row.conflict_count),
    blockerCount: count(activity.blocker_count ?? row.blocker_count),
    writerLeaseRef: text(firstDefined(isolation.writer_lease_ref, row.writer_lease_ref)),
    worktreeRef: text(firstDefined(isolation.worktree_ref, row.worktree_ref)),
    browserIsolationClass: text(firstDefined(
      isolation.browser_isolation_class,
      row.browser_isolation_class,
      'not-applicable'
    ))
  };
}

function identityForRow(row: Record<string, unknown>): {
  identity?: ExactWorkSurfaceIdentity;
  reason: WorkSurfaceQuarantineReason;
} {
  const explicit = 'identity' in row
    ? row.identity
    : 'work_surface_identity' in row
      ? row.work_surface_identity
      : 'authority' in row
        ? row.authority
        : undefined;
  const candidate = explicit !== undefined
    ? asRecord(explicit)
    : explicitIdentityCandidate(row);
  if (!candidate) return { reason: 'missing_exact_identity' };

  return normalizeExactIdentity(candidate);
}

function explicitIdentityCandidate(row: Record<string, unknown>): Record<string, unknown> | undefined {
  const scope = asRecord(row.scope);
  const workstream = firstDefined(row.workstream, scope.workstream);
  const attachment = firstDefined(row.attachment, row.primary_attachment);
  const workSurfaceId = firstDefined(row.work_surface_id, row.surface_id);
  if (workstream === undefined && attachment === undefined && workSurfaceId === undefined) return undefined;
  // A legacy row may expose a surface ID or flat project/continuity fields,
  // but without both generated authority owners it is not a partial identity
  // that Desktop may repair.
  if (workstream === undefined || attachment === undefined) return undefined;
  return {
    workstream,
    attachment,
    work_surface_id: workSurfaceId,
    ...(Object.hasOwn(row, 'continuity_id') ? { continuity_id: row.continuity_id } : {}),
    ...(Object.hasOwn(row, 'runtime_object') ? { runtime_object: row.runtime_object } : {})
  };
}

function normalizeExactIdentity(candidate: Record<string, unknown>): {
  identity?: ExactWorkSurfaceIdentity;
  reason: WorkSurfaceQuarantineReason;
} {
  const rawAttachment = asRecord(candidate.attachment);
  if (!rawAttachment) return { reason: 'invalid_identity' };
  const declaredWorkstream = candidate.workstream === undefined
    ? normalizeWorkstream(rawAttachment.workstream)
    : normalizeWorkstream(candidate.workstream);
  if (!declaredWorkstream) return { reason: 'invalid_identity' };
  // Some generated surface envelopes carry the exact WorkstreamKey beside a
  // primary attachment payload. Bind those explicit fields; never fill the
  // owner from project_root, continuity, a tab, or a latest-row lookup.
  const attachmentWorkstream = rawAttachment.workstream === undefined
    ? declaredWorkstream
    : normalizeWorkstream(rawAttachment.workstream);
  if (!attachmentWorkstream) return { reason: 'invalid_identity' };
  if (!sameWorkstreamKey(attachmentWorkstream, declaredWorkstream)) {
    return { reason: 'foreign_attachment_workstream' };
  }

  const attachment = {
    workstream: attachmentWorkstream,
    continuity_id: rawAttachment.continuity_id ?? null,
    instance_id: rawAttachment.instance_id,
    session_id: rawAttachment.session_id,
    attachment_id: rawAttachment.attachment_id,
    workspace_binding_id: rawAttachment.workspace_binding_id
  };
  if (!validateMissionCanvasContract('AttachmentKey', attachment).valid) {
    return { reason: 'invalid_identity' };
  }

  const continuity = candidate.continuity_id ?? attachment.continuity_id ?? null;
  const runtime = candidate.runtime_object ?? null;
  const identity = {
    workstream: declaredWorkstream,
    continuity_id: continuity,
    attachment,
    runtime_object: runtime,
    work_surface_id: candidate.work_surface_id
  };
  return isExactWorkSurfaceIdentity(identity)
    ? { identity, reason: 'invalid_identity' }
    : { reason: 'invalid_identity' };
}

function normalizeWorkstream(value: unknown): WorkstreamKey | undefined {
  if (!validateMissionCanvasContract('WorkstreamKey', value).valid) return undefined;
  const workstream = value as WorkstreamKey;
  const scope = asRecord(workstream.scope);
  const scopeKey = asRecord(scope.scope_key);
  if (!nonEmpty(workstream.workstream_id)
    || !validScopeKind(scope.scope_kind)
    || scopeKey.scope_kind !== scope.scope_kind
    || !validScopeKind(scopeKey.scope_kind)
    || !nonEmpty(scopeKey.scope_id)
    || !nonEmpty(scopeKey.root_path)
    || !nonEmpty(scopeKey.canonical_name)
    || !nonEmpty(scopeKey.fingerprint)) {
    return undefined;
  }
  return cloneJson(workstream);
}

function validateRowIdentityConsistency(
  row: Record<string, unknown>,
  identity: ExactWorkSurfaceIdentity
): WorkSurfaceQuarantineReason | undefined {
  const attachment = identity.attachment;
  const scope = asRecord(row.scope);
  const topLevelChecks: Array<[unknown, string]> = [
    [row.work_surface_id ?? row.surface_id, identity.work_surface_id],
    [row.instance_id, attachment.instance_id],
    [row.session_id, attachment.session_id],
    [row.attachment_id, attachment.attachment_id],
    [row.workspace_binding_id, attachment.workspace_binding_id]
  ];
  for (const [provided, expected] of topLevelChecks) {
    if (provided !== undefined && provided !== null && (typeof provided !== 'string' || provided.trim() === '' || provided !== expected)) {
      return 'identity_mismatch';
    }
  }

  const providedRuntime = firstDefined(scope.runtime_object, row.runtime_object);
  if (providedRuntime !== undefined && !sameStable(providedRuntime, identity.runtime_object ?? null)) {
    return 'identity_mismatch';
  }

  const providedContinuity = firstDefined(scope.continuity_id, row.continuity_id);
  if (providedContinuity !== undefined && providedContinuity !== null
    && (typeof providedContinuity !== 'string' || providedContinuity.trim() === '' || providedContinuity !== (identity.continuity_id ?? attachment.continuity_id ?? null))) {
    return 'identity_mismatch';
  }

  for (const candidate of [row.attachment, row.primary_attachment]) {
    if (candidate === undefined || candidate === null) continue;
    const raw = asRecord(candidate);
    if (!raw) return 'identity_mismatch';
    for (const [field, expected] of [
      ['instance_id', attachment.instance_id],
      ['session_id', attachment.session_id],
      ['attachment_id', attachment.attachment_id],
      ['workspace_binding_id', attachment.workspace_binding_id]
    ] as Array<[string, string]>) {
      const provided = raw[field];
      if (provided !== undefined && provided !== null
        && (typeof provided !== 'string' || provided.trim() === '' || provided !== expected)) {
        return 'identity_mismatch';
      }
    }
    if (raw.continuity_id !== undefined && raw.continuity_id !== null
      && (typeof raw.continuity_id !== 'string' || raw.continuity_id.trim() === '' || raw.continuity_id !== (attachment.continuity_id ?? null))) {
      return 'identity_mismatch';
    }
    if (raw.workstream !== undefined && !sameWorkstreamKey(raw.workstream, identity.workstream)) {
      return 'foreign_attachment_workstream';
    }
  }

  if (row.workstream !== undefined && !sameWorkstreamKey(row.workstream, identity.workstream)) {
    return 'foreign_attachment_workstream';
  }
  if (scope.workstream !== undefined && !sameWorkstreamKey(scope.workstream, identity.workstream)) {
    return 'foreign_attachment_workstream';
  }
  return undefined;
}

function sameStable(left: unknown, right: unknown): boolean {
  return stableSerialize(left ?? null) === stableSerialize(right ?? null);
}

export function exactWorkSurfaceIdentityKey(identity: ExactWorkSurfaceIdentity): string {
  return stableEncode([
    identity.workstream,
    identity.continuity_id ?? identity.attachment.continuity_id ?? null,
    identity.attachment,
    identity.runtime_object ?? null,
    identity.work_surface_id
  ]);
}

function stableEncode(value: unknown): string {
  return encodeURIComponent(stableSerialize(value));
}

function stableSerialize(value: unknown): string {
  if (Array.isArray(value)) return `[${value.map(stableSerialize).join(',')}]`;
  if (value && typeof value === 'object') {
    return `{${Object.keys(value as Record<string, unknown>).sort().map((key) => `${JSON.stringify(key)}:${stableSerialize((value as Record<string, unknown>)[key])}`).join(',')}}`;
  }
  return JSON.stringify(value) ?? String(value);
}

function firstDefined(...values: unknown[]): unknown {
  return values.find((value) => value !== undefined);
}

function asRecord(value: unknown): Record<string, unknown> {
  return value !== null && typeof value === 'object' && !Array.isArray(value)
    ? value as Record<string, unknown>
    : {};
}

function text(value: unknown): string {
  return typeof value === 'string' ? value.trim() : String(value ?? '').trim();
}

function nonEmpty(value: unknown): value is string {
  return typeof value === 'string' && value.trim().length > 0;
}

function validScopeKind(value: unknown): value is 'project' | 'host' {
  return value === 'project' || value === 'host';
}

function count(value: unknown): number {
  const parsed = Number(value);
  return Number.isFinite(parsed) && parsed >= 0 ? Math.floor(parsed) : 0;
}

function cloneJson<T>(value: T): T {
  if (typeof globalThis.structuredClone === 'function') {
    try {
      return globalThis.structuredClone(value);
    } catch {
      // Svelte 5 $state proxies are not structured-cloneable; JSON round-trip
      // strips the proxy and yields plain data.
      return JSON.parse(JSON.stringify(value)) as T;
    }
  }
  return JSON.parse(JSON.stringify(value)) as T;
}
