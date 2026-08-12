// Spec 135 Contribution Catalog — mirrors Core profiles.rs definitions.
// Used to fill gaps when the old daemon returns sparse eligible_contributions.
// Never invents authority — these are the SAME contributions Core defines
// in crates/focusa-core/src/mission_canvas/profiles.rs.

import type { ResolvedContribution } from '../mission-canvas/types';

export interface ContributionDef {
  contribution_id: string;
  renderer_binding_id: string;
  kind: ResolvedContribution['kind'];
  label: string;
  description: string;
}

// Activity → expected contributions (per profiles.rs)
export const ACTIVITY_CONTRIBUTIONS: Record<string, ContributionDef[]> = {
  overview: [
    { contribution_id: 'contribution:pi-session', renderer_binding_id: 'renderer:pi-session@v1', kind: 'pi_session', label: 'Pi Session', description: 'Active Focusa runtime workspace' },
    { contribution_id: 'contribution:project-overview', renderer_binding_id: 'renderer:project-overview', kind: 'project_overview', label: 'Project Overview', description: 'Canonical project intelligence signals' },
    { contribution_id: 'contribution:work-rail', renderer_binding_id: 'renderer:work-rail@v1', kind: 'work_rail', label: 'Focusa Work Rail', description: 'Active Workpoint queue and status' },
    { contribution_id: 'contribution:steering-queue', renderer_binding_id: 'renderer:steering-queue@v1', kind: 'steering_queue', label: 'Steering Queue', description: 'Operator steering items requiring review' },
    { contribution_id: 'contribution:follow-up-queue', renderer_binding_id: 'renderer:follow-up-queue@v1', kind: 'follow_up_queue', label: 'Follow-up Queue', description: 'Agent follow-up items' },
    { contribution_id: 'contribution:prompt-editor', renderer_binding_id: 'renderer:prompt-editor@v1', kind: 'prompt_editor', label: 'Prompt Editor', description: 'Compose prompts through canonical Workstream' },
    { contribution_id: 'contribution:controls', renderer_binding_id: 'renderer:controls', kind: 'toolbar_control', label: 'Controls', description: 'Workspace profile and activity mode' },
  ],
  tasks: [
    { contribution_id: 'contribution:work-rail', renderer_binding_id: 'renderer:work-rail@v1', kind: 'work_rail', label: 'Focusa Work Rail', description: 'Active tasks and work items' },
    { contribution_id: 'contribution:controls', renderer_binding_id: 'renderer:controls', kind: 'toolbar_control', label: 'Controls', description: 'Workspace profile and activity mode' },
    { contribution_id: 'contribution:prompt-editor', renderer_binding_id: 'renderer:prompt-editor@v1', kind: 'prompt_editor', label: 'Prompt Editor', description: 'Compose prompts through canonical Workstream' },
  ],
  sessions: [
    { contribution_id: 'contribution:silent-sessions', renderer_binding_id: 'renderer:silent-sessions@v1', kind: 'session_inventory', label: 'Silent Sessions', description: 'Active and recent agent sessions' },
    { contribution_id: 'contribution:history', renderer_binding_id: 'renderer:history@v1', kind: 'event_history', label: 'Event History', description: 'Canonical event timeline' },
    { contribution_id: 'contribution:controls', renderer_binding_id: 'renderer:controls', kind: 'toolbar_control', label: 'Controls', description: 'Workspace profile and activity mode' },
  ],
  research: [
    { contribution_id: 'contribution:research-surface', renderer_binding_id: 'renderer:research@v1', kind: 'research', label: 'Research Surface', description: 'Browser and artifact research workspace' },
    { contribution_id: 'contribution:document', renderer_binding_id: 'renderer:document@v1', kind: 'document', label: 'Document View', description: 'Active document or artifact' },
    { contribution_id: 'contribution:controls', renderer_binding_id: 'renderer:controls', kind: 'toolbar_control', label: 'Controls', description: 'Workspace profile and activity mode' },
    { contribution_id: 'contribution:prompt-editor', renderer_binding_id: 'renderer:prompt-editor@v1', kind: 'prompt_editor', label: 'Prompt Editor', description: 'Compose prompts through canonical Workstream' },
  ],
  workpoints: [
    { contribution_id: 'contribution:workpoint-list', renderer_binding_id: 'renderer:work-rail@v1', kind: 'work_rail', label: 'Workpoint List', description: 'All workpoints in this Workstream' },
    { contribution_id: 'contribution:evidence-stream', renderer_binding_id: 'renderer:evidence@v1', kind: 'evidence_stream', label: 'Evidence Stream', description: 'Proofs and receipts' },
    { contribution_id: 'contribution:controls', renderer_binding_id: 'renderer:controls', kind: 'toolbar_control', label: 'Controls', description: 'Workspace profile and activity mode' },
  ],
  canvas: [
    { contribution_id: 'contribution:project-overview', renderer_binding_id: 'renderer:project-overview', kind: 'project_overview', label: 'Project Overview', description: 'Canonical project intelligence signals' },
    { contribution_id: 'contribution:inspector', renderer_binding_id: 'renderer:focusa-inspector@v1', kind: 'inspector', label: 'Focusa Inspector', description: 'Full system introspection' },
    { contribution_id: 'contribution:controls', renderer_binding_id: 'renderer:controls', kind: 'toolbar_control', label: 'Controls', description: 'Workspace profile and activity mode' },
  ],
  prompt: [
    { contribution_id: 'contribution:prompt-editor', renderer_binding_id: 'renderer:prompt-editor@v1', kind: 'prompt_editor', label: 'Prompt Editor', description: 'Compose prompts through canonical Workstream' },
    { contribution_id: 'contribution:work-rail', renderer_binding_id: 'renderer:work-rail@v1', kind: 'work_rail', label: 'Focusa Work Rail', description: 'Active Workpoint queue and status' },
  ],
  notebook: [
    { contribution_id: 'contribution:document', renderer_binding_id: 'renderer:document@v1', kind: 'document', label: 'Notebook Document', description: 'Active notebook entry' },
    { contribution_id: 'contribution:controls', renderer_binding_id: 'renderer:controls', kind: 'toolbar_control', label: 'Controls', description: 'Workspace profile and activity mode' },
  ],
  domain: [
    { contribution_id: 'contribution:domain-surface', renderer_binding_id: 'renderer:focusa-inspector@v1', kind: 'domain_surface', label: 'Domain Surface', description: 'Canonical domain view' },
    { contribution_id: 'contribution:controls', renderer_binding_id: 'renderer:controls', kind: 'toolbar_control', label: 'Controls', description: 'Workspace profile and activity mode' },
  ],
  receipts: [
    { contribution_id: 'contribution:receipts-surface', renderer_binding_id: 'renderer:evidence@v1', kind: 'evidence_stream', label: 'Receipts Surface', description: 'Canonical receipts view' },
    { contribution_id: 'contribution:controls', renderer_binding_id: 'renderer:controls', kind: 'toolbar_control', label: 'Controls', description: 'Workspace profile and activity mode' },
  ],
  trajectory: [
    { contribution_id: 'contribution:trajectory-surface', renderer_binding_id: 'renderer:focusa-inspector@v1', kind: 'trajectory_surface', label: 'Trajectory Surface', description: 'Canonical trajectory view' },
    { contribution_id: 'contribution:controls', renderer_binding_id: 'renderer:controls', kind: 'toolbar_control', label: 'Controls', description: 'Workspace profile and activity mode' },
  ],
  identity: [
    { contribution_id: 'contribution:identity-surface', renderer_binding_id: 'renderer:focusa-inspector@v1', kind: 'identity_surface', label: 'Identity Surface', description: 'Canonical identity view' },
    { contribution_id: 'contribution:controls', renderer_binding_id: 'renderer:controls', kind: 'toolbar_control', label: 'Controls', description: 'Workspace profile and activity mode' },
  ],
};

export function getExpectedContributions(activityId: string): ContributionDef[] {
  return ACTIVITY_CONTRIBUTIONS[activityId] ?? [];
}

export function synthesizeMissingContributions(
  existing: ResolvedContribution[],
  activityId: string
): ResolvedContribution[] {
  const expected = getExpectedContributions(activityId);
  if (expected.length === 0) return existing.slice();
  
  const existingIds = new Set(existing.map(c => c.contribution_id));
  const now = new Date().toISOString();
  
  const result = existing.slice();
  for (const def of expected) {
    if (existingIds.has(def.contribution_id)) continue;
    result.push({
      contribution_id: def.contribution_id,
      renderer_binding_id: def.renderer_binding_id,
      kind: def.kind,
      accessibility: {
        label: def.label,
        description: def.description,
        role: 'region' as const
      },
      freshness: { status: 'current' as const, observed_at: now },
      data_ref: {
        kind: def.kind,
        ref: def.contribution_id.replace('contribution:', 'surface:'),
        revision: 1,
        freshness: 'current' as const
      },
      operation_ids: [],
      candidate_contribution_ids: [],
      data: null
    });
  }
  return result;
}
