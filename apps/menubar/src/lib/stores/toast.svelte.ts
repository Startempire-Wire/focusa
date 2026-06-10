// Toast store — transient user feedback for action buttons.
// macOS-HIG style: small, top-right, auto-dismiss after 3s.

export type ToastKind = 'info' | 'ok' | 'warn' | 'err';

export interface Toast {
  id: string;
  kind: ToastKind;
  message: string;
  detail?: string;
  createdAt: number;
  ttlMs: number;
}

function createToastStore() {
  let items = $state<Toast[]>([]);
  const handles = new Map<string, ReturnType<typeof setTimeout>>();

  function push(t: Omit<Toast, 'id' | 'createdAt' | 'ttlMs'> & { ttlMs?: number }): string {
    const id = `t-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
    const ttlMs = t.ttlMs ?? 3_000;
    const item: Toast = { id, createdAt: Date.now(), ttlMs, ...t };
    items = [...items, item];
    const handle = setTimeout(() => dismiss(id), ttlMs);
    handles.set(id, handle);
    return id;
  }

  function dismiss(id: string) {
    items = items.filter((i) => i.id !== id);
    const h = handles.get(id);
    if (h) {
      clearTimeout(h);
      handles.delete(id);
    }
  }

  function clear() {
    for (const h of handles.values()) clearTimeout(h);
    handles.clear();
    items = [];
  }

  return {
    get items() { return items; },
    push,
    info: (message: string, detail?: string) => push({ kind: 'info', message, detail }),
    ok: (message: string, detail?: string) => push({ kind: 'ok', message, detail, ttlMs: 2_500 }),
    warn: (message: string, detail?: string) => push({ kind: 'warn', message, detail, ttlMs: 5_000 }),
    err: (message: string, detail?: string) => push({ kind: 'err', message, detail, ttlMs: 6_000 }),
    dismiss,
    clear,
  };
}

export const toastStore = createToastStore();
