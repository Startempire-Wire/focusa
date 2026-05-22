import { registerTurns } from "../apps/pi-extension/src/turns.ts";
import { S } from "../apps/pi-extension/src/state.ts";

type Handler = (event: any, ctx: any) => any | Promise<any>;

function assert(cond: any, msg: string) {
  if (!cond) throw new Error(msg);
}

class MockPi {
  handlers = new Map<string, Handler[]>();
  sent: any[] = [];
  on(name: string, handler: Handler) {
    const list = this.handlers.get(name) || [];
    list.push(handler);
    this.handlers.set(name, list);
  }
  async emit(name: string, event: any, ctx: any) {
    const list = this.handlers.get(name) || [];
    let last: any;
    for (const handler of list) last = await handler(event, ctx);
    return last;
  }
  sendMessage(message: any) { this.sent.push(message); }
  ui = { setStatus() {}, notify() {}, setWidget() {} };
}

function jsonResponse(body: any, status = 200) {
  return new Response(JSON.stringify(body), {
    status,
    headers: { "content-type": "application/json" },
  });
}

const frame = {
  id: "frame-spec96-runtime",
  status: "active",
  title: "Pi Task: Spec96 Focus Slice runtime proof",
  goal: "Prove LowMem/tool affordance injection",
  tags: ["session-spec96-runtime"],
  focus_state: {
    intent: "Spec96 Focus Slice proof",
    current_focus: "Verify ResourceMode and TOOL_AFFORDANCES injection",
    current_state: "LowMem runtime mock active",
    decisions: [],
    constraints: [],
    failures: [],
    recent_results: [],
    next_steps: [],
    open_questions: [],
    artifacts: [],
  },
};

(globalThis as any).fetch = async (input: any, init?: RequestInit) => {
  const url = new URL(String(input));
  const route = url.pathname.replace(/^\/v1/, "");
  if (route === "/focus/frame/current") return jsonResponse({ active_frame_id: frame.id, frame });
  if (route === "/ascc/state") return jsonResponse({ frame_id: frame.id, ascc: frame.focus_state });
  if (route === "/focus/stack") return jsonResponse({ active_frame_id: frame.id, stack: { active_id: frame.id, frames: [frame] } });
  if (route === "/memory/semantic") return jsonResponse({ semantic: [] });
  if (route === "/ecs/handles") return jsonResponse({ handles: [] });
  if (route === "/ontology/context") return jsonResponse({ ontology_context: { active_object_set: [], relevant_link_paths: [], valid_next_actions: [], blocked_affordances: [], evidence_handles: [], uncertainty_flags: [] } });
  if (route === "/trajectory/view") return jsonResponse({
    status: "completed",
    canonical: true,
    project_identity: { status: "verified", project_root: "/home/wirebot/focusa", continuity_id: "cont-spec96-runtime", confidence: "high" },
    trajectory: {
      definition_status: "clear",
      long_term_goal: "Spec96 compliant Focus Slice",
      desired_end_state: "ResourceMode and tool affordance guidance visible to the model",
      current_state: "Runtime mock active",
      active_gap: "Assert injected sections",
      similarity_group: { advisory_only: true, must_not_merge_sessions: true },
      evidence_refs: [],
    },
    intelligence_view: {
      context_sufficiency: { score: 1, status: "clear", missing_facts: [], recommended_action: "proceed" },
      next_workpoint_candidate: { workpoint_id: "wp-spec96-runtime", next_slice: "Assert injected sections" },
      do_not_use: [],
    },
  });
  if (route === "/resource/mode") return jsonResponse({
    status: "completed",
    resource_mode: {
      mode: "lowmem",
      reason: "operator_forced",
      budget: { hot_route_timeout_ms: 250, max_items_default: 10, hot_payload_bytes: 32768, max_rehydrate_refs: 8 },
      cold_surfaces_deferred: ["full_lineage_tree", "full_ontology_graph", "deep_work_loop_status", "replay_bundles"],
      transition_omitted_count: 3,
    },
  });
  if (route === "/work-loop/context") return jsonResponse({ status: "completed" });
  return jsonResponse({ status: "completed", route, method: init?.method || "GET" });
};

Object.assign(S, {
  pi: null,
  cfg: {
    enabled: true,
    focusaApiBaseUrl: "http://focusa.test/v1",
    focusaApiTimeoutMs: 250,
    emitMetrics: false,
  },
  focusaAvailable: true,
  activeFrameId: frame.id,
  activeFrameTitle: frame.title,
  activeFrameGoal: frame.goal,
  sessionFrameKey: "session-spec96-runtime",
  sessionCwd: "/home/wirebot/focusa",
  continuityId: "cont-spec96-runtime",
  activeWorkpointPacket: { workpoint_id: "wp-spec96-runtime", canonical: true, mission: "Spec96 Focus Slice proof", next_slice: "Assert injected sections", action_intent: { action_type: "resume_workpoint", status: "ready" }, active_object_refs: [], verification_records: [], blockers: [] },
  activeWorkpointSummary: "WORKPOINT wp-spec96-runtime: mission=Spec96 Focus Slice proof; action=resume_workpoint; next=Assert injected sections; canonical=true",
  currentAsk: { text: "Continue", kind: "instruction", sourceTurnId: "pi-turn-runtime", updatedAt: Date.now() },
  queryScope: { scopeKind: "mission_carryover", carryoverPolicy: "allow_if_relevant", sourceTurnId: "pi-turn-runtime", updatedAt: Date.now() },
  excludedContext: null,
  focusStateCache: { key: null, at: 0, data: null, inflight: null },
  semanticMemoryCache: { at: 0, data: null, inflight: null },
  ecsHandlesCache: { at: 0, data: null, inflight: null },
  turnCount: 42,
});

const pi = new MockPi();
registerTurns(pi as any);
const result = await pi.emit("context", { messages: [{ role: "user", content: [{ type: "text", text: "Continue" }] }] }, {
  getContextUsage: () => ({ tokens: 1000, contextWindow: 128000 }),
});

const injected = result?.messages?.[0]?.content?.[0]?.text || "";
assert(injected.includes("PROJECT_IDENTITY: status=verified"), `missing PROJECT_IDENTITY line:
${injected}`);
assert(injected.includes("TRAJECTORY_GOALS: high=Spec96 compliant Focus Slice"), `missing TRAJECTORY_GOALS line:
${injected}`);
assert(injected.includes("ACTIVE_GAP: Assert injected sections"), `missing ACTIVE_GAP line:
${injected}`);
assert(injected.includes("WORKPOINT_CANDIDATE: id=wp-spec96-runtime"), `missing WORKPOINT_CANDIDATE line:
${injected}`);
assert(injected.includes("CONTEXT_SUFFICIENCY: score=1 status=clear"), `missing CONTEXT_SUFFICIENCY line:
${injected}`);
assert(injected.includes("RESOURCE_MODE: lowmem"), `missing RESOURCE_MODE line:\n${injected}`);
assert(injected.includes("LOWMEM_BUDGET: hot_timeout_ms=250"), `missing LOWMEM_BUDGET line:\n${injected}`);
assert(injected.includes("CONTEXT_POSTURE: surgical_summary_only"), `missing context posture:\n${injected}`);
assert(injected.includes("TOOL_AFFORDANCES:"), `missing TOOL_AFFORDANCES:\n${injected}`);
assert(injected.includes("focusa_traverse — fetch narrow"), `missing traverse affordance:\n${injected}`);
assert(injected.includes("scope_mismatch -> focusa_project_verify"), `missing recovery affordance:\n${injected}`);
assert(injected.includes("full lineage tree / full ontology graph / deep work-loop status by default"), `missing do-not-use affordance:\n${injected}`);
console.log("SPEC96 Focus Slice runtime injection proof passed");
