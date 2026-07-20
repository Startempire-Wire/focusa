import { createFocusaSpec135Client } from "@focusa/spec135-client";
import actionBindings from "../../../docs/contracts/spec135/generated-contract-v1/ui-action-bindings.fixture.json" with { type: "json" };
import {
  FOCUSA_A2UI_CATALOG_ID,
  FocusaA2uiRenderer,
  type A2uiClientAction,
  type A2uiMessage,
} from "../src/index.js";

const scope = {
  project_root: "/example/focusa",
  continuity_id: "focusa-cont-f12-generated-ui",
  attachment_id: "attachment:f12-context",
};
const operationId = "focusa.context.source.commit";
const binding = actionBindings.bindings.find((candidate) => candidate.action_id === operationId);
if (!binding || !binding.control.idempotency_required || !binding.control.receipt_required) {
  throw new Error("generated Context commit action binding is unavailable or incomplete");
}

const client = createFocusaSpec135Client({ baseUrl: window.location.origin });
const status = document.querySelector<HTMLElement>("#commit-result")!;
const surface = document.querySelector<HTMLElement>("#context-surface")!;
const observedActions: A2uiClientAction[] = [];
let lastResponse: unknown;
let lastError: unknown;

const renderer = new FocusaA2uiRenderer({
  allowedActionNames: new Set([binding.action_id]),
  async onAction(action) {
    observedActions.push(action);
    status.textContent = "Committing canonical Context…";
    const query = { ...scope };
    const readVersion = async () => client.GET("/v1/context/sources", { params: { query } });
    let listed = await readVersion();
    if (listed.error || !listed.data) {
      lastError = listed.error;
      showRecovery("Could not read canonical Context state before commit.");
      return;
    }
    const sendCommit = (expected_state_version: number) => client.POST("/v1/context/sources/commit", {
      params: { query },
      body: {
        ...scope,
        idempotency_key: "f12-generated-ui-alpha-seed-v1",
        expected_state_version,
        source_kind: "markdown",
        title: "Generated UI Alpha Context",
        content: "# Mission Context\n\nThis source was committed through the generated A2UI action binding and typed client.",
      },
    });
    let committed = await sendCommit(listed.data.state_version);
    for (let attempt = 0; !committed.data && committed.response.status === 409 && attempt < 2; attempt += 1) {
      listed = await readVersion();
      if (!listed.data) break;
      committed = await sendCommit(listed.data.state_version);
    }
    if (committed.error || !committed.data) {
      lastError = committed.error;
      showRecovery("The Context commit was blocked. Refresh state and retry safely.");
      return;
    }
    lastResponse = committed.data;
    status.textContent = committed.data.replayed
      ? "Context resumed from the existing idempotent commit"
      : "Context committed to canonical Focusa state";
    renderer.processDelta([{
      version: "v0.9",
      updateComponents: {
        surfaceId: "f12-context-commit",
        components: [{
          id: "result",
          component: "FocusaEvidenceSummary",
          label: committed.data.replayed ? "Context commit resumed" : "Context source committed",
          description: `Evidence ${committed.data.evidence_ref}`,
          status: "saved",
          details: `receipt=${committed.data.receipt_ref}; source=${committed.data.source.source_id}; version=${committed.data.state_version}`,
        }],
      },
    }]);
    document.body.dataset.commitStatus = committed.data.replayed ? "replayed" : "committed";
  },
});

function showRecovery(description: string) {
  status.textContent = description;
  renderer.processDelta([{
    version: "v0.9",
    updateComponents: {
      surfaceId: "f12-context-commit",
      components: [{
        id: "result",
        component: "FocusaRecoveryCard",
        label: "Context commit needs recovery",
        description,
        status: "retry",
        details: JSON.stringify(lastError),
      }],
    },
  }]);
  document.body.dataset.commitStatus = "recovery";
}

const snapshot: A2uiMessage[] = [
  {
    version: "v0.9",
    createSurface: { surfaceId: "f12-context-commit", catalogId: FOCUSA_A2UI_CATALOG_ID },
  },
  {
    version: "v0.9",
    updateComponents: {
      surfaceId: "f12-context-commit",
      components: [
        { id: "root", component: "Column", children: ["stage", "commit", "result"] },
        {
          id: "stage",
          component: "FocusaStageShell",
          label: "What Focusa will know",
          description: "A bounded Markdown source will become canonical project Context.",
          status: "ready",
          details: `operation=${binding.action_id}; scope=${JSON.stringify(scope)}`,
        },
        {
          id: "commit",
          component: "FocusaPrimaryAction",
          label: "Commit project Context",
          description: "Creates reducer state, durable event history, Evidence, and a reversible Receipt.",
          primaryActionLabel: "Commit Context",
          action: { event: { name: binding.action_id, context: scope } },
        },
        {
          id: "result",
          component: "FocusaEvidenceSummary",
          label: "Commit proof",
          description: "Evidence and Receipt references will appear here.",
          status: "pending",
        },
      ],
    },
  },
];
renderer.processSnapshot(snapshot);
renderer.mount(surface, "f12-context-commit");

Object.assign(window, {
  focusaContextEval: {
    renderer,
    binding,
    scope,
    observedActions,
    get lastResponse() { return lastResponse; },
    get lastError() { return lastError; },
  },
});
