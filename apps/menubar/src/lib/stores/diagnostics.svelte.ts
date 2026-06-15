// Menubar diagnostics — timestamped local error ledger for operator troubleshooting.

export type MenubarErrorClass =
  | 'network'
  | 'timeout'
  | 'http'
  | 'json_parse'
  | 'keychain'
  | 'global_js'
  | 'unhandled_rejection'
  | 'pairing_start'
  | 'pairing_poll'
  | 'pairing_list'
  | 'pairing_revoke'
  | 'pairing_bootstrap'
  | 'unknown';

export interface DiagnosticEntry {
  ts: string;
  area: string;
  phase: string;
  error_class: MenubarErrorClass;
  message: string;
  url?: string;
  method?: string;
  status?: number;
  failure_class?: string;
  body?: unknown;
  stack?: string;
  context?: Record<string, unknown>;
}

const MAX_ENTRIES = 100;
const STORAGE_KEY = 'focusa_menubar_diagnostics_v1';

function safeString(value: unknown): string {
  if (value instanceof Error) return value.message;
  if (typeof value === 'string') return value;
  try { return JSON.stringify(value); } catch { return String(value); }
}

function classify(error: unknown, fallback: MenubarErrorClass = 'unknown'): MenubarErrorClass {
  const anyErr = error as any;
  const name = String(anyErr?.name || '').toLowerCase();
  const msg = safeString(error).toLowerCase();
  if (name.includes('abort') || msg.includes('timeout') || msg.includes('timed out')) return 'timeout';
  if (anyErr?.status || anyErr?.failure_class || msg.includes('http_')) return 'http';
  if (msg.includes('keychain') || msg.includes('security') || msg.includes('generic-password')) return 'keychain';
  if (msg.includes('failed to fetch') || msg.includes('load failed') || msg.includes('networkerror')) return 'network';
  return fallback;
}

function load(): DiagnosticEntry[] {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return [];
    const parsed = JSON.parse(raw);
    return Array.isArray(parsed) ? parsed.slice(-MAX_ENTRIES) : [];
  } catch {
    return [];
  }
}

function persist(entries: DiagnosticEntry[]) {
  try { localStorage.setItem(STORAGE_KEY, JSON.stringify(entries.slice(-MAX_ENTRIES))); } catch {}
}

function createDiagnosticsStore() {
  let entries = $state<DiagnosticEntry[]>(load());

  function record(entry: Omit<DiagnosticEntry, 'ts' | 'error_class' | 'message'> & { error?: unknown; error_class?: MenubarErrorClass; message?: string }): DiagnosticEntry {
    const error = entry.error;
    const full: DiagnosticEntry = {
      ts: new Date().toISOString(),
      area: entry.area,
      phase: entry.phase,
      error_class: entry.error_class || classify(error, 'unknown'),
      message: entry.message || safeString(error || 'unknown error'),
      url: entry.url ?? (error as any)?.url,
      method: entry.method ?? (error as any)?.method,
      status: entry.status ?? (error as any)?.status,
      failure_class: entry.failure_class ?? (error as any)?.failure_class,
      body: entry.body ?? (error as any)?.body,
      stack: entry.stack ?? (error instanceof Error ? error.stack : undefined),
      context: entry.context,
    };
    entries = [...entries, full].slice(-MAX_ENTRIES);
    persist(entries);
    const parts = [
      `area=${full.area}`,
      `phase=${full.phase}`,
      `class=${full.error_class}`,
      full.status ? `status=${full.status}` : '',
      full.failure_class ? `failure_class=${full.failure_class}` : '',
      full.url ? `url=${full.url}` : '',
      `message=${full.message}`,
    ].filter(Boolean);
    console.error(`[focusa-menubar-diagnostic] ${parts.join(' ')}`);
    return full;
  }

  function clear() {
    entries = [];
    persist(entries);
  }

  function latest(): DiagnosticEntry | null {
    return entries.at(-1) || null;
  }

  function render(scope?: { area?: string; limit?: number }): string {
    const selected = entries
      .filter((e) => !scope?.area || e.area === scope.area)
      .slice(-(scope?.limit || 30));
    return selected.map((e) => JSON.stringify(e)).join('\n');
  }

  return {
    get entries() { return entries; },
    record,
    clear,
    latest,
    render,
  };
}

export const diagnosticsStore = createDiagnosticsStore();

export function installGlobalDiagnostics(): void {
  if (typeof window === 'undefined') return;
  if ((window as any).__focusaDiagnosticsInstalled) return;
  (window as any).__focusaDiagnosticsInstalled = true;
  window.addEventListener('error', (event) => {
    diagnosticsStore.record({
      area: 'global',
      phase: 'window.error',
      error_class: 'global_js',
      error: event.error || event.message,
      message: event.message,
      context: { filename: event.filename, lineno: event.lineno, colno: event.colno },
    });
  });
  window.addEventListener('unhandledrejection', (event) => {
    diagnosticsStore.record({
      area: 'global',
      phase: 'unhandledrejection',
      error_class: 'unhandled_rejection',
      error: event.reason,
    });
  });
}
