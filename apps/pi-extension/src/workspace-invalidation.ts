export interface WorkspaceInvalidationEvent {
  schema?: string;
  event?: string;
  type?: string;
  event_id?: string;
  cursor?: string;
  project_root?: string;
  continuity_id?: string;
  session_id?: string;
  attachment_id?: string;
  work_surface_id?: string;
  invalidate?: string[];
}

export interface InvalidationPlan {
  accepted: boolean;
  reason: string;
  cursor?: string;
  refetchKeys: string[];
  stale: boolean;
  transport: "sse" | "polling_fallback";
}

const NAMED_PREFIXES = [
  "mission_canvas.",
  "workspace.",
  "workpoint.",
  "context.",
  "connectors.",
] as const;

export function isNamedInvalidationKey(key: string): boolean {
  return NAMED_PREFIXES.some((prefix) => key.startsWith(prefix));
}

export function planWorkspaceInvalidation(
  event: WorkspaceInvalidationEvent,
  projectRoot: string,
  continuityId: string,
  visibleKeys: string[],
  subscribedKeys: string[],
  transport: "sse" | "polling_fallback" = "sse"
): InvalidationPlan {
  if (event.schema && event.schema !== "focusa.workspace_event.v1") {
    return { accepted: false, reason: "not_workspace_invalidation", refetchKeys: [], stale: false, transport };
  }
  if (event.project_root && event.project_root !== projectRoot) {
    return { accepted: false, reason: "cross_project_scope", refetchKeys: [], stale: false, transport };
  }
  if (event.continuity_id && event.continuity_id !== continuityId) {
    return { accepted: false, reason: "cross_workstream_scope", refetchKeys: [], stale: false, transport };
  }
  const requested = [...new Set((event.invalidate || []).filter(isNamedInvalidationKey))];
  const allowed = new Set([...visibleKeys, ...subscribedKeys]);
  const refetchKeys = requested.filter((key) => allowed.has(key));
  return {
    accepted: true,
    reason: refetchKeys.length ? "targeted_refetch" : "no_visible_subscription",
    cursor: event.cursor,
    refetchKeys,
    stale: transport === "polling_fallback",
    transport,
  };
}

export function reconnectInvalidationPlan(cursor?: string): InvalidationPlan {
  return {
    accepted: true,
    reason: cursor ? "resume_from_cursor" : "snapshot_fallback",
    cursor,
    refetchKeys: cursor ? [] : ["mission_canvas.summary"],
    stale: true,
    transport: "sse",
  };
}
