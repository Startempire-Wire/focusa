import type { ResolvedWorkspaceProjection } from './types';

// Canonical label resolution following Spec 135 handoff:
// When the daemon provides a raw contribution-id label, derive a
// human-readable label from the renderer binding id. Never invents
// authority — only maps well-known binding ids to display strings.

const RENDERER_LABELS: Readonly<Record<string, string>> = {
  'renderer:controls': 'Controls',
  'renderer:pi-session@v1': 'Pi Session',
  'renderer:project-overview': 'Project Overview',
  'renderer:work-rail@v1': 'Work Rail',
  'renderer:steering-queue@v1': 'Steering Queue',
  'renderer:follow-up-queue@v1': 'Follow-up Queue',
  'renderer:prompt-editor@v1': 'Prompt Editor',
  'renderer:generated-surface@v1': 'Generated Surface',
  'renderer:document@v1': 'Document',
  'renderer:research@v1': 'Research',
  'renderer:evidence@v1': 'Evidence',
  'renderer:session-inventory@v1': 'Session Inventory',
  'renderer:browser-artifact@v1': 'Browser Artifact',
  'renderer:canonical-event-history@v1': 'Event History',
};

/** Returns true if the label is a raw contribution id like "contribution:controls" */
export function isRawIdLabel(label: string): boolean {
  return label.startsWith('contribution:');
}

/** Derive a human-readable label from the renderer binding id */
export function deriveLabel(rendererBindingId: string, fallback: string): string {
  if (!isRawIdLabel(fallback)) return fallback;
  return RENDERER_LABELS[rendererBindingId] ?? fallback;
}

/** Normalize all contribution labels in a projection inline. Returns the same object. */
export function normalizeLabels(projection: ResolvedWorkspaceProjection): ResolvedWorkspaceProjection {
  for (const c of projection.eligible_contributions ?? []) {
    const label = c.accessibility?.label ?? c.contribution_id;
    if (isRawIdLabel(label)) {
      const derived = RENDERER_LABELS[c.renderer_binding_id] ?? c.contribution_id;
      if (c.accessibility) {
        c.accessibility.label = derived;
      }
    }
  }
  return projection;
}
