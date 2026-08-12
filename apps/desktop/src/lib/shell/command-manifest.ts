import { FOCUSA_DESKTOP_WORKSPACES } from './workspace-manifest';
import type { MotionMode } from '$lib/ui/motion';
import type { DesktopSidebarMode } from './sidebar-preferences';

export type PresentationCommandAction =
  | { kind: 'navigate-workspace'; workspaceId: string }
  | { kind: 'set-interface'; interfaceMode: 'canvas' | 'tui' }
  | { kind: 'set-sidebar'; sidebarMode: DesktopSidebarMode }
  | { kind: 'set-motion'; motionMode: MotionMode }
  | { kind: 'select-profile'; profileId: string }
  | { kind: 'select-activity'; activityId: string };

export interface PresentationCommand {
  id: string;
  label: string;
  hint: string;
  keywords: string[];
  action: PresentationCommandAction;
  authority: 'presentation-only';
}

export const FOCUSA_DESKTOP_PRESENTATION_COMMANDS: readonly PresentationCommand[] = [
  ...FOCUSA_DESKTOP_WORKSPACES.map((workspace) => ({
    id: `workspace.${workspace.id}`,
    label: `Open ${workspace.label}`,
    hint: workspace.availability === 'shell' ? 'Available shell' : `Planned M${workspace.milestone}`,
    keywords: ['workspace', workspace.label, workspace.shortLabel],
    action: { kind: 'navigate-workspace' as const, workspaceId: workspace.id },
    authority: 'presentation-only' as const
  })),
  { id: 'interface.canvas', label: 'Show Mission Canvas', hint: 'Replace the complete inner surface', keywords: ['canvas', 'gui', 'interface'], action: { kind: 'set-interface', interfaceMode: 'canvas' }, authority: 'presentation-only' },
  { id: 'interface.tui', label: 'Show Agent TUI', hint: 'Open integrated Pi Work Surface', keywords: ['agent', 'tui', 'pi', 'terminal'], action: { kind: 'set-interface', interfaceMode: 'tui' }, authority: 'presentation-only' },
  { id: 'sidebar.expand', label: 'Expand sidebar', hint: 'Local layout preference', keywords: ['sidebar', 'navigation', 'wide'], action: { kind: 'set-sidebar', sidebarMode: 'expanded' }, authority: 'presentation-only' },
  { id: 'sidebar.compact', label: 'Compact sidebar', hint: 'Local icon rail', keywords: ['sidebar', 'navigation', 'rail'], action: { kind: 'set-sidebar', sidebarMode: 'compact' }, authority: 'presentation-only' },
  ...(['system', 'full', 'reduced'] as const).map((motionMode) => ({ id: `motion.${motionMode}`, label: `Use ${motionMode} motion`, hint: 'Local accessibility preference', keywords: ['motion', 'animation', motionMode], action: { kind: 'set-motion' as const, motionMode }, authority: 'presentation-only' as const }))
];

export function filterPresentationCommands(query: string): readonly PresentationCommand[] {
  const needle = query.trim().toLocaleLowerCase();
  if (!needle) return FOCUSA_DESKTOP_PRESENTATION_COMMANDS;
  return FOCUSA_DESKTOP_PRESENTATION_COMMANDS.filter((command) => [command.label, command.hint, ...command.keywords].some((value) => value.toLocaleLowerCase().includes(needle)));
}
