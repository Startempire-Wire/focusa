/**
 * CUT-005 bounded Desktop handoff seam.
 *
 * The menubar never owns full Mission Canvas content. It keeps only status,
 * resume, pairing, lifecycle, and a single "Open in Desktop" action. This
 * module validates the exact project-bound Workstream authority and emits a
 * desktop-open intent; it never fabricates a scope, CWD, or remembered
 * workspace, and it never binds a wrong Workstream.
 */

export type DesktopHandoffContext = Readonly<{
  workstream: Readonly<{
    scope: Readonly<{
      scope_kind: string;
      scope_key: Readonly<Record<string, unknown>>;
    }>;
    workstream_id: string;
  }>;
  continuity_id?: string | null;
  attachment?: Readonly<Record<string, unknown>> | null;
  workspace_binding_id?: string | null;
  runtime_object?: Readonly<Record<string, unknown>> | null;
  work_surface_id?: string | null;
}>;

export type DesktopHandoffIntent = Readonly<{
  action: 'desktop_open';
  workstream_id: string;
  scope_kind: string;
  continuity_id: string | null;
  attachment_bound: boolean;
  target: 'focusa-desktop';
  failure?: undefined;
}>;

export type DesktopHandoffResult =
  | { ok: true; intent: DesktopHandoffIntent }
  | { ok: false; failure: 'missing_authority' | 'foreign_scope' | 'invalid_scope' };

function record(value: unknown): Record<string, unknown> {
  return value !== null && typeof value === 'object' && !Array.isArray(value)
    ? value as Record<string, unknown>
    : {};
}

function isRecordObject(value: unknown): value is Readonly<Record<string, unknown>> {
  return value !== null && typeof value === 'object' && !Array.isArray(value);
}

function scopeKeyRecord(scope: unknown): Readonly<Record<string, unknown>> {
  return isRecordObject(scope) && isRecordObject((scope as Record<string, unknown>).scope_key)
    ? (scope as Record<string, unknown>).scope_key as Readonly<Record<string, unknown>>
    : {};
}

export const DesktopPresent = {
  /** Bounded handoff validation: exact Workstream authority only. */
  invoke(context: unknown): DesktopHandoffResult {
    if (!isRecordObject(context)) return { ok: false, failure: 'missing_authority' };
    const record = context as Record<string, unknown>;
    const workstream = record.workstream;
    if (!isRecordObject(workstream)) return { ok: false, failure: 'missing_authority' };

    const workstreamRecord = workstream as Record<string, unknown>;
    const scope = workstreamRecord.scope;
    if (!isRecordObject(scope)) return { ok: false, failure: 'invalid_scope' };

    const scopeRecord = scope as Record<string, unknown>;
    if (scopeRecord.scope_kind !== 'project') return { ok: false, failure: 'invalid_scope' };
    if (typeof workstreamRecord.workstream_id !== 'string'
      || workstreamRecord.workstream_id.trim().length === 0) {
      return { ok: false, failure: 'invalid_scope' };
    }
    const scopeKey = scopeKeyRecord(scope);
    if (typeof scopeKey.scope_id !== 'string'
      || typeof scopeKey.root_path !== 'string'
      || scopeKey.root_path.trim().length === 0) {
      return { ok: false, failure: 'invalid_scope' };
    }

    const continuityId = typeof record.continuity_id === 'string' ? record.continuity_id : null;
    const attachment = isRecordObject(record.attachment) ? record.attachment : null;
    if (attachment !== null) {
      const attachmentWorkstream = (attachment as Record<string, unknown>).workstream;
      if (!isRecordObject(attachmentWorkstream)) return { ok: false, failure: 'foreign_scope' };
      const attachmentWorkstreamId = (attachmentWorkstream as Record<string, unknown>).workstream_id;
      if (attachmentWorkstreamId !== workstreamRecord.workstream_id) {
        return { ok: false, failure: 'foreign_scope' };
      }
    }

    return {
      ok: true,
      intent: {
        action: 'desktop_open',
        workstream_id: workstreamRecord.workstream_id,
        scope_kind: 'project',
        continuity_id: continuityId,
        attachment_bound: attachment !== null,
        target: 'focusa-desktop'
      }
    };
  }
};
