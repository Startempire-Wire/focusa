<script lang="ts">
  import {
    sameWorkstreamKey,
    validateMissionCanvasContract
  } from '../../../../../../docs/contracts/spec135/mission-canvas-v1/typescript/mission-canvas-validators.generated';
  import { exactScopeKey } from '../exact-scope';
  import type {
    AttachmentKey,
    ExactWorkSurfaceIdentity,
    OperationBinding,
    ResolvedContribution,
    ResolvedWorkspaceProjection,
    WorkSurfaceProjection,
    WorkstreamAuthorityContext
  } from '../types';
  import { isExactWorkSurfaceIdentity } from '../types';

  type InventoryRowState = 'exact' | 'compatibility' | 'quarantined';

  type SessionInventoryRow = {
    key: string;
    state: InventoryRowState;
    reason?: string;
    sourceContributionId?: string;
    identity?: ExactWorkSurfaceIdentity;
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

  let {
    contribution,
    projection,
    onOperation,
    workSurfaces = []
  }: {
    contribution: ResolvedContribution;
    projection: ResolvedWorkspaceProjection;
    onOperation?: (binding: OperationBinding) => void | Promise<void>;
    /**
     * Canonical Work Surface projections may be supplied by the contribution
     * host. They already carry exact identity; this component never rebuilds
     * one from project_root, continuity_id, a tab, or a latest-row fallback.
     */
    workSurfaces?: readonly WorkSurfaceProjection[];
  } = $props();

  // Session inventory has no local transport or route: its input is the
  // canonical contribution/projection supplied by Core. Approved operation
  // bindings are still dispatched by the shared renderer host.

  const inventoryRows = $derived(projectSessionInventory(projection, contribution, workSurfaces));

  function actionable(row: SessionInventoryRow, binding: OperationBinding): boolean {
    return Boolean(
      row.state === 'exact'
      && row.freshness !== 'stale'
      && row.freshness !== 'unknown'
      && row.freshness !== 'not_applicable'
      && onOperation
      && binding.enabled
      && binding.authority_ref
      && !binding.disabled_reason_ref
      && binding.confirmation !== 'preview'
    );
  }

  function operationLabel(operationId: string): string {
    return operationId.split('.').at(-1) ?? operationId;
  }

  function workstreamKeyLabel(identity: ExactWorkSurfaceIdentity): string {
    return JSON.stringify(identity.workstream);
  }

  function attachmentKeyLabel(attachment: AttachmentKey): string {
    return JSON.stringify(attachment);
  }

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
      bindings: bindingsFor(projection, source, Boolean(identity && watermarkValid), freshness)
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
      bindings: bindingsFor(projection, source, Boolean(exact && watermarkValid), freshness)
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
</script>

{#if inventoryRows.length > 0}
  <section
    class="session-inventory"
    aria-label={contribution.accessibility.label}
    data-session-inventory={contribution.data_ref.ref}
    data-contribution-id={contribution.contribution_id}
    data-projection-revision={projection.projection_revision}
    data-layout-revision={projection.layout_revision}
    data-event-cursor={projection.durable_event_cursor}
  >
    <header>
      <div class="heading">
        <strong>{contribution.accessibility.label}</strong>
        {#if contribution.accessibility.description}
          <span>{contribution.accessibility.description}</span>
        {/if}
      </div>
      <span class="freshness">{contribution.freshness.status}</span>
    </header>

    <ul class="rows" aria-label="Session inventory rows">
      {#each inventoryRows as row (row.key)}
        <li
          class:compatibility={row.state === 'compatibility'}
          class:quarantined={row.state === 'quarantined'}
          class:stale={row.freshness === 'stale' || row.freshness === 'unknown' || row.freshness === 'not_applicable'}
          data-session-inventory-row={row.key}
          data-row-state={row.state}
          data-quarantine-reason={row.reason ?? undefined}
          data-bindable={row.state === 'exact' && row.freshness !== 'stale' && row.freshness !== 'unknown' && row.freshness !== 'not_applicable' ? 'true' : 'false'}
          data-non-actionable={row.state === 'exact' && row.freshness !== 'stale' && row.freshness !== 'unknown' && row.freshness !== 'not_applicable' ? undefined : 'true'}
        >
          <div class="row-heading">
            <div class="row-title">
              <strong>{row.label}</strong>
              <span>{row.kind} · {row.lifecycle} · {row.health}</span>
            </div>
            <span class="row-status">{row.state === 'exact' ? row.freshness : 'compatibility'}</span>
          </div>

          {#if row.identity}
            <dl class="identity" data-workstream-id={row.identity.workstream.workstream_id} data-work-surface-id={row.identity.work_surface_id} data-attachment-id={row.identity.attachment.attachment_id} data-session-id={row.identity.attachment.session_id} data-instance-id={row.identity.attachment.instance_id}>
              <div>
                <dt>Workstream</dt>
                <dd data-workstream-key={workstreamKeyLabel(row.identity)}>{row.identity.workstream.workstream_id}</dd>
              </div>
              <div>
                <dt>Attachment</dt>
                <dd data-attachment-key={attachmentKeyLabel(row.identity.attachment)}>{row.identity.attachment.attachment_id}</dd>
              </div>
              <div>
                <dt>Session</dt>
                <dd>{row.identity.attachment.session_id}</dd>
              </div>
              <div>
                <dt>Instance</dt>
                <dd>{row.identity.attachment.instance_id}</dd>
              </div>
              <div>
                <dt>Surface</dt>
                <dd>{row.identity.work_surface_id}</dd>
              </div>
            </dl>
            <div class="metadata">
              <span>{row.approvals} approvals</span>
              <span>{row.conflicts} conflicts</span>
              <span>{row.writerLease}</span>
              <span>{row.browserIsolation}</span>
              <code>{row.origin}</code>
              <small>revision {row.revision}</small>
            </div>
          {:else}
            <p class="compatibility-copy">
              Compatibility data only. Exact Workstream and Attachment identity is unavailable; this row cannot bind, focus, or steer a Work Surface.
            </p>
          {/if}

          {#if row.bindings.length > 0}
            <div class="actions" aria-label={`${row.label} actions`}>
              {#each row.bindings as binding (binding.operation_id)}
                <button
                  type="button"
                  disabled={!actionable(row, binding)}
                  title={binding.disabled_reason_ref ?? binding.operation_id}
                  aria-label={`${operationLabel(binding.operation_id)} ${row.label}`}
                  onclick={() => void onOperation?.(binding)}
                >{operationLabel(binding.operation_id)}</button>
              {/each}
            </div>
          {/if}
        </li>
      {/each}
    </ul>
  </section>
{/if}

<style>
  .session-inventory{display:grid;gap:var(--space-3);min-width:0;padding:var(--space-3);border:1px solid var(--color-border);border-radius:var(--radius-panel);background:var(--color-panel)}
  header,.row-heading,.metadata,.actions{display:flex;align-items:center;gap:var(--space-3);min-width:0}
  header,.row-heading{justify-content:space-between}
  .heading,.row-title{display:grid;gap:var(--space-1);min-width:0}
  strong{color:var(--color-text);font:var(--type-label)}
  header span,.row-title span,.metadata span,.compatibility-copy{color:var(--color-text-tertiary);font:var(--type-caption)}
  .freshness,.row-status{flex-shrink:0;padding:2px var(--space-2);border:1px solid var(--color-border);border-radius:999px;color:var(--color-success);font:var(--type-caption)}
  .rows{display:grid;gap:var(--space-2);margin:0;padding:0;list-style:none}
  .rows>li{display:grid;gap:var(--space-2);min-width:0;padding:var(--space-3);border:1px solid var(--color-border);border-radius:var(--radius-card);background:var(--color-elevated)}
  .rows>li.compatibility,.rows>li.quarantined,.rows>li.stale{border-color:var(--color-warning)}
  .row-status{color:var(--color-text-secondary)}
  .identity{display:grid;grid-template-columns:repeat(5,minmax(0,1fr));gap:var(--space-2);margin:0}
  .identity div{display:grid;gap:2px;min-width:0}
  dt{color:var(--color-text-tertiary);font:var(--type-caption);text-transform:uppercase;letter-spacing:.06em}
  dd{margin:0;overflow:hidden;color:var(--color-text-secondary);font:var(--type-code);text-overflow:ellipsis;white-space:nowrap}
  .metadata{flex-wrap:wrap;padding-top:var(--space-2);border-top:1px solid var(--color-border)}
  .metadata code,.metadata small{overflow:hidden;color:var(--color-text-tertiary);font:var(--type-caption);text-overflow:ellipsis;white-space:nowrap}
  .compatibility-copy{margin:0;line-height:1.5}
  .actions{justify-content:flex-end}
  button{border:1px solid var(--color-border);border-radius:var(--radius-control);padding:var(--space-1) var(--space-2);background:transparent;color:var(--color-text-secondary);font:var(--type-caption);cursor:pointer}
  button:disabled{cursor:not-allowed;opacity:.45}
  @container mission-canvas (max-width: 760px){.identity{grid-template-columns:repeat(2,minmax(0,1fr))}}
  @media (prefers-reduced-motion: reduce){button{transition:none}}
</style>
