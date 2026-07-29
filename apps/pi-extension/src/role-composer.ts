import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";
import {
  focusaFetch,
  getActiveWorkpointPacket,
  getContinuityId,
  getSessionCwd,
} from "./state.js";

function scopedQuery(projectRoot: string, continuityId: string, attachmentId: string): string {
  return new URLSearchParams({
    project_root: projectRoot,
    continuity_id: continuityId,
    attachment_id: attachmentId,
  }).toString();
}

function attachmentId(): string {
  const workpoint = getActiveWorkpointPacket();
  return String(
    workpoint?.attachment_id ||
      workpoint?.workpoint_id ||
      `attachment:role-composer:${getContinuityId()}`
  );
}

function renderRoleProfile(profile: any): string {
  const alternatives = Array.isArray(profile?.alternatives) ? profile.alternatives : [];
  const lines = [
    `# Role Profile — ${profile?.title || "Untitled"}`,
    "",
    `Status: ${profile?.status || "unknown"} · Revision: ${profile?.revision || 0}`,
    `Purpose: ${profile?.purpose || "—"}`,
    "",
    "## Authority boundary",
    profile?.grants_permissions
      ? "⚠ Role incorrectly claims operational permission."
      : "Role defines responsibility only; PermissionProfile and operator gates remain authoritative.",
    "",
    "## Alternatives",
    ...(alternatives.length
      ? alternatives.map(
          (alternative: any) =>
            `- ${alternative.title}: ${alternative.purpose} (${(alternative.tradeoffs || []).join("; ")})`
        )
      : ["- No alternatives recorded."]),
    "",
    "## Grounding",
    ...[
      ...(profile?.grounding?.context_artifact_refs || []),
      ...(profile?.grounding?.context_claim_refs || []),
      ...(profile?.grounding?.interview_answer_refs || []),
    ].map((reference: string) => `- ${reference}`),
  ];
  return lines.join("\n");
}

export function registerRoleComposer(pi: ExtensionAPI): void {
  pi.registerCommand("focusa-role", {
    description: "Compose, inspect, and approve a grounded Mission Canvas Role Profile",
    handler: async (_args, ctx) => {
      if (!ctx.hasUI) return;
      const projectRoot = getSessionCwd();
      const continuityId = getContinuityId();
      const attachment = attachmentId();
      if (!projectRoot || !continuityId || continuityId === "extension-bootstrap") {
        ctx.ui.notify("Role Composer requires a verified project/workstream scope.", "warning");
        return;
      }
      const query = scopedQuery(projectRoot, continuityId, attachment);
      const current = await focusaFetch(`/roles/profiles?${query}`);
      const action = await ctx.ui.select("Mission Canvas Role Composer", [
        "Create grounded draft",
        "Inspect latest profile",
        "Review latest profile",
      ]);
      if (!action) return;

      if (action === "Inspect latest profile") {
        if (!current?.latest) {
          ctx.ui.notify("No Role Profile exists for this attachment.", "info");
          return;
        }
        pi.sendMessage({
          customType: "focusa-role-profile",
          content: renderRoleProfile(current.latest),
          display: true,
        });
        return;
      }

      if (action === "Review latest profile") {
        const latest = current?.latest;
        if (!latest) {
          ctx.ui.notify("Create a grounded Role draft before review.", "warning");
          return;
        }
        const decision = await ctx.ui.select("Operator decision", ["approve", "reject", "defer"]);
        if (!decision) return;
        const rationale = await ctx.ui.input("Decision rationale");
        if (!rationale?.trim()) return;
        const reviewedBy = (await ctx.ui.input("Reviewer reference", "operator"))?.trim();
        if (!reviewedBy) return;
        const result = await focusaFetch("/roles/profiles/review", {
          method: "POST",
          body: JSON.stringify({
            project_root: projectRoot,
            continuity_id: continuityId,
            attachment_id: attachment,
            role_profile_id: latest.role_profile_id,
            profile_revision: latest.revision,
            idempotency_key: `pi-role-review:${latest.role_profile_id}:${latest.revision}:${decision}`,
            expected_state_version: current.state_version,
            decision,
            reviewed_by: reviewedBy,
            rationale: rationale.trim(),
          }),
        });
        if (!result?.profile) {
          ctx.ui.notify("Role review was rejected; inspect daemon diagnostics.", "error");
          return;
        }
        pi.sendMessage({
          customType: "focusa-role-profile",
          content: renderRoleProfile(result.profile),
          display: true,
        });
        return;
      }

      const sources = await focusaFetch(`/context/sources?${query}`);
      const availableSources = Array.isArray(sources?.sources) ? sources.sources : [];
      if (!availableSources.length) {
        ctx.ui.notify(
          "Role Composer requires Context grounding. Ingest a source for the active attachment first.",
          "warning"
        );
        return;
      }
      const sourceLabels = availableSources.map(
        (source: any) => `${source.title} · ${source.source_id}`
      );
      const selectedSource = await ctx.ui.select("Grounding source", sourceLabels);
      if (!selectedSource) return;
      const source = availableSources[sourceLabels.indexOf(selectedSource)];
      const seed = (await ctx.ui.input("Original operator role seed"))?.trim();
      const title = (await ctx.ui.input("Role title"))?.trim();
      const purpose = (await ctx.ui.editor("Role purpose", seed || ""))?.trim();
      const alternativeTitle = (
        await ctx.ui.input("Alternative role title", `${title || "Role"} reviewer`)
      )?.trim();
      if (!seed || !title || !purpose || !alternativeTitle) return;
      const groundingRef = source.source_id;
      const result = await focusaFetch("/roles/profiles/draft", {
        method: "POST",
        body: JSON.stringify({
          project_root: projectRoot,
          continuity_id: continuityId,
          attachment_id: attachment,
          idempotency_key: `pi-role-draft:${Date.now()}`,
          expected_state_version: sources.state_version,
          original_seed: seed,
          title,
          purpose,
          expertise: [title],
          primary_responsibilities: [`Serve the project as ${title}`],
          secondary_responsibilities: [],
          expected_deliverables: ["Evidence-backed project guidance"],
          quality_standards: ["Trace conclusions to accepted Context"],
          decision_principles: ["Escalate authority and evidence gaps"],
          evidence_expectations: ["Cite source artifacts and verification results"],
          evidence_behavior: "Preserve source links and state uncertainty explicitly.",
          communication_posture: "Concise, direct, and operator-legible.",
          stakeholder_posture: "Respect stakeholder boundaries and explicit handoffs.",
          non_responsibilities: ["Granting operational permission"],
          forbidden_assumptions: ["Role responsibility implies authority"],
          escalation_triggers: ["Missing evidence or permission"],
          handoff_boundaries: ["Consequential mutations require existing approval policy"],
          tool_preferences: [],
          reviewer_lenses: ["evidence", "authority", "scope"],
          alternatives: [
            {
              title: alternativeTitle,
              purpose: `Challenge and review the ${title} role with narrower execution scope.`,
              tradeoffs: ["More review depth with less direct execution"],
              grounding_refs: [groundingRef],
            },
          ],
          context_artifact_refs: [groundingRef],
          context_claim_refs: [],
          interview_answer_refs: [],
          assumptions: [],
          unresolved_questions: [],
          redlines: [],
          permission_profile_refs: [],
          permission_assertions: [],
        }),
      });
      if (!result?.profile) {
        ctx.ui.notify("Role draft was rejected; inspect grounding and daemon diagnostics.", "error");
        return;
      }
      pi.sendMessage({
        customType: "focusa-role-profile",
        content: renderRoleProfile(result.profile),
        display: true,
      });
    },
  });
}
