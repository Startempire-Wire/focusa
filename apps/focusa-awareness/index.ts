interface OpenClawPluginApi {
  registerTool?(tool: {
    name: string;
    label: string;
    description: string;
    parameters: Record<string, unknown>;
    execute(callId: string, args: unknown, signal?: AbortSignal): Promise<{
      content: Array<{ type: "text"; text: string }>;
      details: Record<string, unknown>;
    }>;
  }): void;
  pluginConfig?: Record<string, unknown>;
  logger: {
    info(message: string): void;
    warn(message: string): void;
  };
  on(
    event: "before_prompt_build",
    handler: (
      event: unknown,
      ctx: { sessionKey?: string },
    ) => Promise<{ prependContext: string }>,
  ): void;
}

interface FocusaAwarenessConfig {
  focusaUrl?: string;
  adapterId?: string;
  workspaceId?: string;
  agentId?: string;
  operatorId?: string;
  projectRoot?: string;
  continuityId?: string;
  timeoutMs?: number;
  enabled?: boolean;
}

function cfg(api: OpenClawPluginApi): Required<FocusaAwarenessConfig> {
  const raw = (api.pluginConfig ?? {}) as FocusaAwarenessConfig;
  return {
    focusaUrl:
      raw.focusaUrl || process.env.FOCUSA_API_URL || "http://127.0.0.1:8787",
    adapterId: raw.adapterId || "openclaw",
    workspaceId: raw.workspaceId?.trim() || "",
    agentId: raw.agentId?.trim() || "",
    operatorId: raw.operatorId?.trim() || "",
    projectRoot: raw.projectRoot || process.env.FOCUSA_PROJECT_ROOT || "",
    continuityId: raw.continuityId || process.env.FOCUSA_CONTINUITY_ID || "",
    timeoutMs: typeof raw.timeoutMs === "number" && Number.isFinite(raw.timeoutMs)
      ? Math.trunc(Math.max(100, Math.min(raw.timeoutMs, 5000)))
      : 1500,
    enabled: raw.enabled !== false,
  };
}

function sessionIdFromContext(
  ctx: { sessionKey?: string } | undefined,
): string {
  return typeof ctx?.sessionKey === "string" && ctx.sessionKey.length <= 240
    ? ctx.sessionKey.trim()
    : "";
}

function fallbackCard(
  c: Required<FocusaAwarenessConfig>,
  reason: string,
): string {
  return [
    "# Focusa Utility Card",
    "Status: degraded / scoped awareness unavailable; current execution admission unverified",
    `Agent: adapter=${c.adapterId} workspace=${c.workspaceId} agent=${c.agentId} operator=${c.operatorId}`,
    "Mission: use latest operator instruction and OpenClaw/Wirebot workspace context.",
    "Next anchor: preserve explicit operator intent; mark cognition_degraded=true. Fallback context is advisory, not replacement authority for governed effects.",
    `Configured lookup inputs (not verified binding): project_root=${c.projectRoot || "unbound"}; continuity_id=${c.continuityId || "unbound"}`,
    "Resolve actual Scope/Workstream/continuity/attachment and current operation/frontier through the supported installed adapter before dependent effects; configuration alone proves none of these.",
    "",
    "Use the cohesive project journey and contextual installed capabilities, not a mandatory tool sequence:",
    "- Initialize Bootstrap/Genesis only for missing/interrupted state; reuse valid existing intent, specifications, tasks and evidence.",
    "- Prepare/Act/Reconcile/Advance within existing grants; ordinary choices inside a verified grant do not require renewed permission.",
    "- Diagnose exact failed dependencies, perform supported recovery and verify resumption; independently admitted work continues.",
    "- Active development uses approved reload/deploy/consumer testing; Git push is project-specific and production signed-release controls remain.",
    "- First when uncertain/degraded: call /v1/doctor or run `focusa doctor --json`.",
    "- Before compaction/model switch/fork/risky continuation: checkpoint a project-bound Workpoint.",
    "- After compaction/reload/resume: fetch Workpoint resume; do not trust transcript tail over Workpoint.",
    "- After proof/tests/API/file evidence: capture or link evidence to the active Workpoint.",
    "- Before risky or uncertain next action: record a prediction; after outcome: evaluate it.",
    `- Degraded reason: ${reason}`,
    "Operator steering always wins; Focusa guides, preserves, and audits.",
  ].join("\n");
}

async function fetchCard(
  c: Required<FocusaAwarenessConfig>,
  sessionId: string,
): Promise<string> {
  const qs = new URLSearchParams({
    adapter_id: c.adapterId,
    session_id: sessionId,
    project_root: c.projectRoot,
  });
  if (c.workspaceId) qs.set("workspace_id", c.workspaceId);
  if (c.agentId) qs.set("agent_id", c.agentId);
  if (c.operatorId) qs.set("operator_id", c.operatorId);
  if (c.continuityId) qs.set("continuity_id", c.continuityId);
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), c.timeoutMs);
  try {
    const res = await fetch(
      `${c.focusaUrl.replace(/\/$/, "")}/v1/awareness/card?${qs.toString()}`,
      {
        signal: controller.signal,
        redirect: "error",
        headers: { accept: "application/json" },
      },
    );
    if (!res.ok) throw new Error(`Focusa awareness HTTP ${res.status}`);
    const body = (await res.json()) as { rendered_card?: string };
    if (!body.rendered_card)
      throw new Error("Focusa awareness response missing rendered_card");
    return body.rendered_card;
  } finally {
    clearTimeout(timer);
  }
}

type ReadTool = NonNullable<OpenClawPluginApi["registerTool"]> extends (tool: infer T) => void ? T : never;

function registerBoundedRead(
  api: OpenClawPluginApi,
  c: Required<FocusaAwarenessConfig>,
  metadata: Pick<ReadTool, "name" | "label" | "description">,
  path: string,
  project: (body: Record<string, unknown>) => Record<string, unknown>,
) {
  api.registerTool?.({
    ...metadata,
    parameters: {
      type: "object",
      additionalProperties: false,
      properties: { query: { type: "string", maxLength: 120 } },
    },
    async execute(_callId, args, signal) {
      const reply = (details: Record<string, unknown>) => ({
        content: [{ type: "text" as const, text: JSON.stringify(details) }], details,
      });
      const query = (args as { query?: unknown } | null)?.query;
      if (query !== undefined && (typeof query !== "string" || query.length > 120)) {
        return reply({ status: "invalid_input", message: "Use a search query of at most 120 characters." });
      }
      try {
        const expiry = AbortSignal.timeout(c.timeoutMs);
        const params = new URLSearchParams({ limit: "8", query: typeof query === "string" ? query.trim() : "" });
        const response = await fetch(`${c.focusaUrl.replace(/\/$/, "")}${path}?${params}`, {
          headers: { accept: "application/json" },
          redirect: "error",
          signal: signal ? AbortSignal.any([signal, expiry]) : expiry,
        });
        if (!response.ok) throw new Error("read rejected");
        const body = await response.json() as Record<string, unknown>;
        if (!body || typeof body !== "object" || body.ok === false || body.degraded === true) {
          throw new Error("read did not complete");
        }
        return reply(project(body));
      } catch {
        return reply({ status: signal?.aborted ? "cancelled" : "unavailable", message: "Focusa read did not complete. Other authorized work can continue." });
      }
    },
  });
}

const boundedText = (value: unknown, max: number) => typeof value === "string" ? value.slice(0, max) : "";

function registerReads(api: OpenClawPluginApi, c: Required<FocusaAwarenessConfig>) {
  registerBoundedRead(api, c, {
    name: "focusa_capabilities", label: "Focusa capabilities",
    description: "Search the Focusa catalog. Catalog entries do not imply current OpenClaw tool access.",
  }, "/v1/agent/tools", body => {
    if (!Array.isArray(body.tools)) throw new Error("invalid catalog");
    return { status: "completed", catalog_only: true, total: body.total,
      capabilities: body.tools.slice(0, 8).map(row => ({ id: boundedText(row.capability_id, 160), description: boundedText(row.summary, 240) })) };
  });
  registerBoundedRead(api, c, {
    name: "focusa_reflex_primitives", label: "Focusa recovery guidance",
    description: "Find advisory recovery or evidence guidance for a situation. Does not authorize or execute actions.",
  }, "/v1/reflex/primitives", body => {
    if (body.status !== "completed" || body.degraded === true || body.read_only !== true || body.advisory_only !== true || !Array.isArray(body.items)) {
      throw new Error("unqualified guidance");
    }
    return { status: "completed", advisory_only: true, read_only: true, items: body.items.slice(0, 8).map(row => ({
      id: boundedText(row.primitive_id, 160), family: boundedText(row.family, 80),
      trigger: boundedText(row.trigger, 240), recommended_tool: boundedText(row.recommended_tool, 120),
    })) };
  });
}

const focusaAwareness = {
  id: "focusa-awareness",
  name: "Focusa Awareness",
  description: "Inject Focusa Utility Card into OpenClaw/Wirebot agent starts",
  kind: "extension" as const,

  register(api: OpenClawPluginApi) {
    const c = cfg(api);
    if (!c.enabled) {
      api.logger.info("focusa-awareness: disabled by config");
      return;
    }

    registerReads(api, c);
    api.on(
      "before_prompt_build",
      async (_event: unknown, ctx: { sessionKey?: string }) => {
        const sessionId = sessionIdFromContext(ctx);
        if (!sessionId || !c.projectRoot || !c.continuityId) {
          return { prependContext: "Focusa: no scoped project context for this conversation. Continue ordinary work without assuming project authority." };
        }
        try {
          return { prependContext: await fetchCard(c, sessionId) };
        } catch {
          return { prependContext: fallbackCard(c, "Focusa awareness read did not complete") };
        }
      },
    );

    api.logger.info("focusa-awareness: adapter registered");
  },
};

export default focusaAwareness;
