/* Generated extraction from SessionInventoryContribution.svelte: pure inventory
   projection helpers exposed as module seams for exact-identity rendering. */

  import {
    sameWorkstreamKey,
    validateMissionCanvasContract
  } from '../../../../../docs/contracts/spec135/mission-canvas-v1/typescript/mission-canvas-validators.generated';
  import { exactScopeKey, sameWorkstreamAuthority } from './exact-scope';
  import type {
    AttachmentKey,
    ExactWorkSurfaceIdentity,
    OperationBinding,
    ResolvedContribution,
    ResolvedWorkspaceProjection,
    WorkSurfaceProjection,
    WorkstreamAuthorityContext
  } from './types';
  import { isExactWorkSurfaceIdentity } from './types';

  type InventoryRowState = 'exact' | 'compatibility' | 'quarantined';
  export type InventoryScope = 'aggregate' | 'local';

  export type SessionInventoryRow = {
    key: string;
    state: InventoryRowState;
    reason?: string;
    sourceContributionId?: string;
    identity?: ExactWorkSurfaceIdentity;
    inventoryScope: InventoryScope;
    label: string;
    kind: string;
    lifecycle: string;
    health: string;
    approvals: string;
    conflicts: string;
    writerLease: string;
    browserIsolation: string;
    origin: string;
    freshness: string;
    revision: string;
    bindings: OperationBinding[];
  };

  export interface SessionAttachmentIdentityRender {
    authority: WorkstreamAuthorityContext | undefined;
    identity: ExactWorkSurfaceIdentity | undefined;
    inventoryScope: InventoryScope;
    reason?: string;
  }

  export const SessionAttachmentIdentity = {
    render(
      value: unknown,
      expectedScope?: WorkstreamAuthorityContext | null
    ): SessionAttachmentIdentityRender {
      const authority = authorityContext(value);
      if (!authority) {
        return {
          authority: undefined,
          identity: undefined,
          inventoryScope: 'aggregate',
          reason: 'invalid_identity'
        };
      }

      const identity = exactIdentity(authority);
      if (!identity) {
        return {
          authority,
          identity: undefined,
          inventoryScope: 'aggregate',
          reason: 'missing_exact_identity'
        };
      }

      if (expectedScope === null) {
        return {
          authority,
          identity,
          inventoryScope: 'aggregate',
          reason: 'foreign_scope'
        };
      }

      if (expectedScope !== undefined && !sameWorkstreamAuthority(authority, expectedScope)) {
        return {
          authority,
          identity,
          inventoryScope: 'aggregate',
          reason: 'foreign_scope'
        };
      }

      return {
        authority,
        identity,
        inventoryScope: 'local'
      };
    }
  };

  function renderWorkSurfaceInventory(
    projection: ResolvedWorkspaceProjection,
    contribution: ResolvedContribution,
    canonicalWorkSurfaces: readonly WorkSurfaceProjection[] = []
  ): SessionInventoryRow[] {
    return projectSessionInventory(projection, contribution, canonicalWorkSurfaces);
  }

  export const WorkSurfaceInventory = {
    render: renderWorkSurfaceInventory
  };
function record(value: unknown): Record<string, unknown> {
    return value !== null && typeof value === 'object' && !Array.isArray(value)
      ? value as Record<string, unknown>
      : {};
  }

  function text(...values: unknown[]): string {
    for (const value of values) {
      if (typeof value === 'string' && value.trim()) return value.trim();
      if (typeof value === 'number' && Number.isFinite(value)) return String(value);
    }
    return 'not reported';
  }

  function numberText(value: unknown): string {
    return typeof value === 'number' && Number.isFinite(value) && value >= 0
      ? String(Math.floor(value))
      : 'not reported';
  }

  function authorityContext(value: unknown): WorkstreamAuthorityContext | undefined {
    const authority = record(value);
    const candidate = {
      workstream: authority.workstream,
      continuity_id: authority.continuity_id ?? null,
      attachment: authority.attachment ?? null,
      workspace_binding_id: authority.workspace_binding_id ?? null,
      runtime_object: authority.runtime_object ?? null,
      work_surface_id: authority.work_surface_id ?? null
    } as WorkstreamAuthorityContext;
    return validateMissionCanvasContract('WorkstreamAuthorityContext', candidate).valid
      ? candidate
      : undefined;
  }

  function exactIdentity(value: unknown): ExactWorkSurfaceIdentity | undefined {
    const authority = authorityContext(value);
    if (!authority?.attachment || !authority.work_surface_id) return undefined;
    const identity = {
      workstream: authority.workstream,
      continuity_id: authority.continuity_id ?? authority.attachment.continuity_id ?? null,
      attachment: authority.attachment,
      runtime_object: authority.runtime_object ?? null,
      work_surface_id: authority.work_surface_id
    } as ExactWorkSurfaceIdentity;
    return isExactWorkSurfaceIdentity(identity) ? identity : undefined;
  }

  function isWorkSurfaceContribution(contribution: ResolvedContribution): boolean {
    return contribution.kind === 'focused_work_surface' || contribution.data_ref.kind === 'work_surface';
  }

  function isInventoryContribution(contribution: ResolvedContribution): boolean {
    return contribution.renderer_binding_id === 'renderer:silent-sessions@v1'
      || contribution.semantic_binding_id === 'semantic:silent-sessions'
      || contribution.data_ref.kind === 'session_inventory';
  }

  function validProjectionWatermarks(projection: ResolvedWorkspaceProjection): boolean {
    return Number.isSafeInteger(projection.projection_revision)
      && projection.projection_revision >= 0
      && Number.isSafeInteger(projection.layout_revision)
      && projection.layout_revision >= 0
      && typeof projection.durable_event_cursor === 'string'
      && projection.durable_event_cursor.trim().length > 0;
  }

  function bindingsFor(
    projection: ResolvedWorkspaceProjection,
    source: ResolvedContribution | undefined,
    exact: boolean,
    freshness: string
  ): OperationBinding[] {
    if (!source || !exact || freshness === 'stale' || freshness === 'unknown' || freshness === 'not_applicable') return [];
    return projection.operation_bindings.filter((binding) =>
      binding.target_contribution_id === source.contribution_id
      && source.operation_ids.includes(binding.operation_id)
    );
  }

  function rowFromContribution(
    projection: ResolvedWorkspaceProjection,
    source: ResolvedContribution,
    inventoryContributionId: string | undefined,
    watermarkValid: boolean
  ): SessionInventoryRow | undefined {
    const sourceAuthority = authorityContext(source.authority);
    if (!sourceAuthority || !sameWorkstreamKey(sourceAuthority.workstream, projection.workstream)) return undefined;

    const identity = exactIdentity(source.authority);
    const freshness = text(source.freshness.status);
    const base = {
      sourceContributionId: source.contribution_id,
      label: text(source.accessibility.label, source.data_ref.ref),
      kind: text(source.data_ref.kind, source.kind),
      lifecycle: freshness,
      health: freshness,
      approvals: 'not reported',
      conflicts: 'not reported',
      writerLease: 'not reported',
      browserIsolation: 'not reported',
      origin: text(source.data_ref.ref),
      freshness,
      revision: text(source.data_ref.revision),
      bindings: bindingsFor(projection, source, Boolean(identity && watermarkValid), freshness),
      inventoryScope: (identity ? 'local' : 'aggregate') as InventoryScope
    };

    if (!identity) {
      // A canonical inventory contribution may still describe an aggregate or
      // legacy-compatible row. Keep it visible as observation-only data, but
      // never manufacture an AttachmentKey or a WorkSurfaceId.
      if (source.contribution_id !== inventoryContributionId) return undefined;
      return {
        ...base,
        key: `compatibility:${source.contribution_id}`,
        state: 'compatibility',
        reason: 'missing_exact_identity'
      };
    }

    return {
      ...base,
      key: exactScopeKey(identity) ?? `quarantined:${source.contribution_id}`,
      state: watermarkValid ? 'exact' : 'quarantined',
      ...(watermarkValid ? {} : { reason: 'stale_revision_or_cursor' }),
      identity
    };
  }

  function rowFromSurface(
    projection: ResolvedWorkspaceProjection,
    surface: WorkSurfaceProjection,
    source: ResolvedContribution | undefined,
    watermarkValid: boolean
  ): SessionInventoryRow {
    const identity = surface?.identity;
    const validIdentity = Boolean(identity && exactScopeKey(surface) !== undefined);
    const sameScope = Boolean(validIdentity && identity && sameWorkstreamKey(identity.workstream, projection.workstream));
    const exact = validIdentity && sameScope;
    const freshness = source ? text(source.freshness.status) : 'not reported';
    const base = {
      sourceContributionId: source?.contribution_id,
      label: text(surface?.displayName, surface?.workSurfaceId),
      kind: text(surface?.kind),
      lifecycle: text(surface?.lifecycleState),
      health: text(surface?.health),
      approvals: numberText(surface?.pendingApprovalCount),
      conflicts: numberText(surface?.conflictCount),
      writerLease: text(surface?.writerLeaseRef),
      browserIsolation: text(surface?.browserIsolationClass),
      origin: text(surface?.workSurfaceId),
      freshness,
      revision: source ? text(source.data_ref.revision) : 'not reported',
      bindings: bindingsFor(projection, source, Boolean(exact && watermarkValid), freshness),
      inventoryScope: (exact ? 'local' : 'aggregate') as InventoryScope
    };

    if (!exact) {
      return {
        ...base,
        key: `compatibility:${text(surface?.workSurfaceId)}`,
        state: 'compatibility',
        reason: !validIdentity ? 'missing_exact_identity' : 'foreign_scope'
      };
    }

    return {
      ...base,
      key: identity ? (exactScopeKey(identity) ?? `quarantined:${text(surface?.workSurfaceId)}`) : `quarantined:${text(surface?.workSurfaceId)}`,
      state: watermarkValid ? 'exact' : 'quarantined',
      ...(watermarkValid ? {} : { reason: 'stale_revision_or_cursor' }),
      identity
    };
  }

  function quarantineDuplicates(rows: SessionInventoryRow[]): SessionInventoryRow[] {
    const counts = new Map<string, number>();
    for (const row of rows) counts.set(row.key, (counts.get(row.key) ?? 0) + 1);
    return rows.map((row) => counts.get(row.key)! > 1
      ? { ...row, state: 'quarantined', reason: 'duplicate_identity', bindings: [] }
      : row);
  }

  /**
   * Translate only canonical exact Work Surface projections into inventory
   * rows. Legacy discovered/unbound records are not accepted as authority and
   * never become a Desktop focus target.
   */
  function projectSessionInventory(
    projection: ResolvedWorkspaceProjection,
    contribution: ResolvedContribution,
    canonicalWorkSurfaces: readonly WorkSurfaceProjection[] = []
  ): SessionInventoryRow[] {
    const watermarkValid = validProjectionWatermarks(projection);
    const inventoryContribution = isInventoryContribution(contribution);
    const rows: SessionInventoryRow[] = [];

    if (canonicalWorkSurfaces.length > 0) {
      const sourceBySurface = new Map(
        projection.eligible_contributions
          .filter(isWorkSurfaceContribution)
          .map((candidate) => [exactIdentity(candidate.authority)?.work_surface_id ?? candidate.data_ref.ref, candidate])
      );
      for (const surface of canonicalWorkSurfaces) {
        const surfaceId = surface?.identity?.work_surface_id;
        const source = sourceBySurface.get(typeof surfaceId === 'string' ? surfaceId : '');
        const row = rowFromSurface(projection, surface, source, watermarkValid);
        if (row.reason !== 'foreign_scope' && (row.state !== 'compatibility' || inventoryContribution)) rows.push(row);
      }
    } else {
      const sources = projection.eligible_contributions.filter(isWorkSurfaceContribution);
      for (const source of sources) {
        const row = rowFromContribution(
          projection,
          source,
          inventoryContribution ? contribution.contribution_id : undefined,
          watermarkValid
        );
        if (row) rows.push(row);
      }
      if (rows.length === 0) {
        const row = rowFromContribution(
          projection,
          contribution,
          inventoryContribution ? contribution.contribution_id : undefined,
          watermarkValid
        );
        if (row) rows.push(row);
      }
    }

    return quarantineDuplicates(rows);
  }
