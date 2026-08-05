export type PiAttachmentState = 'unbound' | 'binding' | 'attached' | 'disconnected' | 'error';

export interface PiAttachmentIdentity {
  scopeRef: string;
  workstreamId: string;
  continuityId: string;
  attachmentKey: string;
  sessionId: string;
  instanceId: string;
  workSurfaceId: string;
}

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
  | { kind: 'input'; attachmentKey: string; data: string }
  | { kind: 'resize'; attachmentKey: string; geometry: PiTerminalGeometry }
  | { kind: 'interrupt'; attachmentKey: string }
  | { kind: 'detach'; attachmentKey: string };

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
  return projection.state === 'attached' && Boolean(identity?.scopeRef && identity.workstreamId && identity.continuityId && identity.attachmentKey && identity.sessionId && identity.instanceId && identity.workSurfaceId);
}
