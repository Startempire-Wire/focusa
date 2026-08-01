import type { HostLifecycleStatus, MissionCanvasScope, RichHostLifecycleState } from "./types.js";

export interface ProjectionEnvelope {
  schema: "focusa.resolved_workspace_projection.v1";
  scope: MissionCanvasScope;
  projection_revision: number;
  layout_revision: number;
  durable_event_cursor: string;
  projection_digest: string;
  [key: string]: unknown;
}

export interface ProjectionEvent {
  event_id: string;
  event_kind: string;
  scope: MissionCanvasScope;
  projection_revision: number;
  layout_revision: number;
  [key: string]: unknown;
}

export class MissionCanvasApiClient {
  private projection?: ProjectionEnvelope;
  private eventCursor = "0";

  constructor(
    private readonly baseUrl: string,
    private readonly token: string | undefined,
    readonly scope: MissionCanvasScope,
    private readonly fetchImpl: typeof fetch = fetch
  ) {}

  async ensureProjection(signal?: AbortSignal): Promise<ProjectionEnvelope> {
    try {
      return await this.getProjection(signal);
    } catch (error) {
      if (!String(error).includes("404")) throw error;
    }
    const contributionId = "contribution:pi-session";
    const response = await this.request<{ projection: ProjectionEnvelope }>(
      "POST",
      "/mission-canvas/projection/resolve",
      {
        candidates: [
          {
            contribution_id: contributionId,
            kind: "focused_work_surface",
            semantic_binding_id: "semantic:pi-session",
            renderer_binding_id: "renderer:pi-session@v1",
            priority: 1000,
            applicable_profile_ids: ["software", "general"],
            applicable_activity_mode_ids: ["overview", "sessions"],
            canonical_content_refs: [
              { kind: "pi_session", ref: this.scope.session_id, revision: 0, freshness: "current" },
            ],
            required_capabilities: [],
            required_permissions: [],
            required_operations: ["focusa.agent_execution.prompt"],
            geometry: {
              preferred_regions: ["primary"],
              minimum_span: 6,
              maximum_span: 12,
              preferred_order: 0,
              merge_policy: "never",
              tab_policy: "compatible",
              inspector_side: "none",
            },
          },
        ],
        eligibility: {
          scope: this.scope,
          profile_id: "software",
          activity_mode_id: "overview",
          projection_revision: 1,
          capabilities: [],
          permissions: ["session:read", "session:prompt"],
          available_operations: ["focusa.agent_execution.prompt"],
          meaningful_content: { [contributionId]: true },
          previously_eligible: [],
          observed_at: new Date().toISOString(),
        },
        workspace_profile_revision: 1,
        activity_mode_revision: 1,
        focused_work_surface_id: this.scope.session_id,
        canonical_read_model_revision: 0,
        viewport_width: 1280,
        viewport_height: 800,
        viewport_class: "standard",
        focused_semantic_target: "semantic:pi-session",
        previous_projection_revision: 0,
        previous_layout_revision: 0,
        event_cursor: "mission-canvas:1",
        causation_id: "rich-host-bootstrap",
        idempotency_key: `rich-host-bootstrap:${this.scope.attachment_id}`,
      },
      "mission_canvas:write",
      signal
    );
    this.assertScope(response.projection.scope);
    this.projection = response.projection;
    this.eventCursor = response.projection.durable_event_cursor;
    return response.projection;
  }

  async getProjection(signal?: AbortSignal): Promise<ProjectionEnvelope> {
    const projection = await this.request<ProjectionEnvelope>("GET", "/mission-canvas/projection", undefined, "mission_canvas:read", signal);
    this.assertScope(projection.scope);
    if (this.projection && projection.projection_revision < this.projection.projection_revision) {
      throw new Error("Mission Canvas projection revision regressed");
    }
    this.projection = projection;
    this.eventCursor = projection.durable_event_cursor;
    return projection;
  }

  async events(signal?: AbortSignal): Promise<ProjectionEvent[]> {
    const response = await this.request<{ events: Array<[number, ProjectionEvent]> }>("GET", "/mission-canvas/events", undefined, "mission_canvas:read", signal);
    const events = response.events.map(([, event]) => event);
    for (const event of events) {
      this.assertScope(event.scope);
      if (this.projection && event.projection_revision < this.projection.projection_revision) continue;
      this.eventCursor = String(event.projection_revision);
    }
    return events;
  }

  async appendPiSessionEvent(eventKind: "pi_turn_started" | "pi_turn_completed" | "pi_message_updated" | "pi_tool_started" | "pi_tool_completed", payload: unknown, eventId: string, signal?: AbortSignal): Promise<unknown> {
    return this.request("POST", "/mission-canvas/pi-session/events", {
      scope: this.scope,
      event_id: eventId,
      event_kind: eventKind,
      projection_revision: this.projection?.projection_revision ?? 0,
      layout_revision: this.projection?.layout_revision ?? 0,
      payload,
      occurred_at: new Date().toISOString(),
    }, "mission_canvas:write", signal);
  }

  async updateHostLifecycle(
    action: "launch" | "focus" | "hide" | "close",
    state: RichHostLifecycleState,
    expectedRevision: number | undefined,
    signal?: AbortSignal
  ): Promise<unknown> {
    return this.request("POST", `/mission-canvas/rich-host/${action}`, {
      scope: this.scope,
      document_id: state.host_instance_id,
      revision: state.lifecycle_revision,
      expected_revision: expectedRevision,
      payload: state,
      idempotency_key: `${action}:${state.host_instance_id}:${state.lifecycle_revision}`,
    }, "mission_canvas:host", signal);
  }

  async syncDraft(payload: Record<string, unknown>, draftId: string, revision: number, expectedRevision?: number, signal?: AbortSignal): Promise<unknown> {
    return this.request("POST", "/mission-canvas/drafts/sync", {
      scope: this.scope,
      document_id: draftId,
      revision,
      expected_revision: expectedRevision,
      payload,
      idempotency_key: `draft:${draftId}:${revision}`,
    }, "mission_canvas:draft", signal);
  }

  cachedProjection(): ProjectionEnvelope | undefined {
    return this.projection;
  }

  durableEventCursor(): string {
    return this.eventCursor;
  }

  private async request<T>(method: "GET" | "POST", path: string, body: unknown, permission: string, signal?: AbortSignal): Promise<T> {
    const url = new URL(`${this.baseUrl.replace(/\/$/, "")}${path}`);
    if (method === "GET") {
      for (const [key, value] of Object.entries(this.scope)) if (value != null) url.searchParams.set(key, String(value));
    }
    const response = await this.fetchImpl(url, {
      method,
      signal,
      headers: {
        "content-type": "application/json",
        "x-focusa-permissions": permission,
        ...(this.token ? { authorization: `Bearer ${this.token}` } : {}),
      },
      body: body === undefined ? undefined : JSON.stringify(body),
    });
    if (!response.ok) throw new Error(`Mission Canvas API ${method} ${path} failed: ${response.status} ${await response.text()}`);
    return (await response.json()) as T;
  }

  private assertScope(observed: MissionCanvasScope): void {
    for (const key of ["project_root", "continuity_id", "session_id", "attachment_id"] as const) {
      if (observed[key] !== this.scope[key]) throw new Error(`Mission Canvas scope mismatch: ${key}`);
    }
  }
}

export function lifecycleState(
  scope: MissionCanvasScope,
  status: HostLifecycleStatus,
  revision: number,
  rendererResolution: RichHostLifecycleState["renderer_resolution"],
  processId?: number,
  windowId?: string
): RichHostLifecycleState {
  return {
    host_instance_id: `rich-host:${scope.attachment_id}`,
    scope,
    renderer_resolution: rendererResolution,
    state: status,
    process_id: processId ?? null,
    window_id: windowId ?? null,
    focused: status === "focused",
    durable_event_cursor: "0",
    pi_draft_ref: null,
    canvas_draft_ref: null,
    last_error_ref: null,
    lifecycle_revision: revision,
    updated_at: new Date().toISOString(),
  };
}
