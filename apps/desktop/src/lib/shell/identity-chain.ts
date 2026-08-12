// Spec 158 identity chain resolver:
// Resolves ScopeRef → WorkstreamId → ContinuityId → AttachmentKey
// from a live WorkstreamAuthorityContext. Works with any daemon version
// since it reads canonical contract fields, not daemon-specific data.

import type { WorkstreamAuthorityContext } from '../mission-canvas/types';
import type { DaemonReadStatus } from './daemon-health';

export interface IdentityChainStep {
  label: string;
  value: string | null;
  detail: string;
  resolved: boolean;
}

export interface IdentityChainState {
  steps: IdentityChainStep[];
  overall: 'unbound' | 'partial' | 'resolved';
}

export function resolveIdentityChain(
  authority: WorkstreamAuthorityContext | undefined,
  daemon: DaemonReadStatus
): IdentityChainState {
  if (!authority) {
    return {
      overall: 'unbound',
      steps: [
        { label: 'Daemon', value: null, detail: daemon.kind === 'read-only' ? `v${daemon.version} connected` : 'No daemon', resolved: daemon.kind === 'read-only' },
        { label: 'ScopeRef', value: null, detail: 'No verified project scope', resolved: false },
        { label: 'WorkstreamId', value: null, detail: 'No canonical Workstream selected', resolved: false },
        { label: 'ContinuityId', value: null, detail: 'Lineage inside the Workstream', resolved: false },
        { label: 'AttachmentKey', value: null, detail: 'No exact runtime Attachment', resolved: false },
        { label: 'WorkSurfaceId', value: null, detail: 'No presentation surface binding', resolved: false },
      ]
    };
  }

  const ws = authority.workstream;
  const scope = ws?.scope?.scope_key;
  const att = authority.attachment;

  return {
    overall: authority.work_surface_id ? 'resolved' : 'partial',
    steps: [
      {
        label: 'Daemon',
        value: daemon.kind === 'read-only' ? daemon.version ?? 'connected' : null,
        detail: daemon.kind === 'read-only' ? 'Daemon available' : daemon.label,
        resolved: daemon.kind === 'read-only'
      },
      {
        label: 'ScopeRef',
        value: scope?.scope_id ?? scope?.canonical_name ?? null,
        detail: scope?.root_path ?? scope?.canonical_name ?? 'No verified project scope',
        resolved: !!scope?.scope_id
      },
      {
        label: 'WorkstreamId',
        value: ws?.workstream_id ?? null,
        detail: ws?.workstream_id ? 'Canonical Workstream selected' : 'No canonical Workstream selected',
        resolved: !!ws?.workstream_id
      },
      {
        label: 'ContinuityId',
        value: authority.continuity_id ?? null,
        detail: authority.continuity_id ? 'Lineage bound' : 'Lineage inside the Workstream',
        resolved: !!authority.continuity_id
      },
      {
        label: 'AttachmentKey',
        value: att?.attachment_id ?? null,
        detail: att?.attachment_id
          ? `Bound to ${att.kind ?? 'runtime'}`
          : 'No exact runtime Attachment',
        resolved: !!att?.attachment_id
      },
      {
        label: 'InstanceId',
        value: att?.instance_id ?? null,
        detail: att?.instance_id ? 'Pi runtime instance bound' : 'Awaiting Pi attachment',
        resolved: !!att?.instance_id
      },
      {
        label: 'SessionId',
        value: att?.session_id ?? null,
        detail: att?.session_id ? 'Temporal session bound' : 'Awaiting Pi attachment',
        resolved: !!att?.session_id
      },
      {
        label: 'WorkSurfaceId',
        value: authority.work_surface_id ?? null,
        detail: authority.work_surface_id ? 'Presentation surface bound' : 'No presentation surface binding',
        resolved: !!authority.work_surface_id
      },
    ]
  };
}
