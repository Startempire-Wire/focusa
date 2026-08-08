/* Canonical extraction from CanonicalEventHistoryContribution.svelte: pure history
   projection helpers exposed as seams for bounded verification and tests. */

import type {
  ProjectionLifecycleEvent,
  ResolvedWorkspaceProjection,
  WorkstreamAuthorityContext
} from './types';
import { validateMissionCanvasContract } from '../../../../../docs/contracts/spec135/mission-canvas-v1/typescript/mission-canvas-validators.generated';
import { authorityFromEvent, authorityFromProjection, sameWorkstreamAuthority as sameScope } from './exact-scope';

export interface CanonicalHistoryRow {
  event_id: string;
  event_cursor: string;
  sequence: number;
  event_kind: string;
  projection_revision: number;
  layout_revision: number;
  occurred_at: string;
  payload_ref: string;
}

export interface CanonicalHistoryRenderRejection {
  event?: unknown;
  reason: string;
}

export interface CanonicalHistoryRenderResult {
  rows: CanonicalHistoryRow[];
  rejected: CanonicalHistoryRenderRejection[];
}

export interface CanonicalHistoryRenderInput {
  projection: ResolvedWorkspaceProjection;
  events: readonly unknown[];
  authority?: WorkstreamAuthorityContext;
  maxRows?: number;
}

export const MAX_CANONICAL_HISTORY_ROWS = 200;

export const CanonicalEventHistory = {
  render(
    input: CanonicalHistoryRenderInput | ResolvedWorkspaceProjection,
    fallbackEvents: readonly unknown[] = [],
    fallbackAuthority?: WorkstreamAuthorityContext,
    fallbackMaxRows?: number
  ): CanonicalHistoryRenderResult {
    const normalized = normalizeRenderInput(input, fallbackEvents, fallbackAuthority, fallbackMaxRows);
    const projection = normalized.projection;
    const rows: CanonicalHistoryRow[] = [];
    const rejected: CanonicalHistoryRenderRejection[] = [];

    if (!projection || typeof projection !== 'object' || !normalized.events || !Array.isArray(normalized.events)) {
      return { rows, rejected: [{ reason: 'invalid_render_input' }] };
    }

    const limit = normalizeLimit(normalized.maxRows);
    const projectionAuthority = normalized.authority ?? authorityFromProjection(projection);
    const projectionAuthorityValidation = validateMissionCanvasContract('WorkstreamAuthorityContext', projectionAuthority);
    if (!projectionAuthorityValidation.valid) {
      return { rows, rejected: [{ reason: `invalid_projection_authority:${projectionAuthorityValidation.errors.join(',')}` }] };
    }

    const currentSequence = parseEventCursor(projection.durable_event_cursor);
    if (currentSequence === undefined) {
      return { rows, rejected: [{ reason: 'invalid_projection_cursor' }] };
    }

    const pending: Array<{ event: ProjectionLifecycleEvent; sequence: number }> = [];
    const seenEventIds = new Set<string>();
    const seenEventCursors = new Set<number>();

    for (const candidate of normalized.events) {
      const eventValidation = validateMissionCanvasContract('ProjectionLifecycleEvent', candidate);
      if (!eventValidation.valid) {
        rejected.push({ event: candidate, reason: `invalid_event:${eventValidation.errors.join(',')}` });
        continue;
      }

      const event = candidate as ProjectionLifecycleEvent;
      if (typeof event.event_id !== 'string' || event.event_id.trim().length === 0) {
        rejected.push({ event, reason: 'invalid_event_id' });
        continue;
      }
      if (seenEventIds.has(event.event_id)) {
        rejected.push({ event, reason: 'duplicate_event' });
        continue;
      }
      seenEventIds.add(event.event_id);

      const eventAuthority = authorityFromEvent(event);
      const eventAuthorityValidation = validateMissionCanvasContract('WorkstreamAuthorityContext', eventAuthority);
      if (!eventAuthorityValidation.valid) {
        rejected.push({ event, reason: `invalid_event_scope:${eventAuthorityValidation.errors.join(',')}` });
        continue;
      }
      if (!sameScope(eventAuthority, projectionAuthority)) {
        rejected.push({ event, reason: 'foreign_event_scope' });
        continue;
      }

      const sequence = parseEventCursor(event.event_cursor);
      if (sequence === undefined) {
        rejected.push({ event, reason: 'invalid_event_cursor' });
        continue;
      }
      if (seenEventCursors.has(sequence)) {
        rejected.push({ event, reason: 'duplicate_cursor' });
        continue;
      }
      seenEventCursors.add(sequence);

      if (event.projection_revision < projection.projection_revision) {
        rejected.push({ event, reason: 'projection_revision_stale' });
        continue;
      }
      if (event.layout_revision < projection.layout_revision) {
        rejected.push({ event, reason: 'layout_revision_stale' });
        continue;
      }

      pending.push({ event, sequence });
    }

    pending.sort((left, right) => left.sequence - right.sequence);

    let lastSeenSequence = currentSequence;
    for (const item of pending) {
      if (item.sequence <= lastSeenSequence) {
        rejected.push({ event: item.event, reason: 'event_cursor_stale' });
        continue;
      }
      lastSeenSequence = item.sequence;
      rows.push({
        event_id: item.event.event_id,
        event_cursor: item.event.event_cursor,
        sequence: item.sequence,
        event_kind: item.event.event_kind,
        projection_revision: item.event.projection_revision,
        layout_revision: item.event.layout_revision,
        occurred_at: item.event.occurred_at,
        payload_ref: item.event.payload_ref
      });
    }

    const boundedRows = limit > 0 ? rows.slice(-limit) : rows;
    return { rows: boundedRows, rejected };
  }
};

function normalizeRenderInput(
  input: CanonicalHistoryRenderInput | ResolvedWorkspaceProjection,
  fallbackEvents: readonly unknown[],
  fallbackAuthority?: WorkstreamAuthorityContext,
  fallbackMaxRows?: number
): CanonicalHistoryRenderInput {
  if (input && typeof input === 'object' && 'projection' in input && 'events' in input) {
    return input as CanonicalHistoryRenderInput;
  }
  return {
    projection: input as ResolvedWorkspaceProjection,
    events: fallbackEvents,
    authority: fallbackAuthority,
    maxRows: fallbackMaxRows
  };
}

function normalizeLimit(value: unknown): number {
  return typeof value === 'number' && Number.isSafeInteger(value) && value > 0
    ? value
    : MAX_CANONICAL_HISTORY_ROWS;
}

function parseEventCursor(value: unknown): number | undefined {
  if (typeof value !== 'string') return undefined;
  const cursor = value.trim();
  const raw = cursor.startsWith('event:')
    ? cursor.slice('event:'.length)
    : cursor.startsWith('cursor:')
      ? cursor.slice('cursor:'.length)
      : cursor;
  if (!/^[0-9]+$/.test(raw)) return undefined;
  const parsed = Number(raw);
  return Number.isSafeInteger(parsed) ? parsed : undefined;
}
