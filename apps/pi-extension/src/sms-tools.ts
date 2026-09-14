import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";
import { Type } from "@sinclair/typebox";
import { focusaFetch } from "./state.js";

function result(summary: string, data: any, exposeAuthorizedData = false) {
  const ok = data?.ok !== false && !data?.failure_class;
  const text = exposeAuthorizedData ? `${summary}\n${JSON.stringify(data)}` : summary;
  return {
    content: [{ type: "text" as const, text }],
    details: {
      schema: "focusa.tool_result_v1",
      canonical: true,
      ok,
      status: data?.status || (ok ? "ok" : "blocked"),
      summary,
      data,
    },
  };
}

async function get(path: string) {
  return focusaFetch(path);
}
async function post(path: string, body: unknown) {
  return focusaFetch(path, { method: "POST", body: JSON.stringify(body) });
}

export function registerSmsTools(pi: ExtensionAPI) {
  pi.registerTool({
    name: "focusa_sms_health",
    label: "SMS Broker Health",
    description:
      "Read value-free connector/checkpoint health. Never returns messages, cookies, pairing state, or OTP values.",
    parameters: Type.Object({}),
    async execute() {
      const data = await get("/v1/sms/health");
      return result(`SMS broker → ${data?.status || "unknown"}`, data);
    },
  });
  pi.registerTool({
    name: "focusa_sms_enrollment",
    label: "SMS Enrollment",
    description: "Read value-free customer-owned connector enrollment status.",
    parameters: Type.Object({}),
    async execute() {
      const data = await get("/v1/sms/enrollment");
      return result(`SMS enrollment → ${data?.status || "unknown"}`, data);
    },
  });
  pi.registerTool({
    name: "focusa_sms_threads",
    label: "SMS Threads",
    description:
      "List customer-authorized thread summaries under a separately granted list_threads capability.",
    parameters: Type.Object({
      grant_id: Type.String(),
      consumer_ref: Type.String(),
      limit: Type.Optional(Type.Integer({ minimum: 1, maximum: 200, default: 50 })),
    }),
    async execute(_id, p: any) {
      const query = new URLSearchParams({
        grant_id: p.grant_id,
        consumer_ref: p.consumer_ref,
        limit: String(p.limit || 50),
      });
      const data = await get(`/v1/sms/threads?${query}`);
      return result(`SMS threads → ${data?.threads?.length || 0}`, data, true);
    },
  });
  pi.registerTool({
    name: "focusa_sms_read_thread",
    label: "Read SMS Thread",
    description: "Read a bounded customer-authorized thread. OTP grants do not authorize this tool.",
    parameters: Type.Object({
      thread_handle: Type.String(),
      grant_id: Type.String(),
      consumer_ref: Type.String(),
      limit: Type.Optional(Type.Integer({ minimum: 1, maximum: 200, default: 50 })),
    }),
    async execute(_id, p: any) {
      const data = await get(
        `/v1/sms/threads/${encodeURIComponent(p.thread_handle)}/messages?${new URLSearchParams({ grant_id: p.grant_id, consumer_ref: p.consumer_ref, limit: String(p.limit || 50) })}`
      );
      return result(`SMS thread read → ${data?.messages?.length || 0} messages`, data, true);
    },
  });
  pi.registerTool({
    name: "focusa_sms_search",
    label: "Search SMS",
    description: "Search customer-authorized message scope with bounded results.",
    parameters: Type.Object({
      query: Type.String({ minLength: 1, maxLength: 500 }),
      grant_id: Type.String(),
      consumer_ref: Type.String(),
      limit: Type.Optional(Type.Integer({ minimum: 1, maximum: 200, default: 50 })),
    }),
    async execute(_id, p: any) {
      const query = new URLSearchParams({
        query: p.query,
        grant_id: p.grant_id,
        consumer_ref: p.consumer_ref,
        limit: String(p.limit || 50),
      });
      const data = await get(`/v1/sms/search?${query}`);
      return result(`SMS search → ${data?.matches?.length || 0} matches`, data, true);
    },
  });
  pi.registerTool({
    name: "focusa_sms_send",
    label: "Send SMS",
    description:
      "Send one customer-authorized message. Requires separate send grant, idempotency key, consumer attribution, and confirm=true.",
    parameters: Type.Object({
      recipient_handle: Type.String(),
      body: Type.String({ minLength: 1, maxLength: 5000 }),
      idempotency_key: Type.String(),
      grant_id: Type.String(),
      consumer_ref: Type.String(),
      confirm: Type.Boolean(),
    }),
    async execute(_id, p: any) {
      if (p.confirm !== true)
        return result("SMS send blocked: confirm=true required", {
          status: "blocked",
          failure_class: "approval_required",
        });
      const data = await post("/v1/sms/send", {
        recipient_handles: [p.recipient_handle],
        body: p.body,
        idempotency_key: p.idempotency_key,
        grant_id: p.grant_id,
        consumer_ref: p.consumer_ref,
        confirm: true,
      });
      return result(`SMS send → ${data?.status || "unknown"}`, data);
    },
  });
  pi.registerTool({
    name: "focusa_sms_otp_challenge",
    label: "Register SMS OTP Challenge",
    description:
      "Register an exact provider/target challenge before requesting OTP delivery. Returns a handle, never an OTP.",
    parameters: Type.Object({
      provider: Type.String(),
      target_handle: Type.String(),
      consumer_ref: Type.String(),
      grant_id: Type.String(),
      ttl_seconds: Type.Optional(Type.Integer({ minimum: 30, maximum: 600, default: 300 })),
    }),
    async execute(_id, p: any) {
      const data = await post("/v1/sms/otp/challenges", { ...p, ttl_seconds: p.ttl_seconds || 300 });
      return result(`SMS OTP challenge → ${data?.status || "unknown"}`, data);
    },
  });
  pi.registerTool({
    name: "focusa_sms_otp_inject",
    label: "Inject SMS OTP",
    description:
      "Inject one eligible OTP into its exact bound target. The OTP value never enters model context or tool output.",
    parameters: Type.Object({
      challenge_handle: Type.String(),
      target_handle: Type.String(),
      consumer_ref: Type.String(),
      grant_id: Type.String(),
    }),
    async execute(_id, p: any) {
      const data = await post("/v1/sms/otp/inject", p);
      return result(`SMS OTP injection → injected=${data?.injected === true}`, data);
    },
  });
  pi.registerTool({
    name: "focusa_sms_checkpoint",
    label: "Checkpoint SMS Connector",
    description:
      "Create and verify an encrypted atomic connector checkpoint. Returns value-free receipt metadata only.",
    parameters: Type.Object({
      grant_id: Type.String(),
      consumer_ref: Type.String(),
      confirm: Type.Boolean(),
    }),
    async execute(_id, p: any) {
      if (p.confirm !== true)
        return result("SMS checkpoint blocked: confirm=true required", {
          status: "blocked",
          failure_class: "approval_required",
        });
      const data = await post("/v1/sms/checkpoint", { grant_id: p.grant_id, consumer_ref: p.consumer_ref });
      return result(`SMS checkpoint → ${data?.status || "unknown"}`, data);
    },
  });
  pi.registerTool({
    name: "focusa_sms_events",
    label: "SMS Broker Events",
    description: "Read bounded value-free broker audit events.",
    parameters: Type.Object({
      grant_id: Type.String(),
      consumer_ref: Type.String(),
      since: Type.Optional(Type.String()),
      limit: Type.Optional(Type.Integer({ minimum: 1, maximum: 500, default: 100 })),
    }),
    async execute(_id, p: any) {
      const query = new URLSearchParams({
        grant_id: p.grant_id,
        consumer_ref: p.consumer_ref,
        limit: String(p.limit || 100),
      });
      if (p.since) query.set("since", p.since);
      const data = await get(`/v1/sms/events?${query}`);
      return result(`SMS events → ${data?.events?.length || 0}`, data, true);
    },
  });
  pi.registerTool({
    name: "focusa_sms_revoke",
    label: "Revoke SMS Connector",
    description: "Revoke one customer-owned connector and its grants. Destructive; requires confirm=true.",
    parameters: Type.Object({
      connector_id: Type.String(),
      grant_id: Type.String(),
      consumer_ref: Type.String(),
      confirm: Type.Boolean(),
    }),
    async execute(_id, p: any) {
      if (p.confirm !== true)
        return result("SMS revoke blocked: confirm=true required", {
          status: "blocked",
          failure_class: "approval_required",
        });
      const data = await post("/v1/sms/revoke", {
        connector_id: p.connector_id,
        grant_id: p.grant_id,
        consumer_ref: p.consumer_ref,
        confirm: "REVOKE",
      });
      return result(`SMS revoke → ${data?.status || "unknown"}`, data);
    },
  });
}
