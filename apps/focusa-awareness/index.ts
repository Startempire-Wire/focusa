interface OpenClawPluginApi {
  pluginConfig?: Record<string, unknown>;
  logger: {
    info(message: string): void;
    warn(message: string): void;
  };
  on(
    event: "before_agent_start",
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
    workspaceId: raw.workspaceId || "wirebot",
    agentId: raw.agentId || "wirebot",
    operatorId: raw.operatorId || "verious.smith",
    projectRoot: raw.projectRoot || process.env.FOCUSA_PROJECT_ROOT || "",
    continuityId: raw.continuityId || process.env.FOCUSA_CONTINUITY_ID || "",
    timeoutMs: Number(raw.timeoutMs || 1500),
    enabled: raw.enabled !== false,
  };
}

function sessionIdFromContext(
  ctx: { sessionKey?: string } | undefined,
): string {
  return String(ctx?.sessionKey || "openclaw-session").slice(0, 240);
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
    workspace_id: c.workspaceId,
    agent_id: c.agentId,
    operator_id: c.operatorId,
    session_id: sessionId,
    project_root: c.projectRoot,
  });
  if (c.continuityId) qs.set("continuity_id", c.continuityId);
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), c.timeoutMs);
  try {
    const res = await fetch(
      `${c.focusaUrl.replace(/\/$/, "")}/v1/awareness/card?${qs.toString()}`,
      {
        signal: controller.signal,
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

    api.on(
      "before_agent_start",
      async (_event: unknown, ctx: { sessionKey?: string }) => {
        const sessionId = sessionIdFromContext(ctx);
        if (!c.projectRoot || !c.continuityId) {
          const reason =
            "project_root and continuity_id lookup inputs are missing; no inferred/global fallback. Their presence alone would not establish exact Workstream/attachment or current action admission";
          api.logger.warn(
            `focusa-awareness: blocked injection session=${sessionId.slice(0, 80)} reason=${reason}`,
          );
          return { prependContext: fallbackCard(c, reason) };
        }
        try {
          const card = await fetchCard(c, sessionId);
          api.logger.info(
            `focusa-awareness: injected card session=${sessionId.slice(0, 80)}`,
          );
          return { prependContext: card };
        } catch (err) {
          const reason = String(err instanceof Error ? err.message : err);
          api.logger.warn(
            `focusa-awareness: degraded injection session=${sessionId.slice(0, 80)} reason=${reason}`,
          );
          return { prependContext: fallbackCard(c, reason) };
        }
      },
    );

    api.logger.info(
      `focusa-awareness: active url=${c.focusaUrl} workspace=${c.workspaceId}`,
    );
  },
};

export default focusaAwareness;
