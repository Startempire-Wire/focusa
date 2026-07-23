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

type IngestBody = components["schemas"]["focusa_context_source_ingest_request_v1"];
type RetrieveBody = components["schemas"]["focusa_context_retrieve_request_v1"];

const scope = {
  project_root: "/example/focusa",
  continuity_id: "focusa-cont-c2-generated-ui",
  attachment_id: "attachment:c2-context",
};
const operationId = "focusa.context.retrieve";
const binding = actionBindings.bindings.find((candidate) => candidate.action_id === operationId);
if (!binding || !binding.control.receipt_required) {
  throw new Error("Generated Context retrieval action binding is unavailable");
}

const client = createFocusaSpec135Client({ baseUrl: window.location.origin });
const surface = document.querySelector<HTMLElement>("#context-retrieval-surface");
const status = document.querySelector<HTMLElement>("#retrieval-result");
if (!surface || !status) throw new Error("Context retrieval proof mount missing");

const observedActions: A2uiClientAction[] = [];
let response: unknown;
let lastError: unknown;

const renderer = new FocusaA2uiRenderer({
  allowedActionNames: new Set([binding.action_id]),
  async onAction(action) {
    observedActions.push(action);
    status.textContent = "Synchronizing canonical sources and searching…";
    renderer.processDelta(progressDelta("Indexing exact source scope", 30));
    try {
      let listed = await client.GET("/v1/context/sources", { params: { query: scope } });
      if (listed.error || !listed.data) throw listed.error ?? new Error("Context source list unavailable");
      let version = listed.data.state_version;
      const seeds: IngestBody[] = [
        {
          ...scope,
          idempotency_key: "c2-generated-positive",
          expected_state_version: version,
          source_kind: "markdown",
          title: "Release Policy A",
          content: "The release policy requires signed artifacts for every deployment.",
          source_locator: "file:///release-policy-a.md",
          source_revision: "git:c2-generated-ui",
          mime_type: "text/markdown",
        },
        {
          ...scope,
          idempotency_key: "c2-generated-negative",
          expected_state_version: version,
          source_kind: "markdown",
          title: "Release Policy B",
          content: "The release policy does not require signed artifacts for every deployment.",
          source_locator: "file:///release-policy-b.md",
          source_revision: "git:c2-generated-ui",
          mime_type: "text/markdown",
        },
      ];
      for (const seed of seeds) {
        seed.expected_state_version = version;
        const ingested = await client.POST("/v1/context/sources/ingest", {
          params: { query: scope },
          body: seed,
        });
        if (ingested.error || !ingested.data) throw ingested.error ?? new Error("Context seed unavailable");
        version = ingested.data.state_version;
      }

      renderer.processDelta(progressDelta("Running bounded hybrid retrieval", 70));
      const body: RetrieveBody = {
        ...scope,
        query: "release policy signed artifacts deployment",
        limit: 8,
        mode: "hybrid",
        include_contradictions: true,
      };
      const retrieved = await client.POST("/v1/context/retrieve", {
        params: { query: scope },
        body,
      });
      if (retrieved.error || !retrieved.data) throw retrieved.error ?? new Error("Context retrieval unavailable");
      response = retrieved.data;
      const result = retrieved.data.result;
      const citations = result.hits.map((hit) => hit.citation.citation_id).join(", ");
      const contradiction = result.contradictions[0];
      renderer.processDelta([{
        version: "v0.9",
        updateComponents: {
          surfaceId: "c2-context-retrieval",
          components: [
            {
              id: "index",
              component: "FocusaSourceConnectorCard",
              label: "SQLite FTS5 + sqlite-vec",
              description: `${result.indexed_source_count} canonical sources; ${result.indexed_chunk_count} source-preserving chunks.`,
              status: result.capabilities.degraded_to_lexical ? "degraded" : "healthy",
              details: JSON.stringify(result.capabilities),
            },
            {
              id: "progress",
              component: "FocusaProgressStepper",
              label: "Cited retrieval complete",
              description: `${result.result_count} bounded exact-scope results returned via ${result.mode_used}.`,
              status: "completed",
              progress: 100,
            },
            {
              id: "result",
              component: "FocusaEvidenceSummary",
              label: `${result.result_count} cited Context results`,
              description: `Citations ${citations}`,
              status: "saved",
              details: `evidence=${retrieved.data.evidence_ref}; receipt=${retrieved.data.receipt_ref}; scope=${JSON.stringify(scope)}`,
            },
            {
              id: "contradiction",
              component: "FocusaContradictionCard",
              label: contradiction ? "Contradiction candidate surfaced" : "No contradiction candidate",
              description: contradiction?.summary ?? "Retrieved sources agree for this query.",
              status: contradiction ? "review" : "clear",
              details: contradiction ? `${contradiction.contradiction_id}; citations=${contradiction.left_citation_id},${contradiction.right_citation_id}` : "",
            },
          ],
        },
      }]);
      status.textContent = "Cited Context found with contradiction review";
      document.body.dataset.retrievalStatus = "completed";
    } catch (error) {
      lastError = error;
      status.textContent = "Context retrieval needs recovery";
      renderer.processDelta([{
        version: "v0.9",
        updateComponents: {
          surfaceId: "c2-context-retrieval",
          components: [{
            id: "result",
            component: "FocusaRecoveryCard",
            label: "Cited Context retrieval needs recovery",
            description: "Verify canonical source scope and retry safely.",
            status: "retry",
            details: JSON.stringify(error),
          }],
        },
      }]);
      document.body.dataset.retrievalStatus = "recovery";
    }
  },
});

function progressDelta(label: string, progress: number): A2uiMessage[] {
  return [{
    version: "v0.9",
    updateComponents: {
      surfaceId: "c2-context-retrieval",
      components: [{
        id: "progress",
        component: "FocusaProgressStepper",
        label,
        description: "Retrieval remains bounded to project, workstream, attachment, source, and revision.",
        status: "processing",
        progress,
      }],
    },
  }];
}

const snapshot: A2uiMessage[] = [
  { version: "v0.9", createSurface: { surfaceId: "c2-context-retrieval", catalogId: FOCUSA_A2UI_CATALOG_ID } },
  {
    version: "v0.9",
    updateComponents: {
      surfaceId: "c2-context-retrieval",
      components: [
        { id: "root", component: "Column", children: ["stage", "index", "progress", "retrieve", "result", "contradiction"] },
        {
          id: "stage",
          component: "FocusaStageShell",
          label: "Retrieve project Context",
          description: "Search canonical source chunks with source-preserving citations and contradiction review.",
          status: "ready",
          details: `operation=${binding.action_id}; scope=${JSON.stringify(scope)}`,
        },
        { id: "index", component: "FocusaSourceConnectorCard", label: "Retrieval index", description: "Capabilities will be verified on action.", status: "checking" },
        { id: "progress", component: "FocusaProgressStepper", label: "Ready to search", description: "No Context query has run yet.", status: "ready", progress: 0 },
        {
          id: "retrieve",
          component: "FocusaPrimaryAction",
          label: "Find source-grounded Context",
          description: "Returns bounded cited results, Evidence, a Receipt, and contradiction candidates.",
          primaryActionLabel: "Find Cited Context",
          action: { event: { name: binding.action_id, context: scope } },
        },
        { id: "result", component: "FocusaEvidenceSummary", label: "Retrieval proof", description: "Citation, Evidence, and Receipt references will appear here.", status: "pending" },
        { id: "contradiction", component: "FocusaContradictionCard", label: "Contradiction review", description: "Candidate contradictions will appear here without silently becoming canonical truth.", status: "pending" },
      ],
    },
  },
];
renderer.processSnapshot(snapshot);
renderer.mount(surface, "c2-context-retrieval");

Object.assign(window, {
  focusaContextRetrieveEval: {
    renderer,
    binding,
    scope,
    observedActions,
    get response() { return response; },
    get lastError() { return lastError; },
  },
});
