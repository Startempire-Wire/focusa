export type DesktopSidebarMode = 'expanded' | 'compact';

export interface DesktopSidebarPreferences {
  schema: 'focusa.desktop.sidebar_preferences.v1';
  mode: DesktopSidebarMode;
  widthPx: number;
  collapsedGroups: string[];
}

const STORAGE_KEY = 'focusa.desktop.sidebar_preferences.v1';

export function defaultSidebarPreferences(): DesktopSidebarPreferences {
  return { schema: 'focusa.desktop.sidebar_preferences.v1', mode: 'expanded', widthPx: 248, collapsedGroups: [] };
}

export function readSidebarPreferences(): DesktopSidebarPreferences {
  if (typeof window === 'undefined') return defaultSidebarPreferences();
  try {
    const raw = JSON.parse(window.localStorage.getItem(STORAGE_KEY) ?? 'null') as Partial<DesktopSidebarPreferences> | null;
    if (!raw || raw.schema !== 'focusa.desktop.sidebar_preferences.v1') return defaultSidebarPreferences();
    return {
      schema: 'focusa.desktop.sidebar_preferences.v1',
      mode: raw.mode === 'compact' ? 'compact' : 'expanded',
      widthPx: Math.min(320, Math.max(208, Number(raw.widthPx) || 248)),
      collapsedGroups: Array.isArray(raw.collapsedGroups) ? raw.collapsedGroups.filter((group): group is string => typeof group === 'string') : []
    };
  } catch {
    return defaultSidebarPreferences();
  }
}

export function saveSidebarPreferences(preferences: DesktopSidebarPreferences): void {
  if (typeof window === 'undefined') return;
  try { window.localStorage.setItem(STORAGE_KEY, JSON.stringify(preferences)); } catch { /* Local presentation preference only. */ }
}
