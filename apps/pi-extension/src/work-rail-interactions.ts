import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";
import { focusaFetch, getActiveWorkpointPacket, getContinuityId, getSessionCwd } from "./state.js";

const READ_ACTIONS = [
  "Open or focus related Work Surface",
  "Open Workpoint",
  "Open provider item",
  "Inspect evidence",
  "Inspect change artifact",
  "Inspect Receipt",
  "Inspect history",
  "Inspect session origin and contention",
  "Copy stable reference",
];
const MUTATION_ACTIONS = ["Steer active attachment", "Defer", "Request approval", "Reopen"];

function refs(values: unknown): string {
  return Array.isArray(values) && values.length ? values.join("\n") : "none";
}

function renderReadAction(row: any, action: string): string {
  const value = (() => {
    switch (action) {
      case "Open or focus related Work Surface":
        return refs(row.work_surface_ids);
      case "Open Workpoint":
        return row.workpoint_id;
      case "Open provider item":
        return `${row.provider}:${row.provider_item_id}`;
      case "Inspect evidence":
        return refs(row.evidence_refs);
      case "Inspect change artifact":
        return [row.change_set_ref, ...(row.artifact_refs || [])].filter(Boolean).join("\n") || "none";
      case "Inspect Receipt":
        return row.receipt_ref || "none";
      case "Inspect history":
        return (row.interaction_history || [])
          .map(
            (entry: any) =>
              `${entry.committed_at} · ${entry.action} · ${entry.actor_ref} · ${entry.receipt_ref}`
          )
          .join("\n") || "none";
      case "Inspect session origin and contention":
        return `instance=${row.instance_id || "none"}\nsession=${row.session_id || "none"}\nattachment=${row.attachment_id}`;
      default:
        return `work-rail:${row.work_rail_id}`;
    }
  })();
  return `# Work Rail — ${action}\n\n${value}`;
}

export function registerWorkRailInteractions(pi: ExtensionAPI): void {
  pi.registerCommand("focusa-rail", {
    description: "Open, inspect, preview, and commit actions for a selected Work Rail row",
    handler: async (_args, ctx) => {
      if (!ctx.hasUI) return;
      const packet = getActiveWorkpointPacket();
      const workpoint = packet?.workpoint && typeof packet.workpoint === "object" ? packet.workpoint : packet;
      const projectRoot = getSessionCwd();
      const continuityId = getContinuityId();
      const attachmentId = String(
        workpoint?.attachment_id || packet?.attachment_id || `attachment:rail:${continuityId}`
      );
      const workingSubpathId = String(
        workpoint?.session_identity?.working_subpath_id ||
          packet?.working_subpath_id ||
          "mission-canvas"
      );
      const query = new URLSearchParams({
        project_root: projectRoot,
        working_subpath_id: workingSubpathId,
        continuity_id: continuityId,
        attachment_id: attachmentId,
      });
      const listed = await focusaFetch(`/work-rail?${query}`);
      const rows = Array.isArray(listed?.rows) ? listed.rows : [];
      if (!rows.length) {
        ctx.ui.notify("No canonical Work Rail rows exist in this attachment scope.", "info");
        return;
      }
      const labels = rows.map(
        (row: any) => `${row.focusa_status} · ${row.title} · ${row.provider_item_id}`
      );
      const selected = await ctx.ui.select("Work Rail row", labels);
      if (!selected) return;
      const row = rows[labels.indexOf(selected)];
      const action = await ctx.ui.select("Work Rail action", [...READ_ACTIONS, ...MUTATION_ACTIONS]);
      if (!action) return;
      if (READ_ACTIONS.includes(action)) {
        pi.sendMessage({
          customType: "focusa-work-rail-action",
          content: renderReadAction(row, action),
          display: true,
        });
        return;
      }

      const actionMap: Record<string, string> = {
        "Steer active attachment": "steer",
        Defer: "defer",
        "Request approval": "request_approval",
        Reopen: "reopen",
      };
      const reason =
        action === "Defer"
          ? (await ctx.ui.input("Deferral reason"))?.trim()
          : `${action} requested in Pi Work Rail`;
      if (!reason) return;
      const actorRef = (await ctx.ui.input("Actor reference", "operator"))?.trim();
      if (!actorRef) return;
      const base = {
        project_root: projectRoot,
        working_subpath_id: workingSubpathId,
        continuity_id: continuityId,
        attachment_id: attachmentId,
        idempotency_key: `pi-work-rail:${Date.now()}:${actionMap[action]}`,
        expected_state_version: listed.state_version,
        expected_rail_revision: row.state_revision,
        action: actionMap[action],
        actor_ref: actorRef,
        interaction_reason: reason,
        work_rail_id: row.work_rail_id,
        workpoint_id: row.workpoint_id,
        provider_item_id: row.provider_item_id,
        title: row.title,
        instance_id: row.instance_id,
        session_id: row.session_id,
        work_surface_ids: row.work_surface_ids || [],
        priority: row.priority,
        rank: row.rank,
        change_set_ref: row.change_set_ref,
      };
      const preview = await focusaFetch("/work-rail/mutate", {
        method: "POST",
        body: JSON.stringify({ ...base, side_effect_policy: "preview" }),
      });
      if (!preview?.preview_token || preview?.committed) {
        ctx.ui.notify("Work Rail preview was rejected.", "error");
        return;
      }
      const confirmed = await ctx.ui.confirm(
        `Commit ${action}?`,
        `${row.title}\n${reason}\nPreview: ${preview.preview_token}`
      );
      if (!confirmed) return;
      const committed = await focusaFetch("/work-rail/mutate", {
        method: "POST",
        body: JSON.stringify({
          ...base,
          side_effect_policy: "commit",
          preview_token: preview.preview_token,
        }),
      });
      if (!committed?.committed) {
        ctx.ui.notify("Work Rail commit was rejected; refresh row state and retry.", "error");
        return;
      }
      pi.sendMessage({
        customType: "focusa-work-rail-action",
        content: renderReadAction(committed.row, "Inspect history"),
        display: true,
      });
    },
  });
}
