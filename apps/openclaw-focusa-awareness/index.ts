import type { OpenClawPluginApi } from "openclaw/plugin-sdk";

interface FocusaAwarenessConfig {
  focusaUrl?: string;
  adapterId?: string;
  workspaceId?: string;
  agentId?: string;
  operatorId?: string;
  projectRoot?: string;
  timeoutMs?: number;
  enabled?: boolean;
}

function cfg(api: OpenClawPluginApi): Required<FocusaAwarenessConfig> {
  const raw = (api.pluginConfig ?? {}) as FocusaAwarenessConfig;
  return {
    focusaUrl: raw.focusaUrl || process.env.FOCUSA_API_URL || "http://127.0.0.1:8787",
    adapterId: raw.adapterId || "openclaw",
    workspaceId: raw.workspaceId || "wirebot",
    agentId: raw.agentId || "wirebot",
    operatorId: raw.operatorId || "verious.smith",
    projectRoot: raw.projectRoot || "/data/wirebot/users/verious",
    timeoutMs: Number(raw.timeoutMs || 1500),
    enabled: raw.enabled !== false,
  };
}

function sessionIdFromContext(ctx: { sessionKey?: string } | undefined): string {
  return String(ctx?.sessionKey || "openclaw-session").slice(0, 240);
}

function fallbackCard(c: Required<FocusaAwarenessConfig>, reason: string): string {
  return [
    "# Focusa Utility Card",
    "Status: degraded / awareness endpoint unavailable",
    `Agent: adapter=${c.adapterId} workspace=${c.workspaceId} agent=${c.agentId} operator=${c.operatorId}`,
    "Mission: use latest operator instruction and OpenClaw/Wirebot workspace context.",
    "Next anchor: Focusa unavailable; mark cognition_degraded=true and use explicit fallback context.",
    `Scope: project_root=${c.projectRoot}`,
    "",
    "Use Focusa as agent working memory and governance when available:",
    "- First when uncertain/degraded: call /v1/doctor or run `focusa doctor --json`.",
    "- Before compaction/model switch/fork/risky continuation: checkpoint a scoped Workpoint.",
    "- After compaction/reload/resume: fetch Workpoint resume; do not trust transcript tail over Workpoint.",
    "- After proof/tests/API/file evidence: capture or link evidence to the active Workpoint.",
    "- Before risky or uncertain next action: record a prediction; after outcome: evaluate it.",
    `- Degraded reason: ${reason}`,
    "Operator steering always wins; Focusa guides, preserves, and audits.",
  ].join("\n");
}

async function fetchCard(c: Required<FocusaAwarenessConfig>, sessionId: string): Promise<string> {
  const qs = new URLSearchParams({
    adapter_id: c.adapterId,
    workspace_id: c.workspaceId,
    agent_id: c.agentId,
    operator_id: c.operatorId,
    session_id: sessionId,
    project_root: c.projectRoot,
  });
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), c.timeoutMs);
  try {
    const res = await fetch(`${c.focusaUrl.replace(/\/$/, "")}/v1/awareness/card?${qs.toString()}`, {
      signal: controller.signal,
      headers: { "accept": "application/json" },
    });
    if (!res.ok) throw new Error(`Focusa awareness HTTP ${res.status}`);
    const body = await res.json() as { rendered_card?: string };
    if (!body.rendered_card) throw new Error("Focusa awareness response missing rendered_card");
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

    api.on("before_agent_start", async (_event: unknown, ctx: { sessionKey?: string }) => {
      const sessionId = sessionIdFromContext(ctx);
      try {
        const card = await fetchCard(c, sessionId);
        api.logger.info(`focusa-awareness: injected card session=${sessionId.slice(0, 80)}`);
        return { prependContext: card };
      } catch (err) {
        const reason = String(err instanceof Error ? err.message : err);
        api.logger.warn(`focusa-awareness: degraded injection session=${sessionId.slice(0, 80)} reason=${reason}`);
        return { prependContext: fallbackCard(c, reason) };
      }
    });

    api.logger.info(`focusa-awareness: active url=${c.focusaUrl} workspace=${c.workspaceId}`);
  },
};

export default focusaAwareness;
