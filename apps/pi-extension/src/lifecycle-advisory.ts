import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";

export type LifecycleAdvisoryStatus = "queued" | "skipped_headless" | "failed";

export type LifecycleAdvisoryInput = {
  advisoryKey: string;
  advisoryKind: "unbound_project" | "project_identity_bootstrap";
  title: string;
  content: string;
  reason: string;
  projectRoot: string;
  sessionId?: string;
};

type LifecycleAdvisoryPi = Pick<ExtensionAPI, "sendMessage" | "appendEntry">;

function appendOutcome(
  pi: LifecycleAdvisoryPi,
  input: LifecycleAdvisoryInput,
  status: LifecycleAdvisoryStatus,
  failureClass?: string
): void {
  try {
    pi.appendEntry("focusa-lifecycle-advisory-outcome", {
      schema: "focusa.pi_lifecycle_advisory_outcome.v1",
      advisory_key: input.advisoryKey,
      advisory_kind: input.advisoryKind,
      status,
      reason: input.reason,
      project_root: input.projectRoot,
      session_id: input.sessionId,
      failure_class: failureClass,
      recorded_at: new Date().toISOString(),
    });
  } catch {
    // The advisory itself remains non-triggering even when persistence is unavailable.
  }
}

/**
 * Queue lifecycle guidance without starting an agent turn.
 *
 * `sendUserMessage()` always calls `AgentSession.prompt()` and can race another
 * startup prompt. A custom message with `triggerTurn:false` is visible now and
 * becomes context for the next operator-triggered turn without entering Pi's
 * prompt-processing state.
 */
export function queueLifecycleAdvisory(
  pi: LifecycleAdvisoryPi,
  ctx: any,
  input: LifecycleAdvisoryInput
): LifecycleAdvisoryStatus {
  if (!ctx?.hasUI) {
    appendOutcome(pi, input, "skipped_headless");
    return "skipped_headless";
  }

  try {
    pi.sendMessage(
      {
        customType: "focusa-lifecycle-advisory",
        content: input.content,
        display: true,
        details: {
          schema: "focusa.pi_lifecycle_advisory.v1",
          advisory_key: input.advisoryKey,
          advisory_kind: input.advisoryKind,
          reason: input.reason,
          project_root: input.projectRoot,
          session_id: input.sessionId,
        },
      },
      { triggerTurn: false }
    );
    ctx?.ui?.notify?.(input.title, "warning");
    appendOutcome(pi, input, "queued");
    return "queued";
  } catch (error) {
    const failureClass = error instanceof Error ? error.name || "Error" : "unknown";
    appendOutcome(pi, input, "failed", failureClass);
    ctx?.ui?.notify?.(
      `${input.title} Focusa could not persist the advisory; no automatic agent turn was started.`,
      "warning"
    );
    return "failed";
  }
}
