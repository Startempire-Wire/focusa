export type WorkspaceAvailability = 'shell' | 'planned';

export interface WorkspaceManifestItem {
  id: string;
  label: string;
  shortLabel: string;
  description: string;
  availability: WorkspaceAvailability;
  milestone: 5 | 25 | 50 | 75 | 100;
}

export const FOCUSA_DESKTOP_WORKSPACES: readonly WorkspaceManifestItem[] = [
  { id: 'mission-deck', label: 'Mission Deck', shortLabel: 'Deck', description: 'Orient around verified scope, Workstreams, and active missions.', availability: 'shell', milestone: 5 },
  { id: 'mission-canvas', label: 'Mission Canvas', shortLabel: 'Canvas', description: 'Compose the primary Workstream-aware mission workspace.', availability: 'planned', milestone: 25 },
  { id: 'pi-work-surface', label: 'Pi Work Surface', shortLabel: 'Pi', description: 'Use Pi as an authentic standalone or embedded coding surface.', availability: 'planned', milestone: 75 },
  { id: 'crist', label: 'C.R.I.S.T.', shortLabel: 'C.R.I.S.T.', description: 'Inspect and guide structured reasoning without duplicating authority.', availability: 'planned', milestone: 75 },
  { id: 'context-role', label: 'Context and Role', shortLabel: 'Context', description: 'See exact Context and Role bindings for the selected Workstream.', availability: 'planned', milestone: 50 },
  { id: 'workpoints', label: 'Workpoints', shortLabel: 'Workpoints', description: 'Present truthful Workpoint state and verification posture.', availability: 'planned', milestone: 50 },
  { id: 'trajectory', label: 'Tactical Trajectory', shortLabel: 'Trajectory', description: 'Present the tactical Workstream Trajectory.', availability: 'planned', milestone: 50 },
  { id: 'sessions', label: 'Sessions', shortLabel: 'Sessions', description: 'Inspect temporal runtimes and exact Workstream Attachments.', availability: 'planned', milestone: 50 },
  { id: 'contention', label: 'Contention and Approvals', shortLabel: 'Approvals', description: 'Surface conflicts, writer posture, and approval boundaries.', availability: 'planned', milestone: 75 },
  { id: 'evidence', label: 'Evidence and Receipts', shortLabel: 'Evidence', description: 'Inspect provenance, Evidence, Receipts, and closure proof.', availability: 'planned', milestone: 50 },
  { id: 'documents', label: 'Documents', shortLabel: 'Documents', description: 'Work with Workstream-bound documents and explicit shared context.', availability: 'planned', milestone: 75 },
  { id: 'research', label: 'Research', shortLabel: 'Research', description: 'Run governed research with UIAI Engine browser Evidence.', availability: 'planned', milestone: 75 },
  { id: 'agent-runtime', label: 'Agent Runtime', shortLabel: 'Runtime', description: 'Inspect daemon infrastructure separately from cognitive state.', availability: 'shell', milestone: 5 }
] as const;

export function workspaceById(id: string): WorkspaceManifestItem {
  return FOCUSA_DESKTOP_WORKSPACES.find((item) => item.id === id) ?? FOCUSA_DESKTOP_WORKSPACES[0];
}
