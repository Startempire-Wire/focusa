import { createServer } from "node:http";
import { readFile } from "node:fs/promises";
import { dirname, extname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..", "rich-host");
const scope = { project_root: "/fixture/focusa", continuity_id: "uiai-eval", session_id: "session:uiai", attachment_id: "attachment:uiai", instance_id: null, working_subpath_id: null };
let scenario = process.env.FOCUSA_UIAI_SCENARIO || "populated";
let eventSequence = 1;

function contribution(id, kind, label, data = {}) {
  return {
    contribution_id: `contribution:${id}`,
    kind,
    semantic_binding_id: `semantic:${id}`,
    renderer_binding_id: id === "pi-session" ? "renderer:pi-session@v1" : `renderer:${id}@v1`,
    data_ref: { kind, ref: `${kind}:${id}`, revision: 1, freshness: "current", ...data },
    operation_ids: [],
    authority: { canonical_owner: "Focusa Core", mutation_owner: "Focusa Core", scope, read_only: false },
    freshness: { status: "current", observed_at: "2026-07-31T00:00:00Z" },
    resolved_geometry: { preferred_regions: ["primary"], minimum_span: 1, maximum_span: 12, merge_policy: "compatible", tab_policy: "compatible" },
    accessibility: { label, landmark_role: "region", focus_semantic_id: `semantic:${id}` },
    contribution_revision: 1,
    evidence_refs: [],
  };
}

function projection() {
  const primary = contribution("pi-session", "focused_work_surface", "Active Pi Session", {
    messages: [
      { role: "user", content: "Show the current implementation mission." },
      { role: "assistant", content: "Mission Canvas is resolving canonical work surfaces." },
      { role: "tool", content: { name: "focusa_trajectory_view", status: "completed" } },
    ],
  });
  const inspector = contribution("focusa-inspector", "inspector", "Focusa Inspector", { sections: [{ title: "Mission", status: "active" }, { title: "Evidence", status: "current" }] });
  const rail = contribution("work-rail", "work_rail", "Work Rail", { scope_label: "Project · primary", items: [{ id: "work:1", label: "P10 UIAI evaluation" }, { id: "work:2", label: "Release evidence" }] });
  const queue = contribution("follow-up-queue", "follow_up_queue", "Follow-up Queue", { items: [{ id: "follow:1", label: "Review responsive evidence" }] });
  const editor = contribution("prompt-editor", "prompt_editor", "Prompt Editor", { draft: "" });
  let eligible = [primary, inspector, rail, queue, editor];
  if (scenario === "empty-optionals") eligible = [primary, editor];
  if (scenario === "single-queue") eligible = [primary, queue, editor];
  if (scenario === "zero-queues") eligible = [primary, editor];
  const ids = eligible.map((item) => item.contribution_id);
  const children = eligible.filter((item) => item.kind !== "inspector").map((item, index) => ({ kind: "single", node_id: `layout:item:${index}`, contribution_id: item.contribution_id }));
  const base = children.length === 1 ? children[0] : { kind: "grid", node_id: "layout:grid", columns: scenario === "single-queue" ? 1 : 2, children };
  const layout = eligible.some((item) => item.kind === "inspector")
    ? { kind: "inspector", node_id: "layout:inspector", side: "end", primary: base, inspector_contribution_ids: ["contribution:focusa-inspector"], span: 3 }
    : base;
  return {
    schema: "focusa.resolved_workspace_projection.v1",
    scope,
    workspace_profile_id: "software",
    workspace_profile_revision: 1,
    activity_mode_id: "overview",
    activity_mode_revision: 1,
    focused_work_surface_id: "focused_work_surface:pi-session",
    canonical_read_model_revision: 1,
    candidate_contribution_ids: ["contribution:pi-session", "contribution:focusa-inspector", "contribution:work-rail", "contribution:follow-up-queue", "contribution:prompt-editor"],
    eligible_contributions: eligible,
    omission_diagnostics: ["focusa-inspector", "work-rail", "follow-up-queue"].filter((id) => !ids.includes(`contribution:${id}`)).map((id) => ({ contribution_id: `contribution:${id}`, reason: "no_relevant_content", rule_revision: "adaptive-composition:v1", projection_revision: 1, canonical_input_refs: [], details_ref: null, observed_at: "2026-07-31T00:00:00Z" })),
    layout_tree: layout,
    operation_bindings: [],
    focused_semantic_target: "semantic:pi-session",
    projection_revision: eventSequence,
    layout_revision: eventSequence,
    durable_event_cursor: `event:${eventSequence}`,
    projection_digest: `sha256:${"1".repeat(64)}`,
    resolved_at: "2026-07-31T00:00:00Z",
    evidence_refs: [`evidence:uiai:${scenario}`],
    receipt_refs: [`receipt:uiai:${scenario}`],
  };
}

const server = createServer(async (request, response) => {
  const url = new URL(request.url || "/", "http://127.0.0.1");
  if (url.pathname === "/__fixture/reset" && request.method === "POST") {
    const chunks = [];
    for await (const chunk of request) chunks.push(chunk);
    scenario = JSON.parse(Buffer.concat(chunks).toString() || "{}").scenario || "populated";
    eventSequence += 1;
    return json(response, { scenario, projection: projection(), evidence_ref: `evidence:uiai:${scenario}`, receipt_ref: `receipt:uiai:${scenario}` });
  }
  if (url.pathname === "/__fixture/state") return json(response, { scenario, projection: projection() });
  if (url.pathname === "/v1/mission-canvas/projection") return json(response, projection());
  if (url.pathname === "/v1/mission-canvas/profiles") return json(response, [{ document_id: "profile:software", payload: { profile_id: "software", display_name: "Software Engineering", candidate_contribution_ids: projection().candidate_contribution_ids } }]);
  if (url.pathname === "/v1/mission-canvas/activities") return json(response, [{ document_id: "activity:overview", payload: { activity_mode_id: "overview", display_name: "Overview", candidate_contribution_ids: projection().candidate_contribution_ids } }]);
  if (url.pathname === "/v1/mission-canvas/events") return json(response, { events: [] });
  if (url.pathname.startsWith("/v1/mission-canvas/")) return json(response, { accepted: true, projection: projection(), projection_revision: eventSequence, layout_revision: eventSequence });
  const relative = url.pathname === "/" ? "assets/index.html" : url.pathname.replace(/^\//, "");
  if (!/^(assets\/)?(index\.html|main\.js|a2ui-runtime\.js|styles\.css)$/.test(relative)) return response.writeHead(404).end("Not found");
  const path = join(root, relative.startsWith("assets/") ? relative : `assets/${relative}`);
  let body = await readFile(path);
  if (path.endsWith("index.html")) {
    const bootstrap = JSON.stringify({ daemon_base_url: `http://127.0.0.1:${server.address().port}/v1`, token: null, scope }).replaceAll("<", "\\u003c");
    body = Buffer.from(String(body).replace("<script type=\"module\"", `<script>globalThis.__FOCUSA_RICH_HOST__=${bootstrap}</script><script type=\"module\"`));
  }
  const contentType = extname(path) === ".js" ? "text/javascript" : extname(path) === ".css" ? "text/css" : "text/html";
  response.writeHead(200, { "content-type": `${contentType}; charset=utf-8`, "cache-control": "no-store" });
  response.end(body);
});

function json(response, value) {
  response.writeHead(200, { "content-type": "application/json", "access-control-allow-origin": "*" });
  response.end(JSON.stringify(value));
}

server.listen(Number(process.env.PORT || 0), "127.0.0.1", () => {
  const address = server.address();
  process.stdout.write(`${JSON.stringify({ schema: "focusa.mission_canvas.uiai_harness_ready.v1", url: `http://127.0.0.1:${address.port}/`, reset_url: `http://127.0.0.1:${address.port}/__fixture/reset` })}\n`);
});
for (const signal of ["SIGINT", "SIGTERM"]) process.on(signal, () => server.close(() => process.exit(0)));
