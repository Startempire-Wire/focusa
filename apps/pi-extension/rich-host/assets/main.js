const bootstrap = globalThis.__FOCUSA_RICH_HOST__;
const elements = {
  layout: document.querySelector("#layout"),
  surfaces: document.querySelector("#work-surfaces"),
  connection: document.querySelector("#connection"),
  scope: document.querySelector("#scope"),
  freshness: document.querySelector("#freshness"),
  profile: document.querySelector("#profile"),
  activities: document.querySelector("#activities"),
  dialog: document.querySelector("#workspace-dialog"),
  dialogTitle: document.querySelector("#dialog-title"),
  dialogContent: document.querySelector("#dialog-content"),
  notifications: document.querySelector("#notifications"),
  template: document.querySelector("#contribution-template"),
};
const state = { projection: null, profiles: [], activities: [], abort: null, draggedContributionId: null, scrollPositions: new Map(), piEvents: [], lastEventSequence: 0 };
const customRenderers = new Map();
globalThis.FocusaMissionCanvas = Object.freeze({
  registerRenderer(rendererBindingId, renderer) {
    if (typeof rendererBindingId !== "string" || !rendererBindingId.startsWith("renderer:custom:")) throw new Error("Custom renderer binding must use renderer:custom namespace");
    if (typeof renderer !== "function") throw new Error("Custom renderer must be a function");
    customRenderers.set(rendererBindingId, renderer);
  },
});

function exactScopeQuery(scope = bootstrap?.scope) {
  const query = new URLSearchParams();
  for (const key of ["project_root", "continuity_id", "instance_id", "session_id", "attachment_id", "working_subpath_id"]) {
    if (scope?.[key] != null) query.set(key, String(scope[key]));
  }
  return query;
}

async function request(path, { method = "GET", permission = "mission_canvas:read", body, signal } = {}) {
  const base = bootstrap.daemon_base_url.replace(/\/$/, "");
  const url = new URL(`${base}${path}`);
  if (method === "GET") for (const [key, value] of exactScopeQuery()) url.searchParams.set(key, value);
  const response = await fetch(url, {
    method,
    signal,
    headers: {
      "content-type": "application/json",
      "x-focusa-permissions": permission,
      ...(bootstrap.token ? { authorization: `Bearer ${bootstrap.token}` } : {}),
    },
    body: body === undefined ? undefined : JSON.stringify(body),
  });
  if (!response.ok) throw new Error(`${method} ${path} failed: ${response.status} ${await response.text()}`);
  return response.json();
}

const OPERATION_PATHS = {
  "focusa.agent_execution.prompt": ["POST", "/work-loop/driver/prompt", "work-loop:write"],
  "focusa.agent_execution.abort": ["POST", "/work-loop/driver/abort", "work-loop:write"],
  "focusa.agent_execution.stop": ["POST", "/work-loop/driver/stop", "work-loop:write"],
  "focusa.trajectory.propose_workpoint": ["POST", "/trajectory/propose-workpoint", "trajectory:write"],
};

async function invokeOperation(operationId, payload = {}) {
  if (!operationId) return notify("No canonical delivery operation is available", "warning");
  const descriptor = OPERATION_PATHS[operationId];
  if (!descriptor) return notify(`Operation is not bound in this host: ${operationId}`, "warning");
  const [method, path, permission] = descriptor;
  try {
    await request(path, { method, permission, body: payload });
    notify(`Completed: ${operationId}`);
  } catch (error) {
    notify(String(error), "error", 10000);
  }
}

function assertScope(observed) {
  for (const key of ["project_root", "continuity_id", "session_id", "attachment_id"]) {
    if (observed?.[key] !== bootstrap.scope[key]) throw new Error(`Scope mismatch: ${key}`);
  }
}

function notify(message, level = "info", timeout = 5000) {
  const item = document.createElement("div");
  item.className = `notification ${level}`;
  item.setAttribute("role", level === "error" ? "alert" : "status");
  item.textContent = message;
  elements.notifications.prepend(item);
  while (elements.notifications.children.length > 3) elements.notifications.lastElementChild.remove();
  setTimeout(() => item.remove(), timeout);
}

function contributionById(id) {
  return state.projection?.eligible_contributions?.find((item) => item.contribution_id === id);
}

function renderContribution(id) {
  const contribution = contributionById(id);
  if (!contribution) return null;
  const fragment = elements.template.content.cloneNode(true);
  const article = fragment.querySelector("article");
  article.dataset.contributionId = contribution.contribution_id;
  article.dataset.focusSemanticId = contribution.accessibility?.focus_semantic_id || contribution.semantic_binding_id;
  article.classList.add(contribution.kind);
  article.setAttribute("aria-label", contribution.accessibility?.label || contribution.contribution_id);
  article.querySelector("h2").textContent = contribution.accessibility?.label || contribution.contribution_id;
  article.querySelector(".freshness").textContent = contribution.freshness?.status || "unknown";
  renderContributionContent(article.querySelector(".content"), contribution);
  renderOperations(article.querySelector(".operations"), contribution);
  article.addEventListener("dragstart", () => { state.draggedContributionId = contribution.contribution_id; });
  article.addEventListener("dragover", (event) => { event.preventDefault(); article.classList.add("drop-target"); });
  article.addEventListener("dragleave", () => article.classList.remove("drop-target"));
  article.addEventListener("drop", async (event) => {
    event.preventDefault();
    article.classList.remove("drop-target");
    if (state.draggedContributionId && state.draggedContributionId !== contribution.contribution_id) {
      await mutateLayout("reorder", state.draggedContributionId, contribution.contribution_id);
    }
  });
  return article;
}

function renderContributionContent(container, contribution) {
  const data = contribution.data_ref || {};
  const customRenderer = customRenderers.get(contribution.renderer_binding_id);
  if (customRenderer) {
    const rendered = customRenderer({ contribution, data, scope: bootstrap.scope });
    if (!(rendered instanceof Node)) throw new Error(`Custom renderer did not return a DOM Node: ${contribution.renderer_binding_id}`);
    container.append(rendered);
    return;
  }
  if (contribution.kind === "generated_surface" || contribution.renderer_binding_id?.includes("a2ui")) {
    const surface = document.createElement("focusa-generated-surface");
    surface.allowedActions = (state.projection.operation_bindings || [])
      .filter((binding) => binding.enabled)
      .map((binding) => binding.operation_id);
    const messages = data.a2ui_messages || data.messages || [];
    if (messages.length) surface.snapshot = messages;
    else {
      const recovery = document.createElement("button");
      recovery.type = "button";
      recovery.textContent = data.progress ? `Generating… ${data.progress}%` : "Retry generated surface";
      recovery.addEventListener("click", () => invokeOperation(data.recovery_operation_id, { scope: bootstrap.scope, contribution_id: contribution.contribution_id }));
      surface.append(recovery);
    }
    surface.addEventListener("focusa-operation", (event) => {
      const action = event.detail || {};
      void invokeOperation(action.name || action.operation_id, { ...action, scope: bootstrap.scope });
    });
    container.append(surface);
    return;
  }
  if (contribution.renderer_binding_id?.includes("uiai") || contribution.renderer_binding_id?.includes("browser")) {
    renderBrowserSurface(container, contribution, data);
    return;
  }
  if (contribution.kind === "prompt_editor") {
    const editor = document.createElement("textarea");
    editor.setAttribute("aria-label", "Prompt editor draft");
    editor.placeholder = "Steer the active recipient…";
    editor.value = data.draft || "";
    editor.addEventListener("input", debounce(() => syncDraft(editor.value, contribution), 250));
    editor.addEventListener("keydown", async (event) => {
      if ((event.ctrlKey || event.metaKey) && event.key === "Enter") {
        event.preventDefault();
        await invokeOperation("focusa.agent_execution.prompt", { text: editor.value, scope: bootstrap.scope });
      }
    });
    container.append(editor);
    return;
  }
  if (contribution.renderer_binding_id?.includes("pi-session")) {
    const transcript = document.createElement("ol");
    transcript.className = "transcript";
    const transcriptItems = data.messages || data.transcript || state.piEvents;
    for (const message of virtualWindow(transcriptItems, 200)) {
      const row = document.createElement("li");
      row.className = `transcript-${message.role || "event"}`;
      const role = document.createElement("strong");
      role.textContent = message.role || message.kind || "event";
      const body = document.createElement("pre");
      body.textContent = typeof message.content === "string" ? message.content : JSON.stringify(message.content ?? message, null, 2);
      row.append(role, body);
      transcript.append(row);
    }
    if (!transcript.children.length) transcript.textContent = "Live Pi session connected; no transcript events yet.";
    container.append(transcript);
    return;
  }
  if (["work_rail", "steering_queue", "follow_up_queue"].includes(contribution.kind)) {
    const list = document.createElement("ul");
    for (const item of virtualWindow(data.items || [], 200)) {
      const row = document.createElement("li");
      const label = document.createElement("button");
      label.type = "button";
      label.textContent = item.label || item.title || item.work_item_id || item.id;
      label.title = item.scope_label || data.scope_label || "Current scope";
      label.addEventListener("click", () => invokeOperation(item.operation_id || data.delivery_operation_id, { ...item, scope: bootstrap.scope }));
      row.append(label);
      list.append(row);
    }
    if (!list.children.length) list.className = "empty";
    container.append(list);
    return;
  }
  if (Array.isArray(data.sections) || Array.isArray(data.items) || Array.isArray(data.sources)) {
    const list = document.createElement("ul");
    for (const item of virtualWindow(data.sections || data.items || data.sources, 200)) {
      const row = document.createElement("li");
      row.textContent = item.title || item.label || item.name || item.ref || JSON.stringify(item);
      list.append(row);
    }
    container.append(list);
    return;
  }
  const payload = document.createElement("pre");
  payload.textContent = typeof data.content === "string" ? data.content : JSON.stringify(data, null, 2);
  container.append(payload);
}

function renderBrowserSurface(container, contribution, data) {
  const status = document.createElement("p");
  status.textContent = `UIAI session: ${data.session_id || "unbound"} · ${data.status || "unknown"}`;
  container.append(status);
  if (typeof data.screenshot_url === "string" && /^(blob:|data:image\/|https?:\/\/127\.0\.0\.1[:/])/.test(data.screenshot_url)) {
    const image = document.createElement("img");
    image.src = data.screenshot_url;
    image.alt = data.screenshot_alt || "UIAI browser viewport";
    image.loading = "lazy";
    container.append(image);
  }
  for (const [label, value] of [["Snapshot", data.snapshot], ["Diagnostics", data.diagnostics], ["Artifacts", data.artifacts]]) {
    if (value == null) continue;
    const details = document.createElement("details");
    const summary = document.createElement("summary");
    summary.textContent = label;
    const pre = document.createElement("pre");
    pre.textContent = JSON.stringify(value, null, 2);
    details.append(summary, pre);
    container.append(details);
  }
  const boundary = document.createElement("p");
  boundary.textContent = "Browser execution remains owned by UIAI Engine Cockpit; this surface renders governed evidence and operation bindings only.";
  boundary.className = "product-boundary";
  container.append(boundary);
  container.dataset.uiaiSessionId = data.session_id || "";
  container.dataset.rendererBindingId = contribution.renderer_binding_id;
}

function renderOperations(container, contribution) {
  const bindings = (state.projection.operation_bindings || []).filter((binding) => binding.target_contribution_id === contribution.contribution_id && binding.enabled);
  for (const binding of bindings) {
    const button = document.createElement("button");
    button.type = "button";
    button.textContent = binding.operation_id.split(".").at(-1).replaceAll("_", " ");
    button.dataset.operationId = binding.operation_id;
    button.addEventListener("click", () => invokeOperation(binding.operation_id, {
      scope: bootstrap.scope,
      target_contribution_id: contribution.contribution_id,
      idempotency_key: `operation:${crypto.randomUUID()}`,
    }));
    container.append(button);
  }
  if (!container.children.length) container.remove();
}

function renderLayoutNode(node) {
  if (!node || typeof node !== "object") return null;
  if (node.kind === "single") {
    const contribution = renderContribution(node.contribution_id);
    if (!contribution) return null;
    const container = layoutContainer("single", node.node_id);
    container.append(contribution);
    return container;
  }
  if (node.kind === "tabs") return renderTabs(node);
  if (node.kind === "inspector") return renderInspector(node);
  const children = (node.children || []).map(renderLayoutNode).filter(Boolean);
  if (!children.length) return null;
  const kind = ["split", "stack", "grid"].includes(node.kind) ? node.kind : "stack";
  const container = layoutContainer(kind, node.node_id);
  if (kind === "split") {
    container.classList.add(node.orientation === "horizontal" ? "horizontal" : "vertical");
    container.style.setProperty("--ratio", `${Math.max(0.1, Math.min(0.9, Number(node.ratio) || 0.67))}fr`);
  }
  if (kind === "grid") container.style.setProperty("--columns", String(Math.max(1, Math.min(12, Number(node.columns) || 1))));
  container.append(...children);
  return container;
}

function renderTabs(node) {
  const ids = (node.contribution_ids || []).filter((id) => contributionById(id));
  if (!ids.length) return null;
  const active = ids.includes(node.active_contribution_id) ? node.active_contribution_id : ids[0];
  const container = layoutContainer("tabs", node.node_id);
  const tablist = document.createElement("div");
  tablist.setAttribute("role", "tablist");
  for (const id of ids) {
    const tab = document.createElement("button");
    tab.type = "button";
    tab.setAttribute("role", "tab");
    tab.setAttribute("aria-selected", String(id === active));
    tab.textContent = contributionById(id).accessibility?.label || id;
    tab.addEventListener("click", () => mutateLayout("set_active_tab", id, node.node_id));
    tablist.append(tab);
  }
  const activeContribution = renderContribution(active);
  if (!activeContribution) return null;
  container.append(tablist, activeContribution);
  return container;
}

function renderInspector(node) {
  const primary = renderLayoutNode(node.primary);
  const inspectors = (node.inspector_contribution_ids || []).map(renderContribution).filter(Boolean);
  if (!primary) return inspectors.length ? wrapStack(inspectors, `${node.node_id}:fallback`) : null;
  if (!inspectors.length) return primary;
  const container = layoutContainer("inspector", node.node_id);
  if (node.side === "start") container.classList.add("start");
  const inspectorStack = wrapStack(inspectors, `${node.node_id}:sections`);
  if (node.side === "start") container.append(inspectorStack, primary);
  else container.append(primary, inspectorStack);
  return container;
}

function layoutContainer(kind, id) {
  const container = document.createElement("section");
  container.className = `layout-${kind}`;
  container.dataset.layoutNodeId = id || `layout:${kind}`;
  return container;
}

function wrapStack(children, id) {
  const stack = layoutContainer("stack", id);
  stack.append(...children);
  return stack;
}

function renderProjection(projection) {
  assertScope(projection.scope);
  if (state.projection && projection.projection_revision < state.projection.projection_revision) throw new Error("Projection revision regressed");
  const previousFocus = document.activeElement?.dataset?.focusSemanticId || state.projection?.focused_semantic_target;
  for (const surface of elements.layout.querySelectorAll("[data-contribution-id]")) {
    state.scrollPositions.set(surface.dataset.contributionId, surface.scrollTop);
  }
  state.projection = projection;
  elements.scope.textContent = `${projection.workspace_profile_id} · ${projection.activity_mode_id} · ${projection.scope.attachment_id}`;
  elements.freshness.textContent = `r${projection.projection_revision} / layout ${projection.layout_revision}`;
  renderSurfaceStrip();
  elements.layout.classList.add("recomposing");
  const rendered = renderLayoutNode(projection.layout_tree);
  elements.layout.replaceChildren(...(rendered ? [rendered] : []));
  requestAnimationFrame(() => elements.layout.classList.remove("recomposing"));
  for (const surface of elements.layout.querySelectorAll("[data-contribution-id]")) {
    surface.scrollTop = state.scrollPositions.get(surface.dataset.contributionId) || 0;
  }
  if (previousFocus) elements.layout.querySelector(`[data-focus-semantic-id="${CSS.escape(previousFocus)}"]`)?.focus({ preventScroll: true });
  elements.connection.textContent = "Connected";
}

function renderSurfaceStrip() {
  elements.surfaces.replaceChildren();
  const surfaces = virtualWindow(
    (state.projection.eligible_contributions || []).filter((item) => ["focused_work_surface", "generated_surface", "work_surface_strip"].includes(item.kind)),
    100
  );
  for (const contribution of surfaces) {
    const button = document.createElement("button");
    button.type = "button";
    button.textContent = contribution.accessibility?.label || contribution.contribution_id;
    button.setAttribute("aria-selected", String(contribution.data_ref?.ref === state.projection.focused_work_surface_id));
    button.addEventListener("click", () => document.querySelector(`[data-contribution-id="${CSS.escape(contribution.contribution_id)}"]`)?.focus());
    elements.surfaces.append(button);
  }
  elements.surfaces.classList.toggle("empty", surfaces.length === 0);
}

async function loadNavigation() {
  const [profiles, activities] = await Promise.all([
    request("/mission-canvas/profiles"),
    request("/mission-canvas/activities"),
  ]);
  state.profiles = profiles.filter((item) => item.payload?.candidate_contribution_ids?.length !== 0);
  state.activities = activities.filter((item) => item.payload?.candidate_contribution_ids?.length !== 0);
  elements.profile.replaceChildren();
  for (const item of state.profiles) {
    const option = document.createElement("option");
    option.value = item.payload?.profile_id || item.document_id;
    option.textContent = item.payload?.display_name || option.value;
    option.selected = option.value === state.projection?.workspace_profile_id;
    elements.profile.append(option);
  }
  elements.profile.disabled = !state.profiles.length;
  elements.activities.replaceChildren();
  for (const item of state.activities) {
    const button = document.createElement("button");
    button.type = "button";
    button.textContent = item.payload?.display_name || item.document_id;
    button.setAttribute("aria-pressed", String(item.payload?.activity_mode_id === state.projection?.activity_mode_id));
    button.addEventListener("click", () => selectComposition("activities", item.payload?.activity_mode_id || item.document_id));
    elements.activities.append(button);
  }
}

async function selectComposition(kind, selectionId) {
  const response = await request(`/mission-canvas/${kind}/select`, {
    method: "POST",
    permission: "mission_canvas:write",
    body: {
      scope: bootstrap.scope,
      selection_id: selectionId,
      expected_projection_revision: state.projection.projection_revision,
      idempotency_key: `${kind}:${selectionId}:${crypto.randomUUID()}`,
    },
  });
  renderProjection(response.projection);
  await loadNavigation();
}

async function mutateLayout(action, targetContributionId, secondaryRef) {
  const binding = state.projection.operation_bindings?.find((item) => item.operation_id === "focusa.mission_canvas.layout.mutate" && item.enabled);
  if (!binding) return notify("Layout mutation is unavailable for the current authority/capability scope", "warning");
  const result = await request("/mission-canvas/layout/mutations", {
    method: "POST",
    permission: "mission_canvas:write",
    body: {
      command_id: `layout-command:${crypto.randomUUID()}`,
      scope: bootstrap.scope,
      action,
      attachment_id: bootstrap.scope.attachment_id,
      target_contribution_id: targetContributionId,
      secondary_work_surface_id: secondaryRef,
      expected_projection_revision: state.projection.projection_revision,
      expected_layout_revision: state.projection.layout_revision,
      idempotency_key: `layout:${crypto.randomUUID()}`,
    },
  });
  notify(`Layout ${action} accepted at revision ${result.layout_revision}`);
  await refreshProjection();
}

async function syncDraft(content, contribution) {
  await request("/mission-canvas/drafts/sync", {
    method: "POST",
    permission: "mission_canvas:draft",
    body: {
      scope: bootstrap.scope,
      document_id: `draft:${bootstrap.scope.attachment_id}`,
      revision: Number(contribution.data_ref?.revision || 0) + 1,
      expected_revision: contribution.data_ref?.revision ?? null,
      payload: { content, recipient_ref: state.projection.focused_work_surface_id, owner: "canvas_prompt_editor" },
      idempotency_key: `draft:${crypto.randomUUID()}`,
    },
  });
}

function showDialog(kind) {
  elements.dialogTitle.textContent = kind === "controls" ? "Controls and diagnostics" : kind === "add" ? "Add Surface" : "Manage Workspace";
  elements.dialogContent.replaceChildren();
  if (kind === "controls") {
    const pre = document.createElement("pre");
    pre.textContent = JSON.stringify({
      projection_revision: state.projection.projection_revision,
      layout_revision: state.projection.layout_revision,
      projection_digest: state.projection.projection_digest,
      omissions: state.projection.omission_diagnostics,
      operations: state.projection.operation_bindings,
    }, null, 2);
    elements.dialogContent.append(pre);
  } else {
    const candidates = kind === "add" ? state.projection.omission_diagnostics || [] : state.projection.eligible_contributions || [];
    for (const candidate of candidates) {
      const row = document.createElement("section");
      const label = document.createElement("strong");
      label.textContent = candidate.accessibility?.label || candidate.contribution_id;
      row.append(label);
      const actions = kind === "add" ? ["rehydrate"] : ["focus", "pin", "unpin", "group", "ungroup", "split_horizontal", "split_vertical", "compare", "suspend_projection", "close_projection"];
      for (const action of actions) {
        const button = document.createElement("button");
        button.type = "button";
        button.textContent = action.replaceAll("_", " ");
        button.title = candidate.reason || action;
        button.addEventListener("click", () => mutateLayout(action, candidate.contribution_id, state.projection.focused_work_surface_id || ""));
        row.append(button);
      }
      elements.dialogContent.append(row);
    }
    if (!candidates.length) elements.dialogContent.textContent = "No meaningful surfaces are available.";
  }
  elements.dialog.showModal();
}

async function refreshProjection() {
  state.abort?.abort();
  state.abort = new AbortController();
  const projection = await request("/mission-canvas/projection", { signal: state.abort.signal });
  renderProjection(projection);
}

async function pollEvents() {
  try {
    const response = await request("/mission-canvas/events");
    const unseen = (response.events || []).filter(([sequence]) => sequence > state.lastEventSequence);
    for (const [sequence, event] of unseen) {
      state.lastEventSequence = Math.max(state.lastEventSequence, sequence);
      if (event.event_kind?.startsWith("pi_")) {
        state.piEvents.push({ role: event.event_kind, content: event.payload });
        if (state.piEvents.length > 500) state.piEvents.splice(0, state.piEvents.length - 500);
      }
    }
    const latest = response.events?.at(-1)?.[1];
    if (latest && latest.projection_revision > state.projection.projection_revision) await refreshProjection();
    else if (unseen.some(([, event]) => event.event_kind?.startsWith("pi_"))) renderProjection(state.projection);
  } catch (error) {
    elements.connection.textContent = "Reconnecting";
    notify(String(error), "warning", 3000);
  } finally {
    setTimeout(pollEvents, 2000);
  }
}

function virtualWindow(items, limit) {
  if (!Array.isArray(items) || items.length <= limit) return Array.isArray(items) ? items : [];
  return items.slice(items.length - limit);
}

function debounce(fn, delay) {
  let timer;
  return (...args) => { clearTimeout(timer); timer = setTimeout(() => fn(...args).catch((error) => notify(String(error), "error")), delay); };
}

function bindInteractions() {
  document.querySelector("#add-surface").addEventListener("click", () => showDialog("add"));
  document.querySelector("#manage-workspace").addEventListener("click", () => showDialog("manage"));
  document.querySelector("#controls").addEventListener("click", () => showDialog("controls"));
  elements.profile.addEventListener("change", () => selectComposition("profiles", elements.profile.value).catch((error) => notify(String(error), "error")));
  document.addEventListener("keydown", (event) => {
    if (event.key === "Escape" && elements.dialog.open) elements.dialog.close();
    if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "k") { event.preventDefault(); elements.profile.focus(); }
    if ((event.ctrlKey || event.metaKey) && event.shiftKey && event.key.toLowerCase() === "a") { event.preventDefault(); showDialog("add"); }
    if (event.altKey && ["ArrowLeft", "ArrowRight", "ArrowUp", "ArrowDown"].includes(event.key)) {
      const focusables = [...elements.layout.querySelectorAll("[data-contribution-id]")];
      const current = Math.max(0, focusables.indexOf(document.activeElement));
      const direction = ["ArrowLeft", "ArrowUp"].includes(event.key) ? -1 : 1;
      focusables[(current + direction + focusables.length) % focusables.length]?.focus();
    }
  });
}

async function start() {
  if (!bootstrap?.daemon_base_url || !bootstrap?.scope) throw new Error("Secure host handshake required");
  bindInteractions();
  await refreshProjection();
  await loadNavigation();
  void pollEvents();
}

window.addEventListener("error", (event) => notify(event.error?.message || event.message, "error", 10000));
window.addEventListener("unhandledrejection", (event) => notify(String(event.reason), "error", 10000));
start().catch((error) => {
  elements.connection.textContent = "Unavailable";
  notify(String(error), "error", 15000);
});
