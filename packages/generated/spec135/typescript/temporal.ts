/** Typed Spec 137 client. It never computes deadlines, estimates, or progress locally. */
export interface TemporalScope { project_root: string; continuity_id: string }
export interface TemporalFailure { status: "blocked"; failure_class: string; message: string; next_action: string }
export interface TemporalResult<T = unknown> { schema: string; status: "completed"; canonical: true; [key: string]: T | string | true }
export interface TemporalMutation extends TemporalScope { idempotency_key: string }
export interface DeadlineSetRequest extends TemporalMutation { subject_ref: string; deadline_at: string; timezone: string; readiness_target?: string; completion_target_ref: string; confirm: true }
export interface DeadlineRevisionRequest extends TemporalMutation { deadline_id: string; expected_revision: number; reason: string; deadline_at?: string; confirm: true }
export interface ProgressRecordRequest extends TemporalMutation { item_id: string; kind: string; evidence_refs: string[] }
export type TemporalResponse<T = unknown> = TemporalResult<T> | TemporalFailure;

export interface TemporalClient {
  timeNow(): Promise<TemporalResponse>;
  timeStatus(scope: TemporalScope): Promise<TemporalResponse>;
  deadlines(scope: TemporalScope): Promise<TemporalResponse>;
  deadline(scope: TemporalScope, id: string): Promise<TemporalResponse>;
  deadlineConflicts(scope: TemporalScope): Promise<TemporalResponse>;
  setDeadline(request: DeadlineSetRequest): Promise<TemporalResponse>;
  reviseDeadline(request: DeadlineRevisionRequest): Promise<TemporalResponse>;
  clearDeadline(request: DeadlineRevisionRequest): Promise<TemporalResponse>;
  progress(scope: TemporalScope, itemId: string): Promise<TemporalResponse>;
  recordProgress(request: ProgressRecordRequest): Promise<TemporalResponse>;
  lostTime(scope: TemporalScope, subjectRef: string): Promise<TemporalResponse>;
  cancellation(scope: TemporalScope, id: string): Promise<TemporalResponse>;
}

export function createTemporalClient(baseUrl: string, fetcher: typeof fetch = fetch): TemporalClient {
  const base = baseUrl.replace(/\/$/, "");
  const query = (s: TemporalScope, extra: Record<string, string> = {}) => new URLSearchParams({ ...s, ...extra }).toString();
  const call = async (path: string, init?: RequestInit): Promise<TemporalResponse> => {
    const response = await fetcher(`${base}${path}`, init);
    const body = await response.json() as TemporalResponse;
    if (!response.ok || body.status === "blocked") {
      const failure = body as TemporalFailure;
      throw new Error(`${failure.failure_class ?? "temporal_request_failed"}: ${failure.message ?? response.status}`);
    }
    if (body.status !== "completed" || body.canonical !== true) throw new Error("invalid_temporal_response_envelope");
    return body;
  };
  const post = (path: string, body: unknown) => call(path, { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify(body) });
  return {
    timeNow: () => call("/v1/time/now"),
    timeStatus: (s) => call(`/v1/time/status?${query(s)}`),
    deadlines: (s) => call(`/v1/deadlines?${query(s)}`),
    deadline: (s, id) => call(`/v1/deadline/${encodeURIComponent(id)}?${query(s)}`),
    deadlineConflicts: (s) => call(`/v1/deadline/conflicts?${query(s)}`),
    setDeadline: (r) => post("/v1/deadline/set", r),
    reviseDeadline: (r) => post("/v1/deadline/revise", r),
    clearDeadline: (r) => post("/v1/deadline/clear", r),
    progress: (s, item_id) => call(`/v1/progress/status?${query(s, { item_id })}`),
    recordProgress: (r) => post("/v1/progress/record", r),
    lostTime: (s, subject_ref) => call(`/v1/lost-time/incidents?${query(s, { subject_ref })}`),
    cancellation: (s, id) => call(`/v1/cancellation/${encodeURIComponent(id)}?${query(s)}`),
  };
}
