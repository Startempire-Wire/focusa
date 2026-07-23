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

type ArtifactIntakeBody = components["schemas"]["focusa_workspace_artifact_intake_request_v1"];
type ArtifactIntakeResult = components["schemas"]["focusa_workspace_artifact_intake_result_v1"];

const scope = {
  project_root: "/example/focusa",
  continuity_id: "focusa-cont-u1-generated-ui",
  attachment_id: "attachment:u1-workspace-artifact",
};
const operationId = "focusa.workspace.artifact.intake";
const binding = actionBindings.bindings.find(
  (candidate) => candidate.action_id === operationId,
);
if (!binding?.control.receipt_required) {
  throw new Error("Generated Workspace Artifact intake binding is unavailable");
}

const client = createFocusaSpec135Client({ baseUrl: window.location.origin });
const surface = document.querySelector<HTMLElement>("#workspace-artifact-surface");
const status = document.querySelector<HTMLElement>("#artifact-result");
if (!surface || !status) throw new Error("Workspace Artifact proof mount missing");

const observedActions: A2uiClientAction[] = [];
let response: unknown;
let lastError: unknown;

const renderer = new FocusaA2uiRenderer({
  allowedActionNames: new Set([binding.action_id]),
  async onAction(action) {
    observedActions.push(action);
    status.textContent = "Validating UIAI origin, scope, and trust…";
    renderer.processDelta(progressDelta("Validating bounded descriptor", 35));
    try {
      const body: ArtifactIntakeBody = {
        ...scope,
        idempotency_key: "u1-generated-ui-uiai-screenshot",
        expected_state_version: 0,
        artifact_kind: "image",
        mime_type: "image/png",
        title: "UIAI cited Context retrieval proof",
        summary:
          "UIAI Engine screenshot proving generated cited retrieval while the image blob remains externally owned.",
        handle_ref:
          "uiai-screenshot:session=biw4d3UW:1784588806118638000",
        artifact_path:
          "/Volumes/Macintosh HD/Users/vsmith/.local/share/uiai-engine/shares/session-screenshots/biw4d3UW-1784588806118638000.png",
        sha256:
          "82c78c9fb883577bbaef785728abd640d2a9df66c2bcc5165db34a373e1497e0",
        size_bytes: 116930,
        source_system: "uiai",
        source_ref: "uiai-browser:session=biw4d3UW",
        source_url: "http://127.0.0.1:4173/context-retrieve.html",
        project_identity_ref: "project:focusa",
        workpoint_id: "focusa-mc-u1",
        work_item_ref: "focusa-mc-u1",
        instance_id: "focusa-instance:u1-generated-ui",
        work_surface_id: "surface:u1-workspace-artifact",
        uiai_session_id: "biw4d3UW",
        browser_context_id: "browser-context:biw4d3UW",
        browser_target_id: "context-retrieve-proof",
        diagnostics_refs: [
          "browser-diagnostics:2026-07-20T23:07:04.444Z",
        ],
        evidence_refs: [
          "evidence:context-retrieve:a248110aa8107ce7b8fa3c9d",
        ],
        domain_pack_refs: ["domain-pack:software-delivery"],
        candidate_object_refs: ["candidate-object:cited-context-proof"],
        candidate_link_refs: ["candidate-link:artifact-source"],
        candidate_claim_refs: [],
        verification_policy_refs: ["verification-policy:source-cited"],
        semantic_delta_refs: [],
        citation_refs: ["context-citation:a248110aa8107ce7b8fa3c9d"],
        evidence_status: "verified",
        redaction_status: "secret_safe",
        freshness_status: "captured",
        provenance_status: "verified",
        retention_policy: "project_evidence",
        cleanup_action:
          "close the UIAI session independently after proof capture",
        preferred_renderer: "image_preview",
        fallback_renderer: "bounded_metadata_and_handle",
        render_width: 1440,
        render_height: 1000,
      };

      let linked: ArtifactIntakeResult | undefined;
      let linkError: unknown;
      for (let attempt = 0; attempt < 5; attempt += 1) {
        const listed = await client.GET("/v1/workspace/artifacts", {
          params: { query: scope },
        });
        if (listed.error || !listed.data) {
          throw listed.error ?? new Error("Workspace Artifact list unavailable");
        }
        body.expected_state_version = listed.data.state_version;
        const attemptResult = await client.POST(
          "/v1/workspace/artifacts/intake",
          { params: { query: scope }, body },
        );
        linkError = attemptResult.error;
        if (!attemptResult.error && attemptResult.data) {
          linked = attemptResult.data;
          break;
        }
      }
      if (!linked) {
        throw linkError ?? new Error("Workspace Artifact link unavailable");
      }
      response = linked;
      const artifact = linked.artifact;
      renderer.processDelta([
        {
          version: "v0.9",
          updateComponents: {
            surfaceId: "u1-workspace-artifact",
            components: [
              {
                id: "origin",
                component: "FocusaSourceConnectorCard",
                label: "UIAI artifact linked",
                description: `${artifact.title}; source=${artifact.source.system}; session=${artifact.origin.uiai_session_id}`,
                status: "healthy",
                details: `handle=${artifact.content.handle_ref}; blob_authority=external; sha256=${artifact.content.sha256}`,
              },
              {
                id: "progress",
                component: "FocusaProgressStepper",
                label: "Descriptor linked to exact work surface",
                description: `${artifact.scope.continuity_id}; ${artifact.origin.attachment_id}`,
                status: "completed",
                progress: 100,
              },
              {
                id: "evidence",
                component: "FocusaEvidenceSummary",
                label: "Evidence-backed artifact",
                description: artifact.evidence_refs.join(", "),
                status: "saved",
                details: `diagnostics=${artifact.diagnostics_refs.join(",")}; redaction=${artifact.trust.redaction_status}; provenance=${artifact.trust.provenance_status}`,
              },
              {
                id: "receipt",
                component: "FocusaReceiptCard",
                label: "Artifact link Receipt",
                description: linked.receipt_ref,
                status: linked.replayed ? "replayed" : "completed",
                details: `artifact=${artifact.artifact_id}; external_artifact_authority=${linked.external_artifact_authority}`,
              },
              {
                id: "details",
                component: "FocusaAdvancedDetails",
                label: "Runtime ownership boundary",
                description:
                  "Focusa stores the bounded descriptor and refs; UIAI owns browser state and the large image blob.",
                status: "verified",
                details: JSON.stringify({
                  origin: artifact.origin,
                  renderer: artifact.render.preferred_renderer,
                  fallback: artifact.render.fallback_renderer,
                  cleanup: artifact.retention.cleanup_action,
                  citations: artifact.semantic.citation_refs,
                }),
              },
            ],
          },
        },
      ]);
      status.textContent = "UIAI artifact linked with Evidence and Receipt";
      document.body.dataset.artifactStatus = "completed";
    } catch (error) {
      lastError = error;
      status.textContent = "Workspace Artifact needs recovery";
      renderer.processDelta([
        {
          version: "v0.9",
          updateComponents: {
            surfaceId: "u1-workspace-artifact",
            components: [
              {
                id: "details",
                component: "FocusaRecoveryCard",
                label: "Artifact link needs recovery",
                description:
                  "Refresh the exact attachment scope and retry without importing UIAI runtime state.",
                status: "retry",
                details: JSON.stringify(error),
              },
            ],
          },
        },
      ]);
      document.body.dataset.artifactStatus = "recovery";
    }
  },
});

function progressDelta(label: string, progress: number): A2uiMessage[] {
  return [
    {
      version: "v0.9",
      updateComponents: {
        surfaceId: "u1-workspace-artifact",
        components: [
          {
            id: "progress",
            component: "FocusaProgressStepper",
            label,
            description:
              "The descriptor remains project-, workstream-, attachment-, and origin-scoped.",
            status: "processing",
            progress,
          },
        ],
      },
    },
  ];
}

const snapshot: A2uiMessage[] = [
  {
    version: "v0.9",
    createSurface: {
      surfaceId: "u1-workspace-artifact",
      catalogId: FOCUSA_A2UI_CATALOG_ID,
    },
  },
  {
    version: "v0.9",
    updateComponents: {
      surfaceId: "u1-workspace-artifact",
      components: [
        {
          id: "root",
          component: "Column",
          children: [
            "stage",
            "origin",
            "progress",
            "link",
            "evidence",
            "receipt",
            "details",
          ],
        },
        {
          id: "stage",
          component: "FocusaStageShell",
          label: "Attach UIAI output to this work surface",
          description:
            "Retain source, scope, origin, diagnostics, provenance, Evidence, rendering fallback, and cleanup policy.",
          status: "ready",
          details: `operation=${binding.action_id}; scope=${JSON.stringify(scope)}`,
        },
        {
          id: "origin",
          component: "FocusaSourceConnectorCard",
          label: "UIAI Engine",
          description:
            "A bounded screenshot descriptor is ready; browser runtime and blob storage remain external.",
          status: "ready",
        },
        {
          id: "progress",
          component: "FocusaProgressStepper",
          label: "Ready to validate",
          description: "No canonical artifact link has been written yet.",
          status: "ready",
          progress: 0,
        },
        {
          id: "link",
          component: "FocusaPrimaryAction",
          label: "Link cited UIAI artifact",
          description:
            "Writes only a bounded, Evidence-backed descriptor and returns a Receipt.",
          primaryActionLabel: "Link Workspace Artifact",
          action: { event: { name: binding.action_id, context: scope } },
        },
        {
          id: "evidence",
          component: "FocusaEvidenceSummary",
          label: "Artifact Evidence",
          description: "Evidence and diagnostics refs will appear here.",
          status: "pending",
        },
        {
          id: "receipt",
          component: "FocusaReceiptCard",
          label: "Artifact link Receipt",
          description: "No Receipt yet.",
          status: "pending",
        },
        {
          id: "details",
          component: "FocusaAdvancedDetails",
          label: "Ownership and cleanup",
          description:
            "Inspect the renderer fallback and external-runtime cleanup boundary after linking.",
          status: "pending",
        },
      ],
    },
  },
];
renderer.processSnapshot(snapshot);
renderer.mount(surface, "u1-workspace-artifact");

Object.assign(window, {
  focusaWorkspaceArtifactEval: {
    renderer,
    binding,
    scope,
    observedActions,
    get response() {
      return response;
    },
    get lastError() {
      return lastError;
    },
  },
});
