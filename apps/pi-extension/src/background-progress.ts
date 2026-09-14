import type { AttachmentKey } from "./scoped-state.js";

export const PI_BACKGROUND_COMPLETION_ENTRY_SCHEMA = "focusa.pi_background_completion_entry.v1" as const;

export interface BackgroundJobVisualEvent {
  job_id?: unknown;
  name?: unknown;
  status?: unknown;
  exit_code?: unknown;
  log_path?: unknown;
  output_tail?: unknown;
  started_at?: unknown;
  completed_at?: unknown;
  attachment?: unknown;
}

export interface BackgroundCompletionEntryData {
  schema: typeof PI_BACKGROUND_COMPLETION_ENTRY_SCHEMA;
  attachment: AttachmentKey;
  job_id: string;
  name: string;
  status: string;
  exit_code: number | null;
  log_path?: string;
  output_tail: string;
  completed_at?: string;
}

function cleanTerminalText(value: unknown, maxBytes: number): string {
  return String(value || "")
    .replace(/\x1b\[[0-?]*[ -\/]*[@-~]/g, "")
    .replace(/[\u0000-\u0008\u000b\u000c\u000e-\u001f\u007f]/g, "")
    .slice(0, maxBytes);
}

export function attachmentKeysEqual(left: AttachmentKey | undefined, right: unknown): right is AttachmentKey {
  if (!left || !right || typeof right !== "object") return false;
  const candidate = right as AttachmentKey;
  const leftScope = left.workstream?.root_scope;
  const rightScope = candidate.workstream?.root_scope;
  return (
    !!leftScope &&
    !!rightScope &&
    leftScope.scope_kind === rightScope.scope_kind &&
    leftScope.scope_id === rightScope.scope_id &&
    leftScope.fingerprint === rightScope.fingerprint &&
    leftScope.root_path === rightScope.root_path &&
    left.workstream.continuity_id === candidate.workstream?.continuity_id &&
    left.instance_id === candidate.instance_id &&
    left.session_id === candidate.session_id &&
    left.attachment_id === candidate.attachment_id
  );
}

export function eventTargetsAttachment(
  event: BackgroundJobVisualEvent,
  current: AttachmentKey | undefined
): boolean {
  return attachmentKeysEqual(current, event?.attachment);
}

export function completionEntryData(
  event: BackgroundJobVisualEvent,
  attachment: AttachmentKey
): BackgroundCompletionEntryData {
  return {
    schema: PI_BACKGROUND_COMPLETION_ENTRY_SCHEMA,
    attachment,
    job_id: cleanTerminalText(event.job_id, 160),
    name: cleanTerminalText(event.name || event.job_id || "job", 240),
    status: cleanTerminalText(event.status || "completed", 80),
    exit_code: typeof event.exit_code === "number" ? event.exit_code : null,
    ...(event.log_path ? { log_path: cleanTerminalText(event.log_path, 2048) } : {}),
    output_tail: cleanTerminalText(event.output_tail, 4096),
    ...(event.completed_at ? { completed_at: cleanTerminalText(event.completed_at, 120) } : {}),
  };
}

export function formatBackgroundCompletion(
  data: Partial<BackgroundCompletionEntryData>,
  expanded: boolean
): { headline: string; details: string; ok: boolean } {
  const status = cleanTerminalText(data.status || "completed", 80);
  const exitCode = typeof data.exit_code === "number" ? data.exit_code : null;
  const ok = status === "completed" && (exitCode === null || exitCode === 0);
  const name = cleanTerminalText(data.name || "job", 240);
  const tailLines = cleanTerminalText(data.output_tail, 4096).split("\n").filter(Boolean);
  const tail = (tailLines[tailLines.length - 1] || "").slice(0, expanded ? 4096 : 320);
  const headline = `[bg] ${name} ${status} exit ${exitCode ?? "?"}`;
  const logPath = expanded ? cleanTerminalText(data.log_path, 2048) : "";
  const details = logPath
    ? `\n${logPath}${tail ? `\n${tail}` : ""}`
    : tail
      ? expanded
        ? `\n${tail}`
        : ` · ${tail}`
      : "";
  return { headline, details, ok };
}
