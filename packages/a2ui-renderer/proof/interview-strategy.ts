import { createFocusaSpec135Client, type components } from "@focusa/spec135-client";
import actionBindings from "../../../docs/contracts/spec135/generated-contract-v1/ui-action-bindings.fixture.json" with { type: "json" };
import { FOCUSA_A2UI_CATALOG_ID, FocusaA2uiRenderer, type A2uiClientAction, type A2uiMessage } from "../src/index.js";

type StrategyBody = components["schemas"]["focusa_grill_interview_context_v1"];
const scope = { project_root: "/example/focusa", continuity_id: "focusa-cont-ri2-generated-ui", attachment_id: "attachment:ri2-interview" };
const operationId = "focusa.interview.strategy.grill_with_docs.next_question";
const binding = actionBindings.bindings.find((candidate) => candidate.action_id === operationId);
if (!binding || binding.contracts.input_schema_ref !== "focusa.grill_interview_context.v1") throw new Error("Generated Grill strategy binding unavailable");
const client = createFocusaSpec135Client({ baseUrl: window.location.origin });
const surface = document.querySelector<HTMLElement>("#interview-strategy-surface");
const status = document.querySelector<HTMLElement>("#strategy-result");
if (!surface || !status) throw new Error("Grill strategy proof mount missing");
const observedActions: A2uiClientAction[] = [];
let response: unknown;
let lastError: unknown;

function gap(sourceRef: string, tranche: StrategyBody["gaps"][number]["tranche"], id: string, branch: string, priority: StrategyBody["gaps"][number]["priority"], dependencies: number): StrategyBody["gaps"][number] {
  return { gap_id: id, tranche, decision_branch_id: branch, question: `Which bounded option should govern ${id}?`, reason_for_asking: "This remaining tradeoff is operator-owned.", triggering_gap: `Unresolved operator decision: ${id}`, recommendation: "Choose the smallest reversible option that preserves operator authority.", recommendation_basis_refs: [sourceRef], environment_facts_checked: [sourceRef], contradiction_refs: [], linked_context_refs: [sourceRef], linked_spec_sections: ["135H §4"], domain_term_candidates: [], architecture_decision_candidates: [], decision_required: true, priority, answer_type: "select", readiness_effect: "Closes one readiness dependency.", stop_condition: "Operator selects or explicitly defers this branch.", downstream_dependency_count: dependencies, resolved: false };
}

const renderer = new FocusaA2uiRenderer({
  allowedActionNames: new Set([operationId]),
  async onAction(action) {
    observedActions.push(action);
    document.body.dataset.strategyStatus = "running";
    status.textContent = "Retrieving accepted Context and choosing one dependency-ordered question…";
    try {
      const [contexts, roles] = await Promise.all([
        client.GET("/v1/context/sources", { params: { query: scope } }),
        client.GET("/v1/roles/profiles", { params: { query: scope } }),
      ]);
      if (!contexts.data || !roles.data) throw contexts.error ?? roles.error ?? new Error("Canonical inputs unavailable");
      const sourceRef = contexts.data.sources[0]?.source_id;
      const roleRef = roles.data.approved?.role_profile_id;
      if (!sourceRef || !roleRef) throw new Error("Seed one canonical Context source and approved Role Profile in this exact scope");
      const body: StrategyBody = {
        ...scope, session_id: "interview-session-ri2-generated-ui", approved_role_profile_ref: roleRef, active_branch_id: "scope", completed_tranches: [],
        gaps: [
          gap(sourceRef, "discovery", "desired-outcome", "purpose", "blocker", 9),
          gap(sourceRef, "boundary", "scope-boundary", "scope", "normal", 2),
          gap(sourceRef, "boundary", "non-goal", "scope", "high", 6),
          gap(sourceRef, "failure", "known-failure", "failure", "high", 4),
          gap(sourceRef, "evidence", "acceptance-proof", "evidence", "high", 3),
          gap(sourceRef, "architecture", "adapter-boundary", "architecture", "normal", 2),
          gap(sourceRef, "spec_readiness", "approval-gate", "readiness", "normal", 1),
        ],
      };
      const proposed = await client.POST("/v1/interview/strategy/grill-with-docs/next-question", { params: { query: scope }, body });
      if (!proposed.data) throw proposed.error ?? new Error("Strategy proposal unavailable");
      response = proposed.data;
      if (!proposed.data.result.one_question_only || !proposed.data.result.retrieval_performed_before_question) throw new Error("Strategy violated retrieval-first one-question contract");
      const proposal = proposed.data.result.proposal;
      if (!proposal) throw new Error("Expected one unresolved Grill question");
      renderer.processDelta([{ version: "v0.9", updateComponents: { surfaceId: "ri2-interview-strategy", components: [
        { id: "stage", component: "FocusaStageShell", label: "One decision at a time", description: proposal.question, status: "completed", details: `tranche=${proposal.tranche}; branch=${proposal.decision_branch_id}; ${proposal.branch_progress}` },
        { id: "recommendation", component: "FocusaDecisionGate", label: "Recommended answer", description: proposal.recommendation, status: "ready", details: `basis=${proposal.recommendation_basis_refs.join(", ")}; operator answer is authoritative=${proposal.operator_answer_is_authoritative}` },
        { id: "evidence", component: "FocusaEvidenceSummary", label: "Fact-before-question proof", description: `Checked ${proposal.environment_facts_checked.length} canonical fact ref; linked ${proposal.linked_context_refs.length} Context ref.`, status: "verified", details: `stop=${proposal.stop_condition}` },
      ] } }]);
      status.textContent = `Completed: ${proposal.tranche} / ${proposal.decision_branch_id}; one cited recommendation proposed.`;
      document.body.dataset.strategyStatus = "completed";
    } catch (error) {
      lastError = error;
      status.textContent = `Recovery required: ${error instanceof Error ? error.message : String(error)}`;
      document.body.dataset.strategyStatus = "recovery";
    }
  },
});

const snapshot: A2uiMessage[] = [
  { version: "v0.9", createSurface: { surfaceId: "ri2-interview-strategy", catalogId: FOCUSA_A2UI_CATALOG_ID } },
  { version: "v0.9", updateComponents: { surfaceId: "ri2-interview-strategy", components: [
    { id: "root", component: "Column", children: ["stage", "tranches", "recommendation", "evidence", "run"] },
    { id: "stage", component: "FocusaStageShell", label: "Grill with project knowledge", description: "Retrieve what Focusa can know, then ask only the highest-value operator decision.", status: "ready", details: `strategy=focusa.interview.strategy.grill-with-docs.v1; operation=${operationId}` },
    { id: "tranches", component: "FocusaProgressStepper", label: "Six required tranches", description: "Discovery → Boundary → Failure → Evidence → Architecture → Spec-Readiness", status: "ready", progress: 0 },
    { id: "recommendation", component: "FocusaDecisionGate", label: "Recommendation", description: "Every operator decision includes one cited recommendation; no recommendation is authority.", status: "pending" },
    { id: "evidence", component: "FocusaEvidenceSummary", label: "Source grounding", description: "Canonical Context and approved Role refs are verified in exact scope before asking.", status: "pending" },
    { id: "run", component: "FocusaPrimaryAction", label: "Choose the next question", description: "Preserves the active dependency branch and returns exactly one question.", primaryActionLabel: "Run Grill Strategy", action: { event: { name: operationId, context: scope } } },
  ] } },
];
renderer.processSnapshot(snapshot);
renderer.mount(surface, "ri2-interview-strategy");
Object.assign(window, { focusaInterviewStrategyEval: { renderer, binding, scope, observedActions, get response() { return response; }, get lastError() { return lastError; } } });
