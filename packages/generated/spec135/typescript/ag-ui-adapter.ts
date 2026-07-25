export interface FocusaNativeStreamEvent<T = unknown> {
  event_id: string;
  event_type: string;
  project_root: string;
  continuity_id: string;
  attachment_id: string;
  state_version: number;
  emitted_at: string;
  payload: T;
  evidence_refs?: string[];
}

export interface FocusaAgUiEvent<T = unknown> {
  type: "CUSTOM";
  name: string;
  value: T;
  timestamp: number;
  rawEvent: {
    eventId: string;
    projectRoot: string;
    continuityId: string;
    attachmentId: string;
    stateVersion: number;
    replayCursor: string;
    evidenceRefs: string[];
  };
}

/**
 * Stateless downstream compatibility translation for AG-UI consumers.
 * Focusa's native event stream remains canonical; this function owns no
 * history, retry state, or projection state.
 */
export function toAgUiEvent<T>(event: FocusaNativeStreamEvent<T>): FocusaAgUiEvent<T> {
  if (!event.event_id || !event.project_root || !event.continuity_id || !event.attachment_id) {
    throw new Error("Focusa AG-UI translation requires exact native event scope");
  }
  return {
    type: "CUSTOM",
    name: event.event_type,
    value: event.payload,
    timestamp: Date.parse(event.emitted_at),
    rawEvent: {
      eventId: event.event_id,
      projectRoot: event.project_root,
      continuityId: event.continuity_id,
      attachmentId: event.attachment_id,
      stateVersion: event.state_version,
      replayCursor: event.event_id,
      evidenceRefs: event.evidence_refs ?? [],
    },
  };
}
