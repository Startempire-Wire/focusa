import { sameWorkstreamKey } from '../../../../../docs/contracts/spec135/mission-canvas-v1/typescript/mission-canvas-validators.generated';
import type { ResolvedWorkspaceProjection, WorkstreamKey } from './types';

const LOCATOR_ATTRIBUTES = [
  'data-semantic-object-id',
  'data-contribution-id',
  'data-layout-node',
  'data-work-surface-id',
  'data-session-inventory-row',
  'aria-controls'
] as const;

export type PresentationLocator = Readonly<{ attribute: string; value: string }>;
export type FocusCapture = Readonly<{ locator: PresentationLocator }>;
export type ScrollCapture = Readonly<{ locator: PresentationLocator; left: number; top: number }>;
export type SelectionCapture = Readonly<{
  locator: PresentationLocator;
  start: number;
  end: number;
  direction: 'forward' | 'backward' | 'none';
}>;

export type PresentationStateSnapshot = Readonly<{
  workstream: WorkstreamKey;
  projectionRevision: number;
  focus?: FocusCapture;
  scroll: readonly ScrollCapture[];
  selection?: SelectionCapture;
  activeTab?: PresentationLocator;
}>;

type PresentElement = HTMLElement & {
  selectionStart?: number | null;
  selectionEnd?: number | null;
  selectionDirection?: 'forward' | 'backward' | 'none' | null;
  setSelectionRange?: (start: number, end: number, direction?: 'forward' | 'backward' | 'none') => void;
};

export function captureFocus(root: HTMLElement): FocusCapture | undefined {
  const active = root.ownerDocument.activeElement;
  if (!(active instanceof HTMLElement) || !root.contains(active)) return undefined;
  const locator = stableLocator(active, root);
  return locator ? Object.freeze({ locator }) : undefined;
}

export function captureScroll(root: HTMLElement): readonly ScrollCapture[] {
  const captures: ScrollCapture[] = [];
  const candidates = [root, ...root.querySelectorAll<HTMLElement>(stableSelector())];
  for (const element of candidates) {
    if (element.scrollTop === 0 && element.scrollLeft === 0) continue;
    const locator = stableLocator(element, root);
    if (!locator) continue;
    captures.push(Object.freeze({ locator, left: element.scrollLeft, top: element.scrollTop }));
  }
  return Object.freeze(captures);
}

export function captureSelection(root: HTMLElement): SelectionCapture | undefined {
  const active = root.ownerDocument.activeElement as PresentElement | null;
  if (!active || !root.contains(active)) return undefined;
  if (typeof active.selectionStart !== 'number' || typeof active.selectionEnd !== 'number') return undefined;
  const locator = stableLocator(active, root);
  if (!locator) return undefined;
  return Object.freeze({
    locator,
    start: active.selectionStart,
    end: active.selectionEnd,
    direction: active.selectionDirection ?? 'none'
  });
}

export function capturePresentationState(
  root: HTMLElement,
  projection: ResolvedWorkspaceProjection
): PresentationStateSnapshot {
  const activeTab = root.querySelector<HTMLElement>('[role="tab"][aria-selected="true"]');
  return Object.freeze({
    workstream: structuredClone(projection.workstream),
    projectionRevision: projection.projection_revision,
    focus: captureFocus(root),
    scroll: captureScroll(root),
    selection: captureSelection(root),
    activeTab: activeTab ? stableLocator(activeTab, root) : undefined
  });
}

export function restoreIfStillPresent(
  root: HTMLElement,
  snapshot: PresentationStateSnapshot,
  projection: ResolvedWorkspaceProjection
): boolean {
  if (!sameWorkstreamKey(snapshot.workstream, projection.workstream)) return false;
  if (projection.projection_revision < snapshot.projectionRevision) return false;

  for (const scroll of snapshot.scroll) {
    const element = resolveLocator(root, scroll.locator);
    if (!element) continue;
    element.scrollLeft = scroll.left;
    element.scrollTop = scroll.top;
  }

  if (snapshot.selection) {
    const element = resolveLocator(root, snapshot.selection.locator) as PresentElement | undefined;
    element?.setSelectionRange?.(
      snapshot.selection.start,
      snapshot.selection.end,
      snapshot.selection.direction
    );
  }

  const focusTarget = snapshot.focus ? resolveLocator(root, snapshot.focus.locator) : undefined;
  const activeTab = snapshot.activeTab ? resolveLocator(root, snapshot.activeTab) : undefined;
  if (activeTab?.getAttribute('aria-selected') === 'true' && !focusTarget) activeTab.focus({ preventScroll: true });
  else focusTarget?.focus({ preventScroll: true });
  return true;
}

function stableLocator(element: HTMLElement, root: HTMLElement): PresentationLocator | undefined {
  if (element === root) return Object.freeze({ attribute: 'data-presentation-root', value: 'true' });
  let current: HTMLElement | null = element;
  while (current && root.contains(current)) {
    for (const attribute of LOCATOR_ATTRIBUTES) {
      const value = current.getAttribute(attribute)?.trim();
      if (value) return Object.freeze({ attribute, value });
    }
    if (current === root) break;
    current = current.parentElement;
  }
  return undefined;
}

function resolveLocator(root: HTMLElement, locator: PresentationLocator): HTMLElement | undefined {
  if (locator.attribute === 'data-presentation-root') return root;
  return root.querySelector<HTMLElement>(`[${locator.attribute}="${escapeAttribute(locator.value)}"]`) ?? undefined;
}

function stableSelector(): string {
  return LOCATOR_ATTRIBUTES.map((attribute) => `[${attribute}]`).join(',');
}

function escapeAttribute(value: string): string {
  return value.replace(/\\/g, '\\\\').replace(/"/g, '\\"');
}
