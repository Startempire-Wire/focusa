import { createHash } from "crypto";

// Typed scope, authority, result, and CRDT contracts for Spec 104.
// Canonical state is rooted by ScopeRef before continuity/workstream metadata.

export type ScopeKind = "project" | "host";
export type AuthorityStatus = "canonical" | "advisory" | "blocked" | "degraded";

export interface ScopeRef {
  scope_kind: ScopeKind;
  scope_id: string;
  root_path: string;
  canonical_name: string;
  fingerprint: string;
}

export interface WorkstreamKey {
  root_scope: ScopeRef;
  continuity_id: string;
}

export interface AttachmentKey {
  workstream: WorkstreamKey;
  instance_id: string;
  session_id: string;
  attachment_id: string;
}

/**
 * Pi reloads extension instances when the active native session changes, while
 * tool execution contexts may omit or vary their temporal session id. Keep the
 * latest verified typed attachment inside this extension instance so a later
 * scope-less tool can reuse project_root + continuity_id authority.
 */
export class PiExtensionSessionBinding {
  private attachment: AttachmentKey | undefined;

  bind(key: AttachmentKey): void {
    this.attachment = key;
  }

  resolve(): AttachmentKey | undefined {
    return this.attachment;
  }

  clear(): void {
    this.attachment = undefined;
  }
}

export function attachmentRoutingHints(eventOrParams: any): {
  sessionId?: unknown;
  projectRoot?: unknown;
  continuityId?: unknown;
} {
  const payload =
    eventOrParams?.input && typeof eventOrParams.input === "object"
      ? eventOrParams.input
      : eventOrParams || {};
  return {
    sessionId: eventOrParams?.sessionId || eventOrParams?.session_id,
    projectRoot:
      payload?.source_scope?.root_path || payload?.source_scope?.project_root || payload?.project_root,
    continuityId: payload?.source_scope?.continuity_id || payload?.continuity_id,
  };
}

export interface AuthorityEnvelope {
  status: AuthorityStatus;
  why: string;
}

export interface HumanReadableSummary {
  status: string;
  summary: string;
  next_action: string;
  why: string;
  evidence_refs: string[];
  warnings: string[];
}

export interface ScopedResultEnvelope<T> {
  schema: "focusa.scoped_result.v1";
  scope: WorkstreamKey;
  authority: AuthorityEnvelope;
  human: HumanReadableSummary;
  human_readable?: string;
  data: T;
}

export interface ScopedCrdtRecord<T> {
  schema: "focusa.scoped_state.v1";
  scope: WorkstreamKey;
  record_id: string;
  actor_id: string;
  vector_clock: Record<string, number>;
  lamport_ts: number;
  updated_at: string;
  tombstone: boolean;
  value: T;
}

function nonempty(value: unknown): value is string {
  return typeof value === "string" && value.trim().length > 0;
}

export function isScopeRef(value: unknown): value is ScopeRef {
  const scope = value as ScopeRef | null;
  return Boolean(
    scope &&
    (scope.scope_kind === "project" || scope.scope_kind === "host") &&
    nonempty(scope.scope_id) &&
    nonempty(scope.root_path) &&
    nonempty(scope.canonical_name) &&
    nonempty(scope.fingerprint)
  );
}

export function isWorkstreamKey(value: unknown): value is WorkstreamKey {
  const scope = value as WorkstreamKey | null;
  return Boolean(scope && isScopeRef(scope.root_scope) && nonempty(scope.continuity_id));
}

export function buildProjectWorkstreamKey(
  projectRoot: string,
  continuityId: string,
  canonicalName?: string
): WorkstreamKey {
  const root = String(projectRoot || "")
    .trim()
    .replace(/\/+$/, "");
  const continuity = String(continuityId || "").trim();
  if (!root || !continuity) throw new Error("typed_scope_required");
  const fingerprint = `sha256:${createHash("sha256").update(root).digest("hex")}`;
  const name = canonicalName || root.split("/").filter(Boolean).at(-1) || "project";
  return {
    root_scope: {
      scope_kind: "project",
      scope_id: `project:${fingerprint.slice(7, 23)}`,
      root_path: root,
      canonical_name: name,
      fingerprint,
    },
    continuity_id: continuity,
  };
}

export function scopedQueryParams(scope: WorkstreamKey): URLSearchParams {
  const query = new URLSearchParams();
  query.set("scope_kind", scope.root_scope.scope_kind);
  query.set("scope_id", scope.root_scope.scope_id);
  query.set("root_path", scope.root_scope.root_path);
  query.set("canonical_name", scope.root_scope.canonical_name);
  query.set("fingerprint", scope.root_scope.fingerprint);
  query.set("continuity_id", scope.continuity_id);
  return query;
}

export function sameRootScope(left: ScopeRef, right: ScopeRef): boolean {
  return (
    left.scope_kind === right.scope_kind &&
    left.scope_id === right.scope_id &&
    left.fingerprint === right.fingerprint &&
    left.root_path.replace(/\/+$/, "") === right.root_path.replace(/\/+$/, "")
  );
}

export function sameWorkstream(left: WorkstreamKey, right: WorkstreamKey): boolean {
  return sameRootScope(left.root_scope, right.root_scope) && left.continuity_id === right.continuity_id;
}

export function compareVectorClock(
  left: Record<string, number>,
  right: Record<string, number>
): -1 | 0 | 1 | null {
  let less = false;
  let greater = false;
  for (const actor of new Set([...Object.keys(left), ...Object.keys(right)])) {
    const l = left[actor] || 0;
    const r = right[actor] || 0;
    if (l < r) less = true;
    if (l > r) greater = true;
  }
  if (less && greater) return null;
  if (less) return -1;
  if (greater) return 1;
  return 0;
}

function mergeVectorClock(
  left: Record<string, number>,
  right: Record<string, number>
): Record<string, number> {
  const merged: Record<string, number> = { ...left };
  for (const [actor, counter] of Object.entries(right)) {
    merged[actor] = Math.max(merged[actor] || 0, counter);
  }
  return Object.fromEntries(Object.entries(merged).sort(([left], [right]) => left.localeCompare(right)));
}

function stableValue(value: unknown): string {
  if (Array.isArray(value)) return `[${value.map(stableValue).join(",")}]`;
  if (value && typeof value === "object") {
    return `{${Object.entries(value as Record<string, unknown>)
      .sort(([a], [b]) => a.localeCompare(b))
      .map(([key, item]) => `${JSON.stringify(key)}:${stableValue(item)}`)
      .join(",")}}`;
  }
  return JSON.stringify(value);
}

export function reconcileScopedRecord<T>(
  left: ScopedCrdtRecord<T>,
  right: ScopedCrdtRecord<T>
): ScopedCrdtRecord<T> {
  if (!sameWorkstream(left.scope, right.scope)) throw new Error("scope_mismatch");
  if (left.record_id !== right.record_id) throw new Error("record_mismatch");
  const order = compareVectorClock(left.vector_clock, right.vector_clock);
  let winner: ScopedCrdtRecord<T>;
  if (order === 1) winner = left;
  else if (order === -1) winner = right;
  else {
    const leftTie = [
      left.lamport_ts,
      left.updated_at,
      left.actor_id,
      stableValue([left.tombstone, left.value]),
    ];
    const rightTie = [
      right.lamport_ts,
      right.updated_at,
      right.actor_id,
      stableValue([right.tombstone, right.value]),
    ];
    winner = stableValue(leftTie) >= stableValue(rightTie) ? left : right;
  }
  return {
    ...winner,
    vector_clock: mergeVectorClock(left.vector_clock, right.vector_clock),
    lamport_ts: Math.max(left.lamport_ts, right.lamport_ts),
    updated_at: left.updated_at >= right.updated_at ? left.updated_at : right.updated_at,
  };
}

export function renderScopedResultHuman<T>(envelope: ScopedResultEnvelope<T>): string {
  const body = envelope as any;
  // Rolling upgrades can leave the active daemon on the legacy prediction
  // envelope while the reloaded Pi extension already expects scoped_result.v1.
  // Render bounded legacy responses without inventing canonical authority.
  if (!body?.human || !body?.scope || !body?.authority) {
    const items = Array.isArray(body?.predictions) ? body.predictions.length : undefined;
    return [
      `Status: ${String(body?.status || (body?.error ? "blocked" : "ok"))}`,
      `Summary: ${String(body?.summary || body?.message || body?.error || "Legacy daemon response")}`,
      ...(items === undefined ? [] : [`Predictions: ${items}`]),
      "Authority: legacy envelope; scoped Pi request supplied, canonical authority not inferred",
      "Next action: upgrade/restart the daemon when safe; continue with bounded compatibility output",
    ].join("\n");
  }
  const lines = [
    ...(body.human_readable ? [`Human readable: ${body.human_readable}`] : []),
    `Status: ${body.human.status}`,
    `Summary: ${body.human.summary}`,
    `Scope: ${body.scope.root_scope.scope_kind}:${body.scope.root_scope.canonical_name} · ${body.scope.continuity_id}`,
    `Authority: ${body.authority.status}`,
    `Next action: ${body.human.next_action}`,
    `Why: ${body.human.why || body.authority.why}`,
  ];
  if (body.human.warnings.length) lines.push(`Warnings: ${body.human.warnings.join(" | ")}`);
  if (body.human.evidence_refs.length) lines.push(`Evidence: ${body.human.evidence_refs.join(" | ")}`);
  return lines.join("\n");
}
