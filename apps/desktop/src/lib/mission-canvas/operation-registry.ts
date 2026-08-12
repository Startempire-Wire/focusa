// Canonical Spec 135 Operation Registry — 25 operations sourced from
// docs/contracts/spec135/mission-canvas-v1/operation-registry.json.
// Used as the authoritative source for synthesizing operation bindings
// when the daemon binary returns an empty or sparse bindings list.

import type { OperationBinding } from '../mission-canvas/types';

export interface OperationRegistryEntry {
  operation_id: string;
  confirmation: OperationBinding['confirmation'];
  target_contribution_id: string;
  display_label: string;
  enabled: boolean;
}

const REGISTRY: ReadonlyArray<OperationRegistryEntry> = [
  // Projection management
  { operation_id: 'focusa.mission_canvas.projection.get', confirmation: 'none', target_contribution_id: 'contribution:controls', display_label: 'Get Projection', enabled: true },
  { operation_id: 'focusa.mission_canvas.projection.resolve', confirmation: 'none', target_contribution_id: 'contribution:controls', display_label: 'Resolve Projection', enabled: true },
  // Profile operations
  { operation_id: 'focusa.mission_canvas.profile.list', confirmation: 'none', target_contribution_id: 'contribution:controls', display_label: 'List Profiles', enabled: true },
  { operation_id: 'focusa.mission_canvas.profile.select', confirmation: 'immediate', target_contribution_id: 'contribution:controls', display_label: 'Select Profile', enabled: true },
  { operation_id: 'focusa.mission_canvas.profile.get', confirmation: 'none', target_contribution_id: 'contribution:controls', display_label: 'Get Profile', enabled: true },
  // Activity operations
  { operation_id: 'focusa.mission_canvas.activity.list', confirmation: 'none', target_contribution_id: 'contribution:controls', display_label: 'List Activities', enabled: true },
  { operation_id: 'focusa.mission_canvas.activity.select', confirmation: 'immediate', target_contribution_id: 'contribution:controls', display_label: 'Select Activity', enabled: true },
  // Domain pack
  { operation_id: 'focusa.mission_canvas.domain_pack.install', confirmation: 'explicit', target_contribution_id: 'contribution:controls', display_label: 'Install Domain Pack', enabled: true },
  // Registry
  { operation_id: 'focusa.mission_canvas.registry.list', confirmation: 'none', target_contribution_id: 'contribution:controls', display_label: 'List Registries', enabled: true },
  // Layout memory
  { operation_id: 'focusa.mission_canvas.layout_memory.get', confirmation: 'none', target_contribution_id: 'contribution:controls', display_label: 'Get Layout Memory', enabled: true },
  { operation_id: 'focusa.mission_canvas.layout_memory.update', confirmation: 'none', target_contribution_id: 'contribution:controls', display_label: 'Update Layout Memory', enabled: true },
  // Layout
  { operation_id: 'focusa.mission_canvas.layout.mutate', confirmation: 'immediate', target_contribution_id: 'contribution:controls', display_label: 'Mutate Layout', enabled: true },
  // Rich host
  { operation_id: 'focusa.mission_canvas.rich_host.resolve', confirmation: 'none', target_contribution_id: 'contribution:controls', display_label: 'Resolve Rich Host', enabled: true },
  { operation_id: 'focusa.mission_canvas.rich_host.launch', confirmation: 'none', target_contribution_id: 'contribution:controls', display_label: 'Launch Rich Host', enabled: true },
  { operation_id: 'focusa.mission_canvas.rich_host.focus', confirmation: 'none', target_contribution_id: 'contribution:controls', display_label: 'Focus Rich Host', enabled: true },
  { operation_id: 'focusa.mission_canvas.rich_host.hide', confirmation: 'none', target_contribution_id: 'contribution:controls', display_label: 'Hide Rich Host', enabled: true },
  { operation_id: 'focusa.mission_canvas.rich_host.close', confirmation: 'explicit', target_contribution_id: 'contribution:controls', display_label: 'Close Rich Host', enabled: true },
  // Drafts
  { operation_id: 'focusa.mission_canvas.draft.get', confirmation: 'none', target_contribution_id: 'contribution:prompt-editor', display_label: 'Get Draft', enabled: true },
  { operation_id: 'focusa.mission_canvas.draft.sync', confirmation: 'none', target_contribution_id: 'contribution:prompt-editor', display_label: 'Sync Draft', enabled: true },
  // Recipient
  { operation_id: 'focusa.mission_canvas.recipient.resolve', confirmation: 'none', target_contribution_id: 'contribution:prompt-editor', display_label: 'Resolve Recipient', enabled: true },
  // Recomposition
  { operation_id: 'focusa.mission_canvas.recomposition.evidence.get', confirmation: 'none', target_contribution_id: 'contribution:evidence-stream', display_label: 'Get Recomposition Evidence', enabled: true },
  { operation_id: 'focusa.mission_canvas.recomposition.receipt.get', confirmation: 'none', target_contribution_id: 'contribution:evidence-stream', display_label: 'Get Recomposition Receipt', enabled: true },
  { operation_id: 'focusa.mission_canvas.recomposition.diagnostics.list', confirmation: 'none', target_contribution_id: 'contribution:evidence-stream', display_label: 'List Recomposition Diagnostics', enabled: true },
  // Pi session
  { operation_id: 'focusa.mission_canvas.pi_session.event.append', confirmation: 'none', target_contribution_id: 'contribution:pi-session', display_label: 'Append Pi Event', enabled: true },
  // Events
  { operation_id: 'focusa.mission_canvas.events.stream', confirmation: 'none', target_contribution_id: 'contribution:controls', display_label: 'Stream Events', enabled: true },
];

export function getOperationRegistry(): ReadonlyArray<OperationRegistryEntry> {
  return REGISTRY;
}

export function synthesizeAllOperationBindings(existing: OperationBinding[]): OperationBinding[] {
  const existingIds = new Set(existing.map(b => b.operation_id));
  const synthesized: OperationBinding[] = [...existing];
  for (const entry of REGISTRY) {
    if (existingIds.has(entry.operation_id)) continue;
    synthesized.push({
      operation_id: entry.operation_id,
      target_contribution_id: entry.target_contribution_id,
      enabled: entry.enabled,
      authority_ref: `synthetic:${entry.operation_id}:v0`,
      confirmation: entry.confirmation,
      display: { label: entry.display_label },
      input_schema_ref: 'v1'
    });
  }
  return synthesized;
}
