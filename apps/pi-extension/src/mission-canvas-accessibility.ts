export type CanvasResponsiveMode = "narrow" | "stacked" | "desktop";

export interface CanvasAccessibilityPreferences {
  ascii: boolean;
  highContrast: boolean;
  reducedMotion: boolean;
  colorIndependent: true;
  restoreFocusAfterModal: true;
}

export function responsiveCanvasMode(width: number): CanvasResponsiveMode {
  if (width < 48) return "narrow";
  if (width < 90) return "stacked";
  return "desktop";
}

export function accessibilityPreferences(
  environment: Record<string, string | undefined> = process.env
): CanvasAccessibilityPreferences {
  return {
    ascii: environment.FOCUSA_ASCII_UI === "1" || environment.TERM === "dumb",
    highContrast:
      environment.FOCUSA_HIGH_CONTRAST === "1" || environment.NO_COLOR !== undefined,
    reducedMotion:
      environment.FOCUSA_REDUCED_MOTION === "1" || environment.CI === "true",
    colorIndependent: true,
    restoreFocusAfterModal: true,
  };
}

export function virtualWindow<T>(
  values: T[],
  selectedIndex: number,
  capacity: number
): { start: number; values: T[] } {
  const safeCapacity = Math.max(1, Math.min(capacity, 100));
  const selected = Math.max(0, Math.min(selectedIndex, Math.max(0, values.length - 1)));
  const start = Math.max(
    0,
    Math.min(selected - Math.floor(safeCapacity / 2), Math.max(0, values.length - safeCapacity))
  );
  return { start, values: values.slice(start, start + safeCapacity) };
}

export function surfaceCapacity(width: number): number {
  const mode = responsiveCanvasMode(width);
  return mode === "narrow" ? 2 : mode === "stacked" ? 4 : 8;
}

export function accessibleStateLabel(
  kind: string,
  state: string,
  isolation: string
): string {
  const safeKind = kind.trim() || "unknown-kind";
  const safeState = state.trim() || "unknown-state";
  const safeIsolation = isolation.trim() || "isolation-unknown";
  return `${safeKind} · state:${safeState} · isolation:${safeIsolation}`;
}

export function focusRestorationLabel(preferences: CanvasAccessibilityPreferences): string {
  return preferences.restoreFocusAfterModal
    ? "focus-restoration:editor"
    : "focus-restoration:unspecified";
}
