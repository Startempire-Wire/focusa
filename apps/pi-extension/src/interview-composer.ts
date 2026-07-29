import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";
import {
  focusaFetch,
  getActiveWorkpointPacket,
  getContinuityId,
  getSessionCwd,
} from "./state.js";

function attachmentId(): string {
  const workpoint = getActiveWorkpointPacket();
  return String(
    workpoint?.attachment_id ||
      workpoint?.workpoint_id ||
      `attachment:interview:${getContinuityId()}`
  );
}

function query(projectRoot: string, continuityId: string, attachment: string, sessionId?: string) {
  const params = new URLSearchParams({
    project_root: projectRoot,
    continuity_id: continuityId,
    attachment_id: attachment,
  });
  if (sessionId) params.set("interview_session_id", sessionId);
  return params.toString();
}

function renderSession(session: any): string {
  return [
    `# Grill Interview — ${session?.interview_session_id || "not started"}`,
    "",
    `Status: ${session?.status || "unknown"} · Revision: ${session?.state_revision || 0}`,
    `Current question: ${session?.current_question_id || "none"}`,
    `Answers: ${Array.isArray(session?.answers) ? session.answers.length : 0}`,
    `Branches: ${Array.isArray(session?.branches) ? session.branches.length : 0}`,
    "",
    "Operator answers remain canonical; summaries are linked projections only.",
  ].join("\n");
}

export function registerInterviewComposer(pi: ExtensionAPI): void {
  pi.registerCommand("focusa-interview", {
    description: "Continue the durable Mission Canvas Grill Interview and closure compendium",
    handler: async (_args, ctx) => {
      if (!ctx.hasUI) return;
      const projectRoot = getSessionCwd();
      const continuityId = getContinuityId();
      const attachment = attachmentId();
      if (!projectRoot || !continuityId || continuityId === "extension-bootstrap") {
        ctx.ui.notify("Interview requires a verified project/workstream scope.", "warning");
        return;
      }
      const scopeQuery = query(projectRoot, continuityId, attachment);
      const listed = await focusaFetch(`/interviews/sessions?${scopeQuery}`);
      let session = Array.isArray(listed?.sessions) ? listed.sessions.at(-1) : null;
      const action = await ctx.ui.select("Mission Canvas Interview", [
        "Continue Interview",
        "Add Context",
        "Revisit Answer",
        "Ask About New Context",
        "Resolve Contradiction",
        "Pause Interview",
        "Close and Build Compendium",
      ]);
      if (!action) return;

      const mutate = async (mutation: Record<string, any>) => {
        const fresh = await focusaFetch(`/interviews/sessions?${scopeQuery}`);
        session = Array.isArray(fresh?.sessions) ? fresh.sessions.at(-1) : null;
        return focusaFetch("/interviews/sessions/mutate", {
          method: "POST",
          body: JSON.stringify({
            project_root: projectRoot,
            continuity_id: continuityId,
            attachment_id: attachment,
            interview_session_id: session?.interview_session_id,
            expected_state_version: fresh?.state_version || 0,
            expected_session_revision: session?.state_revision || 0,
            idempotency_key: `pi-interview:${Date.now()}:${mutation.action}`,
            ...mutation,
          }),
        });
      };

      if (action === "Add Context") {
        const content = (
          await ctx.ui.editor("Add source-linked Interview Context", "# Interview Context\n\n")
        )?.trim();
        if (!content) return;
        const sources = await focusaFetch(`/context/sources?${scopeQuery}`);
        const result = await focusaFetch("/context/sources/ingest", {
          method: "POST",
          body: JSON.stringify({
            project_root: projectRoot,
            continuity_id: continuityId,
            attachment_id: attachment,
            idempotency_key: `pi-interview-context:${Date.now()}`,
            expected_state_version: sources?.state_version || 0,
            source_kind: "focusa_native",
            source_locator: `focusa://interview/operator-note/${Date.now()}`,
            source_revision: `operator-note:${Date.now()}`,
            title: "Interview operator context",
            mime_type: "text/markdown",
            content,
            author: "operator",
            sensitivity: "operator_supplied",
            freshness_status: "current",
          }),
        });
        ctx.ui.notify(
          result?.source ? "Interview Context added with provenance." : "Context ingestion rejected.",
          result?.source ? "info" : "error"
        );
        return;
      }

      if (action === "Continue Interview") {
        if (!session) {
          const roles = await focusaFetch(`/roles/profiles?${scopeQuery}`);
          if (!roles?.approved) {
            ctx.ui.notify("Approve a grounded Role Profile before opening Interview.", "warning");
            return;
          }
          const opened = await mutate({
            action: "open",
            approved_role_profile_ref: roles.approved.role_profile_id,
          });
          session = opened?.session;
        } else if (session.status === "paused" || session.status === "closed") {
          const reopened = await mutate({ action: "reopen" });
          session = reopened?.session;
        }
        pi.sendMessage({
          customType: "focusa-interview-session",
          content: renderSession(session),
          display: true,
        });
        return;
      }

      if (!session) {
        ctx.ui.notify("Open Interview before using this action.", "warning");
        return;
      }

      if (action === "Revisit Answer") {
        const answers = Array.isArray(session.answers) ? session.answers : [];
        if (!answers.length) {
          ctx.ui.notify("No answer is available to revisit.", "info");
          return;
        }
        const labels = answers.map(
          (answer: any) => `${answer.question_id} · ${JSON.stringify(answer.answer)}`
        );
        const selected = await ctx.ui.select("Answer to amend", labels);
        if (!selected) return;
        const prior = answers[labels.indexOf(selected)];
        const answer = (await ctx.ui.editor("Amended operator answer", String(prior.answer)))?.trim();
        if (!answer) return;
        const result = await mutate({
          action: "record_answer",
          answer: {
            question_id: prior.question_id,
            answer,
            attachment_refs: [],
            operator_id: "operator",
            confidence: 1,
            notes: "Amended through Pi Interview Composer.",
            supersedes: prior.answer_id,
          },
        });
        ctx.ui.notify(result?.session ? "Answer amendment recorded." : "Answer rejected.", result?.session ? "info" : "error");
        return;
      }

      if (action === "Ask About New Context" || action === "Resolve Contradiction") {
        const question = (await ctx.ui.input(action))?.trim();
        if (!question) return;
        const contradiction = action === "Resolve Contradiction";
        let branchId =
          session.active_branch_id || session.branches?.[0]?.decision_branch_id || "operator-follow-up";
        if (!Array.isArray(session.branches) || !session.branches.length) {
          const branched = await mutate({
            action: "upsert_branch",
            branch: {
              decision_branch_id: branchId,
              tranche: contradiction ? "blockers" : "new_context",
              label: contradiction ? "Resolve contradiction" : "Assess new Context",
            },
          });
          if (!branched?.session) {
            ctx.ui.notify("Interview branch creation rejected.", "error");
            return;
          }
          session = branched.session;
          branchId = session.active_branch_id || branchId;
        }
        const result = await mutate({
          action: "queue_question",
          question: {
            decision_branch_id: branchId,
            question,
            reason_for_asking: contradiction
              ? "Resolve an explicit Context contradiction."
              : "Assess new Context supplied by the operator.",
            triggering_gap: contradiction ? "unresolved_contradiction" : "new_context",
            recommendation: "Answer with source links or explicitly defer.",
            recommendation_basis_refs: [],
            environment_facts_checked: [],
            contradiction_refs: contradiction ? ["operator-reported-contradiction"] : [],
            linked_context_refs: [],
            linked_spec_sections: [],
            decision_required: contradiction,
            priority: contradiction ? "blocker" : "normal",
            answer_type: "long_text",
            sensitivity: "project",
            readiness_effect: contradiction
              ? "Blocks readiness until resolved or explicitly accepted."
              : "Updates readiness from new Context.",
            stop_condition: "Operator answer, explicit deferral, or supersession.",
          },
        });
        ctx.ui.notify(result?.session ? "Interview question queued." : "Question rejected.", result?.session ? "info" : "error");
        return;
      }

      if (action === "Pause Interview") {
        const result = await mutate({ action: "pause" });
        ctx.ui.notify(result?.session ? "Interview paused with exact resume state." : "Pause rejected.", result?.session ? "info" : "error");
        return;
      }

      const closed = await mutate({ action: "close" });
      if (!closed?.session) {
        ctx.ui.notify("Interview close rejected.", "error");
        return;
      }
      const closure = await focusaFetch(
        `/interviews/closure-package?${query(
          projectRoot,
          continuityId,
          attachment,
          closed.session.interview_session_id
        )}`
      );
      pi.sendMessage({
        customType: "focusa-interview-session",
        content: [
          renderSession(closed.session),
          "",
          "## Closure Compendium",
          `Closure: ${closure?.closure_ref || "unavailable"}`,
          `Receipt: ${closure?.receipt_ref || "unavailable"}`,
          `Entries: ${Array.isArray(closure?.compendium) ? closure.compendium.length : 0}`,
        ].join("\n"),
        display: true,
      });
    },
  });
}
