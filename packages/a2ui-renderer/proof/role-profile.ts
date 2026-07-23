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

const scope = {
  project_root: "/example/focusa",
  continuity_id: "focusa-cont-ri1-generated-ui",
  attachment_id: "attachment:ri1-role-profile",
};
const draftOperation = "focusa.role_profile.draft";
const reviewOperation = "focusa.role_profile.review";
const draftBinding = actionBindings.bindings.find(
  (candidate) => candidate.action_id === draftOperation,
);
const reviewBinding = actionBindings.bindings.find(
  (candidate) => candidate.action_id === reviewOperation,
);
if (
  !draftBinding?.control.receipt_required ||
  reviewBinding?.control.confirmation !== "consequential" ||
  !reviewBinding.control.receipt_required
) {
  throw new Error("Generated Role Profile bindings are unavailable or incomplete");
}
const roleDraftBinding = draftBinding!;
const roleReviewBinding = reviewBinding!;

const client = createFocusaSpec135Client({ baseUrl: window.location.origin });
const status = document.querySelector<HTMLElement>("#role-result")!;
const surface = document.querySelector<HTMLElement>("#role-surface")!;
if (!status || !surface) throw new Error("Role Profile proof mount missing");

type RoleMutation =
  components["schemas"]["focusa_project_agent_role_profile_mutation_result_v1"];

const observedActions: A2uiClientAction[] = [];
let lastDraft: RoleMutation | undefined;
let lastReview: RoleMutation | undefined;
let lastError: unknown;

const renderer = new FocusaA2uiRenderer({
  allowedActionNames: new Set([roleDraftBinding.action_id, roleReviewBinding.action_id]),
  async onAction(action) {
    observedActions.push(action);
    try {
      if (action.name === draftOperation) {
        await composeDraft();
      } else if (action.name === reviewOperation) {
        const decision = String(
          (action.context as { decision?: string } | undefined)?.decision ?? "approve",
        );
        await reviewRole(decision);
      }
    } catch (error) {
      showRecovery(error);
    }
  },
});

async function contextState() {
  return client.GET("/v1/context/sources", { params: { query: scope } });
}

async function roleState() {
  return client.GET("/v1/roles/profiles", { params: { query: scope } });
}

async function commitGroundingSource() {
  let listed = await contextState();
  if (!listed.data) throw listed.error ?? new Error("Context state unavailable");
  for (let attempt = 0; attempt < 5; attempt += 1) {
    const committed = await client.POST("/v1/context/sources/commit", {
      params: { query: scope },
      body: {
        ...scope,
        idempotency_key: "ri1-generated-ui-grounding-v1",
        expected_state_version: listed.data.state_version,
        source_kind: "markdown",
        title: "Approved Mission Canvas role grounding",
        content:
          "# Mission\nDeliver Spec 135 through canonical Focusa reducers, generated contracts, Evidence, Receipts, and explicit operator approval.",
      },
    });
    if (committed.data) return committed.data.source.source_id;
    if (committed.response.status !== 409) {
      throw committed.error ?? new Error("Context grounding commit unavailable");
    }
    listed = await contextState();
    if (!listed.data) throw listed.error ?? new Error("Context state unavailable");
  }
  throw new Error("Context grounding writer remained conflicted");
}

async function composeDraft(): Promise<void> {
  status.textContent = "Grounding the Role draft in accepted project Context…";
  const sourceId = await commitGroundingSource();
  let listed = await roleState();
  if (!listed.data) throw listed.error ?? new Error("Role state unavailable");
  for (let attempt = 0; attempt < 5; attempt += 1) {
    const drafted = await client.POST("/v1/roles/profiles/draft", {
      params: { query: scope },
      body: {
        ...scope,
        idempotency_key: "ri1-generated-ui-draft-v1",
        expected_state_version: listed.data.state_version,
        original_seed:
          "Act as the evidence-grounded Focusa Mission Canvas delivery lead.",
        title: "Focusa Mission Canvas delivery lead",
        purpose:
          "Translate accepted Context into safe, test-backed delivery while preserving operator authority.",
        expertise: ["Focusa canonical reducers", "generated UI", "evidence governance"],
        primary_responsibilities: [
          "Implement the accepted Spec 135 critical path",
          "Preserve exact project and workstream scope",
        ],
        secondary_responsibilities: ["Explain bounded recovery paths"],
        expected_deliverables: ["Verified capabilities with Evidence and Receipts"],
        quality_standards: ["Restart-safe", "citation-preserving", "lint-clean"],
        decision_principles: ["Operator steering outranks inferred intent"],
        evidence_expectations: ["Every closure links runtime and UIAI proof"],
        evidence_behavior:
          "Distinguish proposals, observed evidence, and canonical acceptance.",
        communication_posture: "Concise, transparent, and recovery-oriented.",
        stakeholder_posture:
          "Protect operator control and expose every unresolved assumption.",
        non_responsibilities: [
          "Granting operational permissions",
          "Inventing semantic authority",
        ],
        forbidden_assumptions: [
          "A role title implies permission",
          "Uncited claims are accepted",
        ],
        escalation_triggers: [
          "Scope conflict",
          "Missing evidence",
          "Permission ambiguity",
        ],
        handoff_boundaries: [
          "UIAI owns browser execution",
          "The operator owns consequential approval",
        ],
        tool_preferences: ["Operation Registry", "UIAI Engine"],
        reviewer_lenses: ["security", "accessibility", "evidence quality"],
        context_artifact_refs: [sourceId],
        context_claim_refs: [],
        interview_answer_refs: [],
        assumptions: [
          {
            statement: "Spec 135 is the approved delivery contract.",
            source_refs: [sourceId],
            status: "grounded" as const,
          },
        ],
        unresolved_questions: [],
        redlines: [],
        permission_profile_refs: ["permission-profile:operator-controlled"],
        permission_assertions: [],
      },
    });
    if (drafted.data) {
      lastDraft = drafted.data;
      renderDraft(drafted.data);
      document.body.dataset.roleStatus = "pending_operator";
      status.textContent = "Grounded Role draft is ready for explicit operator review";
      return;
    }
    if (drafted.response.status !== 409) {
      throw drafted.error ?? new Error("Role draft unavailable");
    }
    listed = await roleState();
    if (!listed.data) throw listed.error ?? new Error("Role state unavailable");
  }
  throw new Error("Role draft writer remained conflicted");
}

function renderDraft(drafted: RoleMutation): void {
  const profile = drafted.profile;
  renderer.processDelta([
    {
      version: "v0.9",
      updateComponents: {
        surfaceId: "ri1-role-profile",
        components: [
          {
            id: "seed",
            component: "FocusaRoleSeed",
            label: "Original operator seed",
            description: profile.original_seed,
            status: "saved",
            details: `seed_ref=${profile.grounding.operator_seed_ref}`,
          },
          {
            id: "draft",
            component: "FocusaRoleDraft",
            label: profile.title,
            description: profile.purpose,
            status: "pending_operator",
            details: `primary=${profile.primary_responsibilities.join(" | ")}; secondary=${profile.secondary_responsibilities.join(" | ")}; deliverables=${profile.expected_deliverables.join(" | ")}`,
          },
          {
            id: "grounding",
            component: "FocusaGroundingSources",
            label: "Context grounding",
            description: "Every responsibility is reviewable against canonical Context.",
            status: "verified",
            details: `artifacts=${profile.grounding.context_artifact_refs.join(",")}; claims=${profile.grounding.context_claim_refs.join(",") || "none"}`,
          },
          {
            id: "assumptions",
            component: "FocusaContextClaimReview",
            label: "Assumptions exposed",
            description: profile.assumptions
              .map((assumption) => `${assumption.status}: ${assumption.statement}`)
              .join(" | "),
            status: "verified",
            details: `unresolved_questions=${profile.unresolved_questions.length}; forbidden=${profile.forbidden_assumptions.join(" | ")}`,
          },
          {
            id: "boundary",
            component: "FocusaEvidenceSummary",
            label: "Responsibility is not permission",
            description:
              "This Role Profile cannot file, send, trade, modify production, or access an unapproved source.",
            status: "verified",
            details: `grants_permissions=${profile.grants_permissions}; permission_authority=${profile.permission_profile_refs.join(",")}`,
          },
          {
            id: "redline",
            component: "FocusaRedline",
            label: "Before / after redline",
            description:
              profile.redlines.length === 0
                ? "Initial revision — no operator edits yet"
                : profile.redlines.map((line) => `${line.field}: ${line.before} → ${line.after}`).join(" | "),
            status: "ready",
            details: `revision=${profile.revision}; history is append-only`,
          },
          {
            id: "approve",
            component: "FocusaPrimaryAction",
            label: "Approve grounded Role Profile",
            description:
              "Consequential operator action: activates responsibilities, not permissions.",
            primaryActionLabel: "Approve Role Profile",
            action: {
              event: {
                name: roleReviewBinding.action_id,
                context: { ...scope, decision: "approve" },
              },
            },
          },
          {
            id: "defer",
            component: "FocusaNextStepCard",
            label: "Defer Role Profile",
            description: "Keep the revision pending without activating it.",
            primaryActionLabel: "Defer",
            action: {
              event: {
                name: roleReviewBinding.action_id,
                context: { ...scope, decision: "defer" },
              },
            },
          },
          {
            id: "reject",
            component: "FocusaNextStepCard",
            label: "Reject Role Profile",
            description: "Supersede this draft while retaining auditable history.",
            primaryActionLabel: "Reject",
            action: {
              event: {
                name: roleReviewBinding.action_id,
                context: { ...scope, decision: "reject" },
              },
            },
          },
          {
            id: "receipt",
            component: "FocusaReceiptCard",
            label: "Draft revision committed",
            description: `revision=${profile.revision}; ${drafted.evidence_ref}`,
            status: "saved",
            details: `receipt=${drafted.receipt_ref}; state_version=${drafted.state_version}`,
          },
        ],
      },
    },
  ]);
}

async function reviewRole(decision: string): Promise<void> {
  if (!lastDraft) throw new Error("Compose a grounded draft before review");
  const drafted = lastDraft;
  status.textContent = `${decision[0]?.toUpperCase()}${decision.slice(1)}ing the Role Profile…`;
  let listed = await roleState();
  if (!listed.data) throw listed.error ?? new Error("Role state unavailable");
  for (let attempt = 0; attempt < 5; attempt += 1) {
    const reviewed = await client.POST("/v1/roles/profiles/review", {
      params: { query: scope },
      body: {
        ...scope,
        role_profile_id: drafted.profile.role_profile_id,
        profile_revision: drafted.profile.revision,
        idempotency_key: `ri1-generated-ui-${decision}-v2`,
        expected_state_version: listed.data.state_version,
        decision: decision as "approve" | "reject" | "defer",
        reviewed_by: "operator:vsmith",
        rationale:
          "Explicit operator review after inspecting grounding, assumptions, responsibilities, non-responsibilities, and permission separation.",
      },
    });
    if (reviewed.data) {
      lastReview = reviewed.data;
      const profile = reviewed.data.profile;
      renderer.processDelta([
        {
          version: "v0.9",
          updateComponents: {
            surfaceId: "ri1-role-profile",
            components: [
              {
                id: "approval",
                component: "FocusaApprovalCard",
                label: `Role Profile ${profile.status}`,
                description: `Explicit ${profile.review?.decision} by ${profile.review?.reviewed_by}`,
                status: profile.status,
                details: `revision=${profile.revision}; approved_at=${profile.review?.reviewed_at}; permission_grant=${profile.grants_permissions}`,
              },
              {
                id: "receipt",
                component: "FocusaReceiptCard",
                label: "Governed review Receipt",
                description: reviewed.data.evidence_ref,
                status: "completed",
                details: `receipt=${reviewed.data.receipt_ref}; state_version=${reviewed.data.state_version}`,
              },
            ],
          },
        },
      ]);
      document.body.dataset.roleStatus = profile.status;
      status.textContent = `Role Profile ${profile.status}; permission authority remains separate`;
      return;
    }
    if (reviewed.response.status !== 409) {
      throw reviewed.error ?? new Error("Role review unavailable");
    }
    listed = await roleState();
    if (!listed.data) throw listed.error ?? new Error("Role state unavailable");
  }
  throw new Error("Role review writer remained conflicted");
}

function showRecovery(error: unknown): void {
  lastError = error;
  status.textContent = "Role Profile needs bounded recovery";
  renderer.processDelta([
    {
      version: "v0.9",
      updateComponents: {
        surfaceId: "ri1-role-profile",
        components: [
          {
            id: "receipt",
            component: "FocusaRecoveryCard",
            label: "Role Profile needs recovery",
            description: "Refresh canonical state, preserve the operator seed, and retry safely.",
            status: "retry",
            details: JSON.stringify(error),
          },
        ],
      },
    },
  ]);
  document.body.dataset.roleStatus = "recovery";
}

const snapshot: A2uiMessage[] = [
  {
    version: "v0.9",
    createSurface: {
      surfaceId: "ri1-role-profile",
      catalogId: FOCUSA_A2UI_CATALOG_ID,
    },
  },
  {
    version: "v0.9",
    updateComponents: {
      surfaceId: "ri1-role-profile",
      components: [
        {
          id: "root",
          component: "Column",
          children: [
            "stage",
            "compose",
            "seed",
            "draft",
            "grounding",
            "assumptions",
            "boundary",
            "redline",
            "approve",
            "defer",
            "reject",
            "approval",
            "receipt",
          ],
        },
        {
          id: "stage",
          component: "FocusaStageShell",
          label: "Define the expert function — not its permissions",
          description:
            "Ground a versioned Role draft in canonical Context, inspect assumptions and redlines, then approve explicitly.",
          status: "ready",
          details: `draft=${roleDraftBinding.action_id}; review=${roleReviewBinding.action_id}; scope=${JSON.stringify(scope)}`,
        },
        {
          id: "compose",
          component: "FocusaPrimaryAction",
          label: "Compose a grounded Role draft",
          description:
            "Preserves the original seed, Context sources, responsibilities, non-responsibilities, assumptions, and permission boundary.",
          primaryActionLabel: "Compose Role Draft",
          action: { event: { name: roleDraftBinding.action_id, context: scope } },
        },
        {
          id: "seed",
          component: "FocusaRoleSeed",
          label: "Original operator seed",
          description: "The seed will remain visible across every revision.",
          status: "pending",
        },
        {
          id: "draft",
          component: "FocusaRoleDraft",
          label: "Generated Role draft",
          description: "Responsibilities and deliverables will appear here.",
          status: "pending",
        },
        {
          id: "grounding",
          component: "FocusaGroundingSources",
          label: "Context grounding",
          description: "Canonical source and claim refs will appear here.",
          status: "pending",
        },
        {
          id: "assumptions",
          component: "FocusaContextClaimReview",
          label: "Assumptions and unresolved questions",
          description: "Approval is blocked until assumptions are grounded or rejected.",
          status: "pending",
        },
        {
          id: "boundary",
          component: "FocusaEvidenceSummary",
          label: "Responsibility is not permission",
          description: "Role composition never grants operational authority.",
          status: "pending",
        },
        {
          id: "redline",
          component: "FocusaRedline",
          label: "Before / after redline",
          description: "Operator edits remain append-only and reviewable.",
          status: "pending",
        },
        {
          id: "approve",
          component: "FocusaPrimaryAction",
          label: "Explicit approval required",
          description: "Compose and inspect the grounded draft before approval.",
          primaryActionLabel: "Approve after composition",
          action: {
            event: {
              name: roleReviewBinding.action_id,
              context: { ...scope, decision: "approve" },
            },
          },
        },
        {
          id: "defer",
          component: "FocusaNextStepCard",
          label: "Defer available after composition",
          description: "No activation occurs while deferred.",
          status: "pending",
        },
        {
          id: "reject",
          component: "FocusaNextStepCard",
          label: "Reject available after composition",
          description: "Rejected revisions remain auditable.",
          status: "pending",
        },
        {
          id: "approval",
          component: "FocusaApprovalCard",
          label: "No active Role Profile yet",
          description: "An explicit operator review will be recorded here.",
          status: "pending",
        },
        {
          id: "receipt",
          component: "FocusaReceiptCard",
          label: "Role history and Receipt",
          description: "Canonical revision and Receipt refs will appear here.",
          status: "pending",
        },
      ],
    },
  },
];
renderer.processSnapshot(snapshot);
renderer.mount(surface, "ri1-role-profile");

Object.assign(window, {
  focusaRoleProfileEval: {
    renderer,
    scope,
    draftBinding: roleDraftBinding,
    reviewBinding: roleReviewBinding,
    observedActions,
    get lastDraft() {
      return lastDraft;
    },
    get lastReview() {
      return lastReview;
    },
    get lastError() {
      return lastError;
    },
  },
});
