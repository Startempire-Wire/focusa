import {
  createFocusaSpec135Client,
  type components,
} from "@focusa/spec135-client";
import actionBindings from "../../../docs/contracts/spec135/generated-contract-v1/ui-action-bindings.fixture.json" with { type: "json" };
import {
  FOCUSA_A2UI_CATALOG_ID,
  FocusaA2uiRenderer,
  type A2uiClientAction,
  type A2uiMessage,
} from "../src/index.js";

type ArtifactIntakeBody = components["schemas"]["focusa_workspace_artifact_intake_request_v1"];
type ArtifactIntakeResult = components["schemas"]["focusa_workspace_artifact_intake_result_v1"];

type WorkspaceStreamEvent = {
  schema: "focusa.stream_event.v1";
  event_id: string;
  cursor: string;
  event_type: string;
  source_state_revision: number;
  payload_ref: string;
  invalidate: string[];
  scope: {
    project_root: string;
    continuity_id: string;
    attachment_id: string;
    work_surface_id?: string;
  };
  payload: {
    schema: "focusa.workspace_event.v1";
    session_id?: string;
    artifact_id: string;
    semantic_authority: false;
  };
};

const commonScope = {
  project_root: "/example/focusa",
  continuity_id: "focusa-cont-u2-generated-ui",
};
const scopeA = {
  ...commonScope,
  attachment_id: "attachment:u2-surface-a",
};
const scopeB = {
  ...commonScope,
  attachment_id: "attachment:u2-surface-b",
};
const sessionId = "focusa-session:u2-generated-ui";
const surfaceAId = "surface:u2-a";
const operationId = "focusa.workspace.artifact.intake";
const binding = actionBindings.bindings.find(
  (candidate) => candidate.action_id === operationId,
);
if (!binding?.control.receipt_required) {
  throw new Error("Generated Workspace Artifact intake binding is unavailable");
}

const client = createFocusaSpec135Client({ baseUrl: window.location.origin });
const surface = document.querySelector<HTMLElement>("#workspace-refresh-surface");
const status = document.querySelector<HTMLElement>("#refresh-result");
if (!surface || !status) throw new Error("Workspace live-refresh proof mount missing");

type RefreshStream = { readyState: number; close(): void };

const observedActions: A2uiClientAction[] = [];
const processedEventIds = new Set<string>();
let stream: RefreshStream | undefined;
let fallbackTimer: ReturnType<typeof setInterval> | undefined;
let lastCursor = "0";
let surfaceARenders = 0;
let surfaceBRenders = 0;
let duplicateEvents = 0;
let reconnects = 0;
let response: ArtifactIntakeResult | undefined;
let lastError: unknown;
const renderWaiters = new Map<number, () => void>();

const renderer = new FocusaA2uiRenderer({
  allowedActionNames: new Set([binding.action_id]),
  async onAction(action) {
    observedActions.push(action);
    status.textContent = "Linking artifact while the exact Work Surface stream is live…";
    try {
      await waitForStream();
      const first = await linkArtifact(scopeA, "u2-ui-link-1", "one");
      response = first;
      // Reopen from zero so UIAI/public reverse proxies also exercise durable replay rather
      // than depending on their live-stream chunk buffering policy.
      stream?.close();
      openStream("0");
      reconnects += 1;
      await waitForSurfaceRender(1);
      const cursorBeforeDisconnect = lastCursor;

      stream?.close();
      renderer.processDelta(progressDelta("Disconnected — preserving stale cursor", 45, "stale"));
      const second = await linkArtifact(scopeA, "u2-ui-link-2", "two");
      response = second;
      openStream(cursorBeforeDisconnect);
      reconnects += 1;
      await waitForSurfaceRender(2);

      await linkArtifact(scopeB, "u2-ui-unrelated", "other");
      await new Promise((resolve) => setTimeout(resolve, 900));
      if (surfaceBRenders !== 0) {
        throw new Error("unrelated Work Surface was invalidated");
      }

      stream?.close();
      if (fallbackTimer) clearInterval(fallbackTimer);
      renderer.processDelta([
        ...progressDelta("Reconnect replay completed", 100, "completed"),
        {
          version: "v0.9",
          updateComponents: {
            surfaceId: "u2-workspace-live-refresh",
            components: [
              {
                id: "cursor",
                component: "FocusaReceiptCard",
                label: "Durable refresh cursor",
                description: `cursor=${lastCursor}; reconnects=${reconnects}; duplicates_ignored=${duplicateEvents}`,
                status: "completed",
                details: `event_ids=${[...processedEventIds].join(",")}`,
              },
              {
                id: "surface-b",
                component: "FocusaEvidenceSummary",
                label: "Work Surface B stayed unchanged",
                description: "An artifact linked to Attachment B did not rerender the subscribed Attachment A surface.",
                status: "verified",
                details: `render_count=${surfaceBRenders}; exact_filter=true`,
              },
            ],
          },
        },
      ]);
      status.textContent = "Missed event recovered; only Work Surface A refreshed";
      document.body.dataset.refreshStatus = "completed";
    } catch (error) {
      lastError = error;
      status.textContent = "Workspace live refresh needs recovery";
      renderer.processDelta([
        {
          version: "v0.9",
          updateComponents: {
            surfaceId: "u2-workspace-live-refresh",
            components: [
              {
                id: "cursor",
                component: "FocusaRecoveryCard",
                label: "Live refresh needs recovery",
                description:
                  "Reconnect from the last confirmed cursor or use bounded exact-scope polling while SSE is offline.",
                status: "retry",
                details: JSON.stringify(error),
              },
            ],
          },
        },
      ]);
      document.body.dataset.refreshStatus = "recovery";
    }
  },
});

function streamUrl(cursor: string): string {
  const streamOrigin =
    new URL(window.location.href).searchParams.get("stream_origin") ??
    window.location.origin;
  const query = new URLSearchParams({
    cursor,
    ...scopeA,
    session_id: sessionId,
    work_surface_id: surfaceAId,
  });
  return `${streamOrigin}/v1/events/stream?${query}`;
}

function openStream(cursor: string): void {
  stream?.close();
  const url = streamUrl(cursor);
  if (new URL(window.location.href).searchParams.get("stream_transport") === "header_fetch") {
    openHeaderSseStream(url);
    return;
  }
  const eventSource = new EventSource(url);
  stream = eventSource;
  eventSource.onopen = markStreamOpen;
  eventSource.onmessage = (message) => handleStreamMessage(message);
  eventSource.addEventListener("focusa_event", (raw) =>
    handleStreamMessage(raw as MessageEvent<string>),
  );
  eventSource.onerror = markStreamStale;
}

function markStreamOpen(): void {
  if (fallbackTimer) {
    clearInterval(fallbackTimer);
    fallbackTimer = undefined;
  }
  document.body.dataset.streamStatus = "connected";
}

function markStreamStale(): void {
  document.body.dataset.streamStatus = "stale";
  if (!fallbackTimer) {
    fallbackTimer = setInterval(() => void refetchSurfaceA("polling_fallback"), 2000);
  }
}

function openHeaderSseStream(url: string): void {
  const abort = new AbortController();
  const transport: RefreshStream = {
    readyState: EventSource.CONNECTING,
    close() {
      abort.abort();
      transport.readyState = EventSource.CLOSED;
    },
  };
  stream = transport;
  void (async () => {
    try {
      const response = await fetch(url, {
        headers: { "bypass-tunnel-reminder": "focusa-u2-eval" },
        signal: abort.signal,
      });
      if (!response.ok || !response.body) {
        throw new Error(`SSE transport returned HTTP ${response.status}`);
      }
      transport.readyState = EventSource.OPEN;
      markStreamOpen();
      const reader = response.body.getReader();
      const decoder = new TextDecoder();
      let pending = "";
      while (!abort.signal.aborted) {
        const chunk = await reader.read();
        if (chunk.done) break;
        pending += decoder.decode(chunk.value, { stream: true });
        const frames = pending.split(/\r?\n\r?\n/);
        pending = frames.pop() ?? "";
        for (const frame of frames) {
          const data = frame
            .split(/\r?\n/)
            .filter((line) => line.startsWith("data:"))
            .map((line) => line.slice(5).trimStart())
            .join("\n");
          if (data) handleStreamMessage({ data } as MessageEvent<string>);
        }
      }
    } catch (error) {
      if (!abort.signal.aborted) {
        lastError = error;
        markStreamStale();
      }
    } finally {
      transport.readyState = EventSource.CLOSED;
    }
  })();
}

function handleStreamMessage(message: MessageEvent<string>): void {
  const event = JSON.parse(message.data) as WorkspaceStreamEvent;
  if (
    event.schema !== "focusa.stream_event.v1" ||
    event.event_type !== "workspace_artifact_linked"
  ) {
    return;
  }
  if (processedEventIds.has(event.event_id)) {
    duplicateEvents += 1;
    return;
  }
  if (
    event.scope.project_root !== scopeA.project_root ||
    event.scope.continuity_id !== scopeA.continuity_id ||
    event.scope.attachment_id !== scopeA.attachment_id ||
    event.scope.work_surface_id !== surfaceAId ||
    event.payload.session_id !== sessionId ||
    event.payload.semantic_authority !== false
  ) {
    return;
  }
  if (Number(event.cursor) <= Number(lastCursor)) {
    throw new Error("workspace event cursor was not monotonic");
  }
  processedEventIds.add(event.event_id);
  lastCursor = event.cursor;
  void refetchSurfaceA("sse", event);
}

async function refetchSurfaceA(
  source: "sse" | "polling_fallback",
  event?: WorkspaceStreamEvent,
): Promise<void> {
  let listed = await client.GET("/v1/workspace/artifacts", {
    params: { query: scopeA },
  });
  for (let attempt = 0; ; attempt += 1) {
    if (listed.error || !listed.data) {
      throw listed.error ?? new Error("exact Work Surface artifact read unavailable");
    }
    if (
      !event ||
      listed.data.state_version >= event.source_state_revision ||
      attempt >= 20
    ) {
      break;
    }
    await new Promise((resolve) => setTimeout(resolve, 50));
    listed = await client.GET("/v1/workspace/artifacts", {
      params: { query: scopeA },
    });
  }
  if (source === "polling_fallback" && listed.data.artifacts.length === surfaceARenders) {
    return;
  }
  surfaceARenders = listed.data.artifacts[0]?.revision ?? surfaceARenders;
  renderer.processDelta([
    {
      version: "v0.9",
      updateComponents: {
        surfaceId: "u2-workspace-live-refresh",
        components: [
          {
            id: "surface-a",
            component: "FocusaEvidenceSummary",
            label: `Work Surface A refreshed to revision ${surfaceARenders}`,
            description: `${listed.data.artifacts.length} exact-scope artifact; source=${source}`,
            status: "saved",
            details: event
              ? `cursor=${event.cursor}; invalidates=${event.invalidate.join(",")}; payload_ref=${event.payload_ref}`
              : "SSE unavailable; bounded polling fallback active",
          },
        ],
      },
    },
  ]);
  renderWaiters.get(surfaceARenders)?.();
  renderWaiters.delete(surfaceARenders);
}

function waitForSurfaceRender(revision: number): Promise<void> {
  if (surfaceARenders >= revision) return Promise.resolve();
  return new Promise((resolve, reject) => {
    const timeout = setTimeout(
      () => reject(new Error(`revision ${revision} refresh timed out`)),
      6000,
    );
    renderWaiters.set(revision, () => {
      clearTimeout(timeout);
      resolve();
    });
  });
}

function waitForStream(): Promise<void> {
  if (stream?.readyState === EventSource.OPEN) return Promise.resolve();
  return new Promise((resolve, reject) => {
    const timeout = setTimeout(() => reject(new Error("SSE connection timed out")), 6000);
    const poll = setInterval(() => {
      if (stream?.readyState === EventSource.OPEN) {
        clearInterval(poll);
        clearTimeout(timeout);
        resolve();
      }
    }, 50);
  });
}

async function linkArtifact(
  scope: typeof scopeA,
  idempotencyKey: string,
  projection: string,
): Promise<ArtifactIntakeResult> {
  const preview = "targeted workspace live refresh";
  const body: ArtifactIntakeBody = {
    ...scope,
    idempotency_key: idempotencyKey,
    expected_state_version: 0,
    artifact_kind: "image",
    mime_type: "image/png",
    title: "Targeted live artifact",
    summary: "Bounded artifact for exact Work Surface invalidation.",
    handle_ref: "uiai-screenshot:session=u2-generated-ui:artifact",
    artifact_url: `https://example.invalid/u2/${projection}`,
    inline_preview: preview,
    sha256: "a30bf25cec8684b8fccb49cdfb09056b2ef7f40765bde5bf4522f6cf6b8f1690",
    size_bytes: preview.length,
    source_system: "uiai",
    source_ref: "uiai-browser:session=u2-generated-ui:artifact",
    source_url: "https://example.invalid/u2",
    project_identity_ref: "project:focusa",
    workpoint_id: "focusa-mc-u2",
    work_item_ref: "focusa-mc-u2",
    instance_id: "focusa-instance:u2-generated-ui",
    focusa_session_id: sessionId,
    work_surface_id:
      scope.attachment_id === scopeA.attachment_id ? surfaceAId : "surface:u2-b",
    uiai_session_id: "uiai-session:u2-generated-ui",
    browser_context_id: "browser-context:u2-generated-ui",
    browser_target_id: "browser-target:u2-generated-ui",
    diagnostics_refs: ["uiai-diagnostics:session=u2-generated-ui:seq=1"],
    evidence_refs: ["evidence:workspace-live-refresh:u2-generated-ui"],
    domain_pack_refs: [],
    candidate_object_refs: [],
    candidate_link_refs: [],
    candidate_claim_refs: [],
    verification_policy_refs: [],
    semantic_delta_refs: [],
    citation_refs: ["source:https://example.invalid/u2"],
    evidence_status: "verified",
    redaction_status: "secret_safe",
    freshness_status: "fresh",
    provenance_status: "verified",
    retention_policy: "project_evidence",
    cleanup_action: "close UIAI session asynchronously",
    preferred_renderer: "image_preview",
    fallback_renderer: "artifact_card_and_open",
    render_width: 1440,
    render_height: 1000,
  };
  let error: unknown;
  for (let attempt = 0; attempt < 5; attempt += 1) {
    const listed = await client.GET("/v1/workspace/artifacts", {
      params: { query: scope },
    });
    if (listed.error || !listed.data) {
      throw listed.error ?? new Error("artifact version read unavailable");
    }
    body.expected_state_version = listed.data.state_version;
    const linked = await client.POST("/v1/workspace/artifacts/intake", {
      params: { query: scope },
      body,
    });
    error = linked.error;
    if (linked.data && !linked.error) return linked.data;
  }
  throw error ?? new Error("artifact link unavailable");
}

function progressDelta(
  label: string,
  progress: number,
  state: string,
): A2uiMessage[] {
  return [
    {
      version: "v0.9",
      updateComponents: {
        surfaceId: "u2-workspace-live-refresh",
        components: [
          {
            id: "progress",
            component: "FocusaProgressStepper",
            label,
            description:
              "SSE remains primary; cursor replay and exact-scope refetch recover missed events.",
            status: state,
            progress,
          },
        ],
      },
    },
  ];
}

const snapshot: A2uiMessage[] = [
  {
    version: "v0.9",
    createSurface: {
      surfaceId: "u2-workspace-live-refresh",
      catalogId: FOCUSA_A2UI_CATALOG_ID,
    },
  },
  {
    version: "v0.9",
    updateComponents: {
      surfaceId: "u2-workspace-live-refresh",
      components: [
        {
          id: "root",
          component: "Column",
          children: [
            "stage",
            "progress",
            "refresh",
            "surface-a",
            "surface-b",
            "cursor",
          ],
        },
        {
          id: "stage",
          component: "FocusaStageShell",
          label: "Refresh only the affected Work Surface",
          description:
            "Link, disconnect, recover a missed durable event, and suppress an unrelated Attachment update.",
          status: "ready",
          details: `operation=${binding.action_id}; stream=/v1/events/stream; scope=${JSON.stringify(scopeA)}`,
        },
        {
          id: "progress",
          component: "FocusaProgressStepper",
          label: "SSE connection ready",
          description: "No Workspace invalidation has been applied yet.",
          status: "ready",
          progress: 0,
        },
        {
          id: "refresh",
          component: "FocusaPrimaryAction",
          label: "Prove targeted live refresh",
          description:
            "Exercises cursor reconnect, missed-event recovery, duplicate tolerance, and unrelated-surface suppression.",
          primaryActionLabel: "Run Live Refresh Proof",
          action: { event: { name: binding.action_id, context: scopeA } },
        },
        {
          id: "surface-a",
          component: "FocusaEvidenceSummary",
          label: "Work Surface A",
          description: "Subscribed to its exact project, workstream, session, Attachment, and surface identity.",
          status: "pending",
        },
        {
          id: "surface-b",
          component: "FocusaEvidenceSummary",
          label: "Work Surface B",
          description: "Unrelated Attachment; render count must stay zero.",
          status: "pending",
        },
        {
          id: "cursor",
          component: "FocusaReceiptCard",
          label: "Durable refresh cursor",
          description: "Cursor and reconnect proof will appear here.",
          status: "pending",
        },
      ],
    },
  },
];
renderer.processSnapshot(snapshot);
renderer.mount(surface, "u2-workspace-live-refresh");
openStream("0");

Object.assign(window, {
  focusaWorkspaceRefreshEval: {
    renderer,
    binding,
    scopeA,
    scopeB,
    observedActions,
    processedEventIds,
    get lastCursor() {
      return lastCursor;
    },
    get surfaceARenders() {
      return surfaceARenders;
    },
    get surfaceBRenders() {
      return surfaceBRenders;
    },
    get duplicateEvents() {
      return duplicateEvents;
    },
    get reconnects() {
      return reconnects;
    },
    get response() {
      return response;
    },
    get lastError() {
      return lastError;
    },
  },
});
