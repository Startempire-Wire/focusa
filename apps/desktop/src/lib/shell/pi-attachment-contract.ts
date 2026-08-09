import type { AttachmentKey, RuntimeObjectRef, WorkSurfaceId } from '$lib/mission-canvas/types';

export type PiAttachmentState = 'unbound' | 'binding' | 'attached' | 'disconnected' | 'error';

/** Generated AttachmentKey owns Workstream, continuity, runtime and workspace identity. */
export type PiAttachmentIdentity = AttachmentKey & {
  work_surface_id: WorkSurfaceId;
  runtime_object?: RuntimeObjectRef | null;
};

export interface PiTerminalGeometry {
  columns: number;
  rows: number;
  pixelWidth: number;
  pixelHeight: number;
}

export interface PiAttachmentProjection {
  state: PiAttachmentState;
  identity?: PiAttachmentIdentity;
  runtimeLabel: string;
  detail: string;
  canWrite: boolean;
  canSteer: boolean;
  canInterrupt: boolean;
}

export type PiNativeCommand =
  | { kind: 'attach'; identity: PiAttachmentIdentity; geometry: PiTerminalGeometry }
  | { kind: 'input'; attachment_id: PiAttachmentIdentity['attachment_id']; data: string }
  | { kind: 'resize'; attachment_id: PiAttachmentIdentity['attachment_id']; geometry: PiTerminalGeometry }
  | { kind: 'interrupt'; attachment_id: PiAttachmentIdentity['attachment_id'] }
  | { kind: 'detach'; attachment_id: PiAttachmentIdentity['attachment_id'] }
  | { kind: 'close'; attachment_id: PiAttachmentIdentity['attachment_id'] }
  | { kind: 'restart'; attachment_id: PiAttachmentIdentity['attachment_id'] };

export const UNBOUND_PI_ATTACHMENT: PiAttachmentProjection = {
  state: 'unbound',
  runtimeLabel: 'No Pi runtime attached',
  detail: 'Select an exact Workstream and create an Attachment before native terminal input is enabled.',
  canWrite: false,
  canSteer: false,
  canInterrupt: false
};

export function hasExactPiAttachment(projection: PiAttachmentProjection): projection is PiAttachmentProjection & { identity: PiAttachmentIdentity } {
  const identity = projection.identity;
  return projection.state === 'attached' && Boolean(
    identity?.workstream.scope
    && identity.workstream.workstream_id
    && identity.instance_id
    && identity.session_id
    && identity.attachment_id
    && identity.workspace_binding_id
    && identity.work_surface_id
  );
}
