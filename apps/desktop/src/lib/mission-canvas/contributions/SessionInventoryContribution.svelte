<script module lang="ts">
  export { SessionAttachmentIdentity, WorkSurfaceInventory } from '../session-inventory';
  export type { SessionInventoryRow, SessionAttachmentIdentityRender, InventoryScope } from '../session-inventory';
</script>

<script lang="ts">
  import { WorkSurfaceInventory } from '../session-inventory';
  import type { AttachmentKey, ExactWorkSurfaceIdentity, OperationBinding, ResolvedContribution, ResolvedWorkspaceProjection, WorkSurfaceProjection } from '../types';
  import type { SessionInventoryRow, InventoryScope } from '../session-inventory';

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

  const inventoryRows = $derived(WorkSurfaceInventory.render(projection, contribution, workSurfaces));

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

  function inventoryModeLabel(scope: InventoryScope): string {
    return scope === 'local' ? 'local inventory' : 'aggregate inventory';
  }

  function workstreamKeyLabel(identity: ExactWorkSurfaceIdentity): string {
    return JSON.stringify(identity.workstream);
  }

  function attachmentKeyLabel(attachment: AttachmentKey): string {
    return JSON.stringify(attachment);
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
      <span class="focus-hint">Visual focus is local and not canonical activity.</span>
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
          data-session-inventory-mode={row.inventoryScope}
          data-bindable={row.state === 'exact' && row.freshness !== 'stale' && row.freshness !== 'unknown' && row.freshness !== 'not_applicable' ? 'true' : 'false'}
          data-non-actionable={row.state === 'exact' && row.freshness !== 'stale' && row.freshness !== 'unknown' && row.freshness !== 'not_applicable' ? undefined : 'true'}
        >
          <div class="row-heading">
            <div class="row-title">
              <strong>{row.label}</strong>
              <span>{row.kind} · {row.lifecycle} · {row.health}</span>
            </div>
            <span class="inventory-scope" data-session-inventory-mode={row.inventoryScope}>{inventoryModeLabel(row.inventoryScope)}</span>
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
  .focus-hint{color:var(--color-text-tertiary);font:var(--type-caption)}
  .inventory-scope{flex-shrink:0;padding:2px var(--space-2);border:1px solid var(--color-border);border-radius:999px;color:var(--color-text-tertiary);font:var(--type-caption);text-transform:lowercase}
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
