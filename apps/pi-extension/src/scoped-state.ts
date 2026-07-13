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
  const lines = [
    `Status: ${envelope.human.status}`,
    `Summary: ${envelope.human.summary}`,
    `Scope: ${envelope.scope.root_scope.scope_kind}:${envelope.scope.root_scope.canonical_name} · ${envelope.scope.continuity_id}`,
    `Authority: ${envelope.authority.status}`,
    `Next action: ${envelope.human.next_action}`,
    `Why: ${envelope.human.why || envelope.authority.why}`,
  ];
  if (envelope.human.warnings.length)
    lines.push(`Warnings: ${envelope.human.warnings.join(" | ")}`);
  if (envelope.human.evidence_refs.length)
    lines.push(`Evidence: ${envelope.human.evidence_refs.join(" | ")}`);
  return lines.join("\n");
}
