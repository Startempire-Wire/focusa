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

const scope = {
  project_root: "/example/focusa",
  continuity_id: "focusa-cont-c1-generated-ui",
  attachment_id: "attachment:c1-context",
};
const operationId = "focusa.context.source.ingest";
const healthOperationId = "focusa.context.adapter.docling.health";
const binding = actionBindings.bindings.find((candidate) => candidate.action_id === operationId);
const healthBinding = actionBindings.bindings.find((candidate) => candidate.action_id === healthOperationId);
if (!binding || !binding.control.idempotency_required || !binding.control.receipt_required || !healthBinding) {
  throw new Error("generated Context ingestion or Docling health binding is unavailable");
}

const client = createFocusaSpec135Client({ baseUrl: window.location.origin });
const status = document.querySelector<HTMLElement>("#ingest-result")!;
const surface = document.querySelector<HTMLElement>("#context-ingest-surface")!;
const observedActions: A2uiClientAction[] = [];
const responses: Array<components["schemas"]["focusa_context_source_ingest_result_v1"]> = [];
let lastError: unknown;
let adapterHealth: components["schemas"]["focusa_context_adapter_health_v1"] | undefined;

const renderer = new FocusaA2uiRenderer({
  allowedActionNames: new Set([binding.action_id]),
  async onAction(action) {
    observedActions.push(action);
    status.textContent = "Checking Docling and canonical source scope…";
    renderer.processDelta(progressDelta("Checking adapters", 10));

    const health = await client.GET("/v1/context/adapters/docling/health", { params: { query: scope } });
    adapterHealth = health.data;
    if (health.error || !health.data || health.data.status !== "healthy") {
      lastError = health.error ?? health.data;
      showRecovery(health.data?.recovery_action ?? "Start Docling Serve v1 and retry safely.");
      return;
    }

    const query = { ...scope };
    const readVersion = async () => client.GET("/v1/context/sources", { params: { query } });
    const listed = await readVersion();
    if (listed.error || !listed.data) {
      lastError = listed.error;
      showRecovery("Could not read canonical Context state before ingestion.");
      return;
    }
    let version = listed.data.state_version;

    const ingestOne = async (body: Omit<IngestBody, "expected_state_version">) => {
      const result = await client.POST("/v1/context/sources/ingest", {
        params: { query },
        body: { ...body, expected_state_version: version },
      });
      if (result.error || !result.data) throw result.error ?? new Error("Context ingestion returned no data");
      version = result.data.state_version;
      responses.push(result.data);
      return result.data;
    };

    try {
      status.textContent = "Importing Markdown…";
      renderer.processDelta(progressDelta("Importing Markdown", 30));
      await ingestOne({
        ...scope,
        idempotency_key: "c1-ui-markdown-r1",
        source_kind: "markdown",
        source_locator: "README.md",
        source_revision: "git:c1-markdown-r1",
        title: "Mission Canvas Markdown",
        mime_type: "text/markdown",
        content: "# Mission Canvas\n\nReal Markdown imported through generated UI.",
      });

      status.textContent = "Importing code…";
      renderer.processDelta(progressDelta("Importing code", 55));
      await ingestOne({
        ...scope,
        idempotency_key: "c1-ui-code-r1",
        source_kind: "code",
        source_locator: "src/mission.ts",
        source_revision: "git:c1-code-r1",
        title: "Mission source code",
        mime_type: "text/typescript",
        content: "export const mission = 'operational';",
      });

      status.textContent = "Extracting PDF with Docling…";
      renderer.processDelta(progressDelta("Extracting PDF with Docling", 75));
      await ingestOne({
        ...scope,
        idempotency_key: "c1-ui-pdf-r1",
        source_kind: "pdf",
        source_locator: "mission-context.pdf",
        source_revision: "sha256:c1-pdf-r1",
        title: "Mission PDF Context",
        mime_type: "application/pdf",
        content_base64: bytesToBase64(minimalPdf("Mission Canvas PDF Context")),
      });
    } catch (error) {
      lastError = error;
      showRecovery("A source import was blocked. Retained sources remain canonical; retry the failed source.");
      return;
    }

    const evidence = responses.map((item) => item.evidence_ref).join(", ");
    const receipts = responses.map((item) => item.receipt_ref).join(", ");
    renderer.processDelta([{
      version: "v0.9",
      updateComponents: {
        surfaceId: "c1-context-ingest",
        components: [
          {
            id: "adapter",
            component: "FocusaSourceConnectorCard",
            label: "Docling Serve v1",
            description: "PDF extraction adapter is healthy.",
            status: "healthy",
            details: `endpoint=${health.data.endpoint ?? "configured"}`,
          },
          {
            id: "progress",
            component: "FocusaProgressStepper",
            label: "Context import complete",
            description: "Markdown, code, and PDF are canonical and source-scoped.",
            status: "completed",
            progress: 100,
          },
          {
            id: "result",
            component: "FocusaEvidenceSummary",
            label: "Three real sources imported",
            description: `Evidence ${evidence}`,
            status: "saved",
            details: `receipts=${receipts}; scope=${JSON.stringify(scope)}; version=${version}`,
          },
        ],
      },
    }]);
    status.textContent = "Markdown, code, and PDF Context imported";
    document.body.dataset.ingestStatus = "completed";
  },
});

function progressDelta(label: string, progress: number): A2uiMessage[] {
  return [{
    version: "v0.9",
    updateComponents: {
      surfaceId: "c1-context-ingest",
      components: [{
        id: "progress",
        component: "FocusaProgressStepper",
        label,
        description: "Source scope and provenance are retained during ingestion.",
        status: "processing",
        progress,
      }],
    },
  }];
}

function showRecovery(description: string) {
  status.textContent = description;
  renderer.processDelta([{
    version: "v0.9",
    updateComponents: {
      surfaceId: "c1-context-ingest",
      components: [{
        id: "result",
        component: "FocusaRecoveryCard",
        label: "Context ingestion needs recovery",
        description,
        status: "retry",
        details: JSON.stringify(lastError),
      }],
    },
  }]);
  document.body.dataset.ingestStatus = "recovery";
}

function bytesToBase64(bytes: Uint8Array): string {
  let binary = "";
  for (const byte of bytes) binary += String.fromCharCode(byte);
  return btoa(binary);
}

function minimalPdf(text: string): Uint8Array {
  const escaped = text.replaceAll("\\", "\\\\").replaceAll("(", "\\(").replaceAll(")", "\\)");
  const stream = `BT /F1 18 Tf 72 720 Td (${escaped}) Tj ET\n`;
  const objects = [
    "<< /Type /Catalog /Pages 2 0 R >>",
    "<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
    "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources << /Font << /F1 5 0 R >> >> /Contents 4 0 R >>",
    `<< /Length ${new TextEncoder().encode(stream).length} >>\nstream\n${stream}endstream`,
    "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>",
  ];
  let output = "%PDF-1.4\n";
  const offsets = [0];
  for (let index = 0; index < objects.length; index += 1) {
    offsets.push(new TextEncoder().encode(output).length);
    output += `${index + 1} 0 obj\n${objects[index]}\nendobj\n`;
  }
  const xref = new TextEncoder().encode(output).length;
  output += `xref\n0 ${objects.length + 1}\n0000000000 65535 f \n`;
  output += offsets.slice(1).map((offset) => `${String(offset).padStart(10, "0")} 00000 n \n`).join("");
  output += `trailer\n<< /Size ${objects.length + 1} /Root 1 0 R >>\nstartxref\n${xref}\n%%EOF\n`;
  return new TextEncoder().encode(output);
}

const snapshot: A2uiMessage[] = [
  { version: "v0.9", createSurface: { surfaceId: "c1-context-ingest", catalogId: FOCUSA_A2UI_CATALOG_ID } },
  {
    version: "v0.9",
    updateComponents: {
      surfaceId: "c1-context-ingest",
      components: [
        { id: "root", component: "Column", children: ["stage", "adapter", "progress", "ingest", "result"] },
        {
          id: "stage",
          component: "FocusaStageShell",
          label: "Build project Context",
          description: "Import real Markdown, code, and PDF while preserving source scope and provenance.",
          status: "ready",
          details: `operation=${binding.action_id}; scope=${JSON.stringify(scope)}`,
        },
        {
          id: "adapter",
          component: "FocusaSourceConnectorCard",
          label: "Docling Serve v1",
          description: "Health will be checked before PDF extraction.",
          status: "checking",
        },
        {
          id: "progress",
          component: "FocusaProgressStepper",
          label: "Sources ready",
          description: "No source has been imported yet.",
          status: "ready",
          progress: 0,
        },
        {
          id: "ingest",
          component: "FocusaPrimaryAction",
          label: "Import project sources",
          description: "Creates canonical events, source health, Evidence, and reversible Receipts.",
          primaryActionLabel: "Import Sources",
          action: { event: { name: binding.action_id, context: scope } },
        },
        {
          id: "result",
          component: "FocusaEvidenceSummary",
          label: "Import proof",
          description: "Evidence and Receipt references will appear here.",
          status: "pending",
        },
      ],
    },
  },
];
renderer.processSnapshot(snapshot);
renderer.mount(surface, "c1-context-ingest");

Object.assign(window, {
  focusaContextIngestEval: {
    renderer,
    binding,
    healthBinding,
    scope,
    observedActions,
    responses,
    get adapterHealth() { return adapterHealth; },
    get lastError() { return lastError; },
  },
});
