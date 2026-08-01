export const OPERATOR_STATUS_SETTINGS_VERSION = 1;

export type OperatorWidgetId = "time" | "prediction" | "version" | "ota" | "provider-usage";
export type OperatorWidgetState = "ready" | "loading" | "unavailable" | "stale" | "degraded";

export interface OperatorWidgetView {
  id: string;
  label: string;
  order: number;
  text: string;
  state: OperatorWidgetState;
  source: string;
  observedAt?: number;
}

export interface OperatorStatusData {
  now: number;
  timezone?: string;
  deadline?: unknown;
  prediction?: unknown;
  predictionLoading?: boolean;
  predictionObservedAt?: number;
  version?: unknown;
  ota?: unknown;
  otaState?: OperatorWidgetState;
  otaObservedAt?: number;
  provider?: unknown;
  model?: unknown;
  usagePercent?: number | null;
  renewalAt?: unknown;
  providerObservedAt?: number;
}

export interface OperatorStatusWidget {
  id: string;
  label: string;
  order: number;
  defaultEnabled: boolean;
  render(data: OperatorStatusData): OperatorWidgetView;
}

export interface OperatorStatusSettings {
  version: number;
  enabled: Record<string, boolean>;
}

const STALE_AFTER_MS = 60 * 60 * 1_000;

function clean(value: unknown, max = 80): string {
  if (value !== null && typeof value === "object") return "";
  const text = String(value ?? "").replace(/\s+/g, " ").trim();
  if (!text) return "";
  return text.length > max ? `${text.slice(0, Math.max(0, max - 1))}…` : text;
}

function formatDate(value: unknown, timezone?: string): string {
  const text = clean(value, 120);
  if (!text) return "";
  const numeric = Number(text);
  const date = Number.isFinite(numeric)
    ? new Date(numeric > 10_000_000_000 ? numeric : numeric * 1_000)
    : new Date(text);
  if (Number.isNaN(date.getTime())) return clean(text, 28);
  try {
    return new Intl.DateTimeFormat("en-US", {
      month: "short",
      day: "numeric",
      hour: "numeric",
      minute: "2-digit",
      timeZoneName: "short",
      timeZone: timezone || undefined,
    }).format(date);
  } catch {
    return date.toISOString();
  }
}

function clock(data: OperatorStatusData): string {
  try {
    return new Intl.DateTimeFormat("en-US", {
      hour: "numeric",
      minute: "2-digit",
      timeZoneName: "short",
      timeZone: data.timezone || undefined,
    }).format(new Date(data.now));
  } catch {
    return new Date(data.now).toISOString();
  }
}

function freshness(observedAt: number | undefined, now: number): OperatorWidgetState {
  return observedAt !== undefined && now - observedAt > STALE_AFTER_MS ? "stale" : "ready";
}

function view(
  widget: Pick<OperatorStatusWidget, "id" | "label" | "order">,
  text: string,
  state: OperatorWidgetState,
  source: string,
  observedAt?: number
): OperatorWidgetView {
  return { ...widget, text, state, source, observedAt };
}

const timeWidget: OperatorStatusWidget = {
  id: "time",
  label: "Time & deadline",
  order: 10,
  defaultEnabled: true,
  render(data) {
    const deadline = formatDate(data.deadline, data.timezone);
    return view(
      this,
      deadline ? `Time ${clock(data)} · deadline ${deadline}` : `Time ${clock(data)} · deadline unavailable`,
      deadline ? "ready" : "degraded",
      deadline ? "system clock + confirmed deadline" : "system clock; no confirmed deadline"
    );
  },
};

const predictionWidget: OperatorStatusWidget = {
  id: "prediction",
  label: "Next prediction",
  order: 20,
  defaultEnabled: true,
  render(data) {
    const prediction = clean(data.prediction, 100);
    const state = prediction
      ? freshness(data.predictionObservedAt, data.now)
      : data.predictionLoading
        ? "loading"
        : "unavailable";
    return view(
      this,
      prediction ? `Next ${prediction}` : state === "loading" ? "Next prediction loading" : "Next prediction unavailable",
      state,
      "active Workpoint",
      data.predictionObservedAt
    );
  },
};

const versionWidget: OperatorStatusWidget = {
  id: "version",
  label: "Focusa version",
  order: 30,
  defaultEnabled: true,
  render(data) {
    const candidate = clean(data.version, 32).replace(/^focusa-pi-bridge@/, "");
    const version = candidate.toLowerCase() === "unknown" ? "" : candidate;
    return view(this, version ? `Focusa ${version}` : "Focusa version unavailable", version ? "ready" : "unavailable", "extension package");
  },
};

const otaWidget: OperatorStatusWidget = {
  id: "ota",
  label: "OTA status",
  order: 40,
  defaultEnabled: true,
  render(data) {
    const ota = clean(data.ota, 32);
    let state = data.otaState ?? (ota ? "ready" : "unavailable");
    if (state === "ready") state = freshness(data.otaObservedAt, data.now);
    return view(this, ota ? `OTA ${ota}` : "OTA unavailable", state, "OTA activation receipts", data.otaObservedAt);
  },
};

const providerUsageWidget: OperatorStatusWidget = {
  id: "provider-usage",
  label: "Provider usage & renewal",
  order: 50,
  defaultEnabled: true,
  render(data) {
    const provider = clean(data.provider, 20);
    const model = clean(data.model, 24);
    const usage = data.usagePercent;
    const renewal = formatDate(data.renewalAt, data.timezone);
    const hasUsage = typeof usage === "number" && Number.isFinite(usage);
    const identity = [provider, model].filter(Boolean).join("/") || "Provider";
    const details = [hasUsage ? `${Math.round(Math.max(0, Math.min(100, usage)))}% used` : "usage unavailable", renewal ? `renews ${renewal}` : "renewal unavailable"];
    let state: OperatorWidgetState = !provider && !model && !hasUsage && !renewal ? "unavailable" : !hasUsage || !renewal ? "degraded" : "ready";
    if (state !== "unavailable" && freshness(data.providerObservedAt, data.now) === "stale") state = "stale";
    return view(this, `${identity} · ${details.join(" · ")}`, state, "model provider response headers", data.providerObservedAt);
  },
};

export const DEFAULT_OPERATOR_STATUS_WIDGETS: readonly OperatorStatusWidget[] = Object.freeze([
  timeWidget,
  predictionWidget,
  versionWidget,
  otaWidget,
  providerUsageWidget,
]);

export function createOperatorWidgetRegistry(widgets: readonly OperatorStatusWidget[] = DEFAULT_OPERATOR_STATUS_WIDGETS): readonly OperatorStatusWidget[] {
  const ids = new Set<string>();
  for (const widget of widgets) {
    if (!widget.id || ids.has(widget.id)) throw new Error(`Duplicate or empty operator status widget id: ${widget.id}`);
    ids.add(widget.id);
  }
  return Object.freeze([...widgets].sort((a, b) => a.order - b.order || a.id.localeCompare(b.id)));
}

export function migrateOperatorStatusSettings(
  value: unknown,
  legacy: Partial<Record<OperatorWidgetId, boolean>> = {},
  registry = createOperatorWidgetRegistry()
): OperatorStatusSettings {
  const raw = value && typeof value === "object" ? value as Record<string, unknown> : {};
  const rawEnabled = raw.enabled && typeof raw.enabled === "object" ? raw.enabled as Record<string, unknown> : raw;
  const enabled: Record<string, boolean> = {};
  for (const widget of registry) {
    const candidate = rawEnabled[widget.id];
    enabled[widget.id] = typeof candidate === "boolean"
      ? candidate
      : typeof legacy[widget.id as OperatorWidgetId] === "boolean"
        ? legacy[widget.id as OperatorWidgetId]!
        : widget.defaultEnabled;
  }
  return { version: OPERATOR_STATUS_SETTINGS_VERSION, enabled };
}

export function operatorStatusRollbackPatch(settings: OperatorStatusSettings): Record<string, boolean> {
  return {
    operatorStatusTimeEnabled: settings.enabled.time ?? true,
    operatorStatusDeadlineEnabled: settings.enabled.time ?? true,
    operatorStatusPredictionEnabled: settings.enabled.prediction ?? true,
    operatorStatusVersionEnabled: settings.enabled.version ?? true,
    operatorStatusOtaEnabled: settings.enabled.ota ?? true,
    operatorStatusModelUsageEnabled: settings.enabled["provider-usage"] ?? true,
  };
}

export function renderOperatorStatusBar(
  data: OperatorStatusData,
  settings: OperatorStatusSettings,
  maxWidth = 120,
  registry = createOperatorWidgetRegistry()
): { text: string; widgets: OperatorWidgetView[]; hidden: number } {
  const widgets = registry.filter((widget) => settings.enabled[widget.id] ?? widget.defaultEnabled).map((widget) => widget.render(data));
  if (maxWidth <= 0 || widgets.length === 0) return { text: "", widgets, hidden: widgets.length };
  const decorated = widgets.map((item) => `${item.text} [${item.state}; ${item.source}]`);
  let text = "";
  let shown = 0;
  for (const segment of decorated) {
    const candidate = text ? `${text} | ${segment}` : segment;
    const remaining = decorated.length - shown - 1;
    const suffix = remaining > 0 ? ` | +${remaining}` : "";
    if (candidate.length + suffix.length > maxWidth) break;
    text = candidate;
    shown += 1;
  }
  if (shown === 0) {
    const suffix = widgets.length > 1 ? `… +${widgets.length - 1}` : "…";
    text = decorated[0].length <= maxWidth ? decorated[0] : maxWidth <= suffix.length ? suffix.slice(0, maxWidth) : `${decorated[0].slice(0, maxWidth - suffix.length)}${suffix}`;
    shown = 1;
  } else if (shown < widgets.length) {
    const suffix = ` | +${widgets.length - shown}`;
    text = `${text.slice(0, Math.max(0, maxWidth - suffix.length))}${suffix}`;
  }
  return { text, widgets, hidden: widgets.length - shown };
}
